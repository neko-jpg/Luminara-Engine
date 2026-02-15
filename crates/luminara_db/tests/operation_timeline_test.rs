//! Integration tests for operation timeline storage

use luminara_db::{LuminaraDatabase, OperationTimeline};
use serde_json::json;

#[tokio::test]
async fn test_basic_operation_recording() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record an operation
    let op_id = timeline
        .record_operation(
            "SpawnEntity",
            "Spawned player entity",
            vec![json!({"type": "spawn", "entity": "player"})],
            vec![json!({"type": "despawn", "entity": "player"})],
            vec![],
        )
        .await
        .unwrap();

    assert!(op_id.to_string().contains("operation"));

    // Verify operation is in history
    let history = timeline.get_history(10).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation_type, "SpawnEntity");
    assert_eq!(history[0].description, "Spawned player entity");
}

#[tokio::test]
async fn test_undo_redo_cycle() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record multiple operations
    let op1 = timeline
        .record_operation(
            "Op1",
            "First operation",
            vec![json!({"action": "forward1"})],
            vec![json!({"action": "backward1"})],
            vec![],
        )
        .await
        .unwrap();

    let op2 = timeline
        .record_operation(
            "Op2",
            "Second operation",
            vec![json!({"action": "forward2"})],
            vec![json!({"action": "backward2"})],
            vec![],
        )
        .await
        .unwrap();

    // Undo second operation
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op2);
    assert_eq!(inverse_commands[0]["action"], "backward2");

    // Undo first operation
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op1);
    assert_eq!(inverse_commands[0]["action"], "backward1");

    // Nothing more to undo
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_none());

    // Redo first operation
    let redo_result = timeline.redo().await.unwrap();
    assert!(redo_result.is_some());
    let (_redone_id, commands) = redo_result.unwrap();
    assert_eq!(commands[0]["action"], "forward1");

    // Redo second operation
    let redo_result = timeline.redo().await.unwrap();
    assert!(redo_result.is_some());
    let (_redone_id, commands) = redo_result.unwrap();
    assert_eq!(commands[0]["action"], "forward2");

    // Nothing more to redo
    let redo_result = timeline.redo().await.unwrap();
    assert!(redo_result.is_none());
}

#[tokio::test]
async fn test_branch_creation_and_switching() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations on main branch
    timeline
        .record_operation(
            "MainOp1",
            "Main operation 1",
            vec![json!({"branch": "main"})],
            vec![],
            vec![],
        )
        .await
        .unwrap();

    timeline
        .record_operation(
            "MainOp2",
            "Main operation 2",
            vec![json!({"branch": "main"})],
            vec![],
            vec![],
        )
        .await
        .unwrap();

    // Create feature branch
    timeline.create_branch("feature").await.unwrap();
    assert_eq!(timeline.current_branch(), "feature");

    // Record operations on feature branch
    timeline
        .record_operation(
            "FeatureOp1",
            "Feature operation 1",
            vec![json!({"branch": "feature"})],
            vec![],
            vec![],
        )
        .await
        .unwrap();

    // Verify feature branch history
    let feature_history = timeline.get_history(10).await.unwrap();
    assert_eq!(feature_history.len(), 1);
    assert_eq!(feature_history[0].operation_type, "FeatureOp1");

    // Switch back to main
    timeline.switch_branch("main").await.unwrap();
    assert_eq!(timeline.current_branch(), "main");

    // Verify main branch history
    let main_history = timeline.get_history(10).await.unwrap();
    assert_eq!(main_history.len(), 2);
    assert_eq!(main_history[0].operation_type, "MainOp2");
    assert_eq!(main_history[1].operation_type, "MainOp1");
}

#[tokio::test]
async fn test_branch_listing() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Create operations on main
    timeline
        .record_operation("Op1", "Operation 1", vec![], vec![], vec![])
        .await
        .unwrap();

    // Create and populate feature branch
    timeline.create_branch("feature").await.unwrap();
    timeline
        .record_operation("Op2", "Operation 2", vec![], vec![], vec![])
        .await
        .unwrap();

    // Create and populate experimental branch
    timeline.create_branch("experimental").await.unwrap();
    timeline
        .record_operation("Op3", "Operation 3", vec![], vec![], vec![])
        .await
        .unwrap();

    // List all branches
    let branches = timeline.list_branches().await.unwrap();
    assert_eq!(branches.len(), 3);

    let branch_names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
    assert!(branch_names.contains(&"main"));
    assert!(branch_names.contains(&"feature"));
    assert!(branch_names.contains(&"experimental"));

    // Verify operation counts
    let main_branch = branches.iter().find(|b| b.name == "main").unwrap();
    assert_eq!(main_branch.operation_count, 1);

    let feature_branch = branches.iter().find(|b| b.name == "feature").unwrap();
    assert_eq!(feature_branch.operation_count, 1);
}

#[tokio::test]
async fn test_timeline_statistics() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record 5 operations
    for i in 0..5 {
        timeline
            .record_operation(
                format!("Op{}", i),
                format!("Operation {}", i),
                vec![json!({"index": i})],
                vec![json!({"index": i})],
                vec![],
            )
            .await
            .unwrap();
    }

    // Check initial statistics
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.total_operations, 5);
    assert_eq!(stats.branch_operations, 5);
    assert_eq!(stats.current_branch, "main");
    assert_eq!(stats.undoable_operations, 5);
    assert_eq!(stats.redoable_operations, 0);

    // Undo 3 operations
    timeline.undo().await.unwrap();
    timeline.undo().await.unwrap();
    timeline.undo().await.unwrap();

    // Check statistics after undo
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.undoable_operations, 2);
    assert_eq!(stats.redoable_operations, 3);

    // Redo 1 operation
    timeline.redo().await.unwrap();

    // Check statistics after redo
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.undoable_operations, 3);
    assert_eq!(stats.redoable_operations, 2);
}

#[tokio::test]
async fn test_persistent_undo_redo() {
    // Create database and timeline
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations
    let op1 = timeline
        .record_operation(
            "Op1",
            "First operation",
            vec![json!({"value": 1})],
            vec![json!({"value": -1})],
            vec![],
        )
        .await
        .unwrap();

    let op2 = timeline
        .record_operation(
            "Op2",
            "Second operation",
            vec![json!({"value": 2})],
            vec![json!({"value": -2})],
            vec![],
        )
        .await
        .unwrap();

    // Simulate session restart by creating new timeline with same database
    let db2 = timeline.db.clone();
    let mut timeline2 = OperationTimeline::new(db2, None);

    // Set position to the last operation
    timeline2.current_position = Some(op2.clone());

    // Should be able to undo across sessions
    let undo_result = timeline2.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, _) = undo_result.unwrap();
    assert_eq!(undone_id, op2);

    // Should be able to undo the first operation too
    let undo_result = timeline2.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, _) = undo_result.unwrap();
    assert_eq!(undone_id, op1);
}

#[tokio::test]
async fn test_operation_with_affected_entities() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Create some entities
    let entity1 = db
        .store_entity(luminara_db::EntityRecord::new(Some("Entity1".to_string())))
        .await
        .unwrap();

    let entity2 = db
        .store_entity(luminara_db::EntityRecord::new(Some("Entity2".to_string())))
        .await
        .unwrap();

    // Record operation affecting both entities
    let op_id = timeline
        .record_operation(
            "ModifyMultiple",
            "Modified multiple entities",
            vec![json!({"action": "modify"})],
            vec![json!({"action": "restore"})],
            vec![entity1.clone(), entity2.clone()],
        )
        .await
        .unwrap();

    // Load the operation and verify affected entities
    let operation = db.load_operation(&op_id).await.unwrap();
    assert_eq!(operation.affected_entities.len(), 2);
    assert!(operation.affected_entities.contains(&entity1));
    assert!(operation.affected_entities.contains(&entity2));
}

#[tokio::test]
async fn test_branch_deletion() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Create operations on main
    timeline
        .record_operation("MainOp", "Main operation", vec![], vec![], vec![])
        .await
        .unwrap();

    // Create feature branch with operations
    timeline.create_branch("feature").await.unwrap();
    timeline
        .record_operation("FeatureOp", "Feature operation", vec![], vec![], vec![])
        .await
        .unwrap();

    // Switch back to main
    timeline.switch_branch("main").await.unwrap();

    // Delete feature branch
    timeline.delete_branch("feature").await.unwrap();

    // Verify feature branch is gone
    let branches = timeline.list_branches().await.unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
}

#[tokio::test]
async fn test_cannot_delete_current_branch() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    timeline
        .record_operation("Op", "Operation", vec![], vec![], vec![])
        .await
        .unwrap();

    // Try to delete current branch
    let result = timeline.delete_branch("main").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_branch_info() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations
    for i in 0..3 {
        timeline
            .record_operation(
                format!("Op{}", i),
                format!("Operation {}", i),
                vec![],
                vec![],
                vec![],
            )
            .await
            .unwrap();
    }

    // Get branch info
    let info = timeline.get_branch_info("main").await.unwrap();
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.name, "main");
    assert_eq!(info.operation_count, 3);
    assert!(info.head.is_some());
    assert!(info.created_at > 0);
}

#[tokio::test]
async fn test_clear_branch() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations
    for i in 0..5 {
        timeline
            .record_operation(
                format!("Op{}", i),
                format!("Operation {}", i),
                vec![],
                vec![],
                vec![],
            )
            .await
            .unwrap();
    }

    // Verify operations exist
    let history = timeline.get_history(10).await.unwrap();
    assert_eq!(history.len(), 5);

    // Clear branch
    timeline.clear_branch().await.unwrap();

    // Verify branch is empty
    let history = timeline.get_history(10).await.unwrap();
    assert_eq!(history.len(), 0);
    assert!(timeline.current_position().is_none());
}

#[tokio::test]
async fn test_complex_branch_workflow() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Main branch: Op1 -> Op2
    timeline
        .record_operation("Op1", "Operation 1", vec![], vec![], vec![])
        .await
        .unwrap();
    timeline
        .record_operation("Op2", "Operation 2", vec![], vec![], vec![])
        .await
        .unwrap();

    // Create feature branch from Op2
    timeline.create_branch("feature").await.unwrap();
    timeline
        .record_operation("FeatureOp1", "Feature operation 1", vec![], vec![], vec![])
        .await
        .unwrap();
    timeline
        .record_operation("FeatureOp2", "Feature operation 2", vec![], vec![], vec![])
        .await
        .unwrap();

    // Switch back to main and continue
    timeline.switch_branch("main").await.unwrap();
    timeline
        .record_operation("Op3", "Operation 3", vec![], vec![], vec![])
        .await
        .unwrap();

    // Verify main branch has 3 operations
    let main_history = timeline.get_history(10).await.unwrap();
    assert_eq!(main_history.len(), 3);

    // Switch to feature and verify it has 2 operations
    timeline.switch_branch("feature").await.unwrap();
    let feature_history = timeline.get_history(10).await.unwrap();
    assert_eq!(feature_history.len(), 2);

    // Verify total operations across all branches
    let all_ops = timeline.get_all_operations(100).await.unwrap();
    assert_eq!(all_ops.len(), 5);
}
