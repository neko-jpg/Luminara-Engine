//! Property test for ECS component updates
//!
//! **Validates: Requirements 12.1.2**
//!
//! This test verifies that component updates through the EngineHandle
//! correctly modify the ECS World state.

use luminara_editor::EngineHandle;
use luminara_core::{Component, impl_component};
use proptest::prelude::*;

// Test component for property testing
#[derive(Debug, Clone, PartialEq)]
struct TestComponent {
    value: i32,
}

impl_component!(TestComponent);

// Another test component
#[derive(Debug, Clone, PartialEq)]
struct PositionComponent {
    x: f32,
    y: f32,
    z: f32,
}

impl_component!(PositionComponent);

proptest! {
    /// Property 27: ECS Component Update
    ///
    /// **Property**: When a component is added or updated on an entity through EngineHandle,
    /// the component should be retrievable from the World with the same value.
    ///
    /// **Validates: Requirements 12.1.2**
    #[test]
    fn property_component_update_roundtrip(value in -1000i32..1000i32) {
        // Create a mock engine handle
        let handle = EngineHandle::mock();
        
        // Spawn an entity
        let entity = handle.spawn_entity();
        
        // Create a test component
        let component = TestComponent { value };
        
        // Update the component through the handle
        let result = handle.update_component(entity, component.clone());
        prop_assert!(result.is_ok(), "Failed to update component: {:?}", result);
        
        // Verify the component was added to the World
        let world = handle.world();
        let retrieved = world.get_component::<TestComponent>(entity);
        prop_assert!(retrieved.is_some(), "Component not found in World");
        prop_assert_eq!(retrieved.unwrap().value, value, "Component value mismatch");
    }

    /// Property: Multiple component updates should all be reflected in the World
    #[test]
    fn property_multiple_component_updates(
        value1 in -1000i32..1000i32,
        x in -100.0f32..100.0f32,
        y in -100.0f32..100.0f32,
        z in -100.0f32..100.0f32,
    ) {
        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();
        
        // Add first component
        let comp1 = TestComponent { value: value1 };
        handle.update_component(entity, comp1.clone()).unwrap();
        
        // Add second component
        let comp2 = PositionComponent { x, y, z };
        handle.update_component(entity, comp2.clone()).unwrap();
        
        // Verify both components exist
        let world = handle.world();
        let retrieved1 = world.get_component::<TestComponent>(entity);
        let retrieved2 = world.get_component::<PositionComponent>(entity);
        
        prop_assert!(retrieved1.is_some(), "TestComponent not found");
        prop_assert!(retrieved2.is_some(), "PositionComponent not found");
        prop_assert_eq!(retrieved1.unwrap().value, value1);
        prop_assert_eq!(retrieved2.unwrap().x, x);
        prop_assert_eq!(retrieved2.unwrap().y, y);
        prop_assert_eq!(retrieved2.unwrap().z, z);
    }

    /// Property: Updating a component multiple times should reflect the latest value
    #[test]
    fn property_component_update_overwrites(
        initial_value in -1000i32..1000i32,
        updated_value in -1000i32..1000i32,
    ) {
        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();
        
        // Add initial component
        let initial = TestComponent { value: initial_value };
        handle.update_component(entity, initial).unwrap();
        
        // Update with new value
        let updated = TestComponent { value: updated_value };
        handle.update_component(entity, updated).unwrap();
        
        // Verify the latest value is stored
        let world = handle.world();
        let retrieved = world.get_component::<TestComponent>(entity);
        prop_assert!(retrieved.is_some());
        prop_assert_eq!(retrieved.unwrap().value, updated_value, 
            "Component should have the updated value, not the initial value");
    }

    /// Property: Removing a component should make it unavailable in the World
    #[test]
    fn property_component_removal(value in -1000i32..1000i32) {
        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();
        
        // Add component
        let component = TestComponent { value };
        handle.update_component(entity, component).unwrap();
        
        // Verify it exists
        {
            let world = handle.world();
            prop_assert!(world.get_component::<TestComponent>(entity).is_some());
        }
        
        // Remove the component
        let result = handle.remove_component::<TestComponent>(entity);
        prop_assert!(result.is_ok(), "Failed to remove component: {:?}", result);
        
        // Verify it's gone
        let world = handle.world();
        prop_assert!(world.get_component::<TestComponent>(entity).is_none(),
            "Component should be removed from World");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_component_update_basic() {
        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();
        
        let component = TestComponent { value: 42 };
        let result = handle.update_component(entity, component);
        assert!(result.is_ok());
        
        let world = handle.world();
        let retrieved = world.get_component::<TestComponent>(entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
    }

    #[test]
    fn test_component_update_nonexistent_entity() {
        use luminara_core::Entity;
        
        let handle = EngineHandle::mock();
        
        // Create an entity that doesn't exist
        let fake_entity = Entity::from_raw(9999, 0);
        
        let component = TestComponent { value: 42 };
        let result = handle.update_component(fake_entity, component);
        assert!(result.is_err());
    }

    #[test]
    fn test_component_removal_basic() {
        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();
        
        // Add component
        let component = TestComponent { value: 42 };
        handle.update_component(entity, component).unwrap();
        
        // Remove it
        let result = handle.remove_component::<TestComponent>(entity);
        assert!(result.is_ok());
        
        // Verify it's gone
        let world = handle.world();
        assert!(world.get_component::<TestComponent>(entity).is_none());
    }
}

    #[test]
    fn test_duplicate_entity_command() {
        use luminara_editor::core::commands::DuplicateEntityCommand;
        use luminara_editor::services::engine_bridge::{EngineHandle, EditorCommand};

        let handle = EngineHandle::mock();
        let entity = handle.spawn_entity();

        // Initial count
        let count_before = handle.world().entities().len();

        // Execute command
        let command = Box::new(DuplicateEntityCommand::new(entity));
        handle.execute_command(command);

        // Final count
        let count_after = handle.world().entities().len();

        assert_eq!(count_after, count_before + 1, "Entity count should increase by 1 after duplication");
    }
