// ... imports ...
use crate::sandbox::{ScriptSandbox, SandboxConfig};
use crate::static_analyzer::{StaticAnalyzer, StaticIssue, IssueSeverity};
use crate::dry_run::{DryRunner, CodeApplicator, DiffPreview};
use luminara_script::{ScriptId, ScriptError};
use luminara_core::world::World;
use std::time::Duration;

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
}

impl CodeVerificationPipeline {
    pub fn new() -> Result<Self, ScriptError> {
        let config = SandboxConfig {
            allow_filesystem: false,
            allow_network: false,
            max_execution_time: Duration::from_millis(50),
            ..Default::default()
        };

        Ok(Self {
            static_analyzer: StaticAnalyzer::new(),
            sandbox: ScriptSandbox::new(config)?,
            dry_runner: DryRunner::new(),
            applicator: CodeApplicator::new(),
        })
    }

    // Method used by tests (verification only)
    pub fn verify(&mut self, code: &str) -> VerificationResult {
        // 1. Static Analysis
        let issues = self.static_analyzer.analyze(code);
        let errors = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();

        if errors > 0 {
            return VerificationResult {
                passed: false,
                static_issues: issues,
                sandbox_result: None,
                diff: None,
                suggestions: vec!["Fix static analysis errors first.".into()],
            };
        }

        // 2. Sandbox Execution
        let sandbox_res = self.sandbox.run_lua(code);

        // 3. Dry Run (mock) - Needs world, but we might not have it in this simple verify call.
        // If verify is called without world, we skip dry run or mock it?
        // But dry_run requires &World.
        // Let's create a temporary world for dry run estimation or pass None?
        // Actually, let's overload or change signature.
        // The tests call `verify(code)`.
        // Let's make `verify` take `code` and assume empty world if not provided?
        // Or just require `verify_with_world`.

        // For property tests, we just want to check static/sandbox logic.
        // I will add a dummy World construction here if needed or just skip dry run if I can't construct it easily.
        // Luminara core World::new() is cheap.

        let diff = if sandbox_res.is_ok() {
             use luminara_core::world::World;
             let world = World::new();
             Some(self.dry_runner.dry_run(code, &world))
        } else {
            None
        };

        let passed = sandbox_res.is_ok();
        let suggestions = if !passed {
            vec!["Code failed runtime verification in sandbox.".into()]
        } else {
            vec![]
        };

        VerificationResult {
            passed,
            static_issues: issues,
            sandbox_result: Some(sandbox_res),
            diff,
            suggestions,
        }
    }

    pub fn verify_and_apply(&mut self, code: &str, world: &mut World) -> (VerificationResult, Option<ApplyResult>) {
        // ... implementation as before ...
        // 1. Static Analysis
        let issues = self.static_analyzer.analyze(code);
        let errors = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();

        if errors > 0 {
            return (VerificationResult {
                passed: false,
                static_issues: issues,
                sandbox_result: None,
                diff: None,
                suggestions: vec!["Fix static analysis errors first.".into()],
            }, None);
        }

        // 2. Sandbox Execution
        let sandbox_res = self.sandbox.run_lua(code);
        if sandbox_res.is_err() {
             return (VerificationResult {
                passed: false,
                static_issues: issues,
                sandbox_result: Some(sandbox_res),
                diff: None,
                suggestions: vec!["Code failed runtime verification in sandbox.".into()],
            }, None);
        }

        // 3. Dry Run
        let diff = self.dry_runner.dry_run(code, world);

        // 4. Apply
        let apply_result = match self.applicator.apply_with_monitoring(code, world) {
            Ok(_) => ApplyResult { success: true, error: None },
            Err(e) => ApplyResult { success: false, error: Some(e) },
        };

        (VerificationResult {
            passed: true,
            static_issues: issues,
            sandbox_result: Some(sandbox_res),
            diff: Some(diff),
            suggestions: vec![],
        }, Some(apply_result))
    }
}
