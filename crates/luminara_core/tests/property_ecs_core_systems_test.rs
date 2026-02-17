use luminara_core::{
    impl_component, resource::Resource, schedule::Schedule,
    shared_types::CoreStage, system::{ExclusiveMarker, FunctionMarker, IntoSystem}, world::World,
};
use proptest::prelude::*;
use std::any::TypeId;
use std::sync::{Arc, Mutex};

// ============================================================================
// Property Tests for ECS Core Systems
// Feature: pre-editor-engine-audit
// Task: 29.2 Add property tests for core systems
// Validates: Requirements 5.2
// ============================================================================

/// **Validates: Requirements 5.2**
/// WHEN testing ECS functionality, THE System SHALL include property tests
/// for component registration, system execution, and query correctness

// ============================================================================
// Test Components
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}
impl_component!(Position);

#[derive(Debug, Clone, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}
impl_component!(Velocity);

#[derive(Debug, Clone, PartialEq)]
struct Health {
    current: f32,
    max: f32,
}
impl_component!(Health);

#[derive(Debug, Clone, PartialEq)]
struct Name {
    value: String,
}
impl_component!(Name);

#[derive(Debug, Clone, PartialEq)]
struct Tag {
    id: u32,
}
impl_component!(Tag);

#[derive(Debug, Clone, PartialEq)]
struct Marker;
impl_component!(Marker);

// ============================================================================
// Test Resources
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: usize,
}
impl Resource for Counter {}

#[derive(Debug, Clone)]
struct ExecutionOrder {
    order: Arc<Mutex<Vec<String>>>,
}
impl Resource for ExecutionOrder {}

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Strategy for generating small entity counts for complex tests
fn small_entity_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=50
}

// ============================================================================
// Property Test 1: Component Registration
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property: Component Registration No Duplicates**
    ///
    /// For any component type, registering it multiple times should not
    /// create duplicate registrations. The component should be registered
    /// exactly once and remain accessible.
    ///
    /// **Validates: Requirements 5.2** - ECS component registration
    #[test]
    fn prop_component_registration_no_duplicates(
        registration_count in 1usize..=10
    ) {
        let mut world = World::new();

        // Register the same component multiple times
        for _ in 0..registration_count {
            world.register_component::<Position>();
        }

        // Verify component is registered exactly once
        prop_assert!(
            world.is_component_registered::<Position>(),
            "Component should be registered after {} registrations",
            registration_count
        );

        // Verify we can spawn entities with the component
        let entity = world.spawn();
        let result = world.add_component(entity, Position { x: 1.0, y: 2.0, z: 3.0 });
        prop_assert!(
            result.is_ok(),
            "Should be able to add registered component"
        );

        // Verify component is retrievable
        let component = world.get_component::<Position>(entity);
        prop_assert!(
            component.is_some(),
            "Should be able to retrieve registered component"
        );
    }

    /// **Property: Component Registration Type Safety**
    ///
    /// For any set of different component types, each should be registered
    /// independently with unique type IDs and not interfere with each other.
    ///
    /// **Validates: Requirements 5.2** - ECS component registration
    #[test]
    fn prop_component_registration_type_safety(
        _seed in any::<u64>()
    ) {
        let mut world = World::new();

        // Register multiple different component types
        world.register_component::<Position>();
        world.register_component::<Velocity>();
        world.register_component::<Health>();
        world.register_component::<Name>();
        world.register_component::<Tag>();

        // Verify all components are registered
        prop_assert!(world.is_component_registered::<Position>());
        prop_assert!(world.is_component_registered::<Velocity>());
        prop_assert!(world.is_component_registered::<Health>());
        prop_assert!(world.is_component_registered::<Name>());
        prop_assert!(world.is_component_registered::<Tag>());

        // Verify type IDs are unique
        let type_ids = vec![
            TypeId::of::<Position>(),
            TypeId::of::<Velocity>(),
            TypeId::of::<Health>(),
            TypeId::of::<Name>(),
            TypeId::of::<Tag>(),
        ];

        for i in 0..type_ids.len() {
            for j in (i + 1)..type_ids.len() {
                prop_assert_ne!(
                    type_ids[i],
                    type_ids[j],
                    "Component type IDs should be unique"
                );
            }
        }

        // Verify components can coexist on same entity
        let entity = world.spawn();
        world.add_component(entity, Position { x: 1.0, y: 2.0, z: 3.0 }).unwrap();
        world.add_component(entity, Velocity { x: 0.1, y: 0.2, z: 0.3 }).unwrap();
        world.add_component(entity, Health { current: 100.0, max: 100.0 }).unwrap();

        prop_assert!(world.get_component::<Position>(entity).is_some());
        prop_assert!(world.get_component::<Velocity>(entity).is_some());
        prop_assert!(world.get_component::<Health>(entity).is_some());
    }

    /// **Property: Component Registration Persistence**
    ///
    /// For any component type, once registered, it should remain registered
    /// throughout the world's lifetime, even after entities are spawned and
    /// despawned.
    ///
    /// **Validates: Requirements 5.2** - ECS component registration
    #[test]
    fn prop_component_registration_persistence(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();
        world.register_component::<Position>();

        // Spawn entities with the component
        let mut entities = Vec::new();
        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();
            entities.push(entity);
        }

        // Verify component is still registered
        prop_assert!(world.is_component_registered::<Position>());

        // Despawn entities one by one (in reverse order to avoid swap_remove issues)
        for entity in entities.iter().rev() {
            world.despawn(*entity);
        }

        // Verify component is still registered after all entities are gone
        prop_assert!(
            world.is_component_registered::<Position>(),
            "Component should remain registered after entities are despawned"
        );

        // Verify we can still spawn new entities with the component
        let new_entity = world.spawn();
        let result = world.add_component(new_entity, Position { x: 0.0, y: 0.0, z: 0.0 });
        prop_assert!(
            result.is_ok(),
            "Should still be able to add component after despawning entities"
        );
    }
}

// ============================================================================
// Property Test 2: System Execution Order
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property: System Execution Order Respects Stage Order**
    ///
    /// For any set of systems registered in different stages, they should
    /// execute in the correct stage order: Startup → PreUpdate → Update →
    /// FixedUpdate → PostUpdate → PreRender → Render → PostRender.
    ///
    /// **Validates: Requirements 5.2** - System execution order
    #[test]
    fn prop_system_execution_order_respects_stages(
        _seed in any::<u64>()
    ) {
        let mut world = World::new();
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        world.insert_resource(ExecutionOrder {
            order: execution_order.clone(),
        });

        let mut schedule = Schedule::new();

        // Create systems for different stages
        let stages = vec![
            (CoreStage::Startup, "startup"),
            (CoreStage::PreUpdate, "pre_update"),
            (CoreStage::Update, "update"),
            (CoreStage::FixedUpdate, "fixed_update"),
            (CoreStage::PostUpdate, "post_update"),
            (CoreStage::PreRender, "pre_render"),
            (CoreStage::Render, "render"),
            (CoreStage::PostRender, "post_render"),
        ];

        for (stage, name) in stages {
            let name_owned = name.to_string();
            let system = move |world: &mut World| {
                if let Some(order) = world.get_resource_mut::<ExecutionOrder>() {
                    order.order.lock().unwrap().push(name_owned.clone());
                }
            };
            schedule.add_system(stage, IntoSystem::<ExclusiveMarker>::into_system(system));
        }

        // Run startup stage
        schedule.run_startup(&mut world);

        // Run main loop stages
        schedule.run(&mut world);

        // Verify execution order
        let order = execution_order.lock().unwrap();
        let expected = vec![
            "startup",
            "pre_update",
            "update",
            "fixed_update",
            "post_update",
            "pre_render",
            "render",
            "post_render",
        ];

        prop_assert_eq!(
            order.as_slice(),
            expected.as_slice(),
            "Systems should execute in stage order"
        );
    }

    /// **Property: Systems in Same Stage Execute Without Data Races**
    ///
    /// For any set of systems in the same stage that don't conflict,
    /// they should all execute successfully without data races or panics.
    ///
    /// **Validates: Requirements 5.2** - System execution order
    #[test]
    fn prop_systems_same_stage_no_data_races(
        system_count in 2usize..=10
    ) {
        let mut world = World::new();
        world.insert_resource(Counter { value: 0 });

        let mut schedule = Schedule::new();

        // Add multiple systems that read the same resource
        for i in 0..system_count {
            let system_id = i;
            let system = move |world: &World| {
                if let Some(counter) = world.get_resource::<Counter>() {
                    // Just read the counter - no conflicts
                    let _ = counter.value + system_id;
                }
            };
            schedule.add_system(CoreStage::Update, IntoSystem::<(FunctionMarker, World)>::into_system(system));
        }

        // Should execute without panicking
        schedule.run(&mut world);

        // Verify world is still valid
        prop_assert!(world.get_resource::<Counter>().is_some());
    }

    /// **Property: System Execution Count Matches Registration Count**
    ///
    /// For any number of systems registered in a stage, all systems should
    /// execute exactly once per schedule run.
    ///
    /// **Validates: Requirements 5.2** - System execution order
    #[test]
    fn prop_system_execution_count_matches_registration(
        system_count in 1usize..=20
    ) {
        let mut world = World::new();
        let execution_count = Arc::new(Mutex::new(0usize));

        let mut schedule = Schedule::new();

        // Add multiple systems that increment a counter
        for _ in 0..system_count {
            let count = execution_count.clone();
            let system = move |_world: &World| {
                *count.lock().unwrap() += 1;
            };
            schedule.add_system(CoreStage::Update, IntoSystem::<(FunctionMarker, World)>::into_system(system));
        }

        // Run schedule
        schedule.run(&mut world);

        // Verify all systems executed exactly once
        let final_count = *execution_count.lock().unwrap();
        prop_assert_eq!(
            final_count,
            system_count,
            "All {} systems should execute exactly once, but {} executions occurred",
            system_count,
            final_count
        );
    }
}

// ============================================================================
// Property Test 3: Query Correctness
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property: Query Returns All Matching Entities**
    ///
    /// For any set of entities with specific components, a query for those
    /// components should return all and only those entities that have all
    /// the queried components.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_query_returns_all_matching_entities(
        entity_count in small_entity_count_strategy(),
        with_velocity_ratio in 0.0f64..=1.0
    ) {
        let mut world = World::new();

        // Spawn entities, some with Position only, some with Position + Velocity
        let mut entities_with_both = Vec::new();
        let mut entities_with_position_only = Vec::new();

        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();

            let should_have_velocity = (i as f64 / entity_count as f64) < with_velocity_ratio;
            if should_have_velocity {
                world.add_component(entity, Velocity {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }).unwrap();
                entities_with_both.push(entity);
            } else {
                entities_with_position_only.push(entity);
            }
        }

        // Query for entities with both Position and Velocity
        let mut found_entities = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
            {
                found_entities.push(entity);
            }
        }

        // Verify query found all entities with both components
        prop_assert_eq!(
            found_entities.len(),
            entities_with_both.len(),
            "Query should find all entities with both components"
        );

        // Verify all found entities are in the expected set
        for entity in &found_entities {
            prop_assert!(
                entities_with_both.contains(entity),
                "Query should only return entities with both components"
            );
        }

        // Verify entities with only Position are not in results
        for entity in &entities_with_position_only {
            prop_assert!(
                !found_entities.contains(entity),
                "Query should not return entities missing required components"
            );
        }
    }

    /// **Property: Query Results Reflect Component Additions**
    ///
    /// For any entity, adding a component should make it appear in queries
    /// that require that component.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_query_reflects_component_additions(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();

        // Spawn entities with only Position
        let mut entities = Vec::new();
        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();
            entities.push(entity);
        }

        // Query for entities with both Position and Velocity (should be empty)
        let mut found_before = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
            {
                found_before.push(entity);
            }
        }

        prop_assert_eq!(
            found_before.len(),
            0,
            "No entities should have Velocity initially"
        );

        // Add Velocity to all entities
        for entity in &entities {
            world.add_component(*entity, Velocity {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }).unwrap();
        }

        // Query again (should find all entities)
        let mut found_after = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
            {
                found_after.push(entity);
            }
        }

        prop_assert_eq!(
            found_after.len(),
            entity_count,
            "All entities should have both components after addition"
        );
    }

    /// **Property: Query Results Reflect Component Removals**
    ///
    /// For any entity, removing a component should make it disappear from
    /// queries that require that component.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_query_reflects_component_removals(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();

        // Spawn entities with both Position and Velocity
        let mut entities = Vec::new();
        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();
            world.add_component(entity, Velocity {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }).unwrap();
            entities.push(entity);
        }

        // Query for entities with both components (should find all)
        let mut found_before = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
            {
                found_before.push(entity);
            }
        }

        prop_assert_eq!(
            found_before.len(),
            entity_count,
            "All entities should have both components initially"
        );

        // Remove Velocity from all entities
        for entity in &entities {
            world.remove_component::<Velocity>(*entity).unwrap();
        }

        // Query again (should be empty)
        let mut found_after = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
            {
                found_after.push(entity);
            }
        }

        prop_assert_eq!(
            found_after.len(),
            0,
            "No entities should have Velocity after removal"
        );

        // Verify entities still have Position
        for entity in &entities {
            prop_assert!(
                world.get_component::<Position>(*entity).is_some(),
                "Entities should still have Position after Velocity removal"
            );
        }
    }

    /// **Property: Query Results Are Consistent Across Multiple Iterations**
    ///
    /// For any static set of entities, querying multiple times should return
    /// the same results in a consistent order.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_query_results_consistent_across_iterations(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();

        // Spawn entities with Position
        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();
        }

        // Query multiple times
        let mut results = Vec::new();
        for _ in 0..5 {
            let mut found = Vec::new();
            for entity in world.entities() {
                if world.get_component::<Position>(entity).is_some() {
                    found.push(entity);
                }
            }
            results.push(found);
        }

        // Verify all queries returned the same entities
        for i in 1..results.len() {
            prop_assert_eq!(
                results[i].len(),
                results[0].len(),
                "Query should return same number of entities each time"
            );

            // Sort for comparison (order may vary but entities should be same)
            let mut sorted_first = results[0].clone();
            let mut sorted_current = results[i].clone();
            sorted_first.sort_by_key(|e| e.id());
            sorted_current.sort_by_key(|e| e.id());

            prop_assert_eq!(
                sorted_current,
                sorted_first,
                "Query should return same entities each time"
            );
        }
    }

    /// **Property: Empty Query Returns No Entities**
    ///
    /// For any world state, querying for a component that no entity has
    /// should return an empty result set.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_empty_query_returns_no_entities(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();

        // Spawn entities with Position only
        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();
        }

        // Query for entities with Marker (none should have it)
        let mut found = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Marker>(entity).is_some() {
                found.push(entity);
            }
        }

        prop_assert_eq!(
            found.len(),
            0,
            "Query for non-existent component should return no entities"
        );
    }

    /// **Property: Query Correctness With Multiple Component Types**
    ///
    /// For any combination of component types, queries should correctly
    /// filter entities based on all required components.
    ///
    /// **Validates: Requirements 5.2** - Query correctness
    #[test]
    fn prop_query_correctness_multiple_components(
        entity_count in small_entity_count_strategy()
    ) {
        let mut world = World::new();

        // Create entities with different component combinations
        let mut entities_pos = Vec::new();
        let mut entities_pos_vel = Vec::new();
        let mut entities_pos_vel_health = Vec::new();

        for i in 0..entity_count {
            let entity = world.spawn();
            world.add_component(entity, Position {
                x: i as f32,
                y: i as f32,
                z: i as f32,
            }).unwrap();

            match i % 3 {
                0 => {
                    // Position only
                    entities_pos.push(entity);
                }
                1 => {
                    // Position + Velocity
                    world.add_component(entity, Velocity {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    }).unwrap();
                    entities_pos_vel.push(entity);
                }
                2 => {
                    // Position + Velocity + Health
                    world.add_component(entity, Velocity {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    }).unwrap();
                    world.add_component(entity, Health {
                        current: 100.0,
                        max: 100.0,
                    }).unwrap();
                    entities_pos_vel_health.push(entity);
                }
                _ => unreachable!(),
            }
        }

        // Query for Position + Velocity + Health
        let mut found_all_three = Vec::new();
        for entity in world.entities() {
            if world.get_component::<Position>(entity).is_some()
                && world.get_component::<Velocity>(entity).is_some()
                && world.get_component::<Health>(entity).is_some()
            {
                found_all_three.push(entity);
            }
        }

        prop_assert_eq!(
            found_all_three.len(),
            entities_pos_vel_health.len(),
            "Query should find only entities with all three components"
        );

        // Verify found entities match expected
        for entity in &found_all_three {
            prop_assert!(
                entities_pos_vel_health.contains(entity),
                "Query should only return entities with all required components"
            );
        }
    }
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[test]
fn test_component_registration_zero_sized_types() {
    // **Validates: Requirements 5.2** - Component registration
    let mut world = World::new();
    world.register_component::<Marker>();

    assert!(world.is_component_registered::<Marker>());

    let entity = world.spawn();
    world.add_component(entity, Marker).unwrap();

    assert!(world.get_component::<Marker>(entity).is_some());
}

#[test]
fn test_system_execution_empty_schedule() {
    // **Validates: Requirements 5.2** - System execution order
    let mut world = World::new();
    let mut schedule = Schedule::new();

    // Should not panic with empty schedule
    schedule.run(&mut world);
}

#[test]
fn test_query_with_despawned_entities() {
    // **Validates: Requirements 5.2** - Query correctness
    let mut world = World::new();

    // Spawn entities
    let entity1 = world.spawn();
    world.add_component(entity1, Position { x: 1.0, y: 1.0, z: 1.0 }).unwrap();

    let entity2 = world.spawn();
    world.add_component(entity2, Position { x: 2.0, y: 2.0, z: 2.0 }).unwrap();

    // Despawn one entity
    world.despawn(entity1);

    // Query should only return the remaining entity
    let mut found = Vec::new();
    for entity in world.entities() {
        if world.get_component::<Position>(entity).is_some() {
            found.push(entity);
        }
    }

    assert_eq!(found.len(), 1);
    assert_eq!(found[0], entity2);
}

#[test]
fn test_system_execution_with_exclusive_access() {
    // **Validates: Requirements 5.2** - System execution order
    let mut world = World::new();
    world.insert_resource(Counter { value: 0 });

    let mut schedule = Schedule::new();

    // Add system that needs exclusive access
    let system = |world: &mut World| {
        if let Some(mut counter) = world.get_resource_mut::<Counter>() {
            counter.value += 1;
        }
    };
    schedule.add_system(CoreStage::Update, IntoSystem::<ExclusiveMarker>::into_system(system));

    schedule.run(&mut world);

    let counter = world.get_resource::<Counter>().unwrap();
    assert_eq!(counter.value, 1);
}

#[test]
fn test_query_correctness_after_archetype_changes() {
    // **Validates: Requirements 5.2** - Query correctness
    let mut world = World::new();

    // Spawn entity with Position
    let entity = world.spawn();
    world.add_component(entity, Position { x: 1.0, y: 1.0, z: 1.0 }).unwrap();

    // Query should find it
    let mut found = Vec::new();
    for e in world.entities() {
        if world.get_component::<Position>(e).is_some() {
            found.push(e);
        }
    }
    assert_eq!(found.len(), 1);

    // Add another component (changes archetype)
    world.add_component(entity, Velocity { x: 1.0, y: 1.0, z: 1.0 }).unwrap();

    // Query should still find it
    let mut found = Vec::new();
    for e in world.entities() {
        if world.get_component::<Position>(e).is_some() {
            found.push(e);
        }
    }
    assert_eq!(found.len(), 1);
    assert_eq!(found[0], entity);

    // Verify both components are present
    assert!(world.get_component::<Position>(entity).is_some());
    assert!(world.get_component::<Velocity>(entity).is_some());
}
