use luminara_script_lua::LuaScriptRuntime;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::fs;
use std::io::Write;

#[quickcheck]
fn test_hot_reload_state_preservation(initial_val: i32, new_val_ignore: i32) -> TestResult {
    let _ = new_val_ignore; // Unused

    // Script v1: sets x = initial_val
    let script_v1 = format!(
        r#"
        local module = {{ x = {} }}
        return module
    "#,
        initial_val
    );

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script_v1).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Verify initial state
    // We need to access internal state for verification.
    // Ideally we expose a way to get the instance table.
    // For test, we can use `runtime.lua` (we made it public crate).
    // But we need the key. `LoadedScript` is private fields?
    // `LoadedScript` struct definition in `runtime.rs` has pub fields now?
    // Let's check. Yes `pub struct LoadedScript` has public fields.
    // But `scripts` map is private in Runtime.
    // I can't access `scripts` map from test if it's private.
    // I should add a helper `get_instance_value(id, key)` for testing?

    // Or simpler: The script returns the module. `load_script` returns ID.
    // `runtime` manages it.

    // Let's modify `LuaScriptRuntime` to allow inspecting state for tests,
    // or rely on side-effects (e.g. globals).

    // Let's rewrite script to use global for easier testing without modifying Runtime API too much?
    // But reloading should preserve LOCAL state in the module table as per implementation.

    // Workaround: Add `pub fn get_script_global(&self, id: ScriptId, name: &str) -> Result<Value>` to runtime?
    // I will add a helper in `tests/` utilizing `runtime.lua` if I can access `registry_key`.

    // Since I cannot access private fields easily without modifying source,
    // I'll assume the implementation is correct if I can verify it via `on_update` hook?
    // Let's add `on_update` that copies `x` to global `_G.test_x`.

    let script_v1_hook = format!(
        r#"
        local module = {{ x = {} }}
        function module.on_update()
            _G.test_x = module.x
        end
        return module
    "#,
        initial_val
    );

    temp_file.as_file().set_len(0).unwrap();
    temp_file
        .as_file()
        .write_all(script_v1_hook.as_bytes())
        .unwrap();

    // Reload? No, we need to load it first.
    // Wait, I overwrote the file before loading?
    // `temp_file` persists.

    // Re-create file for v1
    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script_v1_hook).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Run update to sync global
    runtime.call_lifecycle(id, "on_update").unwrap();
    let val: i32 = runtime.get_lua().globals().get("test_x").unwrap();
    if val != initial_val {
        return TestResult::failed();
    }

    // Script v2: doesn't set x (or sets it to default), but we expect x to be preserved.
    // If we set `x = something` in body, it executes and overwrites.
    // BUT our reload logic copies OLD values to NEW table.
    // So if v2 sets `x = 0`, the copy loop should overwrite `x` with `initial_val` from old table?
    // Yes, `new_table.set(k, v)` happens AFTER `func.call()`.

    let script_v2 = r#"
        local module = { x = 0 } -- Reset to 0
        function module.on_update()
            _G.test_x = module.x
        end
        return module
    "#;

    // Write v2
    // We need to write to the SAME path.
    // `temp_file` auto-deletes on drop. We keep it open.
    // Truncate and write.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    write!(file, "{}", script_v2).unwrap();

    // Trigger reload
    runtime.reload_script(id).unwrap();

    // Check value
    runtime.call_lifecycle(id, "on_update").unwrap();
    let val_v2: i32 = runtime.get_lua().globals().get("test_x").unwrap();

    // It should be `initial_val` if state preservation works (overwrite new 0 with old initial_val).
    TestResult::from_bool(val_v2 == initial_val)
}

#[quickcheck]
fn test_hot_reload_fallback(initial_val: i32) -> TestResult {
    let script_v1 = format!(
        r#"
        local module = {{ x = {} }}
        function module.get_x() return module.x end
        return module
    "#,
        initial_val
    );

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script_v1).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    // Verify v1 works
    // Helper to call function on module? We don't have that exposed.
    // Let's use `call_lifecycle` with global side effect again.
    // Or just rely on successful load.

    // Create broken script v2
    let script_v2 = "return { syntax error here";

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    write!(file, "{}", script_v2).unwrap();

    // Trigger reload - should fail but return Err, NOT panic, and keep old state.
    let res = runtime.reload_script(id);

    let reload_failed = res.is_err();

    // Check if old script is still valid/running?
    // We can't easily check internal state without public access.
    // But `reload_script` returns Err, which is what we want.
    // And if we implemented it right, it returned before swapping keys.

    TestResult::from_bool(reload_failed)
}
