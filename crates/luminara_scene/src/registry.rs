use luminara_core::shared_types::{Component, Entity, Resource, World};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// Trait for type-erased component operations, specifically deserialization and insertion into World.
pub trait ReflectComponent: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn add_to_entity(&self, world: &mut World, entity: Entity, value: Value) -> Result<(), String>;
}

/// Helper struct to implement ReflectComponent for any Component that is DeserializeOwned.
pub struct ComponentRegistration<T: Component + DeserializeOwned> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Component + DeserializeOwned> ComponentRegistration<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Component + DeserializeOwned> Default for ComponentRegistration<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component + DeserializeOwned> ReflectComponent for ComponentRegistration<T> {
    fn type_name(&self) -> &'static str {
        T::type_name()
    }

    fn add_to_entity(&self, world: &mut World, entity: Entity, value: Value) -> Result<(), String> {
        match serde_json::from_value::<T>(value) {
            Ok(component) => {
                let _ = world.add_component(entity, component);
                Ok(())
            }
            Err(e) => Err(format!(
                "Failed to deserialize component {}: {}",
                T::type_name(),
                e
            )),
        }
    }
}

/// Global registry for component types.
/// Allows looking up component handling logic by type name (string).
#[derive(Default)]
pub struct TypeRegistry {
    registrations: HashMap<String, Box<dyn ReflectComponent>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
        }
    }

    /// Register a component type for automatic deserialization.
    pub fn register<T: Component + DeserializeOwned>(&mut self) {
        let registration = ComponentRegistration::<T>::new();
        self.registrations
            .insert(T::type_name().to_string(), Box::new(registration));
    }

    /// Deserialize a component from JSON value and add it to the entity.
    pub fn deserialize_and_add(
        &self,
        world: &mut World,
        entity: Entity,
        type_name: &str,
        value: Value,
    ) -> Result<(), String> {
        if let Some(registration) = self.registrations.get(type_name) {
            registration.add_to_entity(world, entity, value)
        } else {
            Err(format!(
                "Component type '{}' not registered in TypeRegistry",
                type_name
            ))
        }
    }
}

impl Resource for TypeRegistry {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestComponent {
        value: i32,
    }

    impl Component for TestComponent {
        fn type_name() -> &'static str {
            "TestComponent"
        }
    }

    #[test]
    fn test_registry_deserialization() {
        let mut world = World::new();
        let mut registry = TypeRegistry::new();

        // Register the component
        registry.register::<TestComponent>();

        // Create an entity
        let entity = world.spawn();

        // Deserialize and add component
        let json_value = serde_json::json!({
            "value": 42
        });

        let result = registry.deserialize_and_add(&mut world, entity, "TestComponent", json_value);
        assert!(result.is_ok());

        // Verify component exists on entity
        let component = world.get_component::<TestComponent>(entity);
        assert!(component.is_some());
        assert_eq!(component.unwrap().value, 42);
    }

    #[test]
    fn test_registry_missing_component() {
        let mut world = World::new();
        let registry = TypeRegistry::new();
        let entity = world.spawn();

        let result = registry.deserialize_and_add(
            &mut world,
            entity,
            "MissingComponent",
            serde_json::Value::Null,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Component type 'MissingComponent' not registered in TypeRegistry"
        );
    }

    #[test]
    fn test_registry_invalid_json() {
        let mut world = World::new();
        let mut registry = TypeRegistry::new();
        registry.register::<TestComponent>();
        let entity = world.spawn();

        // Invalid JSON for TestComponent (string instead of int)
        let json_value = serde_json::json!({
            "value": "invalid"
        });

        let result = registry.deserialize_and_add(&mut world, entity, "TestComponent", json_value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to deserialize component"));
    }
}
