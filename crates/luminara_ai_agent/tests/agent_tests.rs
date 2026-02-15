use luminara_ai_agent::{AgentMessage, AgentOrchestrator, AgentRole};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_agent_registration() {
    let mut orchestrator = AgentOrchestrator::new();
    orchestrator.register_agent("architect".into(), AgentRole::SceneArchitect);
    // Verified by compilation and no panic
}

#[test]
fn test_task_decomposition() {
    let orchestrator = AgentOrchestrator::new();
    let tasks = orchestrator.decompose_task("Create a level with mountains");

    assert!(tasks.len() > 0);
    assert!(tasks
        .iter()
        .any(|(role, _)| *role == AgentRole::SceneArchitect));
}

#[test]
fn test_conflict_detection() {
    let orchestrator = AgentOrchestrator::new();
    let ops = vec![("agent1".to_string(), 1), ("agent2".to_string(), 2)];
    let conflicts = orchestrator.detect_conflicts(&ops);
    assert!(!conflicts.is_empty());
}

#[quickcheck]
fn test_message_delivery(content: String) -> TestResult {
    let orchestrator = AgentOrchestrator::new();
    let msg = AgentMessage {
        from: "a1".into(),
        to: "a2".into(),
        content: content.clone(),
        intent: None,
    };

    orchestrator.send_message(msg).unwrap();
    let received = orchestrator.process_messages();

    TestResult::from_bool(received.len() == 1 && received[0].content == content)
}
