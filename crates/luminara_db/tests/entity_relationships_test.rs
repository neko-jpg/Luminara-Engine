//! Tests for entity storage and loading with Record Links and graph traversal

use luminara_db::{ComponentRecord, EntityRecord, LuminaraDatabase};
use serde_json::json;

#[tokio::test]
async fn test_store_and_load_entity_with_components() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create entity
    let entity = EntityRecord::new(Some("Player".to_string())).with_tag("player");
    let entity_id = db.store_entity(entity).await.unwrap();

    // Create components
    let transform_data = json!({
        "position": [0.0, 1.0, 2.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    });

    let mesh_data = json!({
        "vertices": 100,
        "indices": 300
    });

    let transform = ComponentRecord::new(
        "Transform",
        "luminara_scene::Transform",
        transform_data,
        entity_id.clone(),
    );

    let mesh = ComponentRecord::new(
        "Mesh",
        "luminara_render::Mesh",
        mesh_data,
        entity_id.clone(),
    );

    let transform_id = db.store_component(transform).await.unwrap();
    let mesh_id = db.store_component(mesh).await.unwrap();

    // Update entity with component links
    let mut entity = db.load_entity(&entity_id).await.unwrap();
    entity.components = vec![transform_id, mesh_id];
    db.update_entity(&entity_id, entity).await.unwrap();

    // Load entity with components
    let (loaded_entity, components) = db.load_entity_with_components(&entity_id).await.unwrap();

    assert_eq!(loaded_entity.name, Some("Player".to_string()));
    assert_eq!(components.len(), 2);

    // Verify component types
    let component_types: Vec<&str> = components.iter().map(|c| c.type_name.as_str()).collect();
    assert!(component_types.contains(&"Transform"));
    assert!(component_types.contains(&"Mesh"));
}

#[tokio::test]
async fn test_entity_hierarchy() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create parent entity
    let parent = EntityRecord::new(Some("Parent".to_string()));
    let parent_id = db.store_entity(parent).await.unwrap();

    // Create child entities
    let child1 = EntityRecord::new(Some("Child1".to_string()));
    let child1_id = db.store_entity(child1).await.unwrap();

    let child2 = EntityRecord::new(Some("Child2".to_string()));
    let child2_id = db.store_entity(child2).await.unwrap();

    // Update parent with children
    let mut parent = db.load_entity(&parent_id).await.unwrap();
    parent.children = vec![child1_id.clone(), child2_id.clone()];
    db.update_entity(&parent_id, parent).await.unwrap();

    // Update children with parent
    let mut child1 = db.load_entity(&child1_id).await.unwrap();
    child1.parent = Some(parent_id.clone());
    db.update_entity(&child1_id, child1).await.unwrap();

    let mut child2 = db.load_entity(&child2_id).await.unwrap();
    child2.parent = Some(parent_id.clone());
    db.update_entity(&child2_id, child2).await.unwrap();

    // Load hierarchy
    let hierarchy = db.load_entity_hierarchy(&parent_id).await.unwrap();

    assert_eq!(hierarchy.entity.name, Some("Parent".to_string()));
    assert!(hierarchy.parent.is_none());
    assert_eq!(hierarchy.children.len(), 2);

    // Load child hierarchy
    let child_hierarchy = db.load_entity_hierarchy(&child1_id).await.unwrap();
    assert!(child_hierarchy.parent.is_some());
    assert_eq!(
        child_hierarchy.parent.unwrap().name,
        Some("Parent".to_string())
    );
}

#[tokio::test]
async fn test_load_entity_with_relationships() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create parent
    let parent = EntityRecord::new(Some("Parent".to_string()));
    let parent_id = db.store_entity(parent).await.unwrap();

    // Create entity with component
    let entity = EntityRecord::new(Some("Entity".to_string()));
    let entity_id = db.store_entity(entity).await.unwrap();

    let component = ComponentRecord::new("Transform", "test", json!({"x": 1.0}), entity_id.clone());
    let component_id = db.store_component(component).await.unwrap();

    // Create child
    let child = EntityRecord::new(Some("Child".to_string()));
    let child_id = db.store_entity(child).await.unwrap();

    // Setup relationships
    let mut entity = db.load_entity(&entity_id).await.unwrap();
    entity.parent = Some(parent_id.clone());
    entity.children = vec![child_id.clone()];
    entity.components = vec![component_id];
    db.update_entity(&entity_id, entity).await.unwrap();

    // Update parent
    let mut parent = db.load_entity(&parent_id).await.unwrap();
    parent.children = vec![entity_id.clone()];
    db.update_entity(&parent_id, parent).await.unwrap();

    // Update child
    let mut child = db.load_entity(&child_id).await.unwrap();
    child.parent = Some(entity_id.clone());
    db.update_entity(&child_id, child).await.unwrap();

    // Load with all relationships
    let full_entity = db.load_entity_with_relationships(&entity_id).await.unwrap();

    assert_eq!(full_entity.entity.name, Some("Entity".to_string()));
    assert_eq!(full_entity.components.len(), 1);
    assert_eq!(full_entity.components[0].type_name, "Transform");
    assert!(full_entity.hierarchy.parent.is_some());
    assert_eq!(
        full_entity.hierarchy.parent.unwrap().name,
        Some("Parent".to_string())
    );
    assert_eq!(full_entity.hierarchy.children.len(), 1);
    assert_eq!(
        full_entity.hierarchy.children[0].name,
        Some("Child".to_string())
    );
}

#[tokio::test]
async fn test_find_entities_with_component() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create entities
    let entity1 = EntityRecord::new(Some("Entity1".to_string()));
    let entity1_id = db.store_entity(entity1).await.unwrap();

    let entity2 = EntityRecord::new(Some("Entity2".to_string()));
    let entity2_id = db.store_entity(entity2).await.unwrap();

    let entity3 = EntityRecord::new(Some("Entity3".to_string()));
    let entity3_id = db.store_entity(entity3).await.unwrap();

    // Add Transform to entity1 and entity2
    let transform1 = ComponentRecord::new("Transform", "test", json!({}), entity1_id.clone());
    db.store_component(transform1).await.unwrap();

    let transform2 = ComponentRecord::new("Transform", "test", json!({}), entity2_id.clone());
    db.store_component(transform2).await.unwrap();

    // Add Mesh to entity3
    let mesh = ComponentRecord::new("Mesh", "test", json!({}), entity3_id.clone());
    db.store_component(mesh).await.unwrap();

    // Find entities with Transform
    let entities = db.find_entities_with_component("Transform").await.unwrap();
    assert_eq!(entities.len(), 2);

    // Find entities with Mesh
    let entities = db.find_entities_with_component("Mesh").await.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, Some("Entity3".to_string()));
}

#[tokio::test]
async fn test_find_entity_descendants() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create hierarchy: Root -> Child1 -> Grandchild1
    //                        -> Child2
    let root = EntityRecord::new(Some("Root".to_string()));
    let root_id = db.store_entity(root).await.unwrap();

    let child1 = EntityRecord::new(Some("Child1".to_string()));
    let child1_id = db.store_entity(child1).await.unwrap();

    let child2 = EntityRecord::new(Some("Child2".to_string()));
    let child2_id = db.store_entity(child2).await.unwrap();

    let grandchild1 = EntityRecord::new(Some("Grandchild1".to_string()));
    let grandchild1_id = db.store_entity(grandchild1).await.unwrap();

    // Setup relationships
    let mut root = db.load_entity(&root_id).await.unwrap();
    root.children = vec![child1_id.clone(), child2_id.clone()];
    db.update_entity(&root_id, root).await.unwrap();

    let mut child1 = db.load_entity(&child1_id).await.unwrap();
    child1.parent = Some(root_id.clone());
    child1.children = vec![grandchild1_id.clone()];
    db.update_entity(&child1_id, child1).await.unwrap();

    let mut child2 = db.load_entity(&child2_id).await.unwrap();
    child2.parent = Some(root_id.clone());
    db.update_entity(&child2_id, child2).await.unwrap();

    let mut grandchild1 = db.load_entity(&grandchild1_id).await.unwrap();
    grandchild1.parent = Some(child1_id.clone());
    db.update_entity(&grandchild1_id, grandchild1)
        .await
        .unwrap();

    // Find descendants of root (should find all 3 descendants)
    let descendants = db.find_entity_descendants(&root_id).await.unwrap();

    // Should find Child1, Child2, and Grandchild1
    assert_eq!(descendants.len(), 3);

    let names: Vec<String> = descendants.iter().filter_map(|e| e.name.clone()).collect();
    assert!(names.contains(&"Child1".to_string()));
    assert!(names.contains(&"Child2".to_string()));
    assert!(names.contains(&"Grandchild1".to_string()));
}

#[tokio::test]
async fn test_find_entity_ancestors() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create hierarchy: Root -> Child -> Grandchild
    let root = EntityRecord::new(Some("Root".to_string()));
    let root_id = db.store_entity(root).await.unwrap();

    let child = EntityRecord::new(Some("Child".to_string()));
    let child_id = db.store_entity(child).await.unwrap();

    let grandchild = EntityRecord::new(Some("Grandchild".to_string()));
    let grandchild_id = db.store_entity(grandchild).await.unwrap();

    // Setup relationships
    let mut root = db.load_entity(&root_id).await.unwrap();
    root.children = vec![child_id.clone()];
    db.update_entity(&root_id, root).await.unwrap();

    let mut child = db.load_entity(&child_id).await.unwrap();
    child.parent = Some(root_id.clone());
    child.children = vec![grandchild_id.clone()];
    db.update_entity(&child_id, child).await.unwrap();

    let mut grandchild = db.load_entity(&grandchild_id).await.unwrap();
    grandchild.parent = Some(child_id.clone());
    db.update_entity(&grandchild_id, grandchild).await.unwrap();

    // Find ancestors of grandchild (should find child and root)
    let ancestors = db.find_entity_ancestors(&grandchild_id).await.unwrap();

    // Should find Child and Root
    assert_eq!(ancestors.len(), 2);

    let names: Vec<String> = ancestors.iter().filter_map(|e| e.name.clone()).collect();
    assert!(names.contains(&"Child".to_string()));
    assert!(names.contains(&"Root".to_string()));
}

#[tokio::test]
async fn test_complex_entity_graph() {
    let db = LuminaraDatabase::new_memory().await.unwrap();

    // Create a complex scene graph
    let scene = EntityRecord::new(Some("Scene".to_string())).with_tag("scene");
    let scene_id = db.store_entity(scene).await.unwrap();

    let camera = EntityRecord::new(Some("Camera".to_string())).with_tag("camera");
    let camera_id = db.store_entity(camera).await.unwrap();

    let player = EntityRecord::new(Some("Player".to_string())).with_tag("player");
    let player_id = db.store_entity(player).await.unwrap();

    // Add components to player
    let transform = ComponentRecord::new(
        "Transform",
        "test",
        json!({"position": [0.0, 0.0, 0.0]}),
        player_id.clone(),
    );
    let transform_id = db.store_component(transform).await.unwrap();

    let mesh = ComponentRecord::new("Mesh", "test", json!({"vertices": 100}), player_id.clone());
    let mesh_id = db.store_component(mesh).await.unwrap();

    // Setup scene hierarchy
    let mut scene = db.load_entity(&scene_id).await.unwrap();
    scene.children = vec![camera_id.clone(), player_id.clone()];
    db.update_entity(&scene_id, scene).await.unwrap();

    let mut camera = db.load_entity(&camera_id).await.unwrap();
    camera.parent = Some(scene_id.clone());
    db.update_entity(&camera_id, camera).await.unwrap();

    let mut player = db.load_entity(&player_id).await.unwrap();
    player.parent = Some(scene_id.clone());
    player.components = vec![transform_id, mesh_id];
    db.update_entity(&player_id, player).await.unwrap();

    // Query: Find all entities in scene
    let scene_hierarchy = db.load_entity_hierarchy(&scene_id).await.unwrap();
    assert_eq!(scene_hierarchy.children.len(), 2);

    // Query: Find all entities with Transform
    let entities_with_transform = db.find_entities_with_component("Transform").await.unwrap();
    assert_eq!(entities_with_transform.len(), 1);
    assert_eq!(entities_with_transform[0].name, Some("Player".to_string()));

    // Query: Load player with all relationships
    let player_full = db.load_entity_with_relationships(&player_id).await.unwrap();
    assert_eq!(player_full.components.len(), 2);
    assert!(player_full.hierarchy.parent.is_some());
    assert_eq!(
        player_full.hierarchy.parent.unwrap().name,
        Some("Scene".to_string())
    );
}
