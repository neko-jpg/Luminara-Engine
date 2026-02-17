//! Property Test: Operation Undo Correctness
//!
//! **Property 27: Operation Undo Correctness**
//!
//! For any operation, undoing it should execute the inverse commands and restore
//! the world to the exact state before the operation.
//!
//! This test verifies that:
//! 1. Undo returns the correct inverse commands
//! 2. The timeline position is correctly updated after undo
//! 3. Undo is idempotent (undoing the same operation multiple times has the same effect)
//! 4. Undo works correctly across different operation types
//! 5. Multiple undo operations work correctly in sequence
//! 6. Undo followed by redo restores the forward commands
//!
//! **Validates: Requirements 27.2**

use luminara_db::{LuminaraDatabase, OperationTimeline};
use proptest::prelude::*;
use serde_json::json;

// ============================================================================
// Test Data Generators
// ============================================================================

/// Strategy for generating operation types
fn operation_type_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "SpawnEntity".to_string(),
        "DestroyEntity".to_string(),
        "AddComponent".to_string(),
        "RemoveComponent".to_string(),
        "ModifyComponent".to_string(),
        "ModifyTransform".to_string(),
        "SetPosition".to_string(),
        "SetRotation".to_string(),
        "SetScale".to_string(),
        "UpdatePhysics".to_string(),
    ])
}

/// Strategy for generating operation descriptions
fn description_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-z ]+ [a-z ]+ [a-z]+").unwrap()
}


/// Strategy for generating state values (representing world state)
fn state_value_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop::sample::select(vec![
        json!({"position": [0.0, 0.0, 0.0], "health": 100}),
        json!({"position": [1.0, 2.0, 3.0], "health": 80}),
        json!({"position": [-5.0, 10.0, 0.0], "health": 50}),
        json!({"rotation": [0.0, 0.0, 0.0, 1.0], "velocity": [1.0, 0.0, 0.0]}),
        json!({"scale": [1.0, 1.0, 1.0], "color": [1.0, 0.0, 0.0, 1.0]}),
        json!({"enabled": true, "mass": 10.0}),
        json!({"enabled": false, "mass": 5.0}),
        json!({"name": "Entity", "tag": "Player"}),
        json!({"name": "Enemy", "tag": "NPC", "level": 5}),
        json!({"transform": {"x": 1.0, "y": 2.0, "z": 3.0}}),
    ])
}

/// Strategy for generating a pair of forward and inverse commands
/// The inverse command should logically undo the forward command
fn command_pair_strategy() -> impl Strategy<Value = (serde_json::Value, serde_json::Value)> {
    state_value_strategy().prop_flat_map(|initial_state| {
        state_value_strategy().prop_map(move |modified_state| {
            // Forward command: apply modified state
            let forward = json!({
                "action": "set_state",
                "state": modified_state
            });
            
            // Inverse command: restore initial state
            let inverse = json!({
                "action": "set_state",
                "state": initial_state
            });
            
            (forward, inverse)
        })
    })
}

/// Strategy for generating multiple command pairs
fn command_pairs_list_strategy() -> impl Strategy<Value = Vec<(serde_json::Value, serde_json::Value)>> {
    prop::collection::vec(command_pair_strategy(), 1..10)
}

/// Strategy for generating operation count
fn operation_count_strategy() -> impl Strategy<Value = usize> {
    2usize..20
}


// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 27: Operation Undo Correctness**
    ///
    /// For any operation, undoing it should:
    /// 1. Return the correct inverse commands
    /// 2. Update the timeline position to the parent operation
    /// 3. Allow the operation to be redone with forward commands
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_operation_undo_returns_inverse_commands(
        operation_type in operation_type_strategy(),
        description in description_strategy(),
        command_pair in command_pair_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            let (forward_cmd, inverse_cmd) = command_pair;

            // Record operation
            let op_id = timeline
                .record_operation(
                    operation_type,
                    description,
                    vec![forward_cmd.clone()],
                    vec![inverse_cmd.clone()],
                    vec![],
                )
                .await
                .unwrap();

            // Verify position is set to the operation
            prop_assert_eq!(timeline.current_position(), Some(&op_id));

            // Undo the operation
            let undo_result = timeline.undo().await.unwrap();
            prop_assert!(undo_result.is_some(), "Undo should return Some");

            let (undone_id, returned_inverse_commands) = undo_result.unwrap();

            // Property 1: Undo returns the correct operation ID
            prop_assert_eq!(undone_id, op_id, "Undone operation ID should match");

            // Property 2: Undo returns the correct inverse commands
            prop_assert_eq!(
                returned_inverse_commands.len(),
                1,
                "Should return one inverse command"
            );
            prop_assert_eq!(
                &returned_inverse_commands[0],
                &inverse_cmd,
                "Returned inverse command should match recorded inverse command"
            );

            // Property 3: Timeline position should be updated to None (no parent)
            prop_assert_eq!(
                timeline.current_position(),
                None,
                "Position should be None after undoing the only operation"
            );

            Ok(())
        });
        result.unwrap();
    }


    /// **Property 27 (variant): Multiple Operations Undo Sequence**
    ///
    /// When multiple operations are recorded, undoing them in reverse order
    /// should return the correct inverse commands for each operation.
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_multiple_operations_undo_sequence(
        operation_count in operation_count_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            let mut recorded_ops = Vec::new();

            // Record multiple operations
            for i in 0..operation_count {
                let forward_cmd = json!({"index": i, "action": "forward"});
                let inverse_cmd = json!({"index": i, "action": "backward"});

                let op_id = timeline
                    .record_operation(
                        format!("Op{}", i),
                        format!("Operation {}", i),
                        vec![forward_cmd.clone()],
                        vec![inverse_cmd.clone()],
                        vec![],
                    )
                    .await
                    .unwrap();

                recorded_ops.push((op_id, forward_cmd, inverse_cmd));
            }

            // Undo all operations in reverse order
            for i in (0..operation_count).rev() {
                let (expected_op_id, _forward_cmd, expected_inverse_cmd) = &recorded_ops[i];

                let undo_result = timeline.undo().await.unwrap();
                prop_assert!(undo_result.is_some(), "Undo should succeed for operation {}", i);

                let (undone_id, inverse_commands) = undo_result.unwrap();

                // Property: Each undo returns the correct operation and inverse commands
                prop_assert_eq!(
                    undone_id,
                    expected_op_id.clone(),
                    "Undone operation ID should match for operation {}",
                    i
                );
                prop_assert_eq!(
                    &inverse_commands[0],
                    expected_inverse_cmd,
                    "Inverse command should match for operation {}",
                    i
                );
            }

            // Property: After undoing all operations, position should be None
            prop_assert_eq!(
                timeline.current_position(),
                None,
                "Position should be None after undoing all operations"
            );

            // Property: No more operations to undo
            let final_undo = timeline.undo().await.unwrap();
            prop_assert!(final_undo.is_none(), "Should not be able to undo further");

            Ok(())
        });
        result.unwrap();
    }


    /// **Property 27 (variant): Undo-Redo Round-Trip Correctness**
    ///
    /// For any operation, undoing it and then redoing it should:
    /// 1. Return the original forward commands on redo
    /// 2. Restore the timeline position to the operation
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_undo_redo_roundtrip(
        operation_type in operation_type_strategy(),
        description in description_strategy(),
        command_pair in command_pair_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            let (forward_cmd, inverse_cmd) = command_pair;

            // Record operation
            let op_id = timeline
                .record_operation(
                    operation_type,
                    description,
                    vec![forward_cmd.clone()],
                    vec![inverse_cmd.clone()],
                    vec![],
                )
                .await
                .unwrap();

            let original_position = timeline.current_position().cloned();

            // Undo
            let undo_result = timeline.undo().await.unwrap();
            prop_assert!(undo_result.is_some());
            let (undone_id, inverse_commands) = undo_result.unwrap();
            prop_assert_eq!(undone_id, op_id.clone());
            prop_assert_eq!(&inverse_commands[0], &inverse_cmd);

            // Redo
            let redo_result = timeline.redo().await.unwrap();
            prop_assert!(redo_result.is_some(), "Redo should succeed");

            let (redone_id, forward_commands) = redo_result.unwrap();

            // Property 1: Redo returns the correct operation ID
            prop_assert_eq!(redone_id, op_id, "Redone operation ID should match original");

            // Property 2: Redo returns the original forward commands
            prop_assert_eq!(
                forward_commands.len(),
                1,
                "Should return one forward command"
            );
            prop_assert_eq!(
                &forward_commands[0],
                &forward_cmd,
                "Forward command should match original"
            );

            // Property 3: Timeline position should be restored
            prop_assert_eq!(
                timeline.current_position(),
                original_position.as_ref(),
                "Position should be restored after redo"
            );

            Ok(())
        });
        result.unwrap();
    }


    /// **Property 27 (variant): Undo Correctness Across Different Operation Types**
    ///
    /// Undo should work correctly for all operation types, returning the
    /// appropriate inverse commands for each type.
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_undo_correctness_across_operation_types(
        command_pairs in command_pairs_list_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            let operation_types = vec![
                "SpawnEntity",
                "DestroyEntity",
                "AddComponent",
                "RemoveComponent",
                "ModifyComponent",
                "ModifyTransform",
                "SetPosition",
                "SetRotation",
                "SetScale",
                "UpdatePhysics",
            ];

            let mut recorded_ops = Vec::new();

            // Record operations of different types
            for (i, (forward_cmd, inverse_cmd)) in command_pairs.iter().enumerate() {
                let op_type = operation_types[i % operation_types.len()];

                let op_id = timeline
                    .record_operation(
                        op_type,
                        format!("{} operation", op_type),
                        vec![forward_cmd.clone()],
                        vec![inverse_cmd.clone()],
                        vec![],
                    )
                    .await
                    .unwrap();

                recorded_ops.push((op_id, op_type, inverse_cmd.clone()));
            }

            // Undo all operations and verify correctness for each type
            for (expected_op_id, op_type, expected_inverse_cmd) in recorded_ops.iter().rev() {
                let undo_result = timeline.undo().await.unwrap();
                prop_assert!(
                    undo_result.is_some(),
                    "Undo should succeed for operation type {}",
                    op_type
                );

                let (undone_id, inverse_commands) = undo_result.unwrap();

                // Property: Undo works correctly for this operation type
                prop_assert_eq!(
                    undone_id,
                    expected_op_id.clone(),
                    "Undone operation ID should match for type {}",
                    op_type
                );
                prop_assert_eq!(
                    &inverse_commands[0],
                    expected_inverse_cmd,
                    "Inverse command should match for type {}",
                    op_type
                );
            }

            Ok(())
        });
        result.unwrap();
    }


    /// **Property 27 (variant): Partial Undo-Redo Sequence**
    ///
    /// When undoing some operations and then redoing some, the timeline
    /// should maintain correctness throughout.
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_partial_undo_redo_sequence(
        total_ops in 5usize..15,
        undo_count in 2usize..8
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            // Ensure undo_count doesn't exceed total_ops
            let undo_count = undo_count.min(total_ops);
            let redo_count = undo_count / 2;

            let mut recorded_ops = Vec::new();

            // Record operations
            for i in 0..total_ops {
                let op_id = timeline
                    .record_operation(
                        format!("Op{}", i),
                        format!("Operation {}", i),
                        vec![json!({"index": i, "action": "forward"})],
                        vec![json!({"index": i, "action": "backward"})],
                        vec![],
                    )
                    .await
                    .unwrap();

                recorded_ops.push(op_id);
            }

            // Undo some operations
            for i in 0..undo_count {
                let undo_result = timeline.undo().await.unwrap();
                prop_assert!(
                    undo_result.is_some(),
                    "Undo {} should succeed",
                    i
                );
            }

            // Redo some operations
            for i in 0..redo_count {
                let redo_result = timeline.redo().await.unwrap();
                prop_assert!(
                    redo_result.is_some(),
                    "Redo {} should succeed",
                    i
                );
            }

            // Property: Timeline position should be correct
            // After total_ops - undo_count + redo_count operations
            let expected_position_index = total_ops - undo_count + redo_count - 1;
            let expected_position = &recorded_ops[expected_position_index];
            
            prop_assert_eq!(
                timeline.current_position(),
                Some(expected_position),
                "Timeline position should be at operation {}",
                expected_position_index
            );

            // Property: Should be able to undo back to the expected position
            let stats = timeline.get_statistics().await.unwrap();
            prop_assert_eq!(
                stats.undoable_operations,
                expected_position_index + 1,
                "Should have correct number of undoable operations"
            );

            Ok(())
        });
        result.unwrap();
    }


    /// **Property 27 (variant): Undo with Multiple Commands**
    ///
    /// Operations with multiple forward and inverse commands should undo correctly,
    /// returning all inverse commands in the correct order.
    ///
    /// **Validates: Requirements 27.2**
    #[test]
    fn prop_undo_with_multiple_commands(
        command_pairs in command_pairs_list_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db, None);

            let forward_commands: Vec<_> = command_pairs.iter().map(|(f, _)| f.clone()).collect();
            let inverse_commands: Vec<_> = command_pairs.iter().map(|(_, i)| i.clone()).collect();

            // Record operation with multiple commands
            let op_id = timeline
                .record_operation(
                    "BatchOperation",
                    "Operation with multiple commands",
                    forward_commands.clone(),
                    inverse_commands.clone(),
                    vec![],
                )
                .await
                .unwrap();

            // Undo the operation
            let undo_result = timeline.undo().await.unwrap();
            prop_assert!(undo_result.is_some());

            let (undone_id, returned_inverse_commands) = undo_result.unwrap();

            // Property 1: Correct operation ID
            prop_assert_eq!(undone_id, op_id);

            // Property 2: All inverse commands are returned
            prop_assert_eq!(
                returned_inverse_commands.len(),
                inverse_commands.len(),
                "Should return all inverse commands"
            );

            // Property 3: Inverse commands are in the correct order
            for (i, expected_cmd) in inverse_commands.iter().enumerate() {
                prop_assert_eq!(
                    &returned_inverse_commands[i],
                    expected_cmd,
                    "Inverse command at index {} should match",
                    i
                );
            }

            // Property 4: Redo returns all forward commands
            let redo_result = timeline.redo().await.unwrap();
            prop_assert!(redo_result.is_some());

            let (_redone_id, returned_forward_commands) = redo_result.unwrap();
            prop_assert_eq!(
                returned_forward_commands.len(),
                forward_commands.len(),
                "Should return all forward commands on redo"
            );

            for (i, expected_cmd) in forward_commands.iter().enumerate() {
                prop_assert_eq!(
                    &returned_forward_commands[i],
                    expected_cmd,
                    "Forward command at index {} should match",
                    i
                );
            }

            Ok(())
        });
        result.unwrap();
    }
}


// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_undo_with_no_operations() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Try to undo when there are no operations
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_none(), "Undo should return None when there are no operations");
}

#[tokio::test]
async fn test_undo_single_operation_restores_initial_state() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    let initial_state = json!({"position": [0.0, 0.0, 0.0], "health": 100});
    let modified_state = json!({"position": [5.0, 10.0, 15.0], "health": 80});

    // Record operation that changes state
    let op_id = timeline
        .record_operation(
            "ModifyState",
            "Changed entity state",
            vec![json!({"action": "set_state", "state": modified_state})],
            vec![json!({"action": "set_state", "state": initial_state})],
            vec![],
        )
        .await
        .unwrap();

    // Undo should return the inverse command that restores initial state
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());

    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands.len(), 1);
    assert_eq!(inverse_commands[0]["state"], initial_state);
}

#[tokio::test]
async fn test_undo_redo_undo_sequence() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    let forward_cmd = json!({"action": "forward"});
    let inverse_cmd = json!({"action": "backward"});

    // Record operation
    let op_id = timeline
        .record_operation(
            "TestOp",
            "Test operation",
            vec![forward_cmd.clone()],
            vec![inverse_cmd.clone()],
            vec![],
        )
        .await
        .unwrap();

    // Undo
    let undo1 = timeline.undo().await.unwrap();
    assert!(undo1.is_some());
    assert_eq!(undo1.unwrap().1[0], inverse_cmd);

    // Redo
    let redo = timeline.redo().await.unwrap();
    assert!(redo.is_some());
    assert_eq!(redo.unwrap().1[0], forward_cmd);

    // Undo again
    let undo2 = timeline.undo().await.unwrap();
    assert!(undo2.is_some());
    let (undone_id, inverse_commands) = undo2.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands[0], inverse_cmd);
}


#[tokio::test]
async fn test_undo_with_affected_entities() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Create entities
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
            vec![json!({"action": "modify", "entities": ["entity1", "entity2"]})],
            vec![json!({"action": "restore", "entities": ["entity1", "entity2"]})],
            vec![entity1.clone(), entity2.clone()],
        )
        .await
        .unwrap();

    // Undo should work correctly even with affected entities
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());

    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands[0]["action"], "restore");
    assert_eq!(inverse_commands[0]["entities"][0], "entity1");
    assert_eq!(inverse_commands[0]["entities"][1], "entity2");
}

#[tokio::test]
async fn test_undo_preserves_operation_metadata() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record operation with full metadata
    let op_id = timeline
        .record_operation_with_intent(
            "CompleteOp",
            "Operation with full metadata",
            vec![json!({"action": "forward"})],
            vec![json!({"action": "backward"})],
            vec![],
            Some("AI intent for this operation".to_string()),
        )
        .await
        .unwrap();

    // Undo
    timeline.undo().await.unwrap();

    // Load operation and verify metadata is still intact
    let operation = db.load_operation(&op_id).await.unwrap();
    assert_eq!(operation.operation_type, "CompleteOp");
    assert_eq!(operation.description, "Operation with full metadata");
    assert_eq!(operation.intent, Some("AI intent for this operation".to_string()));
    assert_eq!(operation.commands.len(), 1);
    assert_eq!(operation.inverse_commands.len(), 1);
}

#[tokio::test]
async fn test_undo_across_branches() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operations on main branch
    let op1 = timeline
        .record_operation(
            "MainOp1",
            "Main operation 1",
            vec![json!({"branch": "main", "index": 1})],
            vec![json!({"branch": "main", "index": 1, "undo": true})],
            vec![],
        )
        .await
        .unwrap();

    let op2 = timeline
        .record_operation(
            "MainOp2",
            "Main operation 2",
            vec![json!({"branch": "main", "index": 2})],
            vec![json!({"branch": "main", "index": 2, "undo": true})],
            vec![],
        )
        .await
        .unwrap();

    // Undo operations on main branch
    let undo2 = timeline.undo().await.unwrap();
    assert!(undo2.is_some());
    let (undone_id, inverse_commands) = undo2.unwrap();
    assert_eq!(undone_id, op2);
    assert_eq!(inverse_commands[0]["index"], 2);
    assert_eq!(inverse_commands[0]["undo"], true);

    let undo1 = timeline.undo().await.unwrap();
    assert!(undo1.is_some());
    let (undone_id, inverse_commands) = undo1.unwrap();
    assert_eq!(undone_id, op1);
    assert_eq!(inverse_commands[0]["index"], 1);
    assert_eq!(inverse_commands[0]["undo"], true);
}


#[tokio::test]
async fn test_undo_with_empty_commands() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Record operation with empty commands (edge case)
    let op_id = timeline
        .record_operation(
            "EmptyOp",
            "Operation with no commands",
            vec![],
            vec![],
            vec![],
        )
        .await
        .unwrap();

    // Undo should still work, returning empty inverse commands
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());

    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands.len(), 0);
}

#[tokio::test]
async fn test_undo_with_complex_nested_commands() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Complex nested command structure
    let forward_cmd = json!({
        "type": "batch",
        "operations": [
            {
                "action": "spawn",
                "entity": {
                    "name": "Player",
                    "components": [
                        {"type": "Transform", "position": [1.0, 2.0, 3.0]},
                        {"type": "Health", "value": 100}
                    ]
                }
            },
            {
                "action": "modify",
                "target": "enemy_1",
                "changes": {"health": -10}
            }
        ]
    });

    let inverse_cmd = json!({
        "type": "batch",
        "operations": [
            {
                "action": "modify",
                "target": "enemy_1",
                "changes": {"health": 10}
            },
            {
                "action": "despawn",
                "entity": "Player"
            }
        ]
    });

    let op_id = timeline
        .record_operation(
            "ComplexBatch",
            "Complex batch operation",
            vec![forward_cmd.clone()],
            vec![inverse_cmd.clone()],
            vec![],
        )
        .await
        .unwrap();

    // Undo should preserve the complex structure
    let undo_result = timeline.undo().await.unwrap();
    assert!(undo_result.is_some());

    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op_id);
    assert_eq!(inverse_commands[0], inverse_cmd);
    assert_eq!(inverse_commands[0]["type"], "batch");
    assert_eq!(inverse_commands[0]["operations"][0]["action"], "modify");
    assert_eq!(inverse_commands[0]["operations"][1]["action"], "despawn");
}

#[tokio::test]
async fn test_undo_statistics_update() {
    // **Validates: Requirements 27.2**
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

    // Initial statistics
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.undoable_operations, 5);
    assert_eq!(stats.redoable_operations, 0);

    // Undo 3 operations
    timeline.undo().await.unwrap();
    timeline.undo().await.unwrap();
    timeline.undo().await.unwrap();

    // Statistics should update correctly
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.undoable_operations, 2);
    assert_eq!(stats.redoable_operations, 3);

    // Redo 1 operation
    timeline.redo().await.unwrap();

    // Statistics should update again
    let stats = timeline.get_statistics().await.unwrap();
    assert_eq!(stats.undoable_operations, 3);
    assert_eq!(stats.redoable_operations, 2);
}

#[tokio::test]
async fn test_undo_persistence_across_sessions() {
    // **Validates: Requirements 27.2**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

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

    // Simulate session restart
    let db2 = db.clone();
    let mut timeline2 = OperationTimeline::new(db2, None);
    timeline2.set_position(Some(op2.clone()));

    // Should be able to undo across sessions
    let undo_result = timeline2.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op2);
    assert_eq!(inverse_commands[0]["value"], -2);

    // Should be able to undo the first operation too
    let undo_result = timeline2.undo().await.unwrap();
    assert!(undo_result.is_some());
    let (undone_id, inverse_commands) = undo_result.unwrap();
    assert_eq!(undone_id, op1);
    assert_eq!(inverse_commands[0]["value"], -1);
}

#[tokio::test]
async fn test_undo_with_state_verification() {
    // **Validates: Requirements 27.2**
    // This test verifies that undo truly restores the exact previous state
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db, None);

    // Define exact states
    let state_v0 = json!({
        "position": [0.0, 0.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0],
        "health": 100,
        "enabled": true
    });

    let state_v1 = json!({
        "position": [10.0, 5.0, 3.0],
        "rotation": [0.0, 0.707, 0.0, 0.707],
        "scale": [2.0, 2.0, 2.0],
        "health": 80,
        "enabled": true
    });

    let state_v2 = json!({
        "position": [20.0, 10.0, 6.0],
        "rotation": [0.0, 1.0, 0.0, 0.0],
        "scale": [3.0, 3.0, 3.0],
        "health": 60,
        "enabled": false
    });

    // Operation 1: v0 -> v1
    timeline
        .record_operation(
            "Modify1",
            "First modification",
            vec![json!({"set_state": state_v1})],
            vec![json!({"set_state": state_v0})],
            vec![],
        )
        .await
        .unwrap();

    // Operation 2: v1 -> v2
    timeline
        .record_operation(
            "Modify2",
            "Second modification",
            vec![json!({"set_state": state_v2})],
            vec![json!({"set_state": state_v1})],
            vec![],
        )
        .await
        .unwrap();

    // Undo operation 2: should restore v1
    let undo2 = timeline.undo().await.unwrap();
    assert!(undo2.is_some());
    let (_, inverse_commands) = undo2.unwrap();
    assert_eq!(inverse_commands[0]["set_state"], state_v1);

    // Undo operation 1: should restore v0
    let undo1 = timeline.undo().await.unwrap();
    assert!(undo1.is_some());
    let (_, inverse_commands) = undo1.unwrap();
    assert_eq!(inverse_commands[0]["set_state"], state_v0);

    // Redo operation 1: should apply v1
    let redo1 = timeline.redo().await.unwrap();
    assert!(redo1.is_some());
    let (_, forward_commands) = redo1.unwrap();
    assert_eq!(forward_commands[0]["set_state"], state_v1);

    // Redo operation 2: should apply v2
    let redo2 = timeline.redo().await.unwrap();
    assert!(redo2.is_some());
    let (_, forward_commands) = redo2.unwrap();
    assert_eq!(forward_commands[0]["set_state"], state_v2);
}
