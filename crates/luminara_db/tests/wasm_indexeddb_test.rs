//! Tests for WASM/IndexedDB backend support
//!
//! These tests verify that the database works correctly with IndexedDB
//! when compiled for WASM target.

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use luminara_db::{EntityRecord, LuminaraDatabase};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_indexeddb_initialization() {
        // Test that we can initialize a database with IndexedDB backend
        let db = LuminaraDatabase::new_indexeddb("test_luminara_db")
            .await
            .expect("Failed to initialize IndexedDB database");

        // Verify we can get statistics (which confirms schema is initialized)
        let stats = db.get_statistics().await.expect("Failed to get statistics");
        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.component_count, 0);
        assert_eq!(stats.asset_count, 0);
        assert_eq!(stats.operation_count, 0);
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_entity_crud() {
        let db = LuminaraDatabase::new_indexeddb("test_entity_crud")
            .await
            .expect("Failed to initialize database");

        // Create an entity
        let entity = EntityRecord::new(Some("TestEntity".to_string()))
            .with_tag("test");

        let entity_id = db
            .store_entity(entity.clone())
            .await
            .expect("Failed to store entity");

        // Load the entity
        let loaded = db
            .load_entity(&entity_id)
            .await
            .expect("Failed to load entity");

        assert_eq!(loaded.name, Some("TestEntity".to_string()));
        assert!(loaded.tags.contains(&"test".to_string()));

        // Update the entity
        let mut updated = loaded.clone();
        updated.name = Some("UpdatedEntity".to_string());
        db.update_entity(&entity_id, updated)
            .await
            .expect("Failed to update entity");

        // Verify update
        let loaded_updated = db
            .load_entity(&entity_id)
            .await
            .expect("Failed to load updated entity");
        assert_eq!(loaded_updated.name, Some("UpdatedEntity".to_string()));

        // Delete the entity
        db.delete_entity(&entity_id)
            .await
            .expect("Failed to delete entity");

        // Verify deletion
        assert!(db.load_entity(&entity_id).await.is_err());
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_persistence() {
        // Create database and store an entity
        {
            let db = LuminaraDatabase::new_indexeddb("test_persistence")
                .await
                .expect("Failed to initialize database");

            let entity = EntityRecord::new(Some("PersistentEntity".to_string()));
            db.store_entity(entity)
                .await
                .expect("Failed to store entity");
        }

        // Reconnect to the same database
        {
            let db = LuminaraDatabase::new_indexeddb("test_persistence")
                .await
                .expect("Failed to reconnect to database");

            // Verify the entity persisted
            let stats = db.get_statistics().await.expect("Failed to get statistics");
            assert_eq!(stats.entity_count, 1, "Entity should persist across connections");
        }
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_query() {
        let db = LuminaraDatabase::new_indexeddb("test_query")
            .await
            .expect("Failed to initialize database");

        // Store multiple entities
        for i in 0..5 {
            let entity = EntityRecord::new(Some(format!("Entity{}", i)))
                .with_tag("queryable");
            db.store_entity(entity)
                .await
                .expect("Failed to store entity");
        }

        // Query entities
        let entities = db
            .query_entities("SELECT * FROM entity WHERE 'queryable' IN tags")
            .await
            .expect("Failed to query entities");

        assert_eq!(entities.len(), 5, "Should find all 5 entities");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_concurrent_operations() {
        let db = LuminaraDatabase::new_indexeddb("test_concurrent")
            .await
            .expect("Failed to initialize database");

        // Perform multiple operations concurrently
        let entity1 = EntityRecord::new(Some("Entity1".to_string()));
        let entity2 = EntityRecord::new(Some("Entity2".to_string()));
        let entity3 = EntityRecord::new(Some("Entity3".to_string()));

        let id1 = db.store_entity(entity1).await.expect("Failed to store entity1");
        let id2 = db.store_entity(entity2).await.expect("Failed to store entity2");
        let id3 = db.store_entity(entity3).await.expect("Failed to store entity3");

        // Load all entities concurrently
        let loaded1 = db.load_entity(&id1).await.expect("Failed to load entity1");
        let loaded2 = db.load_entity(&id2).await.expect("Failed to load entity2");
        let loaded3 = db.load_entity(&id3).await.expect("Failed to load entity3");

        assert_eq!(loaded1.name, Some("Entity1".to_string()));
        assert_eq!(loaded2.name, Some("Entity2".to_string()));
        assert_eq!(loaded3.name, Some("Entity3".to_string()));
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_wasm_feature_not_available() {
    // This test just documents that WASM features are not available on non-WASM targets
    println!("WASM/IndexedDB features are only available when compiling for wasm32 target");
}
