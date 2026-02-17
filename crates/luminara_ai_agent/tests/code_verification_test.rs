use luminara_ai_agent::{CodeVerificationPipeline, IssueSeverity};

#[test]
fn test_static_analysis_detects_infinite_loop() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "while true do end";
    let result = pipeline.verify(code);

    assert!(!result.passed);
    assert!(!result.static_issues.is_empty());
    assert!(result
        .static_issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error && i.message.contains("infinite loop")));
}

#[test]
fn test_static_analysis_detects_system_calls() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "os.execute('rm -rf /')";
    let result = pipeline.verify(code);

    assert!(!result.passed);
    assert!(!result.static_issues.is_empty());
    assert!(result
        .static_issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error && i.message.contains("system call")));
}

#[test]
fn test_sandbox_execution_success() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "local x = 1 + 1; return x";
    let result = pipeline.verify(code);

    assert!(result.passed);
    assert!(result.static_issues.is_empty() || 
            result.static_issues.iter().all(|i| i.severity != IssueSeverity::Error));
}

#[test]
fn test_fix_suggestions_provided() {
    let mut pipeline = CodeVerificationPipeline::new().unwrap();
    let code = "while true do end";
    let result = pipeline.verify(code);

    assert!(!result.suggestions.is_empty());
    assert!(result.suggestions.iter().any(|s| s.contains("loop") || s.contains("break")));
}
