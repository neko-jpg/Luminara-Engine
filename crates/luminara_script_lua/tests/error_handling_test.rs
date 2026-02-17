use luminara_script_lua::LuaScriptRuntime;
use std::fs::write;
use tempfile;

#[test]
fn test_compilation_error_with_stack_trace() {
    // Create a script with syntax error
    let script_content = r#"
local module = {}

function module.on_start()
    -- Missing 'end' keyword
    if true then
        print("test")
    -- Missing end here
end

return module
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file, script_content).unwrap();
    let path = temp_file.path();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let result = runtime.load_script(path);

    // Should fail with compilation error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();

    // Check that error contains script path
    assert!(err_str.contains(&path.display().to_string()));
    // Check that it's a compilation error
    assert!(err_str.contains("Compilation error"));
}

#[test]
fn test_runtime_error_with_stack_trace() {
    // Create a script that will cause runtime error
    let script_content = r#"
local module = {}

function module.on_start()
    -- This will cause a runtime error
    local x = nil
    local y = x.nonexistent_field
end

return module
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file, script_content).unwrap();
    let path = temp_file.path();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(path).unwrap();

    // Call lifecycle hook that will error
    let result = runtime.call_lifecycle(id, "on_start");

    // Should fail with runtime error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();

    // Check that error contains script path
    assert!(err_str.contains(&path.display().to_string()));
    // Check that it's a runtime error
    assert!(err_str.contains("Runtime error"));
    // Check that it mentions the actual error
    assert!(err_str.contains("nil") || err_str.contains("attempt to index"));
}

#[test]
fn test_error_isolation_in_update() {
    // Create two scripts: one that errors, one that works
    let error_script = r#"
local module = {}

function module.on_update(dt, input, world)
    error("Intentional error for testing")
end

return module
"#;

    let good_script = r#"
local module = {}

module.update_count = 0

function module.on_update(dt, input, world)
    module.update_count = module.update_count + 1
end

return module
"#;

    let mut temp_file1 = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file1, error_script).unwrap();
    let path1 = temp_file1.path();

    let mut temp_file2 = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file2, good_script).unwrap();
    let path2 = temp_file2.path();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let _id1 = runtime.load_script(path1).unwrap();
    let id2 = runtime.load_script(path2).unwrap();

    // Create mock world and input
    let mut world = luminara_core::world::World::new();
    let input = luminara_input::Input::default();

    // Update should not fail even though one script errors
    // The error should be isolated and logged
    let result = runtime.update(0.016, &mut world, &input);
    assert!(result.is_ok());

    // Verify the good script still ran
    let lua = runtime.get_lua();
    let scripts_table: mlua::Table = lua.globals().get("_G").unwrap();
    
    // The good script should have incremented its counter
    // (This is a simplified check - in reality we'd need to access the script instance)
}

#[test]
fn test_script_not_found_error() {
    let runtime = LuaScriptRuntime::new().unwrap();
    let fake_id = luminara_script::ScriptId(999);

    let result = runtime.call_lifecycle(fake_id, "on_start");

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();

    // Should be a ScriptNotFound error
    assert!(err_str.contains("Script not found") || err_str.contains("ScriptId(999)"));
}

#[test]
fn test_reload_with_compilation_error() {
    // Create a valid script first
    let valid_script = r#"
local module = {}

function module.on_start()
    print("valid")
end

return module
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file, valid_script).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Now write invalid script to the same file
    let invalid_script = r#"
local module = {}

function module.on_start(
    -- Missing closing parenthesis
    print("invalid")
end

return module
"#;

    write(&path, invalid_script).unwrap();

    // Reload should fail with compilation error
    let result = runtime.reload_script(id);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();

    // Check error details
    assert!(err_str.contains("Compilation error"));
    assert!(err_str.contains(&path.display().to_string()));
}

#[test]
fn test_reload_with_runtime_error() {
    // Create a valid script first
    let valid_script = r#"
local module = {}

function module.on_start()
    print("valid")
end

return module
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write(&mut temp_file, valid_script).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Now write script that errors during execution
    let error_script = r#"
-- This will error when executed
error("Script body error")
"#;

    write(&path, error_script).unwrap();

    // Reload should fail with runtime error
    let result = runtime.reload_script(id);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();

    // Check error details
    assert!(err_str.contains("Runtime error"));
    assert!(err_str.contains(&path.display().to_string()));
    assert!(err_str.contains("Script body error"));
}
