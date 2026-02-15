//! Basic CRUD operation tests for the database

use luminara_db::{
    schema::AssetMetadata, AssetRecord, ComponentRecord, EntityRecord, LuminaraDatabase,
    OperationRecord,
};
use serde_json::json;

#[tokio::test]
async fn test_entity_crud() {
    // Create in-memory database
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create entity
    let entity = EntityRecord::new(Some("TestEntity".to_string()))
        .with_tag("test")
        .with_tag("player");

    let entity_id = db.store_entity(entity.clone()).await.unwrap();

    // Load entity
    let loaded = db.load_entity(&entity_id).await.unwrap();
    assert_eq!(loaded.name, Some("TestEntity".to_string()));
    assert_eq!(loaded.tags, vec!["test", "player"]);

    // Update entity
    let mut updated = loaded.clone();
    updated.name = Some("UpdatedEntity".to_string());
    db.update_entity(&entity_id, updated).await.unwrap();

    let loaded = db.load_entity(&entity_id).await.unwrap();
    assert_eq!(loaded.name, Some("UpdatedEntity".to_string()));

    // Delete entity
    db.delete_entity(&entity_id).await.unwrap();

    // Verify deletion
    let result = db.load_entity(&entity_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_component_crud() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create entity first
    let entity = EntityRecord::new(Some("TestEntity".to_string()));
    let entity_id = db.store_entity(entity).await.unwrap();

    // Create component
    let component_data = json!({
        "position": [0.0, 1.0, 2.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    });

    let component = ComponentRecord::new(
        "Transform",
        "luminara_scene::Transform",
        component_data,
        entity_id.clone(),
    );

    let component_id = db.store_component(component).await.unwrap();

    // Load component
    let loaded = db.load_component(&component_id).await.unwrap();
    assert_eq!(loaded.type_name, "Transform");
    assert_eq!(loaded.entity, entity_id);

    // Delete component
    db.delete_component(&component_id).await.unwrap();
}

#[tokio::test]
async fn test_asset_crud() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create asset
    let metadata = AssetMetadata {
        size_bytes: 1024,
        modified_timestamp: 1234567890,
        custom: json!({}),
    };

    let asset = AssetRecord::new("assets/textures/player.png", "Texture", "abc123", metadata);

    let asset_id = db.store_asset(asset).await.unwrap();

    // Load asset
    let loaded = db.load_asset(&asset_id).await.unwrap();
    assert_eq!(loaded.path, "assets/textures/player.png");
    assert_eq!(loaded.asset_type, "Texture");

    // Load by path
    let loaded_by_path = db
        .load_asset_by_path("assets/textures/player.png")
        .await
        .unwrap();
    assert_eq!(loaded_by_path.hash, "abc123");

    // Delete asset
    db.delete_asset(&asset_id).await.unwrap();
}

#[tokio::test]
async fn test_operation_crud() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create operation
    let commands = vec![json!({"type": "SpawnEntity", "name": "Player"})];
    let inverse_commands = vec![json!({"type": "DestroyEntity", "id": "entity:123"})];

    let operation = OperationRecord::new(
        "SpawnEntity",
        "Spawn player entity",
        commands,
        inverse_commands,
        1234567890,
    )
    .with_branch("main");

    let operation_id = db.store_operation(operation).await.unwrap();

    // Load operation
    let loaded = db.load_operation(&operation_id).await.unwrap();
    assert_eq!(loaded.operation_type, "SpawnEntity");
    assert_eq!(loaded.branch, Some("main".to_string()));

    // Load history
    let history = db.load_operation_history(10, Some("main")).await.unwrap();
    assert_eq!(history.len(), 1);

    // Delete operation
    db.delete_operation(&operation_id).await.unwrap();
}

#[tokio::test]
async fn test_query_entities() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create multiple entities
    let entity1 = EntityRecord::new(Some("Player".to_string())).with_tag("player");
    let entity2 = EntityRecord::new(Some("Enemy".to_string())).with_tag("enemy");
    let entity3 = EntityRecord::new(Some("Ally".to_string())).with_tag("player");

    db.store_entity(entity1).await.unwrap();
    db.store_entity(entity2).await.unwrap();
    db.store_entity(entity3).await.unwrap();

    // Query entities with "player" tag
    let players = db
        .query_entities("SELECT * FROM entity WHERE 'player' IN tags")
        .await
        .unwrap();
    assert_eq!(players.len(), 2);

    // Query entities with "enemy" tag
    let enemies = db
        .query_entities("SELECT * FROM entity WHERE 'enemy' IN tags")
        .await
        .unwrap();
    assert_eq!(enemies.len(), 1);
}

#[tokio::test]
async fn test_database_statistics() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create some data
    let entity = EntityRecord::new(Some("Test".to_string()));
    let entity_id = db.store_entity(entity).await.unwrap();

    let component = ComponentRecord::new("Transform", "test", json!({}), entity_id);
    db.store_component(component).await.unwrap();

    // Get statistics
    let stats = db.get_statistics().await.unwrap();
    assert_eq!(stats.entity_count, 1);
    assert_eq!(stats.component_count, 1);
}
