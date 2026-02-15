use glam::Vec3;
use luminara_core::World;
use luminara_math::Transform;
use luminara_scene::*;

/// Test that entity hierarchies are correctly serialized and deserialized
/// 
/// Requirements: 8.5 - Preserve parent-child relationships
#[test]
fn test_hierarchy_serialization_preserves_relationships() {
    let mut world = World::new();

    // Create a hierarchy: root -> child1 -> grandchild
    //                           -> child2
    let root = world.spawn();
    world.add_component(root, Name::new("Root"));
    world.add_component(root, Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)));

    let child1 = world.spawn();
    world.add_component(child1, Name::new("Child1"));
    world.add_component(child1, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    set_parent(&mut world, child1, root);

    let child2 = world.spawn();
    world.add_component(child2, Name::new("Child2"));
    world.add_component(child2, Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)));
    set_parent(&mut world, child2, root);

    let grandchild = world.spawn();
    world.add_component(grandchild, Name::new("Grandchild"));
    world.add_component(
        grandchild,
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    );
    set_parent(&mut world, grandchild, child1);

    // Serialize the world to a scene
    let scene = Scene::from_world(&world);

    // Verify the scene structure
    assert_eq!(scene.entities.len(), 1); // Only root should be at top level
    assert_eq!(scene.entities[0].name, "Root");
    assert_eq!(scene.entities[0].children.len(), 2);

    // Check children
    let child1_data = scene.entities[0]
        .children
        .iter()
        .find(|c| c.name == "Child1")
        .expect("Child1 not found");
    let child2_data = scene.entities[0]
        .children
        .iter()
        .find(|c| c.name == "Child2")
        .expect("Child2 not found");

    assert_eq!(child1_data.children.len(), 1);
    assert_eq!(child1_data.children[0].name, "Grandchild");
    assert_eq!(child2_data.children.len(), 0);

    // Deserialize and verify
    let mut new_world = World::new();
    let spawned = scene.spawn_into(&mut new_world);

    assert_eq!(spawned.len(), 4); // root + 2 children + 1 grandchild

    // Verify hierarchy is preserved
    let root_entity = find_entity_by_name(&new_world, "Root").expect("Root not found");
    let child1_entity = find_entity_by_name(&new_world, "Child1").expect("Child1 not found");
    let grandchild_entity =
        find_entity_by_name(&new_world, "Grandchild").expect("Grandchild not found");

    // Check parent relationships
    let child1_parent = new_world
        .get_component::<Parent>(child1_entity)
        .expect("Child1 should have parent");
    assert_eq!(child1_parent.0, root_entity);

    let grandchild_parent = new_world
        .get_component::<Parent>(grandchild_entity)
        .expect("Grandchild should have parent");
    assert_eq!(grandchild_parent.0, child1_entity);

    // Check children relationships
    let root_children = new_world
        .get_component::<Children>(root_entity)
        .expect("Root should have children");
    assert_eq!(root_children.0.len(), 2);
    assert!(root_children.0.contains(&child1_entity));
}

/// Test that entity references are correctly handled during serialization
/// 
/// Requirements: 8.5 - Handle entity references correctly
#[test]
fn test_entity_reference_handling() {
    let mut world = World::new();

    // Create entities with explicit IDs
    let e1 = world.spawn();
    world.add_component(e1, Name::new("Entity1"));

    let e2 = world.spawn();
    world.add_component(e2, Name::new("Entity2"));
    set_parent(&mut world, e2, e1);

    let e3 = world.spawn();
    world.add_component(e3, Name::new("Entity3"));
    set_parent(&mut world, e3, e1);

    // Serialize
    let scene = Scene::from_world(&world);

    // Verify IDs are assigned
    assert!(scene.entities[0].id.is_some());
    assert!(scene.entities[0].children[0].id.is_some());
    assert!(scene.entities[0].children[1].id.is_some());

    // Verify parent references
    let child1_parent = scene.entities[0].children[0].parent;
    let child2_parent = scene.entities[0].children[1].parent;
    assert_eq!(child1_parent, scene.entities[0].id);
    assert_eq!(child2_parent, scene.entities[0].id);

    // Deserialize and verify references are resolved
    let mut new_world = World::new();
    scene.spawn_into(&mut new_world);

    let new_e1 = find_entity_by_name(&new_world, "Entity1").unwrap();
    let new_e2 = find_entity_by_name(&new_world, "Entity2").unwrap();
    let new_e3 = find_entity_by_name(&new_world, "Entity3").unwrap();

    // Verify parent-child relationships are correct
    assert_eq!(
        new_world.get_component::<Parent>(new_e2).unwrap().0,
        new_e1
    );
    assert_eq!(
        new_world.get_component::<Parent>(new_e3).unwrap().0,
        new_e1
    );

    let children = new_world.get_component::<Children>(new_e1).unwrap();
    assert_eq!(children.0.len(), 2);
    assert!(children.0.contains(&new_e2));
    assert!(children.0.contains(&new_e3));
}

/// Test partial loading of entities by name
/// 
/// Requirements: 8.7 - Support partial loading
#[test]
fn test_partial_loading_by_name() {
    let mut world = World::new();

    // Create a complex hierarchy
    let root = world.spawn();
    world.add_component(root, Name::new("Root"));
    world.add_component(root, Transform::IDENTITY);

    let player = world.spawn();
    world.add_component(player, Name::new("Player"));
    world.add_component(player, Transform::IDENTITY);
    set_parent(&mut world, player, root);

    let enemy = world.spawn();
    world.add_component(enemy, Name::new("Enemy"));
    world.add_component(enemy, Transform::IDENTITY);
    set_parent(&mut world, enemy, root);

    let camera = world.spawn();
    world.add_component(camera, Name::new("Camera"));
    world.add_component(camera, Transform::IDENTITY);
    set_parent(&mut world, camera, root);

    // Serialize
    let scene = Scene::from_world(&world);

    // Load only specific entities
    let mut new_world = World::new();
    let spawned = scene.spawn_entities_by_name(&mut new_world, &["Player", "Camera"]);

    // Should only spawn the requested entities
    assert!(spawned.len() >= 2); // At least Player and Camera

    // Verify only requested entities exist
    assert!(find_entity_by_name(&new_world, "Player").is_some());
    assert!(find_entity_by_name(&new_world, "Camera").is_some());
    assert!(find_entity_by_name(&new_world, "Enemy").is_none());
}

/// Test that serialization round-trip preserves all data
/// 
/// Requirements: 8.5, 8.6 - Preserve all entity data
#[test]
fn test_serialization_round_trip_complete() {
    let mut world = World::new();

    // Create entities with various components
    let root = world.spawn();
    world.add_component(root, Name::new("Root"));
    world.add_component(root, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    let mut root_tags = Tag::new();
    root_tags.insert("important");
    root_tags.insert("root");
    world.add_component(root, root_tags);

    let child = world.spawn();
    world.add_component(child, Name::new("Child"));
    world.add_component(
        child,
        Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)),
    );
    let mut child_tags = Tag::new();
    child_tags.insert("child");
    world.add_component(child, child_tags);
    set_parent(&mut world, child, root);

    // Serialize to RON
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();

    // Deserialize from RON
    let deserialized_scene = Scene::from_ron(&ron_string).unwrap();

    // Spawn into new world
    let mut new_world = World::new();
    deserialized_scene.spawn_into(&mut new_world);

    // Verify all data is preserved
    let new_root = find_entity_by_name(&new_world, "Root").unwrap();
    let new_child = find_entity_by_name(&new_world, "Child").unwrap();

    // Check transforms
    let root_transform = new_world.get_component::<Transform>(new_root).unwrap();
    assert_eq!(root_transform.translation, Vec3::new(1.0, 2.0, 3.0));

    let child_transform = new_world.get_component::<Transform>(new_child).unwrap();
    assert_eq!(child_transform.translation, Vec3::new(4.0, 5.0, 6.0));

    // Check tags
    let root_tags = new_world.get_component::<Tag>(new_root).unwrap();
    assert!(root_tags.contains("important"));
    assert!(root_tags.contains("root"));

    let child_tags = new_world.get_component::<Tag>(new_child).unwrap();
    assert!(child_tags.contains("child"));

    // Check hierarchy
    let child_parent = new_world.get_component::<Parent>(new_child).unwrap();
    assert_eq!(child_parent.0, new_root);
}

/// Test serialization of deep hierarchies
/// 
/// Requirements: 8.5 - Handle complex hierarchies
#[test]
fn test_deep_hierarchy_serialization() {
    let mut world = World::new();

    // Create a deep hierarchy (5 levels)
    let mut current = world.spawn();
    world.add_component(current, Name::new("Level0"));
    world.add_component(current, Transform::IDENTITY);

    for i in 1..5 {
        let child = world.spawn();
        world.add_component(child, Name::new(format!("Level{}", i)));
        world.add_component(child, Transform::IDENTITY);
        set_parent(&mut world, child, current);
        current = child;
    }

    // Serialize
    let scene = Scene::from_world(&world);

    // Verify structure
    let mut current_data = &scene.entities[0];
    for i in 0..4 {
        assert_eq!(current_data.name, format!("Level{}", i));
        assert_eq!(current_data.children.len(), 1);
        current_data = &current_data.children[0];
    }
    assert_eq!(current_data.name, "Level4");
    assert_eq!(current_data.children.len(), 0);

    // Deserialize and verify
    let mut new_world = World::new();
    scene.spawn_into(&mut new_world);

    // Verify all levels exist
    for i in 0..5 {
        assert!(find_entity_by_name(&new_world, &format!("Level{}", i)).is_some());
    }

    // Verify hierarchy chain
    let level4 = find_entity_by_name(&new_world, "Level4").unwrap();
    let level3 = find_entity_by_name(&new_world, "Level3").unwrap();
    let level2 = find_entity_by_name(&new_world, "Level2").unwrap();
    let level1 = find_entity_by_name(&new_world, "Level1").unwrap();
    let level0 = find_entity_by_name(&new_world, "Level0").unwrap();

    assert_eq!(
        new_world.get_component::<Parent>(level4).unwrap().0,
        level3
    );
    assert_eq!(
        new_world.get_component::<Parent>(level3).unwrap().0,
        level2
    );
    assert_eq!(
        new_world.get_component::<Parent>(level2).unwrap().0,
        level1
    );
    assert_eq!(
        new_world.get_component::<Parent>(level1).unwrap().0,
        level0
    );
}

/// Test serialization with multiple root entities
/// 
/// Requirements: 8.6 - Include all entities in scene
#[test]
fn test_multiple_roots_serialization() {
    let mut world = World::new();

    // Create multiple root entities
    let root1 = world.spawn();
    world.add_component(root1, Name::new("Root1"));
    world.add_component(root1, Transform::IDENTITY);

    let root2 = world.spawn();
    world.add_component(root2, Name::new("Root2"));
    world.add_component(root2, Transform::IDENTITY);

    let root3 = world.spawn();
    world.add_component(root3, Name::new("Root3"));
    world.add_component(root3, Transform::IDENTITY);

    // Add children to each root
    let child1 = world.spawn();
    world.add_component(child1, Name::new("Child1"));
    world.add_component(child1, Transform::IDENTITY);
    set_parent(&mut world, child1, root1);

    let child2 = world.spawn();
    world.add_component(child2, Name::new("Child2"));
    world.add_component(child2, Transform::IDENTITY);
    set_parent(&mut world, child2, root2);

    // Serialize
    let scene = Scene::from_world(&world);

    // Should have 3 root entities
    assert_eq!(scene.entities.len(), 3);

    // Verify each root has correct children
    let root1_data = scene
        .entities
        .iter()
        .find(|e| e.name == "Root1")
        .expect("Root1 not found");
    assert_eq!(root1_data.children.len(), 1);
    assert_eq!(root1_data.children[0].name, "Child1");

    let root2_data = scene
        .entities
        .iter()
        .find(|e| e.name == "Root2")
        .expect("Root2 not found");
    assert_eq!(root2_data.children.len(), 1);
    assert_eq!(root2_data.children[0].name, "Child2");

    let root3_data = scene
        .entities
        .iter()
        .find(|e| e.name == "Root3")
        .expect("Root3 not found");
    assert_eq!(root3_data.children.len(), 0);

    // Deserialize and verify
    let mut new_world = World::new();
    let spawned = scene.spawn_into(&mut new_world);

    assert_eq!(spawned.len(), 5); // 3 roots + 2 children
}
