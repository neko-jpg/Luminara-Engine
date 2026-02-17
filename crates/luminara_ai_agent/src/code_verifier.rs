// Requirements 25.1: Complete Code Verification Pipeline
// "Static analysis for Lua/WASM"
// "Sandbox execution with resource limits"
// "Automatic rollback with monitoring"
// "Fix suggestions based on common error patterns"

use crate::dry_run::{CodeApplicator, DiffPreview, DryRunner, MonitoringResult};
use crate::sandbox::{SandboxConfig, ScriptSandbox};
use crate::static_analyzer::{IssueSeverity, StaticAnalyzer, StaticIssue};
use luminara_core::world::World;
use luminara_script::ScriptError;

pub struct CodeVerificationPipeline {
    static_analyzer: StaticAnalyzer,
    sandbox: ScriptSandbox,
    dry_runner: DryRunner,
    applicator: CodeApplicator,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub passed: bool,
    pub static_issues: Vec<StaticIssue>,
    pub sandbox_result: Option<Result<(), ScriptError>>,
    pub diff: Option<DiffPreview>,
    pub suggestions: Vec<String>,
}

pub struct ApplyResult {
    pub success: bool,
    pub error: Option<String>,
    pub monitoring: Option<MonitoringResult>,
}

impl CodeVerificationPipeline {
    pub fn new() -> Result<Self, ScriptError> {
        // Use verification config with proper limits per requirement 25.1
        let config = SandboxConfig::for_verification();

        Ok(Self {
            static_analyzer: StaticAnalyzer::new(),
            sandbox: ScriptSandbox::new(config)?,
            dry_runner: DryRunner::new(),
            applicator: CodeApplicator::new(),
        })
    }

    /// Verify code without applying (for testing and validation)
    pub fn verify(&mut self, code: &str) -> VerificationResult {
        // Step 1: Static Analysis
        let issues = self.static_analyzer.analyze(code);
        let errors = issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .count();

        if errors > 0 {
            return VerificationResult {
                passed: false,
                static_issues: issues.clone(),
                sandbox_result: None,
                diff: None,
                suggestions: self.generate_fix_suggestions(&issues),
            };
        }

        // Step 2: Sandbox Execution
        let sandbox_res = self.sandbox.run_lua(code);

        if sandbox_res.is_err() {
            let mut suggestions = self.generate_fix_suggestions(&issues);
            suggestions.push("Code failed runtime verification in sandbox.".into());

            return VerificationResult {
                passed: false,
                static_issues: issues,
                sandbox_result: Some(sandbox_res),
                diff: None,
                suggestions,
            };
        }

        // Step 3: Dry Run (requires world, create temporary one for verification)
        let world = World::new();
        let diff = self.dry_runner.dry_run(code, &world);

        VerificationResult {
            passed: true,
            static_issues: issues,
            sandbox_result: Some(sandbox_res),
            diff: Some(diff),
            suggestions: vec![],
        }
    }

    /// Verify and apply code with full monitoring and rollback
    /// Requirements 25.1: Complete verification pipeline
    pub fn verify_and_apply(
        &mut self,
        code: &str,
        world: &mut World,
    ) -> (VerificationResult, Option<ApplyResult>) {
        // Step 1: Static Analysis
        let issues = self.static_analyzer.analyze(code);
        let errors = issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .count();

        if errors > 0 {
            return (
                VerificationResult {
                    passed: false,
                    static_issues: issues.clone(),
                    sandbox_result: None,
                    diff: None,
                    suggestions: self.generate_fix_suggestions(&issues),
                },
                None,
            );
        }

        // Step 2: Sandbox Execution
        let sandbox_res = self.sandbox.run_lua(code);
        if sandbox_res.is_err() {
            let mut suggestions = self.generate_fix_suggestions(&issues);
            suggestions.push("Code failed runtime verification in sandbox.".into());

            return (
                VerificationResult {
                    passed: false,
                    static_issues: issues,
                    sandbox_result: Some(sandbox_res),
                    diff: None,
                    suggestions,
                },
                None,
            );
        }

        // Step 3: Dry Run - Generate diff preview
        let diff = self.dry_runner.dry_run(code, world);

        // Step 4: Apply with monitoring and automatic rollback
        let monitoring_result = match self.applicator.apply_with_monitoring(code, world) {
            Ok(monitoring) => {
                let success = monitoring.success;
                let error = if !success {
                    Some(monitoring.errors.join("; "))
                } else {
                    None
                };

                ApplyResult {
                    success,
                    error,
                    monitoring: Some(monitoring),
                }
            }
            Err(e) => ApplyResult {
                success: false,
                error: Some(e),
                monitoring: None,
            },
        };

        (
            VerificationResult {
                passed: monitoring_result.success,
                static_issues: issues,
                sandbox_result: Some(sandbox_res),
                diff: Some(diff),
                suggestions: if monitoring_result.success {
                    vec![]
                } else {
                    vec!["Code was rolled back due to runtime errors or anomalies.".into()]
                },
            },
            Some(monitoring_result),
        )
    }

    /// Generate fix suggestions based on common error patterns
    /// Requirements 25.1: "Provide fix suggestions based on common error patterns"
    fn generate_fix_suggestions(&self, issues: &[StaticIssue]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for issue in issues {
            if let Some(suggestion) = &issue.suggestion {
                suggestions.push(suggestion.clone());
            }
        }

        // Add general suggestions based on error patterns
        let has_infinite_loop = issues.iter().any(|i| i.message.contains("infinite loop"));
        let has_system_call = issues.iter().any(|i| i.message.contains("system call"));
        let has_undefined_var = issues.iter().any(|i| i.message.contains("undefined variable"));

        if has_infinite_loop {
            suggestions.push("Consider using bounded loops with explicit termination conditions.".into());
        }

        if has_system_call {
            suggestions.push("Use engine-provided APIs instead of system calls for security.".into());
        }

        if has_undefined_var {
            suggestions.push("Declare all variables with 'local' keyword to avoid global scope pollution.".into());
        }

        if suggestions.is_empty() && !issues.is_empty() {
            suggestions.push("Review the static analysis issues and fix them before proceeding.".into());
        }

        suggestions
    }
}


