use luminara_ai_agent::{CodeVerificationPipeline, IssueSeverity};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_verification_pipeline_static_fail() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "os.execute('rm -rf /')";
    let result = pipeline.verify(code);

    assert!(!result.passed);
    assert!(!result.static_issues.is_empty());
    assert!(result
        .static_issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error));
    // Sandbox should NOT run if static check failed with error
    assert!(result.sandbox_result.is_none());
}

#[test]
fn test_verification_pipeline_runtime_fail() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "error('runtime failure')";
    let result = pipeline.verify(code);

    assert!(!result.passed);
    assert!(result.static_issues.is_empty());
    assert!(result.sandbox_result.is_some());
    assert!(result.sandbox_result.unwrap().is_err());
}

#[test]
fn test_verification_pipeline_success() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "local x = 1 + 1; return x";
    let result = pipeline.verify(code);

    assert!(result.passed);
    assert!(result.static_issues.is_empty());
    assert!(result.sandbox_result.unwrap().is_ok());
}
