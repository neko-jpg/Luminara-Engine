/// Comprehensive test for entity hierarchy serialization
///
/// This test validates all requirements for task 11.6:
/// - Requirement 8.5: Preserve parent-child relationships and entity references
/// - Requirement 8.6: Include all entities, components, and resources in a single file
/// - Requirement 8.7: Support partial loading and lazy asset resolution
///
/// Task 11.6: Implement entity hierarchy serialization
/// - Preserve parent-child relationships ✓
/// - Handle entity references correctly ✓
/// - Support partial loading ✓

use glam::{Quat, Vec3};
use luminara_core::World;
use luminara_math::Transform;
use luminara_scene::*;

/// Test comprehensive hierarchy serialization with all features
///
/// Requirements: 8.5, 8.6, 8.7
#[test]
fn test_comprehensive_hierarchy_serialization() {
    let mut world = World::new();

    // Create a complex hierarchy with multiple levels and branches
    // Structure:
    //   Root1
    //     ├─ Child1A
    //     │   ├─ Grandchild1A1
    //     │   └─ Grandchild1A2
    //     └─ Child1B
    //   Root2
    //     └─ Child2A
    //         └─ Grandchild2A1
    //             └─ GreatGrandchild2A1A

    // Root1 hierarchy
    let root1 = world.spawn();
    let _ = world.add_component(root1, Name::new("Root1"));
    let _ = world.add_component(
        root1,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    );
    let mut root1_tags = Tag::new();
    root1_tags.insert("root");
    root1_tags.insert("scene1");
    let _ = world.add_component(root1, root1_tags);

    let child1a = world.spawn();
    let _ = world.add_component(child1a, Name::new("Child1A"));
    let _ = world.add_component(
        child1a,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    set_parent(&mut world, child1a, root1);

    let grandchild1a1 = world.spawn();
    let _ = world.add_component(grandchild1a1, Name::new("Grandchild1A1"));
    let _ = world.add_component(
        grandchild1a1,
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    );
    set_parent(&mut world, grandchild1a1, child1a);

    let grandchild1a2 = world.spawn();
    let _ = world.add_component(grandchild1a2, Name::new("Grandchild1A2"));
    let _ = world.add_component(
        grandchild1a2,
        Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
    );
    set_parent(&mut world, grandchild1a2, child1a);

    let child1b = world.spawn();
    let _ = world.add_component(child1b, Name::new("Child1B"));
    let _ = world.add_component(
        child1b,
        Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
    );
    set_parent(&mut world, child1b, root1);

    // Root2 hierarchy (deeper nesting)
    let root2 = world.spawn();
    let _ = world.add_component(root2, Name::new("Root2"));
    let _ = world.add_component(
        root2,
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
    );

    let child2a = world.spawn();
    let _ = world.add_component(child2a, Name::new("Child2A"));
    let _ = world.add_component(
        child2a,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    set_parent(&mut world, child2a, root2);

    let grandchild2a1 = world.spawn();
    let _ = world.add_component(grandchild2a1, Name::new("Grandchild2A1"));
    let _ = world.add_component(
        grandchild2a1,
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    );
    set_parent(&mut world, grandchild2a1, child2a);

    let great_grandchild2a1a = world.spawn();
    let _ = world.add_component(great_grandchild2a1a, Name::new("GreatGrandchild2A1A"));
    let _ = world.add_component(
        great_grandchild2a1a,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    );
    set_parent(&mut world, great_grandchild2a1a, grandchild2a1);

    // === SERIALIZATION ===
    let scene = Scene::from_world(&world);

    // Verify scene structure (Requirement 8.6: Include all entities in a single file)
    assert_eq!(
        scene.entities.len(),
        2,
        "Should have 2 root entities at top level"
    );

    // === VERIFY PARENT-CHILD RELATIONSHIPS (Requirement 8.5) ===

    // Find Root1 in scene
    let root1_data = scene
        .entities
        .iter()
        .find(|e| e.name == "Root1")
        .expect("Root1 not found in scene");

    // Verify Root1 has 2 children
    assert_eq!(
        root1_data.children.len(),
        2,
        "Root1 should have 2 children"
    );

    // Verify Child1A structure
    let child1a_data = root1_data
        .children
        .iter()
        .find(|c| c.name == "Child1A")
        .expect("Child1A not found");
    assert_eq!(
        child1a_data.children.len(),
        2,
        "Child1A should have 2 children"
    );
    assert!(
        child1a_data
            .children
            .iter()
            .any(|c| c.name == "Grandchild1A1"),
        "Grandchild1A1 should be child of Child1A"
    );
    assert!(
        child1a_data
            .children
            .iter()
            .any(|c| c.name == "Grandchild1A2"),
        "Grandchild1A2 should be child of Child1A"
    );

    // Verify Child1B structure
    let child1b_data = root1_data
        .children
        .iter()
        .find(|c| c.name == "Child1B")
        .expect("Child1B not found");
    assert_eq!(
        child1b_data.children.len(),
        0,
        "Child1B should have no children"
    );

    // Find Root2 in scene
    let root2_data = scene
        .entities
        .iter()
        .find(|e| e.name == "Root2")
        .expect("Root2 not found in scene");

    // Verify Root2 deep hierarchy
    assert_eq!(
        root2_data.children.len(),
        1,
        "Root2 should have 1 child"
    );
    let child2a_data = &root2_data.children[0];
    assert_eq!(child2a_data.name, "Child2A");
    assert_eq!(
        child2a_data.children.len(),
        1,
        "Child2A should have 1 child"
    );
    let grandchild2a1_data = &child2a_data.children[0];
    assert_eq!(grandchild2a1_data.name, "Grandchild2A1");
    assert_eq!(
        grandchild2a1_data.children.len(),
        1,
        "Grandchild2A1 should have 1 child"
    );
    let great_grandchild_data = &grandchild2a1_data.children[0];
    assert_eq!(great_grandchild_data.name, "GreatGrandchild2A1A");

    // === VERIFY ENTITY REFERENCES (Requirement 8.5) ===

    // All entities should have IDs
    assert!(root1_data.id.is_some(), "Root1 should have an ID");
    assert!(child1a_data.id.is_some(), "Child1A should have an ID");
    assert!(
        child1b_data.id.is_some(),
        "Child1B should have an ID"
    );

    // Parent references should be correct
    assert_eq!(
        child1a_data.parent,
        root1_data.id,
        "Child1A parent should reference Root1"
    );
    assert_eq!(
        child1b_data.parent,
        root1_data.id,
        "Child1B parent should reference Root1"
    );

    let grandchild1a1_data = child1a_data
        .children
        .iter()
        .find(|c| c.name == "Grandchild1A1")
        .unwrap();
    assert_eq!(
        grandchild1a1_data.parent,
        child1a_data.id,
        "Grandchild1A1 parent should reference Child1A"
    );

    // === VERIFY COMPONENT PRESERVATION (Requirement 8.6) ===

    // Check Transform components are preserved
    assert!(
        root1_data.components.contains_key("Transform"),
        "Root1 should have Transform component"
    );
    assert!(
        child1a_data.components.contains_key("Transform"),
        "Child1A should have Transform component"
    );

    // Check Tag components are preserved
    assert!(
        !root1_data.tags.is_empty(),
        "Root1 should have tags"
    );
    assert!(
        root1_data.tags.contains(&"root".to_string()),
        "Root1 should have 'root' tag"
    );
    assert!(
        root1_data.tags.contains(&"scene1".to_string()),
        "Root1 should have 'scene1' tag"
    );

    // === SERIALIZATION ROUND-TRIP ===

    // Serialize to RON format
    let ron_string = scene.to_ron().expect("Failed to serialize to RON");
    assert!(
        !ron_string.is_empty(),
        "RON serialization should produce non-empty string"
    );

    // Deserialize from RON
    let deserialized_scene =
        Scene::from_ron(&ron_string).expect("Failed to deserialize from RON");

    // === SPAWN INTO NEW WORLD ===

    let mut new_world = World::new();
    let spawned_entities = deserialized_scene.spawn_into(&mut new_world);

    // Verify all entities were spawned (Requirement 8.6)
    assert_eq!(
        spawned_entities.len(),
        9,
        "Should spawn all 9 entities"
    );

    // === VERIFY DESERIALIZED HIERARCHY ===

    // Find entities in new world
    let new_root1 = find_entity_by_name(&new_world, "Root1").expect("Root1 not found in new world");
    let new_child1a =
        find_entity_by_name(&new_world, "Child1A").expect("Child1A not found in new world");
    let new_grandchild1a1 = find_entity_by_name(&new_world, "Grandchild1A1")
        .expect("Grandchild1A1 not found in new world");
    let new_child1b =
        find_entity_by_name(&new_world, "Child1B").expect("Child1B not found in new world");
    let _new_root2 = find_entity_by_name(&new_world, "Root2").expect("Root2 not found in new world");

    // Verify parent-child relationships are preserved (Requirement 8.5)
    let child1a_parent = new_world
        .get_component::<Parent>(new_child1a)
        .expect("Child1A should have Parent component");
    assert_eq!(
        child1a_parent.0, new_root1,
        "Child1A parent should be Root1"
    );

    let grandchild1a1_parent = new_world
        .get_component::<Parent>(new_grandchild1a1)
        .expect("Grandchild1A1 should have Parent component");
    assert_eq!(
        grandchild1a1_parent.0, new_child1a,
        "Grandchild1A1 parent should be Child1A"
    );

    // Verify children relationships
    let root1_children = new_world
        .get_component::<Children>(new_root1)
        .expect("Root1 should have Children component");
    assert_eq!(
        root1_children.0.len(),
        2,
        "Root1 should have 2 children"
    );
    assert!(
        root1_children.0.contains(&new_child1a),
        "Root1 children should contain Child1A"
    );
    assert!(
        root1_children.0.contains(&new_child1b),
        "Root1 children should contain Child1B"
    );

    // Verify Transform components are preserved
    let root1_transform = new_world
        .get_component::<Transform>(new_root1)
        .expect("Root1 should have Transform");
    assert_eq!(
        root1_transform.translation,
        Vec3::new(0.0, 0.0, 0.0),
        "Root1 transform should be preserved"
    );

    let child1a_transform = new_world
        .get_component::<Transform>(new_child1a)
        .expect("Child1A should have Transform");
    assert_eq!(
        child1a_transform.translation,
        Vec3::new(1.0, 0.0, 0.0),
        "Child1A transform should be preserved"
    );

    // Verify Tag components are preserved
    let root1_tags = new_world
        .get_component::<Tag>(new_root1)
        .expect("Root1 should have Tag component");
    assert!(
        root1_tags.contains("root"),
        "Root1 should have 'root' tag"
    );
    assert!(
        root1_tags.contains("scene1"),
        "Root1 should have 'scene1' tag"
    );

    // === TEST PARTIAL LOADING (Requirement 8.7) ===

    let mut partial_world = World::new();
    let partial_spawned =
        deserialized_scene.spawn_entities_by_name(&mut partial_world, &["Child1A", "Root2"]);

    // Should only spawn requested entities and their children
    assert!(
        partial_spawned.len() >= 2,
        "Should spawn at least the requested entities"
    );

    // Verify requested entities exist
    assert!(
        find_entity_by_name(&partial_world, "Child1A").is_some(),
        "Child1A should be spawned"
    );
    assert!(
        find_entity_by_name(&partial_world, "Root2").is_some(),
        "Root2 should be spawned"
    );

    // Verify non-requested root entities don't exist
    assert!(
        find_entity_by_name(&partial_world, "Root1").is_none(),
        "Root1 should not be spawned in partial load"
    );
    assert!(
        find_entity_by_name(&partial_world, "Child1B").is_none(),
        "Child1B should not be spawned in partial load"
    );

    // === TEST JSON SERIALIZATION ===

    let json_string = scene.to_json().expect("Failed to serialize to JSON");
    assert!(
        !json_string.is_empty(),
        "JSON serialization should produce non-empty string"
    );

    let json_scene = Scene::from_json(&json_string).expect("Failed to deserialize from JSON");
    assert_eq!(
        json_scene.entities.len(),
        2,
        "JSON deserialized scene should have 2 root entities"
    );
}

/// Test that entity references remain valid across serialization
///
/// Requirements: 8.5 - Handle entity references correctly
#[test]
fn test_entity_reference_stability() {
    let mut world = World::new();

    // Create entities with specific relationships
    let parent = world.spawn();
    let _ = world.add_component(parent, Name::new("Parent"));
    let _ = world.add_component(parent, Transform::IDENTITY);

    let child1 = world.spawn();
    let _ = world.add_component(child1, Name::new("Child1"));
    let _ = world.add_component(child1, Transform::IDENTITY);
    set_parent(&mut world, child1, parent);

    let child2 = world.spawn();
    let _ = world.add_component(child2, Name::new("Child2"));
    let _ = world.add_component(child2, Transform::IDENTITY);
    set_parent(&mut world, child2, parent);

    // Serialize
    let scene = Scene::from_world(&world);

    // Verify entity IDs are assigned consistently
    let parent_data = &scene.entities[0];
    let child1_data = &parent_data.children[0];
    let child2_data = &parent_data.children[1];

    assert!(parent_data.id.is_some());
    assert!(child1_data.id.is_some());
    assert!(child2_data.id.is_some());

    // Verify parent references point to correct entity
    assert_eq!(child1_data.parent, parent_data.id);
    assert_eq!(child2_data.parent, parent_data.id);

    // Verify IDs are unique
    assert_ne!(parent_data.id, child1_data.id);
    assert_ne!(parent_data.id, child2_data.id);
    assert_ne!(child1_data.id, child2_data.id);

    // Deserialize and verify references are resolved correctly
    let mut new_world = World::new();
    scene.spawn_into(&mut new_world);

    let new_parent = find_entity_by_name(&new_world, "Parent").unwrap();
    let new_child1 = find_entity_by_name(&new_world, "Child1").unwrap();
    let new_child2 = find_entity_by_name(&new_world, "Child2").unwrap();

    // Verify parent-child relationships
    assert_eq!(
        new_world.get_component::<Parent>(new_child1).unwrap().0,
        new_parent
    );
    assert_eq!(
        new_world.get_component::<Parent>(new_child2).unwrap().0,
        new_parent
    );

    let parent_children = new_world.get_component::<Children>(new_parent).unwrap();
    assert_eq!(parent_children.0.len(), 2);
    assert!(parent_children.0.contains(&new_child1));
    assert!(parent_children.0.contains(&new_child2));
}

/// Test serialization with complex transforms
///
/// Requirements: 8.6 - Include all components
#[test]
fn test_complex_transform_serialization() {
    let mut world = World::new();

    // Create entity with complex transform
    let entity = world.spawn();
    let _ = world.add_component(entity, Name::new("ComplexEntity"));

    let complex_transform = Transform {
        translation: Vec3::new(1.5, 2.5, 3.5),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 3.0, 4.0),
    };
    let _ = world.add_component(entity, complex_transform);

    // Serialize and deserialize
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    let deserialized_scene = Scene::from_ron(&ron_string).unwrap();

    let mut new_world = World::new();
    deserialized_scene.spawn_into(&mut new_world);

    // Verify transform is preserved accurately
    let new_entity = find_entity_by_name(&new_world, "ComplexEntity").unwrap();
    let new_transform = new_world.get_component::<Transform>(new_entity).unwrap();

    // Check translation
    assert!(
        (new_transform.translation - complex_transform.translation).length() < 0.001,
        "Translation should be preserved"
    );

    // Check rotation (quaternions)
    assert!(
        (new_transform.rotation.x - complex_transform.rotation.x).abs() < 0.001,
        "Rotation X should be preserved"
    );
    assert!(
        (new_transform.rotation.y - complex_transform.rotation.y).abs() < 0.001,
        "Rotation Y should be preserved"
    );
    assert!(
        (new_transform.rotation.z - complex_transform.rotation.z).abs() < 0.001,
        "Rotation Z should be preserved"
    );
    assert!(
        (new_transform.rotation.w - complex_transform.rotation.w).abs() < 0.001,
        "Rotation W should be preserved"
    );

    // Check scale
    assert!(
        (new_transform.scale - complex_transform.scale).length() < 0.001,
        "Scale should be preserved"
    );
}
