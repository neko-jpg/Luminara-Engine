/// Placeholder asset system for displaying temporary assets during loading
///
/// This module provides placeholder assets (meshes, textures) that are displayed
/// while real assets are loading asynchronously. When the real asset completes
/// loading, it is hot-swapped with the placeholder without frame drops.

use crate::Asset;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for placeholder assets
pub struct PlaceholderRegistry {
    placeholders: Arc<RwLock<HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>>>>,
}

impl PlaceholderRegistry {
    pub fn new() -> Self {
        Self {
            placeholders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a placeholder asset for a specific type
    pub fn register<T: Asset>(&self, placeholder: T) {
        let mut placeholders = self.placeholders.write().unwrap();
        placeholders.insert(TypeId::of::<T>(), Arc::new(placeholder));
    }

    /// Get a placeholder asset for a specific type
    pub fn get<T: Asset>(&self) -> Option<Arc<T>> {
        let placeholders = self.placeholders.read().unwrap();
        placeholders
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    /// Check if a placeholder exists for a type
    pub fn has_placeholder<T: Asset>(&self) -> bool {
        let placeholders = self.placeholders.read().unwrap();
        placeholders.contains_key(&TypeId::of::<T>())
    }
}

impl Default for PlaceholderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for creating default placeholder assets
pub trait PlaceholderAsset: Asset {
    /// Create a default placeholder for this asset type
    fn create_placeholder() -> Self
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestAsset {
        value: i32,
    }

    impl Asset for TestAsset {
        fn type_name() -> &'static str {
            "TestAsset"
        }
    }

    #[test]
    fn test_placeholder_registry() {
        let registry = PlaceholderRegistry::new();

        // Register placeholder
        registry.register(TestAsset { value: 42 });

        // Retrieve placeholder
        let placeholder = registry.get::<TestAsset>();
        assert!(placeholder.is_some());
        assert_eq!(placeholder.unwrap().value, 42);
    }

    #[test]
    fn test_has_placeholder() {
        let registry = PlaceholderRegistry::new();

        assert!(!registry.has_placeholder::<TestAsset>());

        registry.register(TestAsset { value: 42 });

        assert!(registry.has_placeholder::<TestAsset>());
    }

    #[test]
    fn test_placeholder_not_found() {
        let registry = PlaceholderRegistry::new();
        let placeholder = registry.get::<TestAsset>();
        assert!(placeholder.is_none());
    }
}
