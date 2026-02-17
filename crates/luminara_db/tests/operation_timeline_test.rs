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

    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
    let db2 = timeline.get_db().clone();
    let mut timeline2 = OperationTimeline::new(db2, None);

    // Set position to the last operation
    timeline2.set_position(Some(op2.clone()));

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

#[tokio::test]
async fn test_operation_with_intent() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record operation with intent
    let op_id = timeline
        .record_operation_with_intent(
            "SpawnEntity",
            "Spawned enemy at position",
            vec![json!({"type": "spawn"})],
            vec![json!({"type": "despawn"})],
            vec![],
            Some("Create an enemy character near the player".to_string()),
        )
        .await
        .unwrap();

    // Load and verify intent is stored
    let operation = db.load_operation(&op_id).await.unwrap();
    assert!(operation.intent.is_some());
    assert_eq!(
        operation.intent.unwrap(),
        "Create an enemy character near the player"
    );
}

#[tokio::test]
async fn test_selective_undo_without_conflicts() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Create two entities
    let entity1 = db
        .store_entity(luminara_db::EntityRecord::new(Some("Entity1".to_string())))
        .await
        .unwrap();

    let entity2 = db
        .store_entity(luminara_db::EntityRecord::new(Some("Entity2".to_string())))
        .await
        .unwrap();

    // Record operations affecting different entities
    let op1 = timeline
        .record_operation(
            "ModifyEntity1",
            "Modified entity 1",
            vec![json!({"entity": "entity1"})],
            vec![json!({"restore": "entity1"})],
            vec![entity1.clone()],
        )
        .await
        .unwrap();

    let _op2 = timeline
        .record_operation(
            "ModifyEntity2",
            "Modified entity 2",
            vec![json!({"entity": "entity2"})],
            vec![json!({"restore": "entity2"})],
            vec![entity2.clone()],
        )
        .await
        .unwrap();

    // Selective undo of op1 should have no conflicts
    let result = timeline.selective_undo(&op1).await.unwrap();
    assert!(result.is_some());

    let (conflicts, inverse_commands) = result.unwrap();
    assert!(conflicts.is_empty(), "Expected no conflicts");
    assert_eq!(inverse_commands.len(), 1);
    assert_eq!(inverse_commands[0]["restore"], "entity1");
}

#[tokio::test]
async fn test_selective_undo_with_conflicts() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Create entity
    let entity = db
        .store_entity(luminara_db::EntityRecord::new(Some("TestEntity".to_string())))
        .await
        .unwrap();

    // Record operations affecting the same entity
    let op1 = timeline
        .record_operation(
            "ModifyEntity",
            "First modification",
            vec![json!({"value": 1})],
            vec![json!({"value": 0})],
            vec![entity.clone()],
        )
        .await
        .unwrap();

    // Delay to ensure different timestamps (timestamps are in seconds)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let op2 = timeline
        .record_operation(
            "ModifyEntity",
            "Second modification",
            vec![json!({"value": 2})],
            vec![json!({"value": 1})],
            vec![entity.clone()],
        )
        .await
        .unwrap();

    // Verify operations were recorded
    let op1_data = db.load_operation(&op1).await.unwrap();
    let op2_data = db.load_operation(&op2).await.unwrap();
    
    assert!(op2_data.timestamp > op1_data.timestamp, "Op2 should have later timestamp");

    // Selective undo of op1 should detect conflict
    let result = timeline.selective_undo(&op1).await.unwrap();
    assert!(result.is_some());

    let (conflicts, _) = result.unwrap();
    assert!(!conflicts.is_empty(), "Expected conflicts");
    assert!(conflicts[0].contains("Second modification"));
}

#[tokio::test]
async fn test_ai_context_generation() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations with intents
    timeline
        .record_operation_with_intent(
            "SpawnEntity",
            "Spawned player",
            vec![],
            vec![],
            vec![],
            Some("Create the player character".to_string()),
        )
        .await
        .unwrap();

    timeline
        .record_operation_with_intent(
            "AddComponent",
            "Added health component",
            vec![],
            vec![],
            vec![],
            Some("Give the player health tracking".to_string()),
        )
        .await
        .unwrap();

    // Generate AI context
    let context = timeline.generate_ai_context(10, false).await.unwrap();

    assert!(context.contains("Operation History"));
    assert!(context.contains("SpawnEntity"));
    assert!(context.contains("AddComponent"));
    assert!(context.contains("Intent: Create the player character"));
    assert!(context.contains("Intent: Give the player health tracking"));
}

#[tokio::test]
async fn test_compact_summary_generation() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations
    timeline
        .record_operation_with_intent(
            "Op1",
            "First operation",
            vec![],
            vec![],
            vec![],
            Some("Do first thing".to_string()),
        )
        .await
        .unwrap();

    timeline
        .record_operation_with_intent(
            "Op2",
            "Second operation",
            vec![],
            vec![],
            vec![],
            Some("Do second thing".to_string()),
        )
        .await
        .unwrap();

    // Generate compact summary
    let summary = timeline.generate_compact_summary(10).await.unwrap();

    assert!(summary.contains("Recent ops"));
    assert!(summary.contains("Op1: Do first thing"));
    assert!(summary.contains("Op2: Do second thing"));
    assert!(summary.contains("→")); // Arrow separator
}

#[tokio::test]
async fn test_ai_context_with_no_operations() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let timeline = OperationTimeline::new(db, None);

    let context = timeline.generate_ai_context(10, false).await.unwrap();
    assert_eq!(context, "No operations in history.");

    let summary = timeline.generate_compact_summary(10).await.unwrap();
    assert_eq!(summary, "No operations.");
}

#[tokio::test]
async fn test_selective_undo_nonexistent_operation() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let timeline = OperationTimeline::new(db, None);

    // Try to undo an operation that doesn't exist
    let fake_id = surrealdb::RecordId::from(("operation", "nonexistent"));
    let result = timeline.selective_undo(&fake_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_operation_metadata_completeness() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    let entity = db
        .store_entity(luminara_db::EntityRecord::new(Some("TestEntity".to_string())))
        .await
        .unwrap();

    // Record operation with all metadata
    let op_id = timeline
        .record_operation_with_intent(
            "CompleteOperation",
            "Operation with all metadata",
            vec![json!({"action": "forward"})],
            vec![json!({"action": "backward"})],
            vec![entity.clone()],
            Some("AI intent for this operation".to_string()),
        )
        .await
        .unwrap();

    // Load and verify all metadata is present
    let operation = db.load_operation(&op_id).await.unwrap();

    // Verify all required fields
    assert_eq!(operation.operation_type, "CompleteOperation");
    assert_eq!(operation.description, "Operation with all metadata");
    assert_eq!(operation.commands.len(), 1);
    assert_eq!(operation.inverse_commands.len(), 1);
    assert_eq!(operation.affected_entities.len(), 1);
    assert!(operation.timestamp > 0);
    assert_eq!(operation.branch, Some("main".to_string()));
    assert_eq!(
        operation.intent,
        Some("AI intent for this operation".to_string())
    );
}

#[tokio::test]
async fn test_undo_redo_preserves_state() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record operation with specific state
    let original_state = json!({"position": [1.0, 2.0, 3.0], "health": 100});
    let modified_state = json!({"position": [4.0, 5.0, 6.0], "health": 80});

    let op_id = timeline
        .record_operation(
            "ModifyState",
            "Modified entity state",
            vec![modified_state.clone()],
            vec![original_state.clone()],
            vec![],
        )
        .await
        .unwrap();

    // Undo
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands[0], original_state);

    // Redo
    let redo_result = timeline.redo().await.unwrap();
    assert!(redo_result.is_some());
    let (_redone_id, commands) = redo_result.unwrap();
    assert_eq!(commands[0], modified_state);
}
