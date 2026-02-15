use luminara_ai_agent::{IssueSeverity, StaticAnalyzer};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_static_analysis_infinite_loop() {
    let analyzer = StaticAnalyzer::new();
    let code = "
        local x = 0
        while true do
            x = x + 1
        end
    ";
    let issues = analyzer.analyze(code);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.message.contains("infinite loop")));
}

#[quickcheck]
fn test_static_analysis_safe_code(code: String) -> TestResult {
    // If we generate random strings, most should be safe unless they contain specific keywords.
    let analyzer = StaticAnalyzer::new();
    let issues = analyzer.analyze(&code);

    let has_keyword = code.contains("while true")
        || code.contains("os.execute")
        || code.contains("io.open")
        || code.contains("1000000");

    if !has_keyword {
        TestResult::from_bool(issues.is_empty())
    } else {
        TestResult::from_bool(!issues.is_empty())
    }
}
