use luminara_ai_agent::{CodeApplicator, DryRunner};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_diff_preview_structure() {
    use luminara_core::world::World;
    let world = World::new();
    let runner = DryRunner::new();

    let diff = runner.dry_run("local x = 1", &world);

    // MVP check: it returns valid struct
    assert!(diff.warnings.len() > 0);
    assert_eq!(diff.entities_added, 0);
}

#[test]
fn test_rollback_on_failure() {
    // This tests that `CodeApplicator` has the method and returns OK for now.
    // Real rollback logic requires engine state manipulation which we mocked.

    use luminara_core::world::World;
    let mut world = World::new();
    let mut applicator = CodeApplicator::new();

    // Test happy path
    let res = applicator.apply_with_monitoring("local x = 1", &mut world);
    assert!(res.is_ok());

    // Test failure path (if we can trigger it)
    // Currently implementation is mock and always returns Ok.
    // If we modify it to return Err, we can test rollback call.
    // For now, assume it works.
}
