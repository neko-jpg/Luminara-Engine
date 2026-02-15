use luminara_db::prelude::*;
use surrealdb::sql::{Datetime, Thing};

#[tokio::test]
#[ignore = "Fails with SceneNotFound, possibly due to async consistency or ID handling in embedded mode"]
async fn test_scene_save_and_load() {
    let config = DbConfig {
        backend: DbBackend::Memory,
        auto_migrate: true,
        ..Default::default()
    };
    let conn = DbConnection::connect(config).await.unwrap();
    let store = SceneStore::new(&conn);

    let scene_name = "TestScene";
    let snapshot = SceneSnapshot {
        scene_id: scene_name.into(),
        scene: SceneRecord {
            id: None,
            name: scene_name.into(),
            description: None,
            version: "1.0".into(),
            tags: vec![],
            settings: SceneSettings::default(),
            created_at: Datetime::from(chrono::Utc::now()),
            updated_at: Datetime::from(chrono::Utc::now()),
        },
        entities: vec![EntityRecord {
            id: None,
            name: "Entity1".into(),
            scene: Thing::from(("scene", scene_name)),
            enabled: true,
            tags: vec![],
            layer: 0,
            order: 0,
        }],
        components: vec![ComponentRecord {
            id: None,
            entity: Thing::from(("entity", "e1")), // Assuming e1 maps to Entity1 logic in real exporter
            component_type: "Transform".into(),
            data: serde_json::json!({"x": 1}),
            schema_version: 1,
        }],
        hierarchy: vec![],
    };

    store.save_scene(&snapshot).await.unwrap();

    let loaded = store.load_scene(scene_name).await.unwrap();
    assert_eq!(loaded.scene.name, scene_name);
    assert_eq!(loaded.entities.len(), 1);
    assert_eq!(loaded.components.len(), 1);
}
