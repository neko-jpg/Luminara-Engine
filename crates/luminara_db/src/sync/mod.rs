use luminara_core::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Persistent {
    pub auto_save: bool,
    pub db_id: Option<String>,
    pub last_saved_hash: Option<u64>,
}

impl Default for Persistent {
    fn default() -> Self {
        Self {
            auto_save: true,
            db_id: None,
            last_saved_hash: None,
        }
    }
}

impl_component!(Persistent);

#[derive(Debug, Clone, Default)]
pub struct SaveExclude;

impl_component!(SaveExclude);

#[derive(Debug, Clone)]
pub struct DbDirty {
    pub changed_components: Vec<String>,
}

impl_component!(DbDirty);

// Temporarily disabled - incomplete implementation
// pub mod snapshot;
// pub mod restore;
// pub mod commands;

// WorldSync module for ECS synchronization
pub mod world_sync;

pub use world_sync::{WorldSync, SyncStatistics, SyncResult};

// Registry Logic

pub trait ComponentSerializer: Send + Sync {
    fn serialize(&self, world: &World, entity: Entity) -> Option<serde_json::Value>;
    fn deserialize(&self, world: &mut World, entity: Entity, data: &serde_json::Value) -> Result<(), crate::error::DbError>;
    fn type_name(&self) -> &'static str;
}

#[derive(Clone, Default)]
pub struct ComponentRegistry {
    serializers: Arc<RwLock<HashMap<String, Arc<dyn ComponentSerializer>>>>,
    type_id_map: Arc<RwLock<HashMap<TypeId, String>>>,
}

impl Resource for ComponentRegistry {}

impl ComponentRegistry {
    pub fn register<T: Component + serde::Serialize + for<'de> serde::Deserialize<'de>>(&self) {
        let name = T::type_name();
        let serializer = Arc::new(TypedComponentSerializer::<T>::default());

        {
            let mut serializers = self.serializers.write().unwrap();
            serializers.insert(name.to_string(), serializer);
        }

        {
            let mut type_map = self.type_id_map.write().unwrap();
            type_map.insert(TypeId::of::<T>(), name.to_string());
        }
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ComponentSerializer>> {
        let serializers = self.serializers.read().unwrap();
        serializers.get(name).cloned()
    }

    pub fn get_all(&self) -> Vec<Arc<dyn ComponentSerializer>> {
        let serializers = self.serializers.read().unwrap();
        serializers.values().cloned().collect()
    }
}

struct TypedComponentSerializer<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for TypedComponentSerializer<T> {
    fn default() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

impl<T: Component + serde::Serialize + for<'de> serde::Deserialize<'de>> ComponentSerializer for TypedComponentSerializer<T> {
    fn serialize(&self, world: &World, entity: Entity) -> Option<serde_json::Value> {
        if let Some(comp) = world.get_component::<T>(entity) {
            serde_json::to_value(comp).ok()
        } else {
            None
        }
    }

    fn deserialize(&self, world: &mut World, entity: Entity, data: &serde_json::Value) -> Result<(), crate::error::DbError> {
        let comp: T = serde_json::from_value(data.clone())?;
        world.add_component(entity, comp).map_err(|_| crate::error::DbError::Other("Failed to add component".into()))?;
        Ok(())
    }

    fn type_name(&self) -> &'static str {
        T::type_name()
    }
}
