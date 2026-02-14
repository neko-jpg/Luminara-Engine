use crate::Asset;
use luminara_core::shared_types::Component;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_path(path: &str) -> Self {
        Self(Uuid::new_v5(&Uuid::NAMESPACE_URL, path.as_bytes()))
    }
}

pub struct Handle<T: Asset> {
    id: AssetId,
    generation: u32,
    _marker: PhantomData<T>,
}

impl<T: Asset> Handle<T> {
    pub fn new(id: AssetId, generation: u32) -> Self {
        Self {
            id,
            generation,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            generation: self.generation,
            _marker: PhantomData,
        }
    }
}

impl<T: Asset> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).finish()
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Asset> Serialize for Handle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T: Asset> Deserialize<'de> for Handle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = AssetId::deserialize(deserializer)?;
        Ok(Handle::new(id, 0)) // Default generation 0 for deserialized handles
    }
}

impl<T: Asset> Component for Handle<T> {
    fn type_name() -> &'static str {
        // This relies on T::type_name() being stable
        std::any::type_name::<Handle<T>>()
    }
}
