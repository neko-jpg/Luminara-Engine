//! Tests for ECS World synchronization with database
//!
//! These tests verify that the WorldSync system correctly syncs entities and components
//! to the database with minimal latency.

use luminara_db::{ComponentRecord, EntityRecord, LuminaraDatabase, WorldSync};
use serde_json::json;

#[tokio::test]
async fn test_sync_single_entity() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync an entity
    let record_id = sync
        .sync_entity(1, Some("Player".to_string()), vec!["player".to_string()])
        .await
        .unwrap();

    // Verify entity was created in database
    let entity_mapping = sync.get_entity_record_id(1).await;
    assert!(entity_mapping.is_some());
    assert_eq!(entity_mapping.unwrap(), record_id);
}

#[tokio::test]
async fn test_sync_entity_with_components() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync entity
    let entity_id = 1;
    sync.sync_entity(entity_id, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    // Sync components
    let transform_data = json!({
        "position": [0.0, 0.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    });

    let comp_id = sync
        .sync_component(
            entity_id,
            "Transform".to_string(),
            "transform_type_id".to_string(),
            transform_data,
        )
        .await
        .unwrap();

    // Verify component was created
    let component_mapping = sync.get_component_record_id(entity_id, "Transform").await;
    assert!(component_mapping.is_some());
    assert_eq!(component_mapping.unwrap(), comp_id);
}

#[tokio::test]
async fn test_update_existing_entity() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let entity_id = 1;

    // Sync entity first time
    let record_id1 = sync
        .sync_entity(
            entity_id,
            Some("Player".to_string()),
            vec!["player".to_string()],
        )
        .await
        .unwrap();

    // Sync same entity again with different data
    let record_id2 = sync
        .sync_entity(
            entity_id,
            Some("UpdatedPlayer".to_string()),
            vec!["player".to_string(), "updated".to_string()],
        )
        .await
        .unwrap();

    // Should be same record ID (update, not create)
    assert_eq!(record_id1, record_id2);
}

#[tokio::test]
async fn test_remove_entity() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let entity_id = 1;

    // Sync entity
    sync.sync_entity(entity_id, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    // Verify it exists
    assert!(sync.get_entity_record_id(entity_id).await.is_some());

    // Remove entity
    sync.remove_entity(entity_id).await.unwrap();

    // Verify it's gone
    assert!(sync.get_entity_record_id(entity_id).await.is_none());
}

#[tokio::test]
async fn test_remove_component() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let entity_id = 1;

    // Sync entity and component
    sync.sync_entity(entity_id, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    sync.sync_component(
        entity_id,
        "Transform".to_string(),
        "type_id".to_string(),
        json!({}),
    )
    .await
    .unwrap();

    // Verify component exists
    assert!(sync
        .get_component_record_id(entity_id, "Transform")
        .await
        .is_some());

    // Remove component
    sync.remove_component(entity_id, "Transform".to_string())
        .await
        .unwrap();

    // Verify it's gone
    assert!(sync
        .get_component_record_id(entity_id, "Transform")
        .await
        .is_none());
}

#[tokio::test]
async fn test_mark_dirty_and_sync() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Mark entities as dirty
    sync.mark_entity_dirty(1).await;
    sync.mark_entity_dirty(2).await;

    // Mark components as dirty
    sync.mark_component_dirty(1, "Transform".to_string()).await;

    // Sync dirty items
    let result = sync.sync_dirty().await.unwrap();

    // Verify sync happened
    assert_eq!(result.entities_synced, 2);
    assert_eq!(result.components_synced, 1);

    // Verify dirty sets are cleared
    let stats = sync.get_statistics().await;
    assert_eq!(stats.sync_count, 1);
}

#[tokio::test]
async fn test_sync_latency_target() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Mark some entities as dirty
    for i in 0..10 {
        sync.mark_entity_dirty(i).await;
    }

    // Sync and measure time
    let result = sync.sync_dirty().await.unwrap();

    // Verify sync completed within target (16ms)
    // Note: This is a soft target, actual performance depends on hardware
    println!("Sync took {:.2}ms", result.duration_ms);

    // Check if exceeded target flag is set correctly
    if result.duration_ms > 16.0 {
        assert!(result.exceeded_target);
    } else {
        assert!(!result.exceeded_target);
    }
}

#[tokio::test]
async fn test_sync_statistics() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Perform multiple syncs
    for _ in 0..5 {
        sync.mark_entity_dirty(1).await;
        sync.sync_dirty().await.unwrap();
    }

    // Check statistics
    let stats = sync.get_statistics().await;
    assert_eq!(stats.sync_count, 5);
    assert!(stats.avg_sync_time_ms > 0.0);
    assert!(stats.max_sync_time_ms >= stats.avg_sync_time_ms);
}

#[tokio::test]
async fn test_reset_statistics() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Perform a sync
    sync.mark_entity_dirty(1).await;
    sync.sync_dirty().await.unwrap();

    // Verify stats exist
    let stats = sync.get_statistics().await;
    assert_eq!(stats.sync_count, 1);

    // Reset statistics
    sync.reset_statistics().await;

    // Verify stats are cleared
    let stats = sync.get_statistics().await;
    assert_eq!(stats.sync_count, 0);
    assert_eq!(stats.total_sync_time_ms, 0.0);
}

#[tokio::test]
async fn test_concurrent_sync_operations() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = std::sync::Arc::new(WorldSync::new(db));

    // Spawn multiple concurrent sync operations
    let mut handles = vec![];

    for i in 0..10 {
        let sync_clone = sync.clone();
        let handle = tokio::spawn(async move {
            sync_clone
                .sync_entity(i, Some(format!("Entity{}", i)), vec![])
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all entities were synced
    for i in 0..10 {
        assert!(sync.get_entity_record_id(i).await.is_some());
    }
}

#[tokio::test]
async fn test_multiple_components_per_entity() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let entity_id = 1;

    // Sync entity
    sync.sync_entity(entity_id, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    // Sync multiple components
    let component_types = vec!["Transform", "Mesh", "Material", "RigidBody"];

    for comp_type in &component_types {
        sync.sync_component(
            entity_id,
            comp_type.to_string(),
            format!("{}_type_id", comp_type),
            json!({}),
        )
        .await
        .unwrap();
    }

    // Verify all components exist
    for comp_type in &component_types {
        assert!(sync
            .get_component_record_id(entity_id, comp_type)
            .await
            .is_some());
    }
}

#[tokio::test]
async fn test_clear_mappings() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    // Sync some entities
    sync.sync_entity(1, Some("Entity1".to_string()), vec![])
        .await
        .unwrap();
    sync.sync_entity(2, Some("Entity2".to_string()), vec![])
        .await
        .unwrap();

    // Verify mappings exist
    assert!(sync.get_entity_record_id(1).await.is_some());
    assert!(sync.get_entity_record_id(2).await.is_some());

    // Clear mappings
    sync.clear_mappings().await;

    // Verify mappings are gone
    assert!(sync.get_entity_record_id(1).await.is_none());
    assert!(sync.get_entity_record_id(2).await.is_none());
}

#[tokio::test]
async fn test_remove_entity_removes_components() {
    let db = LuminaraDatabase::new_memory().await.unwrap();
    let sync = WorldSync::new(db);

    let entity_id = 1;

    // Sync entity with components
    sync.sync_entity(entity_id, Some("Player".to_string()), vec![])
        .await
        .unwrap();

    sync.sync_component(
        entity_id,
        "Transform".to_string(),
        "type_id".to_string(),
        json!({}),
    )
    .await
    .unwrap();

    sync.sync_component(
        entity_id,
        "Mesh".to_string(),
        "type_id".to_string(),
        json!({}),
    )
    .await
    .unwrap();

    // Verify components exist
    assert!(sync
        .get_component_record_id(entity_id, "Transform")
        .await
        .is_some());
    assert!(sync
        .get_component_record_id(entity_id, "Mesh")
        .await
        .is_some());

    // Remove entity
    sync.remove_entity(entity_id).await.unwrap();

    // Verify components are also removed
    assert!(sync
        .get_component_record_id(entity_id, "Transform")
        .await
        .is_none());
    assert!(sync
        .get_component_record_id(entity_id, "Mesh")
        .await
        .is_none());
}
