//! Property Test: Operation Metadata Completeness
//!
//! **Property 26: Operation Metadata Completeness**
//!
//! For any recorded operation, all required metadata fields must be present and valid:
//! - Intent (optional but should be preserved if provided)
//! - Commands (forward execution commands)
//! - Inverse commands (for undo functionality)
//! - Timestamp (Unix timestamp)
//! - Change summary (description)
//!
//! **Validates: Requirements 27.1**

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
        "CreateAsset".to_string(),
        "DeleteAsset".to_string(),
        "ModifyMaterial".to_string(),
        "UpdatePhysics".to_string(),
    ])
}

/// Strategy for generating operation descriptions
fn description_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-z ]+ [a-z ]+ [a-z]+").unwrap()
}

/// Strategy for generating AI intents
fn intent_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(
        prop::string::string_regex("(Create|Modify|Delete|Update) [a-z ]+ (for|to|with) [a-z ]+")
            .unwrap(),
    )
}

/// Strategy for generating command data
fn command_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop::sample::select(vec![
        json!({"action": "spawn", "entity": "player", "position": [0.0, 0.0, 0.0]}),
        json!({"action": "despawn", "entity": "enemy"}),
        json!({"action": "add_component", "type": "Transform", "data": {"x": 1.0}}),
        json!({"action": "remove_component", "type": "RigidBody"}),
        json!({"action": "modify", "field": "health", "value": 100}),
        json!({"action": "set_position", "position": [1.0, 2.0, 3.0]}),
        json!({"action": "set_rotation", "rotation": [0.0, 0.0, 0.0, 1.0]}),
        json!({"action": "set_scale", "scale": [1.0, 1.0, 1.0]}),
        json!({"action": "create_asset", "path": "assets/texture.png"}),
        json!({"action": "update_material", "color": [1.0, 0.0, 0.0, 1.0]}),
    ])
}

/// Strategy for generating a list of commands
fn commands_list_strategy() -> impl Strategy<Value = Vec<serde_json::Value>> {
    prop::collection::vec(command_strategy(), 1..10)
}

/// Strategy for generating inverse commands
fn inverse_commands_list_strategy() -> impl Strategy<Value = Vec<serde_json::Value>> {
    prop::collection::vec(command_strategy(), 1..10)
}

/// Strategy for generating affected entity count
fn affected_entity_count_strategy() -> impl Strategy<Value = usize> {
    0usize..20
}

/// Strategy for generating branch names
fn branch_name_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop::sample::select(vec![
        "main".to_string(),
        "feature".to_string(),
        "experimental".to_string(),
        "bugfix".to_string(),
        "hotfix".to_string(),
    ]))
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 26: Operation Metadata Completeness**
    ///
    /// For any recorded operation, all required metadata fields must be present and valid:
    /// 1. Operation type is non-empty
    /// 2. Description (change summary) is non-empty
    /// 3. Commands list is present (may be empty for some operations)
    /// 4. Inverse commands list is present (may be empty for some operations)
    /// 5. Timestamp is valid (> 0)
    /// 6. Intent is preserved if provided
    /// 7. Branch is set correctly
    /// 8. Affected entities list is present
    ///
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_operation_metadata_completeness(
        operation_type in operation_type_strategy(),
        description in description_strategy(),
        commands in commands_list_strategy(),
        inverse_commands in inverse_commands_list_strategy(),
        intent in intent_strategy(),
        affected_entity_count in affected_entity_count_strategy(),
        branch_name in branch_name_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db.clone(), branch_name.clone());

            // Create affected entities
            let mut affected_entities = Vec::new();
            for i in 0..affected_entity_count {
                let entity_id = db
                    .store_entity(luminara_db::EntityRecord::new(Some(format!("Entity{}", i))))
                    .await
                    .unwrap();
                affected_entities.push(entity_id);
            }

            // Record operation with all metadata
            let op_id = timeline
                .record_operation_with_intent(
                    operation_type.clone(),
                    description.clone(),
                    commands.clone(),
                    inverse_commands.clone(),
                    affected_entities.clone(),
                    intent.clone(),
                )
                .await
                .unwrap();

            // Load the operation and verify all metadata is present
            let operation = db.load_operation(&op_id).await.unwrap();

            // Property 1: Operation type is non-empty and matches
            prop_assert!(
                !operation.operation_type.is_empty(),
                "Operation type should be non-empty"
            );
            prop_assert_eq!(
                operation.operation_type,
                operation_type,
                "Operation type should match recorded value"
            );

            // Property 2: Description (change summary) is non-empty and matches
            prop_assert!(
                !operation.description.is_empty(),
                "Description (change summary) should be non-empty"
            );
            prop_assert_eq!(
                operation.description,
                description,
                "Description should match recorded value"
            );

            // Property 3: Commands list is present and matches
            prop_assert_eq!(
                operation.commands.len(),
                commands.len(),
                "Commands list length should match"
            );
            for (i, cmd) in commands.iter().enumerate() {
                prop_assert_eq!(
                    &operation.commands[i],
                    cmd,
                    "Command at index {} should match", i
                );
            }

            // Property 4: Inverse commands list is present and matches
            prop_assert_eq!(
                operation.inverse_commands.len(),
                inverse_commands.len(),
                "Inverse commands list length should match"
            );
            for (i, inv_cmd) in inverse_commands.iter().enumerate() {
                prop_assert_eq!(
                    &operation.inverse_commands[i],
                    inv_cmd,
                    "Inverse command at index {} should match", i
                );
            }

            // Property 5: Timestamp is valid (> 0)
            prop_assert!(
                operation.timestamp > 0,
                "Timestamp should be valid (> 0), got {}",
                operation.timestamp
            );

            // Property 6: Intent is preserved if provided
            prop_assert_eq!(
                operation.intent,
                intent,
                "Intent should be preserved exactly as provided"
            );

            // Property 7: Branch is set correctly
            let expected_branch = branch_name.or(Some("main".to_string()));
            prop_assert_eq!(
                operation.branch,
                expected_branch,
                "Branch should be set correctly"
            );

            // Property 8: Affected entities list is present and matches
            prop_assert_eq!(
                operation.affected_entities.len(),
                affected_entities.len(),
                "Affected entities list length should match"
            );
            for entity_id in &affected_entities {
                prop_assert!(
                    operation.affected_entities.contains(entity_id),
                    "Affected entities should contain entity {:?}", entity_id
                );
            }

            Ok(())
        });
        result.unwrap();
    }

    /// **Property 26 (variant): Multiple Operations Metadata Completeness**
    ///
    /// When recording multiple operations in sequence, each should maintain
    /// complete and distinct metadata.
    ///
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_multiple_operations_metadata_completeness(
        operation_count in 2usize..20
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db.clone(), None);

            let mut recorded_ops = Vec::new();

            // Record multiple operations
            for i in 0..operation_count {
                let op_type = format!("Operation{}", i);
                let description = format!("Description for operation {}", i);
                let intent = Some(format!("Intent for operation {}", i));
                let commands = vec![json!({"index": i, "action": "forward"})];
                let inverse_commands = vec![json!({"index": i, "action": "backward"})];

                let op_id = timeline
                    .record_operation_with_intent(
                        op_type.clone(),
                        description.clone(),
                        commands.clone(),
                        inverse_commands.clone(),
                        vec![],
                        intent.clone(),
                    )
                    .await
                    .unwrap();

                recorded_ops.push((op_id, op_type, description, intent, commands, inverse_commands));
            }

            // Verify each operation has complete and distinct metadata
            for (op_id, expected_type, expected_desc, expected_intent, expected_cmds, expected_inv_cmds) in recorded_ops {
                let operation = db.load_operation(&op_id).await.unwrap();

                // Verify all fields are present and correct
                prop_assert_eq!(operation.operation_type, expected_type);
                prop_assert_eq!(operation.description, expected_desc);
                prop_assert_eq!(operation.intent, expected_intent);
                prop_assert_eq!(operation.commands, expected_cmds);
                prop_assert_eq!(operation.inverse_commands, expected_inv_cmds);
                prop_assert!(operation.timestamp > 0);
                prop_assert_eq!(operation.branch, Some("main".to_string()));
            }

            Ok(())
        });
        result.unwrap();
    }

    /// **Property 26 (variant): Operation Metadata Persistence**
    ///
    /// Operation metadata should persist correctly across database sessions.
    ///
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_operation_metadata_persistence(
        operation_type in operation_type_strategy(),
        description in description_strategy(),
        intent in intent_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db.clone(), None);

            let commands = vec![json!({"action": "test"})];
            let inverse_commands = vec![json!({"action": "undo_test"})];

            // Record operation
            let op_id = timeline
                .record_operation_with_intent(
                    operation_type.clone(),
                    description.clone(),
                    commands.clone(),
                    inverse_commands.clone(),
                    vec![],
                    intent.clone(),
                )
                .await
                .unwrap();

            // Simulate session restart by creating new timeline with same database
            let db2 = timeline.get_db().clone();
            let _timeline2 = OperationTimeline::new(db2.clone(), None);

            // Load operation from new timeline
            let operation = db2.load_operation(&op_id).await.unwrap();

            // Property: All metadata should be preserved across sessions
            prop_assert_eq!(operation.operation_type, operation_type);
            prop_assert_eq!(operation.description, description);
            prop_assert_eq!(operation.intent, intent);
            prop_assert_eq!(operation.commands, commands);
            prop_assert_eq!(operation.inverse_commands, inverse_commands);
            prop_assert!(operation.timestamp > 0);

            Ok(())
        });
        result.unwrap();
    }

    /// **Property 26 (variant): Empty Commands Handling**
    ///
    /// Operations with empty command lists should still have all other metadata.
    ///
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_empty_commands_metadata_completeness(
        operation_type in operation_type_strategy(),
        description in description_strategy(),
        intent in intent_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut timeline = OperationTimeline::new(db.clone(), None);

            // Record operation with empty commands
            let op_id = timeline
                .record_operation_with_intent(
                    operation_type.clone(),
                    description.clone(),
                    vec![], // Empty commands
                    vec![], // Empty inverse commands
                    vec![],
                    intent.clone(),
                )
                .await
                .unwrap();

            let operation = db.load_operation(&op_id).await.unwrap();

            // Property: All metadata should be present even with empty commands
            prop_assert_eq!(operation.operation_type, operation_type);
            prop_assert_eq!(operation.description, description);
            prop_assert_eq!(operation.intent, intent);
            prop_assert_eq!(operation.commands.len(), 0);
            prop_assert_eq!(operation.inverse_commands.len(), 0);
            prop_assert!(operation.timestamp > 0);
            prop_assert_eq!(operation.branch, Some("main".to_string()));

            Ok(())
        });
        result.unwrap();
    }

    /// **Property 26 (variant): Branch-Specific Metadata**
    ///
    /// Operations on different branches should maintain correct branch metadata.
    ///
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_branch_specific_metadata(
        branch_count in 2usize..5,
        ops_per_branch in 2usize..10
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let mut recorded_ops = Vec::new();

            // Create operations on different branches
            for branch_idx in 0..branch_count {
                let branch_name = format!("branch{}", branch_idx);
                let mut timeline = OperationTimeline::new(db.clone(), Some(branch_name.clone()));

                for op_idx in 0..ops_per_branch {
                    let op_id = timeline
                        .record_operation(
                            format!("Op{}_{}", branch_idx, op_idx),
                            format!("Operation {} on branch {}", op_idx, branch_name),
                            vec![json!({"branch": branch_idx, "op": op_idx})],
                            vec![],
                            vec![],
                        )
                        .await
                        .unwrap();

                    recorded_ops.push((op_id, branch_name.clone()));
                }
            }

            // Verify each operation has correct branch metadata
            for (op_id, expected_branch) in recorded_ops {
                let operation = db.load_operation(&op_id).await.unwrap();

                prop_assert_eq!(
                    operation.branch,
                    Some(expected_branch.clone()),
                    "Operation should have correct branch metadata"
                );
                prop_assert!(operation.timestamp > 0);
                prop_assert!(!operation.operation_type.is_empty());
                prop_assert!(!operation.description.is_empty());
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
async fn test_minimal_operation_metadata() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record operation with minimal metadata (no intent, no affected entities)
    let op_id = timeline
        .record_operation(
            "MinimalOp",
            "Minimal operation",
            vec![json!({"action": "test"})],
            vec![json!({"action": "undo"})],
            vec![],
        )
        .await
        .unwrap();

    let operation = db.load_operation(&op_id).await.unwrap();

    // All required fields should still be present
    assert_eq!(operation.operation_type, "MinimalOp");
    assert_eq!(operation.description, "Minimal operation");
    assert_eq!(operation.commands.len(), 1);
    assert_eq!(operation.inverse_commands.len(), 1);
    assert!(operation.timestamp > 0);
    assert_eq!(operation.branch, Some("main".to_string()));
    assert_eq!(operation.intent, None);
    assert_eq!(operation.affected_entities.len(), 0);
}

#[tokio::test]
async fn test_maximal_operation_metadata() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), Some("feature".to_string()));

    // Create multiple affected entities
    let mut affected_entities = Vec::new();
    for i in 0..10 {
        let entity_id = db
            .store_entity(luminara_db::EntityRecord::new(Some(format!("Entity{}", i))))
            .await
            .unwrap();
        affected_entities.push(entity_id);
    }

    // Record operation with maximal metadata
    let op_id = timeline
        .record_operation_with_intent(
            "MaximalOp",
            "Operation with all possible metadata fields populated",
            vec![
                json!({"action": "step1"}),
                json!({"action": "step2"}),
                json!({"action": "step3"}),
            ],
            vec![
                json!({"action": "undo_step3"}),
                json!({"action": "undo_step2"}),
                json!({"action": "undo_step1"}),
            ],
            affected_entities.clone(),
            Some("This is a detailed AI intent explaining the purpose of this operation".to_string()),
        )
        .await
        .unwrap();

    let operation = db.load_operation(&op_id).await.unwrap();

    // Verify all fields are present and correct
    assert_eq!(operation.operation_type, "MaximalOp");
    assert_eq!(
        operation.description,
        "Operation with all possible metadata fields populated"
    );
    assert_eq!(operation.commands.len(), 3);
    assert_eq!(operation.inverse_commands.len(), 3);
    assert!(operation.timestamp > 0);
    assert_eq!(operation.branch, Some("feature".to_string()));
    assert_eq!(
        operation.intent,
        Some("This is a detailed AI intent explaining the purpose of this operation".to_string())
    );
    assert_eq!(operation.affected_entities.len(), 10);
}

#[tokio::test]
async fn test_operation_metadata_with_special_characters() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Test with special characters in metadata
    let op_id = timeline
        .record_operation_with_intent(
            "Special<>Op",
            "Description with \"quotes\" and 'apostrophes' and \n newlines",
            vec![json!({"text": "Command with special chars: <>&\"'"})],
            vec![json!({"text": "Inverse with unicode: 日本語 🎮"})],
            vec![],
            Some("Intent with special chars: @#$%^&*()".to_string()),
        )
        .await
        .unwrap();

    let operation = db.load_operation(&op_id).await.unwrap();

    // All special characters should be preserved
    assert_eq!(operation.operation_type, "Special<>Op");
    assert!(operation.description.contains("\"quotes\""));
    assert!(operation.description.contains("'apostrophes'"));
    assert_eq!(
        operation.commands[0]["text"],
        "Command with special chars: <>&\"'"
    );
    assert_eq!(
        operation.inverse_commands[0]["text"],
        "Inverse with unicode: 日本語 🎮"
    );
    assert_eq!(
        operation.intent,
        Some("Intent with special chars: @#$%^&*()".to_string())
    );
}

#[tokio::test]
async fn test_operation_metadata_timestamp_ordering() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    let mut op_ids = Vec::new();

    // Record multiple operations with delays
    for i in 0..5 {
        let op_id = timeline
            .record_operation(
                format!("Op{}", i),
                format!("Operation {}", i),
                vec![],
                vec![],
                vec![],
            )
            .await
            .unwrap();
        op_ids.push(op_id);

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Load operations and verify timestamps are ordered
    let mut prev_timestamp = 0;
    for op_id in op_ids {
        let operation = db.load_operation(&op_id).await.unwrap();
        assert!(
            operation.timestamp >= prev_timestamp,
            "Timestamps should be monotonically increasing"
        );
        prev_timestamp = operation.timestamp;
    }
}

#[tokio::test]
async fn test_operation_metadata_with_complex_commands() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Test with complex nested command structures
    let complex_command = json!({
        "type": "batch",
        "operations": [
            {
                "action": "spawn",
                "entity": {
                    "name": "Player",
                    "components": [
                        {"type": "Transform", "position": [0.0, 0.0, 0.0]},
                        {"type": "Health", "value": 100}
                    ]
                }
            },
            {
                "action": "modify",
                "target": "enemy_1",
                "changes": {
                    "health": -10,
                    "status": "damaged"
                }
            }
        ]
    });

    let op_id = timeline
        .record_operation(
            "ComplexOp",
            "Complex operation with nested structures",
            vec![complex_command.clone()],
            vec![json!({"action": "rollback"})],
            vec![],
        )
        .await
        .unwrap();

    let operation = db.load_operation(&op_id).await.unwrap();

    // Complex command structure should be preserved exactly
    assert_eq!(operation.commands[0], complex_command);
    assert_eq!(operation.commands[0]["type"], "batch");
    assert_eq!(operation.commands[0]["operations"][0]["action"], "spawn");
    assert_eq!(
        operation.commands[0]["operations"][0]["entity"]["name"],
        "Player"
    );
}

#[tokio::test]
async fn test_operation_metadata_without_intent() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record operation without intent (using record_operation instead of record_operation_with_intent)
    let op_id = timeline
        .record_operation(
            "NoIntentOp",
            "Operation without AI intent",
            vec![json!({"action": "test"})],
            vec![json!({"action": "undo"})],
            vec![],
        )
        .await
        .unwrap();

    let operation = db.load_operation(&op_id).await.unwrap();

    // Intent should be None, but all other fields should be present
    assert_eq!(operation.operation_type, "NoIntentOp");
    assert_eq!(operation.description, "Operation without AI intent");
    assert_eq!(operation.commands.len(), 1);
    assert_eq!(operation.inverse_commands.len(), 1);
    assert!(operation.timestamp > 0);
    assert_eq!(operation.branch, Some("main".to_string()));
    assert_eq!(operation.intent, None);
}

#[tokio::test]
async fn test_operation_metadata_in_history() {
    // **Validates: Requirements 27.1**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let mut timeline = OperationTimeline::new(db.clone(), None);

    // Record multiple operations
    for i in 0..10 {
        timeline
            .record_operation_with_intent(
                format!("Op{}", i),
                format!("Description {}", i),
                vec![json!({"index": i})],
                vec![json!({"index": i})],
                vec![],
                Some(format!("Intent {}", i)),
            )
            .await
            .unwrap();
    }

    // Get history and verify all operations have complete metadata
    let history = timeline.get_history(100).await.unwrap();
    assert_eq!(history.len(), 10);

    // Verify each operation has complete metadata (don't check order since timestamps may be same)
    for operation in &history {
        // Operation type should match pattern
        assert!(operation.operation_type.starts_with("Op"));
        
        // Description should be non-empty
        assert!(!operation.description.is_empty());
        assert!(operation.description.starts_with("Description"));
        
        // Commands and inverse commands should be present
        assert_eq!(operation.commands.len(), 1);
        assert_eq!(operation.inverse_commands.len(), 1);
        
        // Timestamp should be valid
        assert!(operation.timestamp > 0);
        
        // Branch should be set
        assert_eq!(operation.branch, Some("main".to_string()));
        
        // Intent should be present
        assert!(operation.intent.is_some());
        assert!(operation.intent.as_ref().unwrap().starts_with("Intent"));
    }
    
    // Verify all 10 unique operations are present
    let mut op_types: Vec<_> = history.iter().map(|op| op.operation_type.clone()).collect();
    op_types.sort();
    let expected: Vec<_> = (0..10).map(|i| format!("Op{}", i)).collect();
    assert_eq!(op_types, expected);
}
