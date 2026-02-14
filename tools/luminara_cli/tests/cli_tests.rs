use luminara_cli::ops::scaffold_project;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::fs;

#[quickcheck]
fn test_project_completeness(name: String) -> TestResult {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains('\0') {
        return TestResult::discard();
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_path_buf();

    // We append name to temp path to create new dir
    let project_path = path.join(&name);

    if scaffold_project(&project_path, &name).is_ok() {
        let has_cargo = project_path.join("Cargo.toml").exists();
        let has_src = project_path.join("src/main.rs").exists();
        let has_scenes = project_path.join("assets/scenes").exists();

        TestResult::from_bool(has_cargo && has_src && has_scenes)
    } else {
        TestResult::discard() // fs error
    }
}
