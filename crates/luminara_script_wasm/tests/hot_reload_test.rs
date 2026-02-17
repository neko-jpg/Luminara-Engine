/// **Validates: Requirements 17.3**
/// Tests that WASM hot-reload works correctly
/// Note: WASM modules are stateless, so state must be managed externally

use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};

/// Test basic hot-reload functionality
#[test]
fn test_wasm_hot_reload_basic() {
    let limits = ResourceLimits {
        max_memory: 1024 * 1024, // 1MB
        max_execution_time: std::time::Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load initial WASM module (minimal WAT)
    let wat_v1 = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;
    let wasm_v1 = wat::parse_str(wat_v1).unwrap();
    let id = runtime.load_module(&wasm_v1).unwrap();

    // Verify v1 works
    let module = runtime.modules.get(&id).unwrap();
    let add_func = module
        .instance
        .get_typed_func::<(i32, i32), i32>(&mut runtime.store, "add")
        .unwrap();
    let result = add_func.call(&mut runtime.store, (2, 3)).unwrap();
    assert_eq!(result, 5);

    // Reload with v2 (different implementation)
    let wat_v2 = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                i32.const 10
                i32.add
            )
        )
    "#;
    let wasm_v2 = wat::parse_str(wat_v2).unwrap();
    runtime.reload_module(id, &wasm_v2).unwrap();

    // Verify v2 works (adds 10 to result)
    let module = runtime.modules.get(&id).unwrap();
    let add_func = module
        .instance
        .get_typed_func::<(i32, i32), i32>(&mut runtime.store, "add")
        .unwrap();
    let result = add_func.call(&mut runtime.store, (2, 3)).unwrap();
    assert_eq!(result, 15); // 2 + 3 + 10
}

/// Test that reload fails gracefully on invalid WASM
#[test]
fn test_wasm_hot_reload_error_handling() {
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: std::time::Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load valid module
    let wat_v1 = r#"
        (module
            (func (export "test") (result i32)
                i32.const 42
            )
        )
    "#;
    let wasm_v1 = wat::parse_str(wat_v1).unwrap();
    let id = runtime.load_module(&wasm_v1).unwrap();

    // Try to reload with invalid WASM
    let invalid_wasm = vec![0x00, 0x61, 0x73, 0x6d]; // Incomplete WASM header
    let result = runtime.reload_module(id, &invalid_wasm);

    // Should fail
    assert!(result.is_err());

    // Old module should still work
    let module = runtime.modules.get(&id).unwrap();
    let test_func = module
        .instance
        .get_typed_func::<(), i32>(&mut runtime.store, "test")
        .unwrap();
    let result = test_func.call(&mut runtime.store, ()).unwrap();
    assert_eq!(result, 42);
}

/// Test multiple consecutive reloads
#[test]
fn test_wasm_multiple_reloads() {
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: std::time::Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load initial module
    let wat_v1 = r#"
        (module
            (func (export "get_value") (result i32)
                i32.const 1
            )
        )
    "#;
    let wasm_v1 = wat::parse_str(wat_v1).unwrap();
    let id = runtime.load_module(&wasm_v1).unwrap();

    // Perform multiple reloads
    for i in 2..=5 {
        let wat = format!(
            r#"
            (module
                (func (export "get_value") (result i32)
                    i32.const {}
                )
            )
        "#,
            i
        );
        let wasm = wat::parse_str(&wat).unwrap();
        runtime.reload_module(id, &wasm).unwrap();

        // Verify new version works
        let module = runtime.modules.get(&id).unwrap();
        let func = module
            .instance
            .get_typed_func::<(), i32>(&mut runtime.store, "get_value")
            .unwrap();
        let result = func.call(&mut runtime.store, ()).unwrap();
        assert_eq!(result, i);
    }
}

/// Test that WASM state must be managed externally
/// This documents the expected behavior: WASM modules don't preserve internal state
#[test]
fn test_wasm_stateless_behavior() {
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: std::time::Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Module with mutable global (state)
    let wat_v1 = r#"
        (module
            (global $counter (mut i32) (i32.const 0))
            
            (func (export "increment")
                global.get $counter
                i32.const 1
                i32.add
                global.set $counter
            )
            
            (func (export "get_counter") (result i32)
                global.get $counter
            )
        )
    "#;
    let wasm_v1 = wat::parse_str(wat_v1).unwrap();
    let id = runtime.load_module(&wasm_v1).unwrap();

    // Increment counter
    let module = runtime.modules.get(&id).unwrap();
    let increment = module
        .instance
        .get_typed_func::<(), ()>(&mut runtime.store, "increment")
        .unwrap();
    increment.call(&mut runtime.store, ()).unwrap();
    increment.call(&mut runtime.store, ()).unwrap();
    increment.call(&mut runtime.store, ()).unwrap();

    // Check counter
    let get_counter = module
        .instance
        .get_typed_func::<(), i32>(&mut runtime.store, "get_counter")
        .unwrap();
    let counter_before = get_counter.call(&mut runtime.store, ()).unwrap();
    assert_eq!(counter_before, 3);

    // Reload with same module
    runtime.reload_module(id, &wasm_v1).unwrap();

    // Counter should be reset to 0 (state not preserved)
    let module = runtime.modules.get(&id).unwrap();
    let get_counter = module
        .instance
        .get_typed_func::<(), i32>(&mut runtime.store, "get_counter")
        .unwrap();
    let counter_after = get_counter.call(&mut runtime.store, ()).unwrap();
    assert_eq!(counter_after, 0); // State was NOT preserved
}
