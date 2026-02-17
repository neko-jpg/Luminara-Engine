/// Enhanced error handling infrastructure for Luminara Engine
///
/// Provides rich error messages with:
/// - File locations and line numbers
/// - Syntax highlighting (ANSI colors)
/// - Suggested fixes
/// - Documentation links

use std::fmt;

/// Error location information
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub context: Option<String>,
}

impl ErrorLocation {
    /// Create a new error location
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            context: None,
        }
    }

    /// Add code context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Base trait for all Luminara errors
pub trait LuminaraError: std::error::Error {
    /// Unique error code (e.g., "E001")
    fn code(&self) -> &str;

    /// File location where error occurred
    fn location(&self) -> Option<&ErrorLocation>;

    /// Suggested fix
    fn suggestion(&self) -> Option<&str>;

    /// Documentation link
    fn docs_link(&self) -> Option<&str>;

    /// Format error with colors and context
    fn format_detailed(&self) -> String {
        let mut output = String::new();

        // Error header with color
        if supports_color() {
            output.push_str(&format!(
                "\x1b[1;31merror[{}]\x1b[0m: \x1b[1m{}\x1b[0m\n",
                self.code(),
                self
            ));
        } else {
            output.push_str(&format!("error[{}]: {}\n", self.code(), self));
        }

        // Location
        if let Some(loc) = self.location() {
            if supports_color() {
                output.push_str(&format!(
                    "  \x1b[1;34m-->\x1b[0m {}:{}:{}\n",
                    loc.file, loc.line, loc.column
                ));
            } else {
                output.push_str(&format!("  --> {}:{}:{}\n", loc.file, loc.line, loc.column));
            }

            // Context with line numbers
            if let Some(context) = &loc.context {
                if supports_color() {
                    output.push_str("   \x1b[1;34m|\x1b[0m\n");
                    for (i, line) in context.lines().enumerate() {
                        let line_num = loc.line + i;
                        output.push_str(&format!(
                            "\x1b[1;34m{:3} |\x1b[0m {}\n",
                            line_num, line
                        ));
                    }
                    output.push_str("   \x1b[1;34m|\x1b[0m\n");
                } else {
                    output.push_str("   |\n");
                    for (i, line) in context.lines().enumerate() {
                        let line_num = loc.line + i;
                        output.push_str(&format!("{:3} | {}\n", line_num, line));
                    }
                    output.push_str("   |\n");
                }
            }
        }

        // Suggestion
        if let Some(suggestion) = self.suggestion() {
            if supports_color() {
                output.push_str(&format!("   \x1b[1;32m= help:\x1b[0m {}\n", suggestion));
            } else {
                output.push_str(&format!("   = help: {}\n", suggestion));
            }
        }

        // Documentation link
        if let Some(docs) = self.docs_link() {
            if supports_color() {
                output.push_str(&format!("   \x1b[1;36m= docs:\x1b[0m {}\n", docs));
            } else {
                output.push_str(&format!("   = docs: {}\n", docs));
            }
        }

        output
    }
}

/// Check if terminal supports ANSI colors
pub fn supports_color() -> bool {
    #[cfg(windows)]
    {
        // Try to enable ANSI colors on Windows 10+
        // This is a simplified check - in production, use a crate like `supports-color`
        std::env::var("TERM").is_ok() || std::env::var("WT_SESSION").is_ok()
    }

    #[cfg(not(windows))]
    {
        // Unix-like systems typically support ANSI colors
        std::env::var("TERM").is_ok() && std::env::var("TERM").unwrap() != "dumb"
    }
}

/// Macro to create errors with location information
#[macro_export]
macro_rules! luminara_error {
    ($code:expr, $msg:expr) => {
        $crate::error::GenericError {
            code: $code.to_string(),
            message: $msg.to_string(),
            location: Some($crate::error::ErrorLocation::new(
                file!(),
                line!() as usize,
                column!() as usize,
            )),
            suggestion: None,
            docs_link: None,
        }
    };

    ($code:expr, $msg:expr, suggestion: $suggestion:expr) => {
        $crate::error::GenericError {
            code: $code.to_string(),
            message: $msg.to_string(),
            location: Some($crate::error::ErrorLocation::new(
                file!(),
                line!() as usize,
                column!() as usize,
            )),
            suggestion: Some($suggestion.to_string()),
            docs_link: None,
        }
    };

    ($code:expr, $msg:expr, suggestion: $suggestion:expr, docs: $docs:expr) => {
        $crate::error::GenericError {
            code: $code.to_string(),
            message: $msg.to_string(),
            location: Some($crate::error::ErrorLocation::new(
                file!(),
                line!() as usize,
                column!() as usize,
            )),
            suggestion: Some($suggestion.to_string()),
            docs_link: Some($docs.to_string()),
        }
    };
}

/// Generic error implementation
#[derive(Debug, Clone)]
pub struct GenericError {
    pub code: String,
    pub message: String,
    pub location: Option<ErrorLocation>,
    pub suggestion: Option<String>,
    pub docs_link: Option<String>,
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GenericError {}

impl LuminaraError for GenericError {
    fn code(&self) -> &str {
        &self.code
    }

    fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }

    fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    fn docs_link(&self) -> Option<&str> {
        self.docs_link.as_deref()
    }
}

/// World operation errors
#[derive(Debug, thiserror::Error)]
pub enum WorldError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(crate::entity::Entity),
    #[error("Component error")]
    ComponentError,
    #[error("Archetype error: {0}")]
    ArchetypeError(String),
}

/// Install custom panic handler with enhanced error messages
pub fn install_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap();
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic"
        };

        if supports_color() {
            eprintln!(
                "\n\x1b[1;31merror[PANIC]\x1b[0m: \x1b[1m{}\x1b[0m\n\
                 \x1b[1;34m  -->\x1b[0m {}:{}:{}\n\
                 \x1b[1;32m  = help:\x1b[0m This is a bug in Luminara Engine or your code\n\
                 \x1b[1;36m  = docs:\x1b[0m https://luminara-engine.org/docs/troubleshooting/panics\n",
                message,
                location.file(),
                location.line(),
                location.column()
            );
        } else {
            eprintln!(
                "\nerror[PANIC]: {}\n\
                   --> {}:{}:{}\n\
                   = help: This is a bug in Luminara Engine or your code\n\
                   = docs: https://luminara-engine.org/docs/troubleshooting/panics\n",
                message,
                location.file(),
                location.line(),
                location.column()
            );
        }

        // Print backtrace if RUST_BACKTRACE is set
        if std::env::var("RUST_BACKTRACE").is_ok() {
            eprintln!("\nBacktrace:");
            eprintln!("{:?}", std::backtrace::Backtrace::capture());
        } else {
            eprintln!("\nRun with RUST_BACKTRACE=1 for a backtrace.");
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_location() {
        let loc = ErrorLocation::new("src/test.rs", 42, 5);
        assert_eq!(loc.file, "src/test.rs");
        assert_eq!(loc.line, 42);
        assert_eq!(loc.column, 5);
        assert!(loc.context.is_none());
    }

    #[test]
    fn test_error_location_with_context() {
        let loc = ErrorLocation::new("src/test.rs", 42, 5)
            .with_context("let x = 42;");
        assert!(loc.context.is_some());
        assert_eq!(loc.context.unwrap(), "let x = 42;");
    }

    #[test]
    fn test_generic_error() {
        let error = GenericError {
            code: "E001".to_string(),
            message: "Test error".to_string(),
            location: Some(ErrorLocation::new("src/test.rs", 42, 5)),
            suggestion: Some("Fix the test".to_string()),
            docs_link: Some("https://example.com/docs".to_string()),
        };

        assert_eq!(error.code(), "E001");
        assert!(error.location().is_some());
        assert_eq!(error.suggestion(), Some("Fix the test"));
        assert_eq!(error.docs_link(), Some("https://example.com/docs"));
    }

    #[test]
    fn test_error_formatting() {
        let error = GenericError {
            code: "E001".to_string(),
            message: "Test error".to_string(),
            location: Some(
                ErrorLocation::new("src/test.rs", 42, 5)
                    .with_context("let x = 42;"),
            ),
            suggestion: Some("Fix the test".to_string()),
            docs_link: Some("https://example.com/docs".to_string()),
        };

        let formatted = error.format_detailed();
        assert!(formatted.contains("error[E001]"));
        assert!(formatted.contains("src/test.rs:42:5"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("help:"));
        assert!(formatted.contains("docs:"));
    }

    #[test]
    fn test_luminara_error_macro() {
        let error = luminara_error!("E001", "Test error");
        assert_eq!(error.code, "E001");
        assert_eq!(error.message, "Test error");
        assert!(error.location.is_some());

        let error_with_suggestion = luminara_error!(
            "E002",
            "Another error",
            suggestion: "Try this fix"
        );
        assert_eq!(error_with_suggestion.suggestion, Some("Try this fix".to_string()));

        let error_with_docs = luminara_error!(
            "E003",
            "Yet another error",
            suggestion: "Fix it",
            docs: "https://example.com"
        );
        assert_eq!(error_with_docs.docs_link, Some("https://example.com".to_string()));
    }
}
