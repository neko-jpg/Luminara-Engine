use luminara_ai_agent::OperationTimeline;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_operation_recording(prompt: String) -> TestResult {
    let mut timeline = OperationTimeline::new();
    let id = timeline.record(prompt.clone(), "Response".into(), vec![], vec![]);

    // Check if we can record multiple without crash
    // And if ID increments
    let id2 = timeline.record("Prompt 2".into(), "Response 2".into(), vec![], vec![]);

    TestResult::from_bool(id2 > id)
}

#[test]
fn test_undo_round_trip() {
    use luminara_core::world::World;
    let mut world = World::new();
    let mut timeline = OperationTimeline::new();

    timeline.record("1".into(), "".into(), vec![], vec![]);
    timeline.record("2".into(), "".into(), vec![], vec![]);

    // Undo "2"
    let res = timeline.undo(&mut world);
    assert!(res.is_ok());

    // Undo "1"
    let res = timeline.undo(&mut world);
    assert!(res.is_ok());

    // Undo again (fail)
    let res = timeline.undo(&mut world); // Should fail as head is None or root
    assert!(res.is_err());
}

#[test]
fn test_branching() {
    use luminara_core::world::World;
    let mut world = World::new();
    let mut timeline = OperationTimeline::new();

    timeline.record("root".into(), "".into(), vec![], vec![]);
    timeline.create_branch("master").unwrap();

    timeline.record("feature".into(), "".into(), vec![], vec![]);
    timeline.create_branch("feature-branch").unwrap();

    // Switch back to master (root)
    let res = timeline.checkout_branch("master", &mut world);
    assert!(res.is_ok());

    // Record on master
    timeline.record("master-commit".into(), "".into(), vec![], vec![]);

    // Verify structure implicitly by no errors
}
