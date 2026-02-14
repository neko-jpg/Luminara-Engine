use luminara_script_lua::LuaScriptRuntime;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::io::Write;

#[quickcheck]
fn test_lifecycle_hook_invocation(has_start: bool, has_update: bool) -> TestResult {
    let mut script_content = String::from("local module = {}\n");
    if has_start {
        script_content.push_str("function module.on_start() _G.start_check = true end\n");
    }
    if has_update {
        script_content.push_str("function module.on_update() _G.update_check = true end\n");
    }
    script_content.push_str("return module\n");

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(temp_file, "{}", script_content).unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();
    let id = runtime.load_script(&path).unwrap();

    let lua = runtime.get_lua();
    // Reset globals
    let _ = lua.globals().set("start_check", false);
    let _ = lua.globals().set("update_check", false);

    // Call hooks
    let _ = runtime.call_lifecycle(id, "on_start");
    let _ = runtime.call_lifecycle(id, "on_update");

    // Verify
    let start_check: bool = lua.globals().get("start_check").unwrap_or(false);
    let update_check: bool = lua.globals().get("update_check").unwrap_or(false);

    let start_ok = start_check == has_start;
    let update_ok = update_check == has_update;

    TestResult::from_bool(start_ok && update_ok)
}
