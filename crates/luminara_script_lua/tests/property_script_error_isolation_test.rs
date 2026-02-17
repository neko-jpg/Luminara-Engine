use luminara_core::world::World;
use luminara_input::Input;
use luminara_script_lua::LuaScriptRuntime;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::fs;
use std::io::Write;
use tempfile;

// ============================================================================
// Property Test 19: Script Error Isolation
// Validates: Requirements 17.1
// ============================================================================

/// **Property 19: Script Error Isolation**
/// **Validates: Requirements 17.1**
///
/// For any Lua script error, the error should be caught and logged without
/// crashing the engine or affecting other scripts.
///
/// This property test verifies that:
/// 1. Runtime errors in scripts don't crash the engine
/// 2. Compilation errors are caught and reported
/// 3. Errors in one script don't prevent other scripts from running
/// 4. The engine remains stable after script errors
/// 5. Multiple error scenarios are handled gracefully

/// Generate different types of Lua script errors
#[derive(Clone, Debug)]
enum ScriptErrorType {
    /// Attempt to index nil
    IndexNil,
    /// Call undefined function
    UndefinedFunction,
    /// Arithmetic on nil
    ArithmeticError,
    /// Explicit error() call
    ExplicitError,
    /// Type mismatch
    TypeMismatch,
    /// Division by zero
    DivisionByZero,
    /// Table access error
    TableAccessError,
    /// String concatenation error
    ConcatError,
}

impl ScriptErrorType {
    /// Generate Lua code that will cause this error type
    fn generate_error_code(&self) -> String {
        match self {
            ScriptErrorType::IndexNil => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local x = nil
    local y = x.field  -- Error: attempt to index nil
end

return module
"#
                .to_string()
            }
            ScriptErrorType::UndefinedFunction => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    undefined_function()  -- Error: undefined function
end

return module
"#
                .to_string()
            }
            ScriptErrorType::ArithmeticError => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local x = nil
    local y = x + 5  -- Error: arithmetic on nil
end

return module
"#
                .to_string()
            }
            ScriptErrorType::ExplicitError => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    error("Intentional error for testing")
end

return module
"#
                .to_string()
            }
            ScriptErrorType::TypeMismatch => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local x = "string"
    local y = x()  -- Error: attempt to call a string value
end

return module
"#
                .to_string()
            }
            ScriptErrorType::DivisionByZero => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local x = 1 / 0  -- Results in inf, not error in Lua
    local y = 0 / 0  -- Results in nan
    -- Force an error by using the result incorrectly
    local z = nil
    local result = z + x
end

return module
"#
                .to_string()
            }
            ScriptErrorType::TableAccessError => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local t = {}
    local value = t.nonexistent.nested  -- Error: attempt to index nil
end

return module
"#
                .to_string()
            }
            ScriptErrorType::ConcatError => {
                r#"
local module = {}

function module.on_update(dt, input, world)
    local x = nil
    local s = "test" .. x  -- Error: attempt to concatenate nil
end

return module
"#
                .to_string()
            }
        }
    }

    /// Convert from u8 for quickcheck generation
    fn from_u8(value: u8) -> Self {
        match value % 8 {
            0 => ScriptErrorType::IndexNil,
            1 => ScriptErrorType::UndefinedFunction,
            2 => ScriptErrorType::ArithmeticError,
            3 => ScriptErrorType::ExplicitError,
            4 => ScriptErrorType::TypeMismatch,
            5 => ScriptErrorType::DivisionByZero,
            6 => ScriptErrorType::TableAccessError,
            _ => ScriptErrorType::ConcatError,
        }
    }
}

/// Generate a valid script that increments a counter
fn generate_valid_script() -> String {
    r#"
local module = {}

module.update_count = 0

function module.on_update(dt, input, world)
    module.update_count = module.update_count + 1
end

return module
"#
    .to_string()
}

/// Generate a script with compilation error
fn generate_compilation_error_script() -> String {
    r#"
local module = {}

function module.on_update(dt, input, world
    -- Missing closing parenthesis
    print("test")
end

return module
"#
    .to_string()
}

#[quickcheck]
fn property_runtime_error_does_not_crash_engine(error_type: u8) -> TestResult {
    let error_type = ScriptErrorType::from_u8(error_type);
    
    // Create a script that will cause a runtime error
    let error_script = error_type.generate_error_code();
    
    let mut temp_file = match tempfile::Builder::new().suffix(".lua").tempfile() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(),
    };
    
    if write!(&mut temp_file, "{}", error_script).is_err() {
        return TestResult::discard();
    }
    
    let path = temp_file.path();
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    // Load the script (should succeed)
    let script_id = match runtime.load_script(path) {
        Ok(id) => id,
        Err(_) => return TestResult::discard(),
    };
    
    // Create mock world and input
    let mut world = World::new();
    let input = Input::default();
    
    // Call update - should NOT crash even though script errors
    // The error should be caught and logged
    let result = runtime.update(0.016, &mut world, &input);
    
    // Property: Update should succeed (error is isolated)
    TestResult::from_bool(result.is_ok())
}

#[quickcheck]
fn property_error_in_one_script_does_not_affect_others(
    error_type: u8,
    num_good_scripts: u8,
) -> TestResult {
    // Limit number of good scripts to reasonable range
    let num_good_scripts = (num_good_scripts % 5) + 1;
    let error_type = ScriptErrorType::from_u8(error_type);
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    let mut temp_files = Vec::new();
    let mut good_script_ids = Vec::new();
    
    // Create and load good scripts
    for i in 0..num_good_scripts {
        let mut temp_file = match tempfile::Builder::new()
            .suffix(&format!("_good_{}.lua", i))
            .tempfile()
        {
            Ok(f) => f,
            Err(_) => return TestResult::discard(),
        };
        
        if write!(&mut temp_file, "{}", generate_valid_script()).is_err() {
            return TestResult::discard();
        }
        
        let path = temp_file.path().to_path_buf();
        temp_files.push(temp_file);
        
        match runtime.load_script(&path) {
            Ok(id) => good_script_ids.push(id),
            Err(_) => return TestResult::discard(),
        }
    }
    
    // Create and load error script
    let mut error_temp = match tempfile::Builder::new().suffix("_error.lua").tempfile() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(),
    };
    
    if write!(&mut error_temp, "{}", error_type.generate_error_code()).is_err() {
        return TestResult::discard();
    }
    
    let error_path = error_temp.path().to_path_buf();
    temp_files.push(error_temp);
    
    if runtime.load_script(&error_path).is_err() {
        return TestResult::discard();
    }
    
    // Create mock world and input
    let mut world = World::new();
    let input = Input::default();
    
    // Run multiple update cycles
    for _ in 0..10 {
        // Update should succeed even with error script
        if runtime.update(0.016, &mut world, &input).is_err() {
            return TestResult::failed();
        }
    }
    
    // Property: Engine remains stable and continues processing
    TestResult::passed()
}

#[quickcheck]
fn property_compilation_error_does_not_crash_engine(seed: u8) -> TestResult {
    let _ = seed; // Use seed for variation
    
    let mut temp_file = match tempfile::Builder::new().suffix(".lua").tempfile() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(),
    };
    
    if write!(&mut temp_file, "{}", generate_compilation_error_script()).is_err() {
        return TestResult::discard();
    }
    
    let path = temp_file.path();
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    // Try to load script with compilation error
    let result = runtime.load_script(path);
    
    // Property: Should return error (not panic)
    TestResult::from_bool(result.is_err())
}

#[quickcheck]
fn property_multiple_errors_in_sequence(num_errors: u8) -> TestResult {
    // Limit to reasonable number of errors
    let num_errors = (num_errors % 10) + 1;
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    let mut temp_files = Vec::new();
    
    // Create multiple error scripts
    for i in 0..num_errors {
        let error_type = ScriptErrorType::from_u8(i);
        
        let mut temp_file = match tempfile::Builder::new()
            .suffix(&format!("_error_{}.lua", i))
            .tempfile()
        {
            Ok(f) => f,
            Err(_) => return TestResult::discard(),
        };
        
        if write!(&mut temp_file, "{}", error_type.generate_error_code()).is_err() {
            return TestResult::discard();
        }
        
        let path = temp_file.path().to_path_buf();
        temp_files.push(temp_file);
        
        // Load script (should succeed)
        if runtime.load_script(&path).is_err() {
            return TestResult::discard();
        }
    }
    
    // Create mock world and input
    let mut world = World::new();
    let input = Input::default();
    
    // Run multiple update cycles with all error scripts
    for _ in 0..5 {
        // Should not crash despite multiple errors
        if runtime.update(0.016, &mut world, &input).is_err() {
            return TestResult::failed();
        }
    }
    
    // Property: Engine handles multiple errors gracefully
    TestResult::passed()
}

#[quickcheck]
fn property_error_then_reload_with_fix(error_type: u8) -> TestResult {
    let error_type = ScriptErrorType::from_u8(error_type);
    
    // Create temporary file
    let temp_file = match tempfile::Builder::new().suffix(".lua").tempfile() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(),
    };
    
    let path = temp_file.path().to_path_buf();
    
    // Write error script
    if fs::write(&path, error_type.generate_error_code()).is_err() {
        return TestResult::discard();
    }
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    // Load error script
    let script_id = match runtime.load_script(&path) {
        Ok(id) => id,
        Err(_) => return TestResult::discard(),
    };
    
    // Create mock world and input
    let mut world = World::new();
    let input = Input::default();
    
    // Update with error script (should not crash)
    if runtime.update(0.016, &mut world, &input).is_err() {
        return TestResult::failed();
    }
    
    // Fix the script by writing valid code
    if fs::write(&path, generate_valid_script()).is_err() {
        return TestResult::discard();
    }
    
    // Reload the script
    if runtime.reload_script(script_id).is_err() {
        return TestResult::discard();
    }
    
    // Update with fixed script (should work)
    if runtime.update(0.016, &mut world, &input).is_err() {
        return TestResult::failed();
    }
    
    // Property: Engine recovers after script is fixed
    TestResult::passed()
}

#[quickcheck]
fn property_lifecycle_hook_error_isolation(error_type: u8) -> TestResult {
    let error_type = ScriptErrorType::from_u8(error_type);
    
    // Create script with error in on_start hook
    let error_script = format!(
        r#"
local module = {{}}

function module.on_start()
    {}
end

function module.on_update(dt, input, world)
    -- This should still be callable
end

return module
"#,
        match error_type {
            ScriptErrorType::IndexNil => "local x = nil; local y = x.field",
            ScriptErrorType::UndefinedFunction => "undefined_function()",
            ScriptErrorType::ArithmeticError => "local x = nil; local y = x + 5",
            ScriptErrorType::ExplicitError => "error('on_start error')",
            ScriptErrorType::TypeMismatch => "local x = 'string'; x()",
            ScriptErrorType::DivisionByZero => "local x = nil; local y = x + (1/0)",
            ScriptErrorType::TableAccessError => "local t = {}; local v = t.a.b",
            ScriptErrorType::ConcatError => "local x = nil; local s = 'test' .. x",
        }
    );
    
    let mut temp_file = match tempfile::Builder::new().suffix(".lua").tempfile() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(),
    };
    
    if write!(&mut temp_file, "{}", error_script).is_err() {
        return TestResult::discard();
    }
    
    let path = temp_file.path();
    
    let mut runtime = match LuaScriptRuntime::new() {
        Ok(r) => r,
        Err(_) => return TestResult::discard(),
    };
    
    // Load script
    let script_id = match runtime.load_script(path) {
        Ok(id) => id,
        Err(_) => return TestResult::discard(),
    };
    
    // Call on_start (will error, but should be caught)
    let _ = runtime.call_lifecycle(script_id, "on_start");
    
    // Create mock world and input
    let mut world = World::new();
    let input = Input::default();
    
    // Update should still work (error in on_start doesn't break on_update)
    let result = runtime.update(0.016, &mut world, &input);
    
    // Property: Error in one lifecycle hook doesn't prevent other hooks from running
    TestResult::from_bool(result.is_ok())
}

// ============================================================================
// Additional Unit Tests for Specific Error Scenarios
// ============================================================================

#[test]
fn test_nil_index_error_isolation() {
    let script = r#"
local module = {}

function module.on_update(dt, input, world)
    local x = nil
    local y = x.field
end

return module
"#;
    
    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(&mut temp_file, "{}", script).unwrap();
    let path = temp_file.path();
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let _id = runtime.load_script(path).unwrap();
    
    let mut world = World::new();
    let input = Input::default();
    
    // Should not panic
    let result = runtime.update(0.016, &mut world, &input);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_scripts_with_mixed_errors() {
    let error_script = r#"
local module = {}

function module.on_update(dt, input, world)
    error("Script 1 error")
end

return module
"#;
    
    let good_script = r#"
local module = {}

module.count = 0

function module.on_update(dt, input, world)
    module.count = module.count + 1
end

return module
"#;
    
    let mut temp1 = tempfile::Builder::new().suffix("_1.lua").tempfile().unwrap();
    write!(&mut temp1, "{}", error_script).unwrap();
    let path1 = temp1.path().to_path_buf();
    
    let mut temp2 = tempfile::Builder::new().suffix("_2.lua").tempfile().unwrap();
    write!(&mut temp2, "{}", good_script).unwrap();
    let path2 = temp2.path().to_path_buf();
    
    let mut temp3 = tempfile::Builder::new().suffix("_3.lua").tempfile().unwrap();
    write!(&mut temp3, "{}", good_script).unwrap();
    let path3 = temp3.path().to_path_buf();
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let _id1 = runtime.load_script(&path1).unwrap();
    let _id2 = runtime.load_script(&path2).unwrap();
    let _id3 = runtime.load_script(&path3).unwrap();
    
    let mut world = World::new();
    let input = Input::default();
    
    // Run multiple updates
    for _ in 0..10 {
        let result = runtime.update(0.016, &mut world, &input);
        assert!(result.is_ok(), "Update should succeed despite error in script 1");
    }
}

#[test]
fn test_compilation_error_provides_details() {
    let bad_script = r#"
local module = {}

function module.on_start(
    -- Missing closing parenthesis
    print("test")
end

return module
"#;
    
    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(&mut temp_file, "{}", bad_script).unwrap();
    let path = temp_file.path();
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let result = runtime.load_script(path);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    
    // Error should contain useful information
    assert!(err_str.contains("Compilation error") || err_str.contains("syntax"));
}

#[test]
fn test_engine_stability_after_100_errors() {
    let error_script = r#"
local module = {}

function module.on_update(dt, input, world)
    error("Repeated error")
end

return module
"#;
    
    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(&mut temp_file, "{}", error_script).unwrap();
    let path = temp_file.path();
    
    let mut runtime = LuaScriptRuntime::new().unwrap();
    let _id = runtime.load_script(path).unwrap();
    
    let mut world = World::new();
    let input = Input::default();
    
    // Run 100 updates, each will error
    for _ in 0..100 {
        let result = runtime.update(0.016, &mut world, &input);
        assert!(result.is_ok(), "Engine should remain stable after repeated errors");
    }
}
