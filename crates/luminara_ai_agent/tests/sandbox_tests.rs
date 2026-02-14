use luminara_ai_agent::{ScriptSandbox, SandboxConfig};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_sandbox_fs_restriction() {
    let config = SandboxConfig {
        allow_filesystem: false,
        ..Default::default()
    };

    let sandbox = ScriptSandbox::new(config).unwrap();

    // Try to access io module
    let result = sandbox.run_lua("return io.open('test.txt')");
    // Should fail because io is nil
    assert!(result.is_err());

    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("attempt to index a nil value") || err_msg.contains("global 'io' is nil"));
}

#[quickcheck]
fn test_sandbox_resource_limits_config(max_mem: usize) -> TestResult {
    // Just verify configuration is accepted
    let config = SandboxConfig {
        max_memory: max_mem,
        ..Default::default()
    };
    let sandbox = ScriptSandbox::new(config);
    TestResult::from_bool(sandbox.is_ok())
}
