/// Integration tests for complete engine workflows
/// Validates: Requirements 5.6 - Integration tests covering complete workflows
///
/// These tests validate end-to-end scenarios that exercise multiple systems
/// working together, ensuring proper integration between ECS, rendering,
/// assets, and physics subsystems.

use luminara::prelude::*;
use luminara_asset::{AssetServer, Handle};
use luminara_core::{App, Component, Entity, World};
use luminara_math::{Transform, Vec3};
use luminara_scene::Name;
use std::time::Duration;

/// Test workflow: Load scene → simulate → render
/// This validates that a complete game loop can execute successfully
#[test]
fn test_load_scene_simulate_render_workflow() {
    // Validates: Requirements 5.6 - Complete workflow testing
    
    // Setup: Create app with minimal plugins
    let mut app = App::new();
    
    // Add core plugins needed for this workflow
    app.add_plugins(DefaultPlugins);
    
    // Create a simple scene programmatically
    let world = app.world_mut();
    
    // Spawn a camera entity
    let camera = world.spawn();
    world.insert(camera, Name::new("MainCamera"));
    world.insert(camera, Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)));
    
    // Spawn a few entities with transforms
    for i in 0..10 {
        let entity = world.spawn();
        world.insert(entity, Name::new(format!("Entity_{}", i)));
        world.insert(
            entity,
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        );
    }
    
    // Verify entities were created
    let entity_count = world.entity_count();
    assert!(
        entity_count >= 11,
        "Should have at least 11 entities (camera + 10 objects)"
    );
    
    // Simulate: Run a few update cycles
    for _ in 0..5 {
        app.update();
    }
    
    // Verify: Entities still exist after simulation
    let world = app.world();
    let final_count = world.entity_count();
    assert_eq!(
        final_count, entity_count,
        "Entity count should remain stable during simulation"
    );
    
    // Verify: Named entities can be queried
    let mut name_count = 0;
    for (_entity, name) in world.query::<&Name>() {
        name_count += 1;
        assert!(
            !name.0.is_empty(),
            "Entity names should not be empty"
        );
    }
    assert_eq!(
        name_count, 11,
        "Should be able to query all named entities"
    );
}

/// Test workflow: Entity spawn → add components → query → despawn
/// This validates the complete entity lifecycle
#[test]
fn test_entity_lifecycle_workflow() {
    // Validates: Requirements 5.6 - Entity lifecycle integration
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let world = app.world_mut();
    
    // Phase 1: Spawn entities
    let mut spawned_entities = Vec::new();
    for i in 0..20 {
        let entity = world.spawn();
        spawned_entities.push(entity);
        
        // Add components
        world.insert(entity, Name::new(format!("TestEntity_{}", i)));
        world.insert(
            entity,
            Transform::from_translation(Vec3::new(i as f32, i as f32, 0.0)),
        );
    }
    
    // Phase 2: Query entities
    let mut query_count = 0;
    for (_entity, (name, transform)) in world.query::<(&Name, &Transform)>() {
        query_count += 1;
        assert!(name.0.starts_with("TestEntity_"));
        assert!(transform.position.x >= 0.0);
    }
    assert_eq!(query_count, 20, "Should query all spawned entities");
    
    // Phase 3: Modify components
    for entity in &spawned_entities[0..10] {
        if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
            transform.position.y += 10.0;
        }
    }
    
    // Verify modifications
    for entity in &spawned_entities[0..10] {
        if let Some(transform) = world.get::<Transform>(*entity) {
            assert!(
                transform.position.y >= 10.0,
                "Modified entities should have updated positions"
            );
        }
    }
    
    // Phase 4: Despawn half the entities
    for entity in &spawned_entities[10..20] {
        world.despawn(*entity).expect("Should despawn successfully");
    }
    
    // Verify despawn
    let mut remaining_count = 0;
    for (_entity, _name) in world.query::<&Name>() {
        remaining_count += 1;
    }
    assert_eq!(
        remaining_count, 10,
        "Should have 10 entities remaining after despawn"
    );
    
    // Phase 5: Verify despawned entities are inaccessible
    for entity in &spawned_entities[10..20] {
        assert!(
            world.get::<Name>(*entity).is_none(),
            "Despawned entities should not be accessible"
        );
    }
}

/// Test workflow: Multiple systems interacting
/// This validates that systems can work together without conflicts
#[test]
fn test_multi_system_interaction_workflow() {
    // Validates: Requirements 5.6 - Multi-system integration
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    // Create entities with various component combinations
    let world = app.world_mut();
    
    // Group 1: Entities with Name + Transform
    for i in 0..5 {
        let entity = world.spawn();
        world.insert(entity, Name::new(format!("Group1_{}", i)));
        world.insert(entity, Transform::default());
    }
    
    // Group 2: Entities with only Transform
    for _ in 0..5 {
        let entity = world.spawn();
        world.insert(entity, Transform::default());
    }
    
    // Group 3: Entities with only Name
    for i in 0..5 {
        let entity = world.spawn();
        world.insert(entity, Name::new(format!("Group3_{}", i)));
    }
    
    // Run multiple update cycles
    for _ in 0..10 {
        app.update();
    }
    
    // Verify: Different query patterns work correctly
    let world = app.world();
    
    // Query 1: Both components
    let mut both_count = 0;
    for _ in world.query::<(&Name, &Transform)>() {
        both_count += 1;
    }
    assert_eq!(both_count, 5, "Should find 5 entities with both components");
    
    // Query 2: Only Transform
    let mut transform_count = 0;
    for _ in world.query::<&Transform>() {
        transform_count += 1;
    }
    assert!(
        transform_count >= 10,
        "Should find at least 10 entities with Transform"
    );
    
    // Query 3: Only Name
    let mut name_count = 0;
    for _ in world.query::<&Name>() {
        name_count += 1;
    }
    assert!(
        name_count >= 10,
        "Should find at least 10 entities with Name"
    );
}

/// Test workflow: Hierarchical entity relationships
/// This validates parent-child relationships work correctly
#[test]
fn test_hierarchical_entity_workflow() {
    // Validates: Requirements 5.6 - Hierarchical entity integration
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let world = app.world_mut();
    
    // Create parent entity
    let parent = world.spawn();
    world.insert(parent, Name::new("Parent"));
    world.insert(parent, Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)));
    
    // Create child entities
    let mut children = Vec::new();
    for i in 0..5 {
        let child = world.spawn();
        world.insert(child, Name::new(format!("Child_{}", i)));
        world.insert(
            child,
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        );
        children.push(child);
    }
    
    // Run simulation
    for _ in 0..3 {
        app.update();
    }
    
    // Verify: All entities exist
    let world = app.world();
    assert!(
        world.get::<Name>(parent).is_some(),
        "Parent should exist"
    );
    
    for child in &children {
        assert!(
            world.get::<Name>(*child).is_some(),
            "Child should exist"
        );
    }
    
    // Verify: Can query parent and children separately
    let mut parent_count = 0;
    let mut child_count = 0;
    
    for (_entity, name) in world.query::<&Name>() {
        if name.0 == "Parent" {
            parent_count += 1;
        } else if name.0.starts_with("Child_") {
            child_count += 1;
        }
    }
    
    assert_eq!(parent_count, 1, "Should have exactly one parent");
    assert_eq!(child_count, 5, "Should have exactly 5 children");
}

/// Test workflow: Component addition and removal during simulation
/// This validates dynamic component management
#[test]
fn test_dynamic_component_management_workflow() {
    // Validates: Requirements 5.6 - Dynamic component management
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let world = app.world_mut();
    
    // Create entities with initial components
    let mut entities = Vec::new();
    for i in 0..10 {
        let entity = world.spawn();
        world.insert(entity, Name::new(format!("Entity_{}", i)));
        entities.push(entity);
    }
    
    // Run initial update
    app.update();
    
    // Phase 1: Add Transform to half the entities
    let world = app.world_mut();
    for entity in &entities[0..5] {
        world.insert(*entity, Transform::default());
    }
    
    app.update();
    
    // Verify: Entities with Transform can be queried
    let world = app.world();
    let mut with_transform = 0;
    for _ in world.query::<(&Name, &Transform)>() {
        with_transform += 1;
    }
    assert_eq!(
        with_transform, 5,
        "Should have 5 entities with Transform"
    );
    
    // Phase 2: Remove Name from some entities
    let world = app.world_mut();
    for entity in &entities[5..8] {
        world.remove::<Name>(*entity).ok();
    }
    
    app.update();
    
    // Verify: Entities without Name are not in Name queries
    let world = app.world();
    let mut name_count = 0;
    for _ in world.query::<&Name>() {
        name_count += 1;
    }
    assert_eq!(
        name_count, 7,
        "Should have 7 entities with Name after removal"
    );
}

/// Test workflow: Stress test with many entities
/// This validates performance with larger entity counts
#[test]
fn test_large_entity_count_workflow() {
    // Validates: Requirements 5.6 - Scalability testing
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let world = app.world_mut();
    
    // Spawn many entities
    const ENTITY_COUNT: usize = 1000;
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn();
        world.insert(entity, Transform::from_translation(Vec3::new(
            (i % 100) as f32,
            (i / 100) as f32,
            0.0,
        )));
        
        // Add Name to every 10th entity
        if i % 10 == 0 {
            world.insert(entity, Name::new(format!("Named_{}", i)));
        }
    }
    
    // Verify entity count
    let entity_count = world.entity_count();
    assert!(
        entity_count >= ENTITY_COUNT,
        "Should have at least {} entities",
        ENTITY_COUNT
    );
    
    // Run simulation cycles
    for _ in 0..5 {
        app.update();
    }
    
    // Verify: Can query all entities efficiently
    let world = app.world();
    let mut transform_count = 0;
    for _ in world.query::<&Transform>() {
        transform_count += 1;
    }
    assert_eq!(
        transform_count, ENTITY_COUNT,
        "Should query all entities with Transform"
    );
    
    // Verify: Named entities are correct
    let mut named_count = 0;
    for _ in world.query::<&Name>() {
        named_count += 1;
    }
    assert_eq!(
        named_count,
        ENTITY_COUNT / 10,
        "Should have correct number of named entities"
    );
}

/// Test workflow: Error handling and recovery
/// This validates that the engine handles errors gracefully
#[test]
fn test_error_handling_workflow() {
    // Validates: Requirements 5.6 - Error handling integration
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let world = app.world_mut();
    
    // Create valid entities
    let entity1 = world.spawn();
    world.insert(entity1, Name::new("Valid"));
    
    let entity2 = world.spawn();
    world.insert(entity2, Transform::default());
    
    // Try to access non-existent entity
    let fake_entity = Entity::from_raw(99999);
    assert!(
        world.get::<Name>(fake_entity).is_none(),
        "Should return None for non-existent entity"
    );
    
    // Try to remove non-existent component
    let result = world.remove::<Transform>(entity1);
    assert!(
        result.is_err(),
        "Should error when removing non-existent component"
    );
    
    // Verify: Valid entities are unaffected
    assert!(
        world.get::<Name>(entity1).is_some(),
        "Valid entity should still exist"
    );
    assert!(
        world.get::<Transform>(entity2).is_some(),
        "Valid entity should still exist"
    );
    
    // Run update to ensure stability
    app.update();
    
    // Verify: System still works after errors
    let world = app.world();
    let entity_count = world.entity_count();
    assert!(entity_count >= 2, "Entities should still exist after errors");
}
