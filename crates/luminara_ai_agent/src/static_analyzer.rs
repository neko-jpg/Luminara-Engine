// Requirements 8.1, 8.8
// "Static Analyzer... Lua AST parser... dangerous patterns... suggestions"

// We use `full-moon` or `mlua`'s parser if available?
// `mlua` doesn't expose AST easily.
// `full-moon` is a Lua 5.1/5.2/5.3/5.4 parser in Rust.
// Or we use regex/basic pattern matching for MVP if full parser is heavy.
// Requirement 8.1 says "Set up Lua AST parser".
// So we should use a parser.
// Let's add `full-moon` to dependencies.

// But wait, `full-moon` is heavy.
// Let's check `luminara_ai_agent/Cargo.toml` dependencies.
// I can add it now.

// Pattern detection:
// 1. Infinite loops: `while true do` without break (hard to prove statically, but we can flag `while true`).
// 2. Undefined vars: Requires scope analysis.
// 3. System calls: `os.execute`, `io.open` (already blocked by sandbox, but static analysis adds early warning).
// 4. Resource exhaustion: large tables?

use std::collections::HashSet;

pub struct StaticAnalyzer {
    // Configuration or rules
}

#[derive(Debug, Clone)]
pub struct StaticIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub line: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl StaticAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn analyze(&self, code: &str) -> Vec<StaticIssue> {
        let mut issues = Vec::new();

        // MVP: Regex-based or simple string matching if full parser not available yet.
        // Or simpler parser.
        // Let's look for "while true" and "os.*" "io.*".

        for (i, line) in code.lines().enumerate() {
            let line_num = i + 1;

            if line.contains("while true") {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Warning,
                    message: "Potential infinite loop detected: 'while true'".into(),
                    line: line_num,
                    suggestion: Some("Ensure there is a 'break' condition.".into()),
                });
            }

            if line.contains("os.execute") || line.contains("io.open") {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Error,
                    message: "System call detected. IO/OS modules are restricted.".into(),
                    line: line_num,
                    suggestion: Some("Remove system calls.".into()),
                });
            }

            // "Undefined variable" - hard without AST.
            // "Resource exhaustion" - "for i=1,1000000"
            if line.contains("1000000") { // Very naive heuristic
                 issues.push(StaticIssue {
                    severity: IssueSeverity::Warning,
                    message: "Large loop iteration count detected.".into(),
                    line: line_num,
                    suggestion: Some("Reduce iteration count or use time-slicing.".into()),
                });
            }
        }

        issues
    }
}
