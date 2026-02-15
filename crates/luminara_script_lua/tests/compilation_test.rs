use luminara_script_lua::LuaScriptRuntime;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::io::Write;

#[quickcheck]
fn test_script_compilation_caching(content: String) -> TestResult {
    // Avoid empty content or content that might be invalid Lua syntax if we cared about execution,
    // but here we just care about loading.
    if content.contains('\0') {
        return TestResult::discard();
    }

    let mut temp_file = tempfile::Builder::new().suffix(".lua").tempfile().unwrap();
    write!(
        temp_file,
        "local x = \"{}\"; return x",
        content.escape_default()
    )
    .unwrap();
    let path = temp_file.path().to_path_buf();

    let mut runtime = LuaScriptRuntime::new().unwrap();

    let id1 = runtime.load_script(&path);
    let id2 = runtime.load_script(&path);

    match (id1, id2) {
        (Ok(id1), Ok(id2)) => TestResult::from_bool(id1 == id2),
        _ => TestResult::failed(),
    }
}
