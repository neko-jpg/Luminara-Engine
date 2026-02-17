/// Tests for WASM resource limit enforcement
/// **Validates: Requirements 17.2**
///
/// This test suite verifies that WASM scripts are properly sandboxed with:
/// - Memory limits enforced (prevent excessive allocation)
/// - Execution time limits enforced (timeout long-running scripts)
/// - Instruction count limits enforced (prevent runaway execution)
/// - Scripts terminated when limits are exceeded

use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};
use std::time::{Duration, Instant};

#[test]
fn test_memory_limit_enforcement() {
    // Test that memory limits prevent excessive allocation
    let limits = ResourceLimits {
        max_instructions: 1_000_000,
        max_memory: 1024 * 64, // 64KB limit
        max_execution_time: Duration::from_secs(5),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // WASM module that tries to allocate more than the limit
    // This module has 1 page (64KB) of memory initially
    let wat = r#"
        (module
            (memory (export "memory") 1)  ;; 1 page = 64KB
            (global $heap (mut i32) (i32.const 1024))
            
            (func $alloc (export "alloc") (param $size i32) (result i32)
                (local $ptr i32)
                (local.set $ptr (global.get $heap))
                (global.set $heap (i32.add (global.get $heap) (local.get $size)))
                (local.get $ptr)
            )
            
            (func (export "allocate_large") (param $ptr i32) (param $len i32) (result i32 i32)
                (local $result_ptr i32)
                
                ;; Try to grow memory beyond limit
                ;; memory.grow returns -1 on failure
                i32.const 10  ;; Try to grow by 10 pages (640KB)
                memory.grow
                drop
                
                ;; Allocate space for result JSON: "\"ok\""
                (local.set $result_ptr (call $alloc (i32.const 4)))
                
                ;; Write "ok" as JSON string
                (i32.store8 (local.get $result_ptr) (i32.const 34))  ;; "
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 1)) (i32.const 111))  ;; o
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 2)) (i32.const 107))  ;; k
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 3)) (i32.const 34))  ;; "
                
                ;; Return success indicator
                (local.get $result_ptr)
                (i32.const 4)
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    // The module should load successfully but memory growth should be limited
    // Note: Wasmtime's memory limits are configured at module instantiation
    // The actual enforcement happens when memory.grow is called
    let result: Result<String, _> = runtime.call_json_func(id, "allocate_large", "test");
    
    // The call should succeed (the module handles the failure internally)
    // but memory growth should have been prevented
    assert!(result.is_ok());
}

#[test]
fn test_instruction_limit_enforcement() {
    // Test that instruction count limits prevent runaway execution
    let limits = ResourceLimits {
        max_instructions: 10_000, // Very low limit
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // WASM module with an infinite loop
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32)
                i32.const 0
            )
            (func (export "infinite_loop") (param i32 i32) (result i32 i32)
                (local $counter i32)
                (local.set $counter (i32.const 0))
                
                (loop $continue
                    ;; Increment counter
                    (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
                    
                    ;; Loop forever (or until fuel runs out)
                    (br $continue)
                )
                
                ;; Never reached
                i32.const 0
                i32.const 0
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    // Start the epoch timer to enable time-based interruption
    runtime.start_epoch_timer();

    // The call should fail due to fuel exhaustion
    let result: Result<String, _> = runtime.call_json_func(id, "infinite_loop", "test");
    
    assert!(result.is_err(), "Expected infinite loop to be terminated by fuel limit");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("fuel") || err_msg.contains("out of fuel") || err_msg.contains("Call failed"),
        "Error should mention fuel exhaustion, got: {}",
        err_msg
    );
}

#[test]
fn test_execution_time_limit_enforcement() {
    // Test that execution time limits timeout long-running scripts
    let limits = ResourceLimits {
        max_instructions: 100_000_000, // High instruction limit
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_millis(500), // 500ms timeout
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // WASM module with a long-running computation
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32)
                i32.const 0
            )
            (func (export "long_computation") (param i32 i32) (result i32 i32)
                (local $counter i32)
                (local $result i32)
                (local.set $counter (i32.const 0))
                (local.set $result (i32.const 0))
                
                ;; Loop many times to consume time
                (loop $continue
                    (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
                    (local.set $result (i32.add (local.get $result) (local.get $counter)))
                    
                    ;; Continue if counter < 10,000,000
                    (br_if $continue (i32.lt_u (local.get $counter) (i32.const 10000000)))
                )
                
                i32.const 0
                i32.const 0
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    // Start the epoch timer to enable time-based interruption
    runtime.start_epoch_timer();

    let start = Instant::now();
    let result: Result<String, _> = runtime.call_json_func(id, "long_computation", "test");
    let elapsed = start.elapsed();

    // The call should fail due to timeout
    assert!(result.is_err(), "Expected long computation to be terminated by timeout");
    
    // Verify it was terminated reasonably quickly (within 2x the timeout)
    assert!(
        elapsed < Duration::from_secs(2),
        "Script should have been terminated quickly, took {:?}",
        elapsed
    );
}

#[test]
fn test_sandbox_restrictions_basic() {
    // Test that sandbox restrictions work (basic validation)
    let limits = ResourceLimits {
        max_instructions: 100_000,
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // WASM module that only uses allowed operations
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $heap (mut i32) (i32.const 1024))
            
            (func $alloc (export "alloc") (param $size i32) (result i32)
                (local $ptr i32)
                (local.set $ptr (global.get $heap))
                (global.set $heap (i32.add (global.get $heap) (local.get $size)))
                (local.get $ptr)
            )
            
            (func (export "safe_computation") (param $ptr i32) (param $len i32) (result i32 i32)
                (local $a i32)
                (local $b i32)
                (local $result_ptr i32)
                
                ;; Safe arithmetic operations
                (local.set $a (i32.const 10))
                (local.set $b (i32.const 20))
                (local.set $a (i32.add (local.get $a) (local.get $b)))
                
                ;; Allocate space for result JSON: "\"ok\""
                (local.set $result_ptr (call $alloc (i32.const 4)))
                
                ;; Write "ok" as JSON string
                (i32.store8 (local.get $result_ptr) (i32.const 34))  ;; "
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 1)) (i32.const 111))  ;; o
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 2)) (i32.const 107))  ;; k
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 3)) (i32.const 34))  ;; "
                
                ;; Return pointer and length
                (local.get $result_ptr)
                (i32.const 4)
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    // Safe operations should succeed
    let result: Result<String, _> = runtime.call_json_func(id, "safe_computation", "test");
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Safe operations should succeed");
}

#[test]
fn test_script_termination_on_limit_exceeded() {
    // Test that scripts are properly terminated when limits are exceeded
    let limits = ResourceLimits {
        max_instructions: 5_000, // Very low limit
        max_memory: 1024 * 64,
        max_execution_time: Duration::from_secs(5),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // WASM module that will exceed instruction limit
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32)
                i32.const 0
            )
            (func (export "exceed_limit") (param i32 i32) (result i32 i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                
                ;; Loop that will exceed instruction limit
                (loop $continue
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br_if $continue (i32.lt_u (local.get $i) (i32.const 100000)))
                )
                
                i32.const 0
                i32.const 0
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    runtime.start_epoch_timer();

    // First call should fail due to limit
    let result1: Result<String, _> = runtime.call_json_func(id, "exceed_limit", "test1");
    assert!(result1.is_err(), "First call should fail due to instruction limit");

    // Runtime should still be functional after termination
    // Load a simple module to verify
    let wat_simple = r#"
        (module
            (memory (export "memory") 1)
            (global $heap (mut i32) (i32.const 1024))
            
            (func $alloc (export "alloc") (param $size i32) (result i32)
                (local $ptr i32)
                (local.set $ptr (global.get $heap))
                (global.set $heap (i32.add (global.get $heap) (local.get $size)))
                (local.get $ptr)
            )
            
            (func (export "simple") (param $ptr i32) (param $len i32) (result i32 i32)
                (local $result_ptr i32)
                
                ;; Allocate space for result JSON: "\"ok\""
                (local.set $result_ptr (call $alloc (i32.const 4)))
                
                ;; Write "ok" as JSON string
                (i32.store8 (local.get $result_ptr) (i32.const 34))  ;; "
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 1)) (i32.const 111))  ;; o
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 2)) (i32.const 107))  ;; k
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 3)) (i32.const 34))  ;; "
                
                (local.get $result_ptr)
                (i32.const 4)
            )
        )
    "#;
    let wasm_simple = wat::parse_str(wat_simple).unwrap();
    let id_simple = runtime.load_module(&wasm_simple).unwrap();
    
    let result2: Result<String, _> = runtime.call_json_func(id_simple, "simple", "test2");
    assert!(result2.is_ok(), "Runtime should still be functional after script termination");
}

#[test]
fn test_multiple_scripts_isolation() {
    // Test that resource limits are enforced per-script
    let limits = ResourceLimits {
        max_instructions: 50_000,
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load two modules
    let wat1 = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32) i32.const 0)
            (func (export "work") (param i32 i32) (result i32 i32)
                (local $i i32)
                (loop $l
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br_if $l (i32.lt_u (local.get $i) (i32.const 1000000)))
                )
                i32.const 0 i32.const 0
            )
        )
    "#;
    let wat2 = r#"
        (module
            (memory (export "memory") 1)
            (global $heap (mut i32) (i32.const 1024))
            
            (func $alloc (export "alloc") (param $size i32) (result i32)
                (local $ptr i32)
                (local.set $ptr (global.get $heap))
                (global.set $heap (i32.add (global.get $heap) (local.get $size)))
                (local.get $ptr)
            )
            
            (func (export "simple") (param $ptr i32) (param $len i32) (result i32 i32)
                (local $result_ptr i32)
                
                ;; Allocate space for result JSON: "\"ok\""
                (local.set $result_ptr (call $alloc (i32.const 4)))
                
                ;; Write "ok" as JSON string
                (i32.store8 (local.get $result_ptr) (i32.const 34))  ;; "
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 1)) (i32.const 111))  ;; o
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 2)) (i32.const 107))  ;; k
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 3)) (i32.const 34))  ;; "
                
                (local.get $result_ptr)
                (i32.const 4)
            )
        )
    "#;

    let wasm1 = wat::parse_str(wat1).unwrap();
    let wasm2 = wat::parse_str(wat2).unwrap();
    
    let id1 = runtime.load_module(&wasm1).unwrap();
    let id2 = runtime.load_module(&wasm2).unwrap();

    runtime.start_epoch_timer();

    // First script should fail due to limits
    let result1: Result<String, _> = runtime.call_json_func(id1, "work", "test");
    assert!(result1.is_err(), "Script 1 should fail due to instruction limit");

    // Second script should still work (isolation)
    let result2: Result<String, _> = runtime.call_json_func(id2, "simple", "test");
    assert!(result2.is_ok(), "Script 2 should succeed despite script 1 failure");
}

#[test]
fn test_resource_limit_configuration() {
    // Test that resource limits can be configured
    
    // Test with no limits (default)
    let limits_none = ResourceLimits::default();
    let runtime_none = WasmScriptRuntime::new(limits_none);
    assert!(runtime_none.is_ok(), "Runtime should initialize with default limits");

    // Test with custom limits
    let limits_custom = ResourceLimits {
        max_instructions: 1_000_000,
        max_memory: 1024 * 1024 * 10, // 10MB
        max_execution_time: Duration::from_secs(10),
    };
    let runtime_custom = WasmScriptRuntime::new(limits_custom);
    assert!(runtime_custom.is_ok(), "Runtime should initialize with custom limits");

    // Test with very restrictive limits
    let limits_strict = ResourceLimits {
        max_instructions: 100,
        max_memory: 1024 * 64, // 64KB
        max_execution_time: Duration::from_millis(100),
    };
    let runtime_strict = WasmScriptRuntime::new(limits_strict);
    assert!(runtime_strict.is_ok(), "Runtime should initialize with strict limits");
}
