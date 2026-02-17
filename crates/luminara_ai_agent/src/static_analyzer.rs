// Requirements 25.1: Static analysis for Lua scripts
// Detects: infinite loops, undefined variables, type errors, dangerous patterns

use std::collections::{HashMap, HashSet};

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

        // Track variable declarations and usage
        let mut declared_vars = HashSet::new();
        let mut used_vars = HashSet::new();
        
        // Lua built-ins that are always available
        let builtins: HashSet<&str> = [
            "print", "type", "tonumber", "tostring", "pairs", "ipairs",
            "next", "select", "assert", "error", "pcall", "xpcall",
            "math", "string", "table", "_G", "_VERSION", "require",
        ].iter().copied().collect();

        for (i, line) in code.lines().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Detect infinite loops
            if trimmed.contains("while true") && !trimmed.contains("break") {
                // Check if break exists in subsequent lines (simple heuristic)
                let has_break = code.lines().skip(i).take(20).any(|l| l.contains("break"));
                if !has_break {
                    issues.push(StaticIssue {
                        severity: IssueSeverity::Error,
                        message: "Potential infinite loop detected: 'while true' without break".into(),
                        line: line_num,
                        suggestion: Some("Add a break condition or use a bounded loop.".into()),
                    });
                }
            }

            // Detect repeat until false (infinite loop)
            if trimmed.contains("repeat") {
                // Check if "until false" appears in the same line or subsequent lines
                let remaining_code = code.lines().skip(i).take(20).collect::<Vec<_>>().join(" ");
                if remaining_code.contains("until false") {
                    issues.push(StaticIssue {
                        severity: IssueSeverity::Error,
                        message: "Infinite loop detected: 'repeat until false'".into(),
                        line: line_num,
                        suggestion: Some("Use a proper termination condition.".into()),
                    });
                }
            }

            // Detect system calls (security)
            if trimmed.contains("os.execute") || trimmed.contains("io.open") || 
               trimmed.contains("io.popen") || trimmed.contains("loadfile") ||
               trimmed.contains("dofile") {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Error,
                    message: "Dangerous system call detected. IO/OS modules are restricted.".into(),
                    line: line_num,
                    suggestion: Some("Remove system calls. Use engine APIs instead.".into()),
                });
            }

            // Detect large loop iterations (resource exhaustion)
            if let Some(count) = extract_loop_count(trimmed) {
                if count > 10000 {
                    issues.push(StaticIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Large loop iteration count detected: {}", count),
                        line: line_num,
                        suggestion: Some("Consider reducing iteration count or using coroutines.".into()),
                    });
                }
            }

            // Track variable declarations (local x = ...)
            if let Some(var_name) = extract_local_declaration(trimmed) {
                declared_vars.insert(var_name);
            }

            // Track variable usage
            for var in extract_variable_usage(trimmed) {
                if !builtins.contains(var.as_str()) {
                    used_vars.insert(var);
                }
            }

            // Detect type errors (basic)
            if trimmed.contains("nil + ") || trimmed.contains("+ nil") {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Error,
                    message: "Type error: attempting arithmetic on nil value".into(),
                    line: line_num,
                    suggestion: Some("Check for nil before arithmetic operations.".into()),
                });
            }

            // Detect string concatenation with nil
            if trimmed.contains("nil .. ") || trimmed.contains(".. nil") {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Error,
                    message: "Type error: attempting to concatenate nil value".into(),
                    line: line_num,
                    suggestion: Some("Check for nil before string concatenation.".into()),
                });
            }
        }

        // Check for undefined variables
        for var in &used_vars {
            if !declared_vars.contains(var) {
                issues.push(StaticIssue {
                    severity: IssueSeverity::Warning,
                    message: format!("Potentially undefined variable: '{}'", var),
                    line: 0, // We'd need better tracking for exact line
                    suggestion: Some(format!("Declare '{}' with 'local' or ensure it's defined.", var)),
                });
            }
        }

        issues
    }

    /// Analyze WASM module for safety
    pub fn analyze_wasm(&self, _wasm_bytes: &[u8]) -> Vec<StaticIssue> {
        let mut issues = Vec::new();
        
        // For WASM, we check:
        // 1. Allowed imports only
        // 2. No dangerous instructions
        // 3. Memory limits
        
        // This would require wasmparser crate for full implementation
        // For now, return placeholder
        
        issues.push(StaticIssue {
            severity: IssueSeverity::Info,
            message: "WASM validation not yet fully implemented".into(),
            line: 0,
            suggestion: Some("WASM modules will be validated at runtime".into()),
        });
        
        issues
    }
}

// Helper functions for parsing

fn extract_loop_count(line: &str) -> Option<usize> {
    // Match patterns like "for i=1,10000" or "for i = 1, 10000"
    if line.contains("for ") && line.contains("=") {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            if let Some(num_str) = parts[1].split_whitespace().next() {
                return num_str.parse::<usize>().ok();
            }
        }
    }
    None
}

fn extract_local_declaration(line: &str) -> Option<String> {
    // Match "local varname = " or "local varname,"
    if line.starts_with("local ") {
        let after_local = &line[6..];
        if let Some(var_name) = after_local.split_whitespace().next() {
            let clean_name = var_name.trim_end_matches(',').trim_end_matches('=');
            if !clean_name.is_empty() {
                return Some(clean_name.to_string());
            }
        }
    }
    None
}

fn extract_variable_usage(line: &str) -> Vec<String> {
    // Very simple: extract words that look like identifiers
    // This is a naive implementation - a real parser would be better
    let mut vars = Vec::new();
    
    for word in line.split_whitespace() {
        // Remove common punctuation but keep dots for table access detection
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '.');
        
        // Handle table access (e.g., math.sqrt, string.upper)
        // Only extract the base table name if it contains a dot
        let identifier = if clean.contains('.') {
            // Skip table access - these are method calls on builtins
            continue;
        } else {
            clean
        };
        
        // Handle function calls - extract just the function name before '('
        let identifier = if let Some(paren_pos) = identifier.find('(') {
            &identifier[..paren_pos]
        } else {
            identifier
        };
        
        if !identifier.is_empty() && identifier.chars().next().unwrap().is_alphabetic() {
            // Skip Lua keywords
            if !is_lua_keyword(identifier) {
                vars.push(identifier.to_string());
            }
        }
    }
    
    vars
}

fn is_lua_keyword(word: &str) -> bool {
    matches!(word, 
        "and" | "break" | "do" | "else" | "elseif" | "end" | "false" | 
        "for" | "function" | "if" | "in" | "local" | "nil" | "not" | 
        "or" | "repeat" | "return" | "then" | "true" | "until" | "while"
    )
}
