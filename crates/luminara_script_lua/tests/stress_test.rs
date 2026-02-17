/// **Validates: Requirements 17.5**
/// Stress tests for Lua script execution under load
/// Tests many scripts simultaneously, rapid hot-reload cycles, and high API call frequency

use luminara_script_lua::LuaScriptRuntime;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

/// Test many scripts running simultaneously
#[test]
fn test_many_scripts_simultaneously() {
    const SCRIPT_COUNT: usize = 100;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_ids = Vec::new();
    let mut temp_files = Vec::new();

    // Create and load many scripts
    for i in 0..SCRIPT_COUNT {
        let script = format!(
            r#"
            local module = {{ 
                id = {},
                counter = 0
            }}
            
            function module.on_update()
                module.counter = module.counter + 1
                _G["script_{}_counter"] = module.counter
            end
            
            return module
        "#,
            i, i
        );

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_ids.push(id);
        temp_files.push(temp_file);
    }

    assert_eq!(script_ids.len(), SCRIPT_COUNT);

    // Execute all scripts multiple times
    for _ in 0..10 {
        for &id in &script_ids {
            runtime.call_lifecycle(id, "on_update").unwrap();
        }
    }

    // Verify all scripts executed correctly
    for i in 0..SCRIPT_COUNT {
        let counter: i32 = runtime
            .get_lua()
            .globals()
            .get(format!("script_{}_counter", i))
            .unwrap();
        assert_eq!(counter, 10, "Script {} counter mismatch", i);
    }
}

/// Test rapid hot-reload cycles
#[test]
fn test_rapid_hot_reload_cycles() {
    const RELOAD_COUNT: usize = 100;
    
    let script_v1 = r#"
        local module = { 
            value = 42,
            reload_count = 0
        }
        
        function module.on_update()
            _G.test_value = module.value
            _G.test_reload_count = module.reload_count
        end
        
        return module
    "#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script_v1).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Verify initial state
    runtime.call_lifecycle(id, "on_update").unwrap();
    let value: i32 = runtime.get_lua().globals().get("test_value").unwrap();
    assert_eq!(value, 42);

    let start = Instant::now();

    // Perform rapid reloads
    for i in 0..RELOAD_COUNT {
        let script_reload = format!(
            r#"
            local module = {{ 
                value = 0,
                reload_count = {}
            }}
            
            function module.on_update()
                _G.test_value = module.value
                _G.test_reload_count = module.reload_count
            end
            
            return module
        "#,
            i + 1
        );

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        write!(file, "{}", script_reload).unwrap();
        drop(file);

        runtime.reload_script(id).unwrap();

        // Verify state preserved (value should still be 42)
        runtime.call_lifecycle(id, "on_update").unwrap();
        let value_after: i32 = runtime.get_lua().globals().get("test_value").unwrap();
        assert_eq!(value_after, 42, "Value not preserved after reload {}", i);
    }

    let elapsed = start.elapsed();
    println!(
        "Completed {} reloads in {:?} ({:.2} reloads/sec)",
        RELOAD_COUNT,
        elapsed,
        RELOAD_COUNT as f64 / elapsed.as_secs_f64()
    );

    // Should complete in reasonable time (< 10 seconds for 100 reloads)
    assert!(
        elapsed < Duration::from_secs(10),
        "Reloads took too long: {:?}",
        elapsed
    );
}

/// Test high API call frequency
#[test]
fn test_high_api_call_frequency() {
    const CALL_COUNT: usize = 1000;
    
    let script = r#"
        local module = { 
            total_calls = 0
        }
        
        function module.on_update()
            module.total_calls = module.total_calls + 1
            _G.test_total_calls = module.total_calls
        end
        
        function module.get_value()
            return 42
        end
        
        return module
    "#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    let start = Instant::now();

    // Make many API calls
    for _ in 0..CALL_COUNT {
        runtime.call_lifecycle(id, "on_update").unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "Completed {} API calls in {:?} ({:.2} calls/sec)",
        CALL_COUNT,
        elapsed,
        CALL_COUNT as f64 / elapsed.as_secs_f64()
    );

    // Verify all calls executed
    let total_calls: i32 = runtime.get_lua().globals().get("test_total_calls").unwrap();
    assert_eq!(total_calls, CALL_COUNT as i32);

    // Should complete in reasonable time (< 5 seconds for 1000 calls)
    assert!(
        elapsed < Duration::from_secs(5),
        "API calls took too long: {:?}",
        elapsed
    );
}

/// Test memory stability with many scripts
#[test]
fn test_memory_stability_many_scripts() {
    const SCRIPT_COUNT: usize = 200;
    const ITERATIONS: usize = 10;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_ids = Vec::new();
    let mut temp_files = Vec::new();

    // Create scripts with various data structures
    for i in 0..SCRIPT_COUNT {
        let script = format!(
            r#"
            local module = {{ 
                id = {},
                data = {{
                    numbers = {{ 1, 2, 3, 4, 5 }},
                    strings = {{ "a", "b", "c" }},
                    nested = {{
                        x = 1.0,
                        y = 2.0,
                        z = 3.0
                    }}
                }},
                counter = 0
            }}
            
            function module.on_update()
                module.counter = module.counter + 1
                -- Create some temporary data
                local temp = {{}}
                for j = 1, 10 do
                    temp[j] = j * module.counter
                end
            end
            
            return module
        "#,
            i
        );

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_ids.push(id);
        temp_files.push(temp_file);
    }

    // Run multiple iterations
    for iteration in 0..ITERATIONS {
        for &id in &script_ids {
            runtime.call_lifecycle(id, "on_update").unwrap();
        }
        
        // Force garbage collection periodically
        if iteration % 3 == 0 {
            runtime.get_lua().gc_collect().unwrap();
        }
    }

    // All scripts should still be functional
    for &id in &script_ids {
        runtime.call_lifecycle(id, "on_update").unwrap();
    }
}

/// Test concurrent script execution with shared state
#[test]
fn test_concurrent_script_execution() {
    const SCRIPT_COUNT: usize = 50;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_ids = Vec::new();
    let mut temp_files = Vec::new();

    // Initialize shared counter
    runtime.get_lua().globals().set("shared_counter", 0).unwrap();

    // Create scripts that modify shared state
    for i in 0..SCRIPT_COUNT {
        let script = format!(
            r#"
            local module = {{ id = {} }}
            
            function module.on_update()
                local current = _G.shared_counter or 0
                _G.shared_counter = current + 1
            end
            
            return module
        "#,
            i
        );

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_ids.push(id);
        temp_files.push(temp_file);
    }

    // Execute all scripts
    for &id in &script_ids {
        runtime.call_lifecycle(id, "on_update").unwrap();
    }

    // Verify shared counter
    let counter: i32 = runtime.get_lua().globals().get("shared_counter").unwrap();
    assert_eq!(counter, SCRIPT_COUNT as i32);
}

/// Test error isolation - one script error shouldn't crash others
#[test]
fn test_error_isolation_under_stress() {
    const SCRIPT_COUNT: usize = 50;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_ids = Vec::new();
    let mut temp_files = Vec::new();

    // Create mix of good and bad scripts
    for i in 0..SCRIPT_COUNT {
        let script = if i % 5 == 0 {
            // Every 5th script has an error
            format!(
                r#"
                local module = {{ id = {} }}
                
                function module.on_update()
                    error("Intentional error in script {}")
                end
                
                return module
            "#,
                i, i
            )
        } else {
            // Normal script
            format!(
                r#"
                local module = {{ id = {}, counter = 0 }}
                
                function module.on_update()
                    module.counter = module.counter + 1
                    _G["script_{}_counter"] = module.counter
                end
                
                return module
            "#,
                i, i
            )
        };

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_ids.push(id);
        temp_files.push(temp_file);
    }

    // Execute all scripts - errors should be isolated
    for (i, &id) in script_ids.iter().enumerate() {
        let result = runtime.call_lifecycle(id, "on_update");
        
        if i % 5 == 0 {
            // Error scripts should fail
            assert!(result.is_err(), "Script {} should have errored", i);
        } else {
            // Good scripts should succeed
            assert!(result.is_ok(), "Script {} should have succeeded", i);
        }
    }

    // Verify good scripts still work
    for i in 0..SCRIPT_COUNT {
        if i % 5 != 0 {
            let counter: i32 = runtime
                .get_lua()
                .globals()
                .get(format!("script_{}_counter", i))
                .unwrap();
            assert_eq!(counter, 1, "Script {} counter mismatch", i);
        }
    }
}

/// Test performance with complex scripts
#[test]
fn test_complex_script_performance() {
    const SCRIPT_COUNT: usize = 20;
    const ITERATIONS: usize = 100;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_ids = Vec::new();
    let mut temp_files = Vec::new();

    // Create scripts with complex logic
    for i in 0..SCRIPT_COUNT {
        let script = format!(
            r#"
            local module = {{ 
                id = {},
                entities = {{}},
                frame_count = 0
            }}
            
            function module.on_update()
                module.frame_count = module.frame_count + 1
                
                -- Simulate entity processing
                for j = 1, 10 do
                    local entity = {{
                        position = {{ x = j * 1.0, y = j * 2.0, z = j * 3.0 }},
                        velocity = {{ x = 0.1, y = 0.2, z = 0.3 }},
                        health = 100
                    }}
                    
                    -- Update position
                    entity.position.x = entity.position.x + entity.velocity.x
                    entity.position.y = entity.position.y + entity.velocity.y
                    entity.position.z = entity.position.z + entity.velocity.z
                    
                    module.entities[j] = entity
                end
                
                _G["script_{}_frame"] = module.frame_count
            end
            
            return module
        "#,
            i, i
        );

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_ids.push(id);
        temp_files.push(temp_file);
    }

    let start = Instant::now();

    // Execute all scripts multiple times
    for _ in 0..ITERATIONS {
        for &id in &script_ids {
            runtime.call_lifecycle(id, "on_update").unwrap();
        }
    }

    let elapsed = start.elapsed();
    let total_calls = SCRIPT_COUNT * ITERATIONS;
    println!(
        "Completed {} complex script calls in {:?} ({:.2} calls/sec)",
        total_calls,
        elapsed,
        total_calls as f64 / elapsed.as_secs_f64()
    );

    // Verify all scripts executed correctly
    for i in 0..SCRIPT_COUNT {
        let frame: i32 = runtime
            .get_lua()
            .globals()
            .get(format!("script_{}_frame", i))
            .unwrap();
        assert_eq!(frame, ITERATIONS as i32, "Script {} frame mismatch", i);
    }

    // Should maintain reasonable performance
    assert!(
        elapsed < Duration::from_secs(10),
        "Complex scripts took too long: {:?}",
        elapsed
    );
}

/// Test reload stability under stress
#[test]
fn test_reload_stability_under_stress() {
    const SCRIPT_COUNT: usize = 20;
    const RELOAD_CYCLES: usize = 10;
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let mut script_data = Vec::new();

    // Create multiple scripts
    for i in 0..SCRIPT_COUNT {
        let script = format!(
            r#"
            local module = {{ 
                id = {},
                value = {}
            }}
            
            function module.on_update()
                _G["script_{}_value"] = module.value
            end
            
            return module
        "#,
            i, i * 10, i
        );

        let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
        write!(temp_file, "{}", script).unwrap();
        let path = temp_file.path().to_path_buf();

        let id = runtime.load_script(&path).unwrap();
        script_data.push((id, temp_file, i * 10));
    }

    // Perform multiple reload cycles on all scripts
    for cycle in 0..RELOAD_CYCLES {
        for (id, temp_file, expected_value) in &script_data {
            let script_reload = format!(
                r#"
                local module = {{ 
                    id = {},
                    value = 0
                }}
                
                function module.on_update()
                    _G["script_{}_value"] = module.value
                end
                
                return module
            "#,
                cycle, id.0
            );

            let path = temp_file.path();
            let mut file = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap();
            write!(file, "{}", script_reload).unwrap();
            drop(file);

            runtime.reload_script(*id).unwrap();

            // Verify value preserved
            runtime.call_lifecycle(*id, "on_update").unwrap();
            let value: i32 = runtime
                .get_lua()
                .globals()
                .get(format!("script_{}_value", id.0))
                .unwrap();
            assert_eq!(
                value, *expected_value as i32,
                "Script {} value not preserved in cycle {}",
                id.0, cycle
            );
        }
    }
}
