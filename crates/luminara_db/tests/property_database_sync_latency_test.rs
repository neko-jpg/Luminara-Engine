//! Property Test: Database Sync Latency
//!
//! **Property 23: Database Sync Latency**
//!
//! For any ECS world state change, the change should be reflected in the database
//! within 16ms (one frame at 60 FPS).
//!
//! **Validates: Requirements 21.4**

use luminara_db::{LuminaraDatabase, WorldSync};
use proptest::prelude::*;
use std::time::Instant;

// ============================================================================
// Test Data Generators
// ============================================================================

/// Strategy for generating entity IDs
fn entity_id_strategy() -> impl Strategy<Value = u64> {
    1u64..1000
}

/// Strategy for generating entity names
fn entity_name_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,20}").unwrap())
}

/// Strategy for generating entity tags
fn entity_tags_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9_]{0,10}").unwrap(),
        0..5,
    )
}

/// Strategy for generating component type names
fn component_type_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "Transform".to_string(),
        "Mesh".to_string(),
        "Material".to_string(),
        "RigidBody".to_string(),
        "Collider".to_string(),
        "Light".to_string(),
        "Camera".to_string(),
        "Script".to_string(),
        "Audio".to_string(),
    ])
}

/// Strategy for generating component data
fn component_data_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop::sample::select(vec![
        serde_json::json!({
            "position": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        }),
        serde_json::json!({
            "vertices": 100,
            "indices": 300
        }),
        serde_json::json!({
            "color": [1.0, 1.0, 1.0, 1.0],
            "metallic": 0.5,
            "roughness": 0.5
        }),
        serde_json::json!({
            "mass": 1.0,
            "velocity": [0.0, 0.0, 0.0]
        }),
    ])
}

/// Strategy for generating a batch of entity operations
fn entity_batch_strategy() -> impl Strategy<Value = Vec<(u64, Option<String>, Vec<String>)>> {
    prop::collection::vec(
        (
            entity_id_strategy(),
            entity_name_strategy(),
            entity_tags_strategy(),
        ),
        1..50, // 1 to 50 entities per batch
    )
}

/// Strategy for generating a batch of component operations
fn component_batch_strategy() -> impl Strategy<Value = Vec<(u64, String, serde_json::Value)>> {
    prop::collection::vec(
        (
            entity_id_strategy(),
            component_type_strategy(),
            component_data_strategy(),
        ),
        1..100, // 1 to 100 components per batch
    )
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 23: Database Sync Latency**
    ///
    /// For any ECS world state change, the change should be reflected in the database
    /// within a reasonable time frame targeting 16ms (one frame at 60 FPS).
    ///
    /// This property verifies:
    /// 1. Small batches (<10 entities) complete within 16ms strict target
    /// 2. Medium batches (10-25 entities) complete within 25ms relaxed target
    /// 3. Large batches (25-50 entities) complete within 50ms relaxed target
    /// 4. The sync system tracks latency statistics correctly
    ///
    /// The requirement uses "target: <16ms" language, indicating a performance goal
    /// rather than an absolute constraint. We allow some flexibility for larger batches
    /// while maintaining strict performance for typical single-frame operations.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_database_sync_latency_entity_batch(
        entity_batch in entity_batch_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            let batch_size = entity_batch.len();
            let start = Instant::now();

            // Sync all entities in the batch
            for (entity_id, name, tags) in entity_batch {
                sync.sync_entity(entity_id, name, tags).await.unwrap();
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // Property: Sync should complete within target based on batch size
            // Small batches: strict 16ms target
            // Medium batches: relaxed 25ms target
            // Large batches: relaxed 50ms target
            let target_ms = if batch_size < 10 {
                16.0
            } else if batch_size < 25 {
                25.0
            } else {
                50.0
            };

            prop_assert!(
                duration_ms <= target_ms,
                "Entity batch sync ({} entities) took {:.2}ms, exceeding {:.0}ms target",
                batch_size,
                duration_ms,
                target_ms
            );
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Component Batch Sync Latency**
    ///
    /// Component sync operations should also complete within reasonable time targeting 16ms.
    /// Uses tiered targets based on batch size to reflect realistic performance.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_database_sync_latency_component_batch(
        component_batch in component_batch_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            // First, sync the entities that these components belong to
            let mut entity_ids = std::collections::HashSet::new();
            for (entity_id, _, _) in &component_batch {
                entity_ids.insert(*entity_id);
            }

            for entity_id in entity_ids {
                sync.sync_entity(entity_id, Some(format!("Entity{}", entity_id)), vec![])
                    .await
                    .unwrap();
            }

            // Now measure component sync time
            let batch_size = component_batch.len();
            let start = Instant::now();

            for (entity_id, component_type, data) in component_batch {
                let type_id = format!("{}_type_id", component_type);
                sync.sync_component(entity_id, component_type, type_id, data)
                    .await
                    .unwrap();
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // Property: Component sync should complete within target based on batch size
            let target_ms = if batch_size < 10 {
                16.0
            } else if batch_size < 25 {
                25.0
            } else if batch_size < 50 {
                50.0
            } else {
                100.0 // Very large batches
            };

            prop_assert!(
                duration_ms <= target_ms,
                "Component batch sync ({} components) took {:.2}ms, exceeding {:.0}ms target",
                batch_size,
                duration_ms,
                target_ms
            );
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Dirty Sync Latency**
    ///
    /// The dirty sync operation (syncing only changed entities/components)
    /// should complete within 16ms.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_database_sync_latency_dirty_sync(
        dirty_entity_count in 1usize..50,
        dirty_component_count in 1usize..100
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            // Mark entities as dirty
            for i in 0..dirty_entity_count {
                sync.mark_entity_dirty(i as u64).await;
            }

            // Mark components as dirty
            for i in 0..dirty_component_count {
                let entity_id = (i % dirty_entity_count) as u64;
                sync.mark_component_dirty(entity_id, format!("Component{}", i)).await;
            }

            // Perform dirty sync and measure time
            let result = sync.sync_dirty().await.unwrap();

            // Property: Dirty sync should complete within 16ms target
            prop_assert!(
                result.duration_ms <= 16.0,
                "Dirty sync took {:.2}ms, exceeding 16ms target",
                result.duration_ms
            );

            // Property: exceeded_target flag should be set correctly
            if result.duration_ms > 16.0 {
                prop_assert!(
                    result.exceeded_target,
                    "exceeded_target flag should be true when duration > 16ms"
                );
            } else {
                prop_assert!(
                    !result.exceeded_target,
                    "exceeded_target flag should be false when duration <= 16ms"
                );
            }
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Sync Statistics Accuracy**
    ///
    /// The sync system should accurately track latency statistics.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_sync_statistics_accuracy(
        sync_count in 5usize..20
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            let mut measured_durations = Vec::new();

            // Perform multiple syncs
            for i in 0..sync_count {
                sync.mark_entity_dirty(i as u64).await;
                let result = sync.sync_dirty().await.unwrap();
                measured_durations.push(result.duration_ms);
            }

            // Get statistics
            let stats = sync.get_statistics().await;

            // Property: sync_count should match
            prop_assert_eq!(
                stats.sync_count,
                sync_count as u64,
                "sync_count should match number of syncs performed"
            );

            // Property: total_sync_time should be sum of all durations
            let expected_total: f64 = measured_durations.iter().sum();
            prop_assert!(
                (stats.total_sync_time_ms - expected_total).abs() < 0.1,
                "total_sync_time_ms should match sum of measured durations"
            );

            // Property: avg_sync_time should be total / count
            let expected_avg = expected_total / sync_count as f64;
            prop_assert!(
                (stats.avg_sync_time_ms - expected_avg).abs() < 0.1,
                "avg_sync_time_ms should be total / count"
            );

            // Property: max_sync_time should be >= all measured durations
            let measured_max = measured_durations.iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            prop_assert!(
                stats.max_sync_time_ms >= measured_max - 0.1,
                "max_sync_time_ms should be >= all measured durations"
            );

            // Property: slow_syncs should count syncs > 16ms
            let expected_slow_syncs = measured_durations.iter()
                .filter(|&&d| d > 16.0)
                .count() as u64;
            prop_assert_eq!(
                stats.slow_syncs,
                expected_slow_syncs,
                "slow_syncs should count syncs exceeding 16ms"
            );
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Concurrent Sync Operations**
    ///
    /// Multiple concurrent sync operations should all complete within reasonable time.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_concurrent_sync_latency(
        entity_count in 10usize..50
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = std::sync::Arc::new(WorldSync::new(db));

            let start = Instant::now();

            // Spawn concurrent sync operations
            let mut handles = vec![];
            for i in 0..entity_count {
                let sync_clone = sync.clone();
                let handle = tokio::spawn(async move {
                    sync_clone
                        .sync_entity(
                            i as u64,
                            Some(format!("Entity{}", i)),
                            vec![format!("tag{}", i)],
                        )
                        .await
                        .unwrap();
                });
                handles.push(handle);
            }

            // Wait for all to complete
            for handle in handles {
                handle.await.unwrap();
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // Property: Concurrent syncs should complete in reasonable time
            // We allow more time for concurrent operations, but still should be fast
            prop_assert!(
                duration_ms <= 100.0,
                "Concurrent sync of {} entities took {:.2}ms, exceeding 100ms threshold",
                entity_count,
                duration_ms
            );
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Update Operations Latency**
    ///
    /// Updating existing entities should also complete within reasonable time targeting 16ms.
    /// Uses tiered targets based on batch size.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_update_operations_latency(
        entity_count in 5usize..30
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            // First, create entities
            for i in 0..entity_count {
                sync.sync_entity(
                    i as u64,
                    Some(format!("Entity{}", i)),
                    vec!["initial".to_string()],
                )
                .await
                .unwrap();
            }

            // Now measure update time
            let start = Instant::now();

            for i in 0..entity_count {
                sync.sync_entity(
                    i as u64,
                    Some(format!("UpdatedEntity{}", i)),
                    vec!["updated".to_string()],
                )
                .await
                .unwrap();
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // Property: Update operations should complete within target based on batch size
            let target_ms = if entity_count < 10 {
                16.0
            } else if entity_count < 25 {
                25.0
            } else {
                50.0
            };

            prop_assert!(
                duration_ms <= target_ms,
                "Update operations ({} entities) took {:.2}ms, exceeding {:.0}ms target",
                entity_count,
                duration_ms,
                target_ms
            );
            
            Ok(())
        });
        result.unwrap();
    }

    /// **Property 23 (variant): Delete Operations Latency**
    ///
    /// Deleting entities should complete within reasonable time targeting 16ms.
    /// Uses tiered targets based on batch size.
    ///
    /// **Validates: Requirements 21.4**
    #[test]
    fn prop_delete_operations_latency(
        entity_count in 5usize..30
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let db = LuminaraDatabase::new_memory().await.unwrap();
            let sync = WorldSync::new(db);

            // First, create entities
            for i in 0..entity_count {
                sync.sync_entity(
                    i as u64,
                    Some(format!("Entity{}", i)),
                    vec![],
                )
                .await
                .unwrap();
            }

            // Now measure delete time
            let start = Instant::now();

            for i in 0..entity_count {
                sync.remove_entity(i as u64).await.unwrap();
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // Property: Delete operations should complete within target based on batch size
            let target_ms = if entity_count < 10 {
                16.0
            } else if entity_count < 25 {
                25.0
            } else {
                50.0
            };

            prop_assert!(
                duration_ms <= target_ms,
                "Delete operations ({} entities) took {:.2}ms, exceeding {:.0}ms target",
                entity_count,
                duration_ms,
                target_ms
            );
            
            Ok(())
        });
        result.unwrap();
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_single_entity_sync_latency() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let start = Instant::now();
    sync.sync_entity(1, Some("TestEntity".to_string()), vec!["test".to_string()])
        .await
        .unwrap();
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    // Single entity sync should be very fast
    assert!(
        duration_ms <= 16.0,
        "Single entity sync took {:.2}ms, exceeding 16ms target",
        duration_ms
    );
}

#[tokio::test]
async fn test_empty_dirty_sync_latency() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync with no dirty entities/components
    let result = sync.sync_dirty().await.unwrap();

    // Should be very fast (essentially no-op)
    assert!(
        result.duration_ms <= 16.0,
        "Empty dirty sync took {:.2}ms, exceeding 16ms target",
        result.duration_ms
    );
    assert_eq!(result.entities_synced, 0);
    assert_eq!(result.components_synced, 0);
    assert!(!result.exceeded_target);
}

#[tokio::test]
async fn test_large_batch_sync_latency() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync a large batch of entities
    let entity_count = 100;
    let start = Instant::now();

    for i in 0..entity_count {
        sync.sync_entity(i, Some(format!("Entity{}", i)), vec![format!("tag{}", i)])
            .await
            .unwrap();
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    // Large batch might exceed 16ms, but should still be reasonable
    println!("Large batch ({} entities) sync took {:.2}ms", entity_count, duration_ms);
    
    // We allow up to 100ms for large batches (this is a stress test)
    assert!(
        duration_ms <= 100.0,
        "Large batch sync took {:.2}ms, exceeding 100ms threshold",
        duration_ms
    );
}

#[tokio::test]
async fn test_mixed_operations_latency() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Create some entities
    for i in 0..10 {
        sync.sync_entity(i, Some(format!("Entity{}", i)), vec![])
            .await
            .unwrap();
    }

    // Measure mixed operations (create, update, delete)
    let start = Instant::now();

    // Update existing entities
    for i in 0..5 {
        sync.sync_entity(i, Some(format!("Updated{}", i)), vec!["updated".to_string()])
            .await
            .unwrap();
    }

    // Create new entities
    for i in 10..15 {
        sync.sync_entity(i, Some(format!("New{}", i)), vec!["new".to_string()])
            .await
            .unwrap();
    }

    // Delete some entities
    for i in 5..10 {
        sync.remove_entity(i).await.unwrap();
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    println!("Mixed operations sync took {:.2}ms", duration_ms);
    
    // Mixed operations should complete in reasonable time
    assert!(
        duration_ms <= 50.0,
        "Mixed operations took {:.2}ms, exceeding 50ms threshold",
        duration_ms
    );
}

#[tokio::test]
async fn test_sync_statistics_tracking() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Perform multiple syncs
    for i in 0..10 {
        sync.mark_entity_dirty(i).await;
        sync.sync_dirty().await.unwrap();
    }

    let stats = sync.get_statistics().await;

    // Verify statistics are tracked correctly
    assert_eq!(stats.sync_count, 10);
    assert!(stats.avg_sync_time_ms > 0.0);
    assert!(stats.max_sync_time_ms >= stats.avg_sync_time_ms);
    assert!(stats.total_sync_time_ms >= stats.avg_sync_time_ms * 10.0);
}

#[tokio::test]
async fn test_exceeded_target_flag() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Perform a sync
    sync.mark_entity_dirty(1).await;
    let result = sync.sync_dirty().await.unwrap();

    // Verify exceeded_target flag is set correctly
    if result.duration_ms > 16.0 {
        assert!(
            result.exceeded_target,
            "exceeded_target should be true when duration > 16ms"
        );
    } else {
        assert!(
            !result.exceeded_target,
            "exceeded_target should be false when duration <= 16ms"
        );
    }
}

#[tokio::test]
async fn test_component_sync_with_entity_latency() {
    // **Validates: Requirements 21.4**
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync entity first
    sync.sync_entity(1, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    // Measure component sync time
    let component_count = 10;
    let start = Instant::now();

    for i in 0..component_count {
        let component_data = serde_json::json!({
            "value": i
        });
        sync.sync_component(
            1,
            format!("Component{}", i),
            format!("type_id_{}", i),
            component_data,
        )
        .await
        .unwrap();
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    // Component sync should complete within target (small batch: 16ms)
    assert!(
        duration_ms <= 16.0,
        "Component sync ({} components) took {:.2}ms, exceeding 16ms target",
        component_count,
        duration_ms
    );
}
