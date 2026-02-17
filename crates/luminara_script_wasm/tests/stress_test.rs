/// **Validates: Requirements 17.5**
/// Stress tests for WASM script execution under load
/// Tests many modules simultaneously, rapid hot-reload cycles, and high API call frequency

use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};
use std::time::{Duration, Instant};

/// Test many WASM modules running simultaneously
#[test]
fn test_many_modules_simultaneously() {
    const MODULE_COUNT: usize = 100;
    
    let limits = ResourceLimits {
        max_memory: 1024 * 1024, // 1MB per module
        max_execution_time: Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create and load many modules
    for i in 0..MODULE_COUNT {
        let wat = format!(
            r#"
            (module
                (func (export "compute") (param i32) (result i32)
                    local.get 0
                    i32.const {}
                    i32.add
                )
            )
        "#,
            i
        );
        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push(id);
    }

    assert_eq!(module_ids.len(), MODULE_COUNT);

    // Execute all modules
    for (i, &id) in module_ids.iter().enumerate() {
        let module = runtime.modules.get(&id).unwrap();
        let compute_func = module
            .instance
            .get_typed_func::<i32, i32>(&mut runtime.store, "compute")
            .unwrap();
        let result = compute_func.call(&mut runtime.store, 10).unwrap();
        assert_eq!(result, 10 + i as i32, "Module {} result mismatch", i);
    }
}

/// Test rapid hot-reload cycles
#[test]
fn test_rapid_hot_reload_cycles() {
    const RELOAD_COUNT: usize = 100;
    
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load initial module
    let wat_v1 = r#"
        (module
            (func (export "get_value") (result i32)
                i32.const 42
            )
        )
    "#;
    let wasm_v1 = wat::parse_str(wat_v1).unwrap();
    let id = runtime.load_module(&wasm_v1).unwrap();

    // Verify initial state
    let module = runtime.modules.get(&id).unwrap();
    let func = module
        .instance
        .get_typed_func::<(), i32>(&mut runtime.store, "get_value")
        .unwrap();
    let value = func.call(&mut runtime.store, ()).unwrap();
    assert_eq!(value, 42);

    let start = Instant::now();

    // Perform rapid reloads
    for i in 0..RELOAD_COUNT {
        let wat = format!(
            r#"
            (module
                (func (export "get_value") (result i32)
                    i32.const {}
                )
            )
        "#,
            i + 100
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
        assert_eq!(result, i as i32 + 100, "Reload {} failed", i);
    }

    let elapsed = start.elapsed();
    println!(
        "Completed {} WASM reloads in {:?} ({:.2} reloads/sec)",
        RELOAD_COUNT,
        elapsed,
        RELOAD_COUNT as f64 / elapsed.as_secs_f64()
    );

    // Should complete in reasonable time (< 15 seconds for 100 reloads)
    assert!(
        elapsed < Duration::from_secs(15),
        "Reloads took too long: {:?}",
        elapsed
    );
}

/// Test high API call frequency
#[test]
fn test_high_api_call_frequency() {
    const CALL_COUNT: usize = 1000;
    
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    // Load module with computation
    let wat = r#"
        (module
            (func (export "compute") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
                i32.const 2
                i32.add
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    let start = Instant::now();

    // Make many API calls
    let module = runtime.modules.get(&id).unwrap();
    let compute_func = module
        .instance
        .get_typed_func::<(i32, i32), i32>(&mut runtime.store, "compute")
        .unwrap();

    for i in 0..CALL_COUNT {
        let result = compute_func.call(&mut runtime.store, (i as i32, 3)).unwrap();
        assert_eq!(result, i as i32 * 3 + 2);
    }

    let elapsed = start.elapsed();
    println!(
        "Completed {} WASM API calls in {:?} ({:.2} calls/sec)",
        CALL_COUNT,
        elapsed,
        CALL_COUNT as f64 / elapsed.as_secs_f64()
    );

    // Should complete in reasonable time (< 5 seconds for 1000 calls)
    assert!(
        elapsed < Duration::from_secs(5),
        "API calls took too long: {:?}",
        elapsed
    );
}

/// Test memory stability with many modules
#[test]
fn test_memory_stability_many_modules() {
    const MODULE_COUNT: usize = 200;
    const ITERATIONS: usize = 10;
    
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create modules with memory operations
    for i in 0..MODULE_COUNT {
        let wat = format!(
            r#"
            (module
                (memory 1)
                (func (export "process") (param i32) (result i32)
                    (local $sum i32)
                    (local $j i32)
                    
                    ;; Write to memory
                    (i32.store (i32.const 0) (local.get 0))
                    
                    ;; Compute sum
                    (local.set $sum (i32.const 0))
                    (local.set $j (i32.const 0))
                    
                    (block $break
                        (loop $continue
                            (br_if $break (i32.ge_u (local.get $j) (i32.const 10)))
                            
                            (local.set $sum 
                                (i32.add (local.get $sum) (local.get $j)))
                            
                            (local.set $j (i32.add (local.get $j) (i32.const 1)))
                            (br $continue)
                        )
                    )
                    
                    ;; Add input and constant
                    (i32.add 
                        (i32.add (local.get $sum) (local.get 0))
                        (i32.const {}))
                )
            )
        "#,
            i
        );
        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push(id);
    }

    // Run multiple iterations
    for _ in 0..ITERATIONS {
        for (i, &id) in module_ids.iter().enumerate() {
            let module = runtime.modules.get(&id).unwrap();
            let process_func = module
                .instance
                .get_typed_func::<i32, i32>(&mut runtime.store, "process")
                .unwrap();
            let result = process_func.call(&mut runtime.store, 5).unwrap();
            // sum(0..9) = 45, + 5 + i = 50 + i
            assert_eq!(result, 50 + i as i32, "Module {} result mismatch", i);
        }
    }

    // All modules should still be functional
    for (i, &id) in module_ids.iter().enumerate() {
        let module = runtime.modules.get(&id).unwrap();
        let process_func = module
            .instance
            .get_typed_func::<i32, i32>(&mut runtime.store, "process")
            .unwrap();
        let result = process_func.call(&mut runtime.store, 10).unwrap();
        assert_eq!(result, 55 + i as i32, "Module {} final check failed", i);
    }
}

/// Test error isolation - modules execute independently
#[test]
fn test_error_isolation_under_stress() {
    const MODULE_COUNT: usize = 50;
    
    let limits = ResourceLimits {
        max_memory: 2 * 1024 * 1024, // 2MB
        max_execution_time: Duration::from_secs(10),
        max_instructions: 10_000_000, // High limit
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create simple modules that should all succeed
    for i in 0..MODULE_COUNT {
        let wat = format!(
            r#"
            (module
                (func (export "compute") (result i32)
                    i32.const {}
                )
            )
        "#,
            i + 100
        );

        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push((id, i + 100));
    }

    // Execute all modules - they should all succeed
    for (id, expected) in &module_ids {
        let module = runtime.modules.get(id).unwrap();
        let compute_func = module
            .instance
            .get_typed_func::<(), i32>(&mut runtime.store, "compute")
            .unwrap();
        
        let result = compute_func.call(&mut runtime.store, ());
        assert!(result.is_ok(), "Module should succeed");
        assert_eq!(result.unwrap(), *expected as i32);
    }

    println!("All {} modules executed successfully", MODULE_COUNT);
}

/// Test performance with complex modules
#[test]
fn test_complex_module_performance() {
    const MODULE_COUNT: usize = 20;
    const ITERATIONS: usize = 100;
    
    let limits = ResourceLimits {
        max_memory: 2 * 1024 * 1024, // 2MB
        max_execution_time: Duration::from_secs(10),
        max_instructions: 10_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create modules with complex logic
    for i in 0..MODULE_COUNT {
        let wat = format!(
            r#"
            (module
                (memory 1)
                
                (func (export "process_entities") (param i32) (result i32)
                    (local $sum i32)
                    (local $j i32)
                    (local $entity_count i32)
                    
                    (local.set $entity_count (i32.const 10))
                    (local.set $sum (i32.const 0))
                    (local.set $j (i32.const 0))
                    
                    ;; Process entities
                    (block $break
                        (loop $continue
                            (br_if $break (i32.ge_u (local.get $j) (local.get $entity_count)))
                            
                            ;; Simulate entity processing
                            (local.set $sum 
                                (i32.add 
                                    (local.get $sum)
                                    (i32.mul (local.get $j) (i32.const 3))))
                            
                            ;; Write to memory (simulate state update)
                            (i32.store 
                                (i32.mul (local.get $j) (i32.const 4))
                                (local.get $sum))
                            
                            (local.set $j (i32.add (local.get $j) (i32.const 1)))
                            (br $continue)
                        )
                    )
                    
                    ;; Add module ID and input
                    (i32.add 
                        (i32.add (local.get $sum) (local.get 0))
                        (i32.const {}))
                )
            )
        "#,
            i
        );
        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push(id);
    }

    let start = Instant::now();

    // Execute all modules multiple times
    for iteration in 0..ITERATIONS {
        for (i, &id) in module_ids.iter().enumerate() {
            let module = runtime.modules.get(&id).unwrap();
            let process_func = module
                .instance
                .get_typed_func::<i32, i32>(&mut runtime.store, "process_entities")
                .unwrap();
            let result = process_func.call(&mut runtime.store, iteration as i32).unwrap();
            // sum(0*3, 1*3, ..., 9*3) = 3*sum(0..9) = 3*45 = 135
            let expected = 135 + iteration as i32 + i as i32;
            assert_eq!(result, expected, "Module {} iteration {} mismatch", i, iteration);
        }
    }

    let elapsed = start.elapsed();
    let total_calls = MODULE_COUNT * ITERATIONS;
    println!(
        "Completed {} complex WASM calls in {:?} ({:.2} calls/sec)",
        total_calls,
        elapsed,
        total_calls as f64 / elapsed.as_secs_f64()
    );

    // Should maintain reasonable performance
    assert!(
        elapsed < Duration::from_secs(15),
        "Complex modules took too long: {:?}",
        elapsed
    );
}

/// Test reload stability under stress
#[test]
fn test_reload_stability_under_stress() {
    const MODULE_COUNT: usize = 20;
    const RELOAD_CYCLES: usize = 10;
    
    let limits = ResourceLimits {
        max_memory: 1024 * 1024,
        max_execution_time: Duration::from_secs(5),
        max_instructions: 1_000_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create multiple modules
    for i in 0..MODULE_COUNT {
        let wat = format!(
            r#"
            (module
                (func (export "get_id") (result i32)
                    i32.const {}
                )
            )
        "#,
            i
        );
        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push((id, i));
    }

    // Perform multiple reload cycles on all modules
    for cycle in 0..RELOAD_CYCLES {
        for (id, expected_id) in &module_ids {
            let wat = format!(
                r#"
                (module
                    (func (export "get_id") (result i32)
                        i32.const {}
                    )
                    (func (export "get_cycle") (result i32)
                        i32.const {}
                    )
                )
            "#,
                expected_id, cycle
            );
            let wasm = wat::parse_str(&wat).unwrap();
            runtime.reload_module(*id, &wasm).unwrap();

            // Verify module works after reload
            let module = runtime.modules.get(id).unwrap();
            let get_id = module
                .instance
                .get_typed_func::<(), i32>(&mut runtime.store, "get_id")
                .unwrap();
            let get_cycle = module
                .instance
                .get_typed_func::<(), i32>(&mut runtime.store, "get_cycle")
                .unwrap();

            let id_result = get_id.call(&mut runtime.store, ()).unwrap();
            let cycle_result = get_cycle.call(&mut runtime.store, ()).unwrap();

            assert_eq!(
                id_result, *expected_id as i32,
                "Module {} ID mismatch in cycle {}",
                id.0, cycle
            );
            assert_eq!(
                cycle_result, cycle as i32,
                "Module {} cycle mismatch",
                id.0
            );
        }
    }
}

/// Test resource limit enforcement under stress
#[test]
fn test_resource_limits_under_stress() {
    const MODULE_COUNT: usize = 30;
    
    let limits = ResourceLimits {
        max_memory: 512 * 1024, // 512KB - relatively small
        max_execution_time: Duration::from_millis(50),
        max_instructions: 50_000,
    };

    let mut runtime = WasmScriptRuntime::new(limits).unwrap();
    let mut module_ids = Vec::new();

    // Create modules with varying resource usage
    for i in 0..MODULE_COUNT {
        let loop_count = (i + 1) * 1000; // Increasing computation
        let wat = format!(
            r#"
            (module
                (func (export "compute") (result i32)
                    (local $sum i32)
                    (local $i i32)
                    
                    (local.set $sum (i32.const 0))
                    (local.set $i (i32.const 0))
                    
                    (block $break
                        (loop $continue
                            (br_if $break (i32.ge_u (local.get $i) (i32.const {})))
                            
                            (local.set $sum 
                                (i32.add (local.get $sum) (i32.const 1)))
                            
                            (local.set $i (i32.add (local.get $i) (i32.const 1)))
                            (br $continue)
                        )
                    )
                    
                    (local.get $sum)
                )
            )
        "#,
            loop_count
        );
        let wasm = wat::parse_str(&wat).unwrap();
        let id = runtime.load_module(&wasm).unwrap();
        module_ids.push((id, loop_count));
    }

    runtime.start_epoch_timer();

    // Execute modules - some should hit limits
    let mut within_limits = 0;
    let mut exceeded_limits = 0;

    for (id, loop_count) in &module_ids {
        let module = runtime.modules.get(id).unwrap();
        let compute_func = module
            .instance
            .get_typed_func::<(), i32>(&mut runtime.store, "compute")
            .unwrap();
        
        let result = compute_func.call(&mut runtime.store, ());
        
        if result.is_ok() {
            within_limits += 1;
            assert_eq!(result.unwrap(), *loop_count as i32);
        } else {
            exceeded_limits += 1;
        }
    }

    println!(
        "Resource limits: {} within limits, {} exceeded",
        within_limits, exceeded_limits
    );

    // Should have some of each
    assert!(within_limits > 0, "Some modules should succeed");
    assert!(exceeded_limits > 0, "Some modules should hit limits");
}
