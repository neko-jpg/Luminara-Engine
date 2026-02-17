use crate::Asset;
use luminara_core::shared_types::Component;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
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

    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_nil()
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

impl<T: Asset> Default for Handle<T> {
    fn default() -> Self {
        Self::new(AssetId::default(), 0)
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

// Implement Reflect for Handle<T>
impl<T: Asset> luminara_core::Reflect for Handle<T> {
    fn type_info(&self) -> &luminara_core::TypeInfo {
        use std::any::TypeId;
        use std::sync::OnceLock;
        static INFO: OnceLock<luminara_core::TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| luminara_core::TypeInfo {
            type_name: format!("Handle<{}>", T::type_name()),
            type_id: TypeId::of::<Handle<T>>(),
            kind: luminara_core::TypeKind::Value,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn luminara_core::Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn luminara_core::Reflect> {
        None
    }

    fn set_field(
        &mut self,
        name: &str,
        _value: Box<dyn luminara_core::Reflect>,
    ) -> Result<(), luminara_core::ReflectError> {
        Err(luminara_core::ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn luminara_core::Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.id).unwrap_or(serde_json::Value::Null)
    }

    fn deserialize_json(
        &mut self,
        value: &serde_json::Value,
    ) -> Result<(), luminara_core::ReflectError> {
        let id: AssetId = serde_json::from_value(value.clone())
            .map_err(|e| luminara_core::ReflectError::DeserializationError(e.to_string()))?;
        *self = Handle::new(id, 0);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<T: Asset> Handle<T> {
    /// Check if this handle is ready (asset is loaded)
    pub fn is_loaded(&self, server: &crate::AssetServer) -> bool {
        use crate::LoadState;
        matches!(server.load_state(self.id), LoadState::Loaded)
    }

    /// Get the load state of this handle
    pub fn load_state(&self, server: &crate::AssetServer) -> crate::LoadState {
        server.load_state(self.id)
    }

    /// Resolve this handle to the actual asset (non-blocking)
    /// Returns None if the asset is not yet loaded
    pub fn resolve(&self, server: &crate::AssetServer) -> Option<Arc<T>> {
        server.get(self)
    }
}
