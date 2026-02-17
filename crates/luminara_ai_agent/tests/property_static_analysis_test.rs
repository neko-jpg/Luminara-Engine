// **Property 24: Static Analysis Issue Detection**
// **Validates: Requirements 25.1**
//
// This property test verifies that the static analyzer correctly detects:
// - Infinite loops (while true without break, repeat until false)
// - Undefined variables
// - Type errors (nil arithmetic, nil concatenation)
// - Dangerous system calls (os.execute, io.open, etc.)
//
// The test generates random Lua code snippets with known issues and verifies
// that the static analyzer detects all dangerous patterns without false negatives.

use luminara_ai_agent::{IssueSeverity, StaticAnalyzer};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

/// Generator for Lua code snippets with known issues
#[derive(Clone, Debug)]
enum LuaCodeWithIssue {
    InfiniteWhileLoop,
    InfiniteRepeatLoop,
    SystemCall(SystemCallType),
    UndefinedVariable(String),
    NilArithmetic,
    NilConcatenation,
    LargeLoop(usize),
    ValidCode,
}

#[derive(Clone, Debug)]
enum SystemCallType {
    OsExecute,
    IoOpen,
    IoPopen,
    LoadFile,
    DoFile,
}

impl Arbitrary for LuaCodeWithIssue {
    fn arbitrary(g: &mut Gen) -> Self {
        let choice = u8::arbitrary(g) % 9;
        match choice {
            0 => LuaCodeWithIssue::InfiniteWhileLoop,
            1 => LuaCodeWithIssue::InfiniteRepeatLoop,
            2 => {
                let call_type = match u8::arbitrary(g) % 5 {
                    0 => SystemCallType::OsExecute,
                    1 => SystemCallType::IoOpen,
                    2 => SystemCallType::IoPopen,
                    3 => SystemCallType::LoadFile,
                    _ => SystemCallType::DoFile,
                };
                LuaCodeWithIssue::SystemCall(call_type)
            }
            3 => {
                let var_name = format!("undefined_var_{}", u32::arbitrary(g) % 1000);
                LuaCodeWithIssue::UndefinedVariable(var_name)
            }
            4 => LuaCodeWithIssue::NilArithmetic,
            5 => LuaCodeWithIssue::NilConcatenation,
            6 => {
                let count = (u32::arbitrary(g) % 50000) as usize + 10001;
                LuaCodeWithIssue::LargeLoop(count)
            }
            7 => LuaCodeWithIssue::ValidCode,
            _ => LuaCodeWithIssue::ValidCode,
        }
    }
}

impl LuaCodeWithIssue {
    fn to_code(&self) -> String {
        match self {
            LuaCodeWithIssue::InfiniteWhileLoop => {
                "while true do\n  local x = 1\nend".to_string()
            }
            LuaCodeWithIssue::InfiniteRepeatLoop => {
                "repeat\n  local x = 1\nuntil false".to_string()
            }
            LuaCodeWithIssue::SystemCall(call_type) => match call_type {
                SystemCallType::OsExecute => "os.execute('ls')".to_string(),
                SystemCallType::IoOpen => "io.open('file.txt', 'r')".to_string(),
                SystemCallType::IoPopen => "io.popen('cat file.txt')".to_string(),
                SystemCallType::LoadFile => "loadfile('script.lua')".to_string(),
                SystemCallType::DoFile => "dofile('script.lua')".to_string(),
            },
            LuaCodeWithIssue::UndefinedVariable(var) => {
                format!("local x = {}", var)
            }
            LuaCodeWithIssue::NilArithmetic => "local x = nil + 5".to_string(),
            LuaCodeWithIssue::NilConcatenation => "local x = nil .. 'text'".to_string(),
            LuaCodeWithIssue::LargeLoop(count) => {
                format!("for i = 1, {} do\n  local x = i\nend", count)
            }
            LuaCodeWithIssue::ValidCode => {
                "local x = 1 + 1\nlocal y = x * 2\nreturn y".to_string()
            }
        }
    }

    fn expected_issue_type(&self) -> Option<ExpectedIssue> {
        match self {
            LuaCodeWithIssue::InfiniteWhileLoop => Some(ExpectedIssue::InfiniteLoop),
            LuaCodeWithIssue::InfiniteRepeatLoop => Some(ExpectedIssue::InfiniteLoop),
            LuaCodeWithIssue::SystemCall(_) => Some(ExpectedIssue::SystemCall),
            LuaCodeWithIssue::UndefinedVariable(_) => Some(ExpectedIssue::UndefinedVariable),
            LuaCodeWithIssue::NilArithmetic => Some(ExpectedIssue::TypeError),
            LuaCodeWithIssue::NilConcatenation => Some(ExpectedIssue::TypeError),
            LuaCodeWithIssue::LargeLoop(_) => Some(ExpectedIssue::LargeLoop),
            LuaCodeWithIssue::ValidCode => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ExpectedIssue {
    InfiniteLoop,
    SystemCall,
    UndefinedVariable,
    TypeError,
    LargeLoop,
}

/// **Property 24: Static Analysis Issue Detection**
/// **Validates: Requirements 25.1**
///
/// For any generated Lua code with a known issue, the static analyzer
/// MUST detect the issue and report it with appropriate severity.
#[quickcheck]
fn prop_static_analysis_detects_infinite_loops(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(ExpectedIssue::InfiniteLoop) => {
            // Must detect infinite loop
            let has_infinite_loop_error = issues.iter().any(|issue| {
                issue.severity == IssueSeverity::Error
                    && issue.message.to_lowercase().contains("infinite loop")
            });

            TestResult::from_bool(has_infinite_loop_error)
        }
        Some(_) => TestResult::discard(), // Not testing infinite loops in this property
        None => TestResult::discard(),    // Valid code, skip
    }
}

/// **Property 24: Static Analysis Issue Detection - System Calls**
/// **Validates: Requirements 25.1**
#[quickcheck]
fn prop_static_analysis_detects_system_calls(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(ExpectedIssue::SystemCall) => {
            // Must detect dangerous system call
            let has_system_call_error = issues.iter().any(|issue| {
                issue.severity == IssueSeverity::Error
                    && (issue.message.to_lowercase().contains("system call")
                        || issue.message.to_lowercase().contains("dangerous"))
            });

            TestResult::from_bool(has_system_call_error)
        }
        Some(_) => TestResult::discard(),
        None => TestResult::discard(),
    }
}

/// **Property 24: Static Analysis Issue Detection - Undefined Variables**
/// **Validates: Requirements 25.1**
#[quickcheck]
fn prop_static_analysis_detects_undefined_variables(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(ExpectedIssue::UndefinedVariable) => {
            // Must detect undefined variable (at least as warning)
            let has_undefined_var_issue = issues.iter().any(|issue| {
                (issue.severity == IssueSeverity::Warning
                    || issue.severity == IssueSeverity::Error)
                    && issue.message.to_lowercase().contains("undefined")
            });

            TestResult::from_bool(has_undefined_var_issue)
        }
        Some(_) => TestResult::discard(),
        None => TestResult::discard(),
    }
}

/// **Property 24: Static Analysis Issue Detection - Type Errors**
/// **Validates: Requirements 25.1**
#[quickcheck]
fn prop_static_analysis_detects_type_errors(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(ExpectedIssue::TypeError) => {
            // Must detect type error
            let has_type_error = issues.iter().any(|issue| {
                issue.severity == IssueSeverity::Error
                    && issue.message.to_lowercase().contains("type error")
            });

            TestResult::from_bool(has_type_error)
        }
        Some(_) => TestResult::discard(),
        None => TestResult::discard(),
    }
}

/// **Property 24: Static Analysis Issue Detection - Large Loops**
/// **Validates: Requirements 25.1**
#[quickcheck]
fn prop_static_analysis_detects_large_loops(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(ExpectedIssue::LargeLoop) => {
            // Must detect large loop (at least as warning)
            let has_large_loop_warning = issues.iter().any(|issue| {
                (issue.severity == IssueSeverity::Warning
                    || issue.severity == IssueSeverity::Error)
                    && issue.message.to_lowercase().contains("large loop")
            });

            TestResult::from_bool(has_large_loop_warning)
        }
        Some(_) => TestResult::discard(),
        None => TestResult::discard(),
    }
}

/// **Property 24: Static Analysis No False Negatives**
/// **Validates: Requirements 25.1**
///
/// For any code with a known dangerous pattern, the analyzer MUST NOT
/// return an empty issue list (no false negatives).
#[quickcheck]
fn prop_static_analysis_no_false_negatives(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        Some(_expected_issue) => {
            // Must have at least one issue detected
            TestResult::from_bool(!issues.is_empty())
        }
        None => TestResult::discard(), // Valid code may have no issues
    }
}

/// **Property 24: Static Analysis Valid Code**
/// **Validates: Requirements 25.1**
///
/// Valid code should not produce any errors (warnings are acceptable).
#[quickcheck]
fn prop_static_analysis_valid_code_no_errors(code_gen: LuaCodeWithIssue) -> TestResult {
    let analyzer = StaticAnalyzer::new();
    let code = code_gen.to_code();
    let issues = analyzer.analyze(&code);

    match code_gen.expected_issue_type() {
        None => {
            // Valid code should not have errors
            let has_errors = issues
                .iter()
                .any(|issue| issue.severity == IssueSeverity::Error);
            TestResult::from_bool(!has_errors)
        }
        Some(_) => TestResult::discard(),
    }
}

// Additional unit tests for specific patterns

#[test]
fn test_detects_while_true_without_break() {
    let analyzer = StaticAnalyzer::new();
    let code = "while true do\n  print('loop')\nend";
    let issues = analyzer.analyze(code);

    assert!(
        issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error
                && i.message.contains("infinite loop")),
        "Should detect infinite while loop"
    );
}

#[test]
fn test_allows_while_true_with_break() {
    let analyzer = StaticAnalyzer::new();
    let code = "while true do\n  if condition then break end\nend";
    let issues = analyzer.analyze(code);

    // Should not report infinite loop error if break is present
    let has_infinite_loop_error = issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error && i.message.contains("infinite loop"));

    assert!(
        !has_infinite_loop_error,
        "Should not report infinite loop when break is present"
    );
}

#[test]
fn test_detects_repeat_until_false() {
    let analyzer = StaticAnalyzer::new();
    let code = "repeat\n  print('loop')\nuntil false";
    let issues = analyzer.analyze(code);

    assert!(
        issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error
                && i.message.to_lowercase().contains("infinite loop")),
        "Should detect repeat until false. Found {} issues", issues.len()
    );
}

#[test]
fn test_detects_all_system_calls() {
    let analyzer = StaticAnalyzer::new();
    let dangerous_calls = vec![
        "os.execute('cmd')",
        "io.open('file.txt')",
        "io.popen('cmd')",
        "loadfile('script.lua')",
        "dofile('script.lua')",
    ];

    for call in dangerous_calls {
        let issues = analyzer.analyze(call);
        assert!(
            issues.iter().any(|i| i.severity == IssueSeverity::Error
                && (i.message.contains("system call") || i.message.contains("Dangerous"))),
            "Should detect dangerous call: {}",
            call
        );
    }
}

#[test]
fn test_detects_nil_arithmetic() {
    let analyzer = StaticAnalyzer::new();
    let code = "local x = nil + 5";
    let issues = analyzer.analyze(code);

    assert!(
        issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error && i.message.contains("Type error")),
        "Should detect nil arithmetic"
    );
}

#[test]
fn test_detects_nil_concatenation() {
    let analyzer = StaticAnalyzer::new();
    let code = "local x = 'text' .. nil";
    let issues = analyzer.analyze(code);

    assert!(
        issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error && i.message.contains("Type error")),
        "Should detect nil concatenation"
    );
}

#[test]
fn test_detects_large_loop_iterations() {
    let analyzer = StaticAnalyzer::new();
    let code = "for i = 1, 50000 do\n  print(i)\nend";
    let issues = analyzer.analyze(code);

    assert!(
        issues.iter().any(|i| (i.severity == IssueSeverity::Warning
            || i.severity == IssueSeverity::Error)
            && i.message.contains("Large loop")),
        "Should detect large loop iterations"
    );
}

#[test]
fn test_detects_undefined_variable_usage() {
    let analyzer = StaticAnalyzer::new();
    let code = "local x = undefined_variable + 1";
    let issues = analyzer.analyze(code);

    assert!(
        issues.iter().any(|i| (i.severity == IssueSeverity::Warning
            || i.severity == IssueSeverity::Error)
            && i.message.contains("undefined")),
        "Should detect undefined variable"
    );
}

#[test]
fn test_valid_code_no_errors() {
    let analyzer = StaticAnalyzer::new();
    let code = r#"
        local x = 1
        local y = 2
        local z = x + y
        return z
    "#;
    let issues = analyzer.analyze(code);

    let has_errors = issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Error);

    assert!(!has_errors, "Valid code should not produce errors");
}

#[test]
fn test_allows_lua_builtins() {
    let analyzer = StaticAnalyzer::new();
    let code = r#"
        local x = math.sqrt(16)
        local y = string.upper("hello")
        print(x, y)
    "#;
    let issues = analyzer.analyze(code);

    // Debug: print all issues
    for issue in &issues {
        eprintln!("Issue: {:?}", issue);
    }

    // Should not report undefined variables for builtins
    let has_undefined_builtin = issues.iter().any(|i| {
        i.message.contains("math")
            || i.message.contains("string")
            || i.message.contains("print")
    });

    assert!(
        !has_undefined_builtin,
        "Should not report builtins as undefined. Found {} issues", issues.len()
    );
}
