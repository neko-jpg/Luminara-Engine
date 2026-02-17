//! Tests for asset dependency tracking with graph traversal
//!
//! This test suite validates Requirement 21.3:
//! - Store asset relationships in graph
//! - Query asset dependencies with SurrealQL
//! - Support transitive dependency queries

use luminara_db::{schema::AssetMetadata, AssetRecord, ComponentRecord, EntityRecord, LuminaraDatabase};
use serde_json::json;

#[tokio::test]
async fn test_direct_asset_dependencies() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create a texture asset
    let texture = AssetRecord::new(
        "textures/albedo.png",
        "Texture",
        "hash123",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture_id = db.store_asset(texture).await.unwrap();

    // Create a material that depends on the texture
    let material = AssetRecord::new(
        "materials/metal.mat",
        "Material",
        "hash456",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture_id.clone());
    let material_id = db.store_asset(material).await.unwrap();

    // Find dependencies of the material
    let deps = db.find_asset_dependencies(&material_id).await.unwrap();

    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].path, "textures/albedo.png");
    assert_eq!(deps[0].asset_type, "Texture");
}

#[tokio::test]
async fn test_transitive_asset_dependencies() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create dependency chain: Mesh -> Material -> Texture
    let texture = AssetRecord::new(
        "textures/albedo.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture_id = db.store_asset(texture).await.unwrap();

    let material = AssetRecord::new(
        "materials/metal.mat",
        "Material",
        "hash2",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture_id.clone());
    let material_id = db.store_asset(material).await.unwrap();

    let mesh = AssetRecord::new(
        "meshes/cube.mesh",
        "Mesh",
        "hash3",
        AssetMetadata {
            size_bytes: 2048,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(material_id.clone());
    let mesh_id = db.store_asset(mesh).await.unwrap();

    // Find transitive dependencies of the mesh
    let deps = db
        .find_asset_dependencies_transitive(&mesh_id)
        .await
        .unwrap();

    // Should find both material and texture
    assert_eq!(deps.len(), 2);

    let paths: Vec<&str> = deps.iter().map(|a| a.path.as_str()).collect();
    assert!(paths.contains(&"materials/metal.mat"));
    assert!(paths.contains(&"textures/albedo.png"));
}

#[tokio::test]
async fn test_complex_dependency_graph() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create a complex dependency graph:
    // Material1 -> Texture1, Texture2
    // Material2 -> Texture2, Texture3
    // Mesh -> Material1, Material2

    let texture1 = AssetRecord::new(
        "textures/albedo.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture1_id = db.store_asset(texture1).await.unwrap();

    let texture2 = AssetRecord::new(
        "textures/normal.png",
        "Texture",
        "hash2",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture2_id = db.store_asset(texture2).await.unwrap();

    let texture3 = AssetRecord::new(
        "textures/roughness.png",
        "Texture",
        "hash3",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture3_id = db.store_asset(texture3).await.unwrap();

    let material1 = AssetRecord::new(
        "materials/metal.mat",
        "Material",
        "hash4",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture1_id.clone())
    .with_dependency(texture2_id.clone());
    let material1_id = db.store_asset(material1).await.unwrap();

    let material2 = AssetRecord::new(
        "materials/wood.mat",
        "Material",
        "hash5",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture2_id.clone())
    .with_dependency(texture3_id.clone());
    let material2_id = db.store_asset(material2).await.unwrap();

    let mesh = AssetRecord::new(
        "meshes/cube.mesh",
        "Mesh",
        "hash6",
        AssetMetadata {
            size_bytes: 2048,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(material1_id.clone())
    .with_dependency(material2_id.clone());
    let mesh_id = db.store_asset(mesh).await.unwrap();

    // Find transitive dependencies of the mesh
    let deps = db
        .find_asset_dependencies_transitive(&mesh_id)
        .await
        .unwrap();

    // Should find 2 materials and 3 textures (texture2 is shared)
    assert_eq!(deps.len(), 5);

    // Count by type
    let material_count = deps.iter().filter(|a| a.asset_type == "Material").count();
    let texture_count = deps.iter().filter(|a| a.asset_type == "Texture").count();

    assert_eq!(material_count, 2);
    assert_eq!(texture_count, 3);
}

#[tokio::test]
async fn test_circular_dependencies() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create circular dependency: Asset1 -> Asset2 -> Asset1
    let asset1 = AssetRecord::new(
        "assets/asset1.dat",
        "Data",
        "hash1",
        AssetMetadata {
            size_bytes: 100,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let asset1_id = db.store_asset(asset1).await.unwrap();

    let asset2 = AssetRecord::new(
        "assets/asset2.dat",
        "Data",
        "hash2",
        AssetMetadata {
            size_bytes: 100,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(asset1_id.clone());
    let asset2_id = db.store_asset(asset2).await.unwrap();

    // Update asset1 to depend on asset2 (creating cycle)
    let mut asset1 = db.load_asset(&asset1_id).await.unwrap();
    asset1.dependencies.push(asset2_id.clone());
    db.update_asset(&asset1_id, asset1).await.unwrap();

    // Find transitive dependencies should handle the cycle
    let deps = db
        .find_asset_dependencies_transitive(&asset1_id)
        .await
        .unwrap();

    // Should find asset2 once (not infinite loop)
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].path, "assets/asset2.dat");
}

#[tokio::test]
async fn test_find_asset_dependents() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create a texture used by multiple materials
    let texture = AssetRecord::new(
        "textures/shared.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture_id = db.store_asset(texture).await.unwrap();

    // Create materials that depend on the texture
    let material1 = AssetRecord::new(
        "materials/mat1.mat",
        "Material",
        "hash2",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture_id.clone());
    db.store_asset(material1).await.unwrap();

    let material2 = AssetRecord::new(
        "materials/mat2.mat",
        "Material",
        "hash3",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture_id.clone());
    db.store_asset(material2).await.unwrap();

    let material3 = AssetRecord::new(
        "materials/mat3.mat",
        "Material",
        "hash4",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture_id.clone());
    db.store_asset(material3).await.unwrap();

    // Find what depends on the texture
    let dependents = db.find_asset_dependents(&texture_id).await.unwrap();

    assert_eq!(dependents.len(), 3);
    assert!(dependents.iter().all(|a| a.asset_type == "Material"));
}

#[tokio::test]
async fn test_find_assets_in_scene() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create a scene with entities
    let scene = EntityRecord::new(Some("TestScene".to_string())).with_tag("scene");
    let scene_id = db.store_entity(scene).await.unwrap();

    let entity1 = EntityRecord::new(Some("Entity1".to_string()));
    let entity1_id = db.store_entity(entity1).await.unwrap();

    let entity2 = EntityRecord::new(Some("Entity2".to_string()));
    let entity2_id = db.store_entity(entity2).await.unwrap();

    // Setup scene hierarchy
    let mut scene = db.load_entity(&scene_id).await.unwrap();
    scene.children = vec![entity1_id.clone(), entity2_id.clone()];
    db.update_entity(&scene_id, scene).await.unwrap();

    let mut entity1 = db.load_entity(&entity1_id).await.unwrap();
    entity1.parent = Some(scene_id.clone());
    db.update_entity(&entity1_id, entity1).await.unwrap();

    let mut entity2 = db.load_entity(&entity2_id).await.unwrap();
    entity2.parent = Some(scene_id.clone());
    db.update_entity(&entity2_id, entity2).await.unwrap();

    // Create textures
    let texture1 = AssetRecord::new(
        "textures/tex1.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture1_id = db.store_asset(texture1).await.unwrap();

    let texture2 = AssetRecord::new(
        "textures/tex2.png",
        "Texture",
        "hash2",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture2_id = db.store_asset(texture2).await.unwrap();

    // Add components with asset references
    let component1 = ComponentRecord::new(
        "Material",
        "test",
        json!({
            "albedo": {
                "asset_id": texture1_id.to_string()
            }
        }),
        entity1_id.clone(),
    );
    db.store_component(component1).await.unwrap();

    let component2 = ComponentRecord::new(
        "Material",
        "test",
        json!({
            "albedo": {
                "asset_id": texture2_id.to_string()
            }
        }),
        entity2_id.clone(),
    );
    db.store_component(component2).await.unwrap();

    // Find all textures in the scene
    let textures = db.find_assets_in_scene(&scene_id, "Texture").await.unwrap();

    assert_eq!(textures.len(), 2);
    let paths: Vec<&str> = textures.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"textures/tex1.png"));
    assert!(paths.contains(&"textures/tex2.png"));
}

#[tokio::test]
async fn test_find_textures_used_by_materials_in_scene() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create scene
    let scene = EntityRecord::new(Some("TestScene".to_string()));
    let scene_id = db.store_entity(scene).await.unwrap();

    let entity = EntityRecord::new(Some("Entity".to_string()));
    let entity_id = db.store_entity(entity).await.unwrap();

    // Setup hierarchy
    let mut scene = db.load_entity(&scene_id).await.unwrap();
    scene.children = vec![entity_id.clone()];
    db.update_entity(&scene_id, scene).await.unwrap();

    let mut entity = db.load_entity(&entity_id).await.unwrap();
    entity.parent = Some(scene_id.clone());
    db.update_entity(&entity_id, entity).await.unwrap();

    // Create textures
    let texture1 = AssetRecord::new(
        "textures/albedo.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture1_id = db.store_asset(texture1).await.unwrap();

    let texture2 = AssetRecord::new(
        "textures/normal.png",
        "Texture",
        "hash2",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let texture2_id = db.store_asset(texture2).await.unwrap();

    // Create material that depends on textures
    let material = AssetRecord::new(
        "materials/pbr.mat",
        "Material",
        "hash3",
        AssetMetadata {
            size_bytes: 512,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    )
    .with_dependency(texture1_id.clone())
    .with_dependency(texture2_id.clone());
    let material_id = db.store_asset(material).await.unwrap();

    // Add component with material reference
    let component = ComponentRecord::new(
        "MeshRenderer",
        "test",
        json!({
            "material": {
                "asset_id": material_id.to_string()
            }
        }),
        entity_id.clone(),
    );
    db.store_component(component).await.unwrap();

    // Find textures used by materials in the scene
    let textures = db
        .find_textures_used_by_materials_in_scene(&scene_id)
        .await
        .unwrap();

    assert_eq!(textures.len(), 2);
    let paths: Vec<&str> = textures.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"textures/albedo.png"));
    assert!(paths.contains(&"textures/normal.png"));
}

#[tokio::test]
async fn test_query_assets_by_type() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create various assets
    let texture1 = AssetRecord::new(
        "textures/tex1.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    db.store_asset(texture1).await.unwrap();

    let texture2 = AssetRecord::new(
        "textures/tex2.png",
        "Texture",
        "hash2",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    db.store_asset(texture2).await.unwrap();

    let mesh = AssetRecord::new(
        "meshes/cube.mesh",
        "Mesh",
        "hash3",
        AssetMetadata {
            size_bytes: 2048,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    db.store_asset(mesh).await.unwrap();

    // Query for textures
    let textures = db
        .query_assets("SELECT * FROM asset WHERE asset_type = 'Texture'")
        .await
        .unwrap();

    assert_eq!(textures.len(), 2);
    assert!(textures.iter().all(|a| a.asset_type == "Texture"));

    // Query for meshes
    let meshes = db
        .query_assets("SELECT * FROM asset WHERE asset_type = 'Mesh'")
        .await
        .unwrap();

    assert_eq!(meshes.len(), 1);
    assert_eq!(meshes[0].path, "meshes/cube.mesh");
}

#[tokio::test]
async fn test_no_dependencies() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create an asset with no dependencies
    let asset = AssetRecord::new(
        "textures/standalone.png",
        "Texture",
        "hash1",
        AssetMetadata {
            size_bytes: 1024,
            modified_timestamp: 1234567890,
            custom: json!({}),
        },
    );
    let asset_id = db.store_asset(asset).await.unwrap();

    // Find dependencies (should be empty)
    let deps = db.find_asset_dependencies(&asset_id).await.unwrap();
    assert_eq!(deps.len(), 0);

    let deps_transitive = db
        .find_asset_dependencies_transitive(&asset_id)
        .await
        .unwrap();
    assert_eq!(deps_transitive.len(), 0);

    // Find dependents (should be empty)
    let dependents = db.find_asset_dependents(&asset_id).await.unwrap();
    assert_eq!(dependents.len(), 0);
}
