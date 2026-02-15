//! Runtime reflection system for editor integration.
//!
//! This module provides runtime type information and dynamic field access
//! for components and other engine types. The reflection system enables
//! the editor to inspect and modify engine state without compile-time knowledge
//! of specific types.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

/// Trait for types that support runtime reflection.
///
/// Types implementing Reflect can provide metadata about their structure,
/// access fields dynamically, and serialize/deserialize their values.
pub trait Reflect: Send + Sync + 'static {
    /// Get type information for this reflected type.
    fn type_info(&self) -> &TypeInfo;

    /// Get a reference to a field by name.
    ///
    /// Returns None if the field doesn't exist or this type doesn't support field access.
    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    /// Get a mutable reference to a field by name.
    ///
    /// Returns None if the field doesn't exist or this type doesn't support field access.
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    /// Set a field value by name.
    ///
    /// Returns an error if the field doesn't exist, the type doesn't match,
    /// or this type doesn't support field modification.
    fn set_field(&mut self, name: &str, value: Box<dyn Reflect>) -> Result<(), ReflectError>;

    /// Clone this value as a reflected boxed value.
    fn clone_value(&self) -> Box<dyn Reflect>;

    /// Serialize to JSON value.
    fn serialize_json(&self) -> serde_json::Value;

    /// Deserialize from JSON value.
    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError>;

    /// Downcast to concrete type reference.
    fn as_any(&self) -> &dyn Any;

    /// Downcast to concrete type mutable reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get a reference to a nested field by path (e.g., "position.x").
    ///
    /// The path is split by '.' and each segment is used to traverse the field hierarchy.
    /// Returns None if any field in the path doesn't exist.
    fn field_path(&self, path: &str) -> Option<&dyn Reflect>
    where
        Self: Sized,
    {
        let mut current: &dyn Reflect = self;
        for segment in path.split('.') {
            current = current.field(segment)?;
        }
        Some(current)
    }

    /// Get a mutable reference to a nested field by path (e.g., "position.x").
    ///
    /// The path is split by '.' and each segment is used to traverse the field hierarchy.
    /// Returns None if any field in the path doesn't exist.
    fn field_path_mut(&mut self, path: &str) -> Option<&mut dyn Reflect>
    where
        Self: Sized,
    {
        let segments: Vec<&str> = path.split('.').collect();
        if segments.is_empty() {
            return None;
        }

        // Handle single segment case
        if segments.len() == 1 {
            return self.field_mut(segments[0]);
        }

        // Navigate to the parent of the final field
        let mut current: &mut dyn Reflect = self;
        for segment in &segments[..segments.len() - 1] {
            current = current.field_mut(segment)?;
        }

        // Get the final field
        current.field_mut(segments[segments.len() - 1])
    }

    /// Set a nested field value by path (e.g., "position.x").
    ///
    /// The path is split by '.' and each segment is used to traverse the field hierarchy.
    /// Returns an error if any field in the path doesn't exist or if the type doesn't match.
    fn set_field_path(&mut self, path: &str, value: Box<dyn Reflect>) -> Result<(), ReflectError>
    where
        Self: Sized,
    {
        let segments: Vec<&str> = path.split('.').collect();
        if segments.is_empty() {
            return Err(ReflectError::FieldNotFound(path.to_string()));
        }

        // Handle single segment case
        if segments.len() == 1 {
            return self.set_field(segments[0], value);
        }

        // Navigate to the parent of the final field
        let mut current: &mut dyn Reflect = self;
        for segment in &segments[..segments.len() - 1] {
            current = current.field_mut(segment)
                .ok_or_else(|| ReflectError::FieldNotFound(segment.to_string()))?;
        }

        // Set the final field
        let final_segment = segments[segments.len() - 1];
        current.set_field(final_segment, value)
    }

    /// Get enum variant information (if this is an enum).
    ///
    /// Returns None if this type is not an enum.
    fn enum_variant(&self) -> Option<EnumVariant> {
        None
    }

    /// Get the number of elements in a collection (if this is a collection).
    ///
    /// Returns None if this type is not a collection.
    fn collection_len(&self) -> Option<usize> {
        None
    }

    /// Get an element from a collection by index.
    ///
    /// Returns None if this type is not a collection or the index is out of bounds.
    fn collection_get(&self, index: usize) -> Option<&dyn Reflect> {
        None
    }

    /// Get a mutable element from a collection by index.
    ///
    /// Returns None if this type is not a collection or the index is out of bounds.
    fn collection_get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        None
    }

    /// Iterate over collection elements.
    ///
    /// Returns an empty iterator if this type is not a collection.
    fn collection_iter(&self) -> Box<dyn Iterator<Item = &dyn Reflect> + '_> {
        Box::new(std::iter::empty())
    }

    /// Get a value from a map by key (for HashMap-like collections).
    ///
    /// Returns None if this type is not a map or the key doesn't exist.
    fn map_get(&self, key: &str) -> Option<&dyn Reflect> {
        None
    }

    /// Get a mutable value from a map by key (for HashMap-like collections).
    ///
    /// Returns None if this type is not a map or the key doesn't exist.
    fn map_get_mut(&mut self, key: &str) -> Option<&mut dyn Reflect> {
        None
    }

    /// Get all keys from a map (for HashMap-like collections).
    ///
    /// Returns an empty vector if this type is not a map.
    fn map_keys(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Metadata about a reflected type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// The fully qualified type name (e.g., "luminara_scene::Transform")
    pub type_name: String,
    /// The TypeId for runtime type checking
    pub type_id: TypeId,
    /// The kind of type (struct, enum, tuple, etc.)
    pub kind: TypeKind,
    /// Field information (for structs and tuples)
    pub fields: Vec<FieldInfo>,
}

/// The kind of reflected type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// A struct with named fields
    Struct,
    /// An enum with variants
    Enum,
    /// A tuple with indexed fields
    Tuple,
    /// A list/array/vector
    List,
    /// A map/dictionary
    Map,
    /// A primitive value (no fields)
    Value,
}

/// Metadata about a field in a reflected type.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// The field name (or index as string for tuples)
    pub name: String,
    /// The type name of the field
    pub type_name: String,
    /// The TypeId of the field
    pub type_id: TypeId,
    /// Optional description for documentation
    pub description: Option<String>,
    /// Optional default value as JSON
    pub default_value: Option<serde_json::Value>,
}

/// Information about an enum variant.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The name of the variant
    pub variant_name: String,
    /// The discriminant value (if available)
    pub discriminant: Option<isize>,
    /// Fields in the variant (for tuple or struct variants)
    pub fields: Vec<FieldInfo>,
}

/// Errors that can occur during reflection operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ReflectError {
    #[error("Field '{0}' not found")]
    FieldNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Cannot set field '{0}' on this type")]
    CannotSetField(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Type '{0}' is not registered")]
    TypeNotRegistered(String),
}

/// Registry for reflected types.
///
/// The registry maintains metadata about all types that support reflection
/// and provides factory functions to construct instances.
pub struct ReflectRegistry {
    types: HashMap<TypeId, TypeInfo>,
    type_names: HashMap<String, TypeId>,
    constructors: HashMap<TypeId, Box<dyn Fn() -> Box<dyn Reflect> + Send + Sync>>,
}

impl ReflectRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            type_names: HashMap::new(),
            constructors: HashMap::new(),
        }
    }

    /// Register a type for reflection.
    ///
    /// The type must implement Reflect and Default.
    pub fn register<T: Reflect + Default>(&mut self) {
        let type_id = TypeId::of::<T>();
        let instance = T::default();
        let type_info = instance.type_info().clone();

        self.types.insert(type_id, type_info.clone());
        self.type_names
            .insert(type_info.type_name.clone(), type_id);
        self.constructors
            .insert(type_id, Box::new(|| Box::new(T::default())));
    }

    /// Get type information by TypeId.
    pub fn get_type_info(&self, type_id: TypeId) -> Option<&TypeInfo> {
        self.types.get(&type_id)
    }

    /// Get type information by type name.
    pub fn get_type_info_by_name(&self, type_name: &str) -> Option<&TypeInfo> {
        self.type_names
            .get(type_name)
            .and_then(|id| self.types.get(id))
    }

    /// Construct a new instance by type name.
    pub fn construct(&self, type_name: &str) -> Option<Box<dyn Reflect>> {
        self.type_names
            .get(type_name)
            .and_then(|id| self.constructors.get(id))
            .map(|ctor| ctor())
    }

    /// Construct a new instance by TypeId.
    pub fn construct_by_id(&self, type_id: TypeId) -> Option<Box<dyn Reflect>> {
        self.constructors.get(&type_id).map(|ctor| ctor())
    }

    /// Get all registered type names.
    pub fn type_names(&self) -> impl Iterator<Item = &String> {
        self.type_names.keys()
    }

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl Default for ReflectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ReflectRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReflectRegistry")
            .field("type_count", &self.types.len())
            .field("type_names", &self.type_names.keys())
            .finish()
    }
}

// Implement Reflect for common primitive types

macro_rules! impl_reflect_primitive {
    ($ty:ty, $type_name:expr) => {
        impl Reflect for $ty {
            fn type_info(&self) -> &TypeInfo {
                use std::sync::OnceLock;
                static INFO: OnceLock<TypeInfo> = OnceLock::new();
                INFO.get_or_init(|| TypeInfo {
                    type_name: $type_name.to_string(),
                    type_id: TypeId::of::<$ty>(),
                    kind: TypeKind::Value,
                    fields: Vec::new(),
                })
            }

            fn field(&self, _name: &str) -> Option<&dyn Reflect> {
                None
            }

            fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
                None
            }

            fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
                Err(ReflectError::FieldNotFound(name.to_string()))
            }

            fn clone_value(&self) -> Box<dyn Reflect> {
                Box::new(*self)
            }

            fn serialize_json(&self) -> serde_json::Value {
                serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
            }

            fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
                *self = serde_json::from_value(value.clone())
                    .map_err(|e| ReflectError::DeserializationError(e.to_string()))?;
                Ok(())
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    };
}

impl_reflect_primitive!(f32, "f32");
impl_reflect_primitive!(f64, "f64");
impl_reflect_primitive!(i8, "i8");
impl_reflect_primitive!(i16, "i16");
impl_reflect_primitive!(i32, "i32");
impl_reflect_primitive!(i64, "i64");
impl_reflect_primitive!(u8, "u8");
impl_reflect_primitive!(u16, "u16");
impl_reflect_primitive!(u32, "u32");
impl_reflect_primitive!(u64, "u64");
impl_reflect_primitive!(bool, "bool");

impl Reflect for String {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: "String".to_string(),
            type_id: TypeId::of::<String>(),
            kind: TypeKind::Value,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        Err(ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::Value::String(self.clone())
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if let serde_json::Value::String(s) = value {
            *self = s.clone();
            Ok(())
        } else {
            Err(ReflectError::DeserializationError(
                "Expected string".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Implement Reflect for Option<T>
impl<T: Reflect + Clone> Reflect for Option<T> {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: format!("Option<{}>", std::any::type_name::<T>()),
            type_id: TypeId::of::<Option<T>>(),
            kind: TypeKind::Value,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        Err(ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        match self {
            Some(v) => v.serialize_json(),
            None => serde_json::Value::Null,
        }
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if value.is_null() {
            *self = None;
            Ok(())
        } else {
            // For now, we can't deserialize without a default value
            Err(ReflectError::DeserializationError(
                "Cannot deserialize Option without existing value".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Implement Reflect for Vec<T>
impl<T: Reflect + Clone> Reflect for Vec<T> {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: format!("Vec<{}>", std::any::type_name::<T>()),
            type_id: TypeId::of::<Vec<T>>(),
            kind: TypeKind::List,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        Err(ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.serialize_json()).collect())
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if let serde_json::Value::Array(arr) = value {
            self.clear();
            for item in arr {
                // For now, we can't deserialize without knowing how to construct T
                return Err(ReflectError::DeserializationError(
                    "Cannot deserialize Vec without type information".to_string(),
                ));
            }
            Ok(())
        } else {
            Err(ReflectError::DeserializationError(
                "Expected array".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn collection_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn collection_get(&self, index: usize) -> Option<&dyn Reflect> {
        self.get(index).map(|v| v as &dyn Reflect)
    }

    fn collection_get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.get_mut(index).map(|v| v as &mut dyn Reflect)
    }

    fn collection_iter(&self) -> Box<dyn Iterator<Item = &dyn Reflect> + '_> {
        Box::new(self.iter().map(|v| v as &dyn Reflect))
    }
}

// Implement Reflect for HashMap<String, T>
impl<T: Reflect + Clone> Reflect for HashMap<String, T> {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: format!("HashMap<String, {}>", std::any::type_name::<T>()),
            type_id: TypeId::of::<HashMap<String, T>>(),
            kind: TypeKind::Map,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        Err(ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        let map: serde_json::Map<String, serde_json::Value> = self
            .iter()
            .map(|(k, v)| (k.clone(), v.serialize_json()))
            .collect();
        serde_json::Value::Object(map)
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if let serde_json::Value::Object(_map) = value {
            self.clear();
            // For now, we can't deserialize without knowing how to construct T
            Err(ReflectError::DeserializationError(
                "Cannot deserialize HashMap without type information".to_string(),
            ))
        } else {
            Err(ReflectError::DeserializationError(
                "Expected object".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn collection_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn map_get(&self, key: &str) -> Option<&dyn Reflect> {
        self.get(key).map(|v| v as &dyn Reflect)
    }

    fn map_get_mut(&mut self, key: &str) -> Option<&mut dyn Reflect> {
        self.get_mut(key).map(|v| v as &mut dyn Reflect)
    }

    fn map_keys(&self) -> Vec<String> {
        self.keys().cloned().collect()
    }
}

// Implement Reflect for glam types (Vec3, Quat)
// These are implemented here to avoid circular dependencies
impl Reflect for glam::Vec3 {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: "Vec3".to_string(),
            type_id: TypeId::of::<glam::Vec3>(),
            kind: TypeKind::Struct,
            fields: vec![
                FieldInfo {
                    name: "x".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
                FieldInfo {
                    name: "y".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
                FieldInfo {
                    name: "z".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
            ],
        })
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "x" => Some(&self.x as &dyn Reflect),
            "y" => Some(&self.y as &dyn Reflect),
            "z" => Some(&self.z as &dyn Reflect),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "x" => Some(&mut self.x as &mut dyn Reflect),
            "y" => Some(&mut self.y as &mut dyn Reflect),
            "z" => Some(&mut self.z as &mut dyn Reflect),
            _ => None,
        }
    }

    fn set_field(&mut self, name: &str, value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        match name {
            "x" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.x = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "y" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.y = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "z" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.z = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            _ => Err(ReflectError::FieldNotFound(name.to_string())),
        }
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(*self)
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
        })
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if let serde_json::Value::Object(map) = value {
            if let Some(x) = map.get("x").and_then(|v| v.as_f64()) {
                self.x = x as f32;
            }
            if let Some(y) = map.get("y").and_then(|v| v.as_f64()) {
                self.y = y as f32;
            }
            if let Some(z) = map.get("z").and_then(|v| v.as_f64()) {
                self.z = z as f32;
            }
            Ok(())
        } else {
            Err(ReflectError::DeserializationError(
                "Expected object".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Reflect for glam::Quat {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: "Quat".to_string(),
            type_id: TypeId::of::<glam::Quat>(),
            kind: TypeKind::Struct,
            fields: vec![
                FieldInfo {
                    name: "x".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
                FieldInfo {
                    name: "y".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
                FieldInfo {
                    name: "z".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
                FieldInfo {
                    name: "w".to_string(),
                    type_name: "f32".to_string(),
                    type_id: TypeId::of::<f32>(),
                    description: None,
                    default_value: None,
                },
            ],
        })
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "x" => Some(&self.x as &dyn Reflect),
            "y" => Some(&self.y as &dyn Reflect),
            "z" => Some(&self.z as &dyn Reflect),
            "w" => Some(&self.w as &dyn Reflect),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "x" => Some(&mut self.x as &mut dyn Reflect),
            "y" => Some(&mut self.y as &mut dyn Reflect),
            "z" => Some(&mut self.z as &mut dyn Reflect),
            "w" => Some(&mut self.w as &mut dyn Reflect),
            _ => None,
        }
    }

    fn set_field(&mut self, name: &str, value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        match name {
            "x" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.x = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "y" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.y = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "z" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.z = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "w" => {
                if let Some(v) = value.as_any().downcast_ref::<f32>() {
                    self.w = *v;
                    Ok(())
                } else {
                    Err(ReflectError::TypeMismatch {
                        expected: "f32".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            _ => Err(ReflectError::FieldNotFound(name.to_string())),
        }
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(*self)
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
            "w": self.w,
        })
    }

    fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), ReflectError> {
        if let serde_json::Value::Object(map) = value {
            if let Some(x) = map.get("x").and_then(|v| v.as_f64()) {
                self.x = x as f32;
            }
            if let Some(y) = map.get("y").and_then(|v| v.as_f64()) {
                self.y = y as f32;
            }
            if let Some(z) = map.get("z").and_then(|v| v.as_f64()) {
                self.z = z as f32;
            }
            if let Some(w) = map.get("w").and_then(|v| v.as_f64()) {
                self.w = w as f32;
            }
            Ok(())
        } else {
            Err(ReflectError::DeserializationError(
                "Expected object".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_reflection() {
        let mut value = 42i32;
        let type_info = value.type_info();

        assert_eq!(type_info.type_name, "i32");
        assert_eq!(type_info.kind, TypeKind::Value);
        assert!(type_info.fields.is_empty());

        // Test serialization
        let json = value.serialize_json();
        assert_eq!(json, serde_json::json!(42));

        // Test deserialization
        value.deserialize_json(&serde_json::json!(100)).unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_string_reflection() {
        let mut value = String::from("hello");
        let type_info = value.type_info();

        assert_eq!(type_info.type_name, "String");
        assert_eq!(type_info.kind, TypeKind::Value);

        // Test serialization
        let json = value.serialize_json();
        assert_eq!(json, serde_json::json!("hello"));

        // Test deserialization
        value
            .deserialize_json(&serde_json::json!("world"))
            .unwrap();
        assert_eq!(value, "world");
    }

    #[test]
    fn test_registry() {
        let mut registry = ReflectRegistry::new();

        // Register i32
        registry.register::<i32>();

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        // Get type info
        let type_info = registry.get_type_info_by_name("i32").unwrap();
        assert_eq!(type_info.type_name, "i32");

        // Construct instance
        let instance = registry.construct("i32").unwrap();
        assert_eq!(instance.type_info().type_name, "i32");
    }

    #[test]
    fn test_vec_collection_reflection() {
        let mut vec = vec![1i32, 2, 3, 4, 5];

        // Test type info
        let type_info = vec.type_info();
        assert_eq!(type_info.kind, TypeKind::List);

        // Test collection length
        assert_eq!(vec.collection_len(), Some(5));

        // Test collection get
        let elem = vec.collection_get(2).unwrap();
        let value = elem.as_any().downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 3);

        // Test collection get_mut
        let elem_mut = vec.collection_get_mut(2).unwrap();
        let value_mut = elem_mut.as_any_mut().downcast_mut::<i32>().unwrap();
        *value_mut = 10;
        assert_eq!(vec[2], 10);

        // Test collection iteration
        let mut sum = 0;
        for elem in vec.collection_iter() {
            if let Some(v) = elem.as_any().downcast_ref::<i32>() {
                sum += v;
            }
        }
        assert_eq!(sum, 1 + 2 + 10 + 4 + 5);

        // Test out of bounds
        assert!(vec.collection_get(10).is_none());
    }

    #[test]
    fn test_hashmap_reflection() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1i32);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);

        // Test type info
        let type_info = map.type_info();
        assert_eq!(type_info.kind, TypeKind::Map);

        // Test collection length
        assert_eq!(map.collection_len(), Some(3));

        // Test map get
        let value = map.map_get("b").unwrap();
        let int_value = value.as_any().downcast_ref::<i32>().unwrap();
        assert_eq!(*int_value, 2);

        // Test map get_mut
        let value_mut = map.map_get_mut("b").unwrap();
        let int_value_mut = value_mut.as_any_mut().downcast_mut::<i32>().unwrap();
        *int_value_mut = 20;
        assert_eq!(map["b"], 20);

        // Test map keys
        let mut keys = map.map_keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);

        // Test missing key
        assert!(map.map_get("d").is_none());
    }

    #[test]
    fn test_nested_vec_reflection() {
        let mut vec_of_vecs = vec![vec![1i32, 2], vec![3, 4, 5]];

        // Test outer collection
        assert_eq!(vec_of_vecs.collection_len(), Some(2));

        // Test inner collection
        let inner = vec_of_vecs.collection_get(1).unwrap();
        assert_eq!(inner.collection_len(), Some(3));

        // Test nested element access
        let inner_elem = inner.collection_get(2).unwrap();
        let value = inner_elem.as_any().downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 5);

        // Test nested mutation
        let inner_mut = vec_of_vecs.collection_get_mut(0).unwrap();
        let elem_mut = inner_mut.collection_get_mut(1).unwrap();
        let value_mut = elem_mut.as_any_mut().downcast_mut::<i32>().unwrap();
        *value_mut = 100;
        assert_eq!(vec_of_vecs[0][1], 100);
    }

    #[test]
    fn test_vec_serialization() {
        let vec = vec![1i32, 2, 3];
        let json = vec.serialize_json();

        assert_eq!(json, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_hashmap_serialization() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), 10i32);
        map.insert("y".to_string(), 20);

        let json = map.serialize_json();

        // Check that it's an object with the right keys
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["x"], 10);
        assert_eq!(obj["y"], 20);
    }

    // Test enum reflection with a simple enum
    #[derive(Debug, Clone, PartialEq)]
    enum TestEnum {
        Unit,
        Tuple(i32, String),
        Struct { x: i32, y: String },
    }

    impl Reflect for TestEnum {
        fn type_info(&self) -> &TypeInfo {
            use std::sync::OnceLock;
            static INFO: OnceLock<TypeInfo> = OnceLock::new();
            INFO.get_or_init(|| TypeInfo {
                type_name: "TestEnum".to_string(),
                type_id: TypeId::of::<TestEnum>(),
                kind: TypeKind::Enum,
                fields: Vec::new(),
            })
        }

        fn field(&self, _name: &str) -> Option<&dyn Reflect> {
            None
        }

        fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
            None
        }

        fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
            Err(ReflectError::FieldNotFound(name.to_string()))
        }

        fn clone_value(&self) -> Box<dyn Reflect> {
            Box::new(self.clone())
        }

        fn serialize_json(&self) -> serde_json::Value {
            match self {
                TestEnum::Unit => serde_json::json!({"variant": "Unit"}),
                TestEnum::Tuple(a, b) => serde_json::json!({
                    "variant": "Tuple",
                    "fields": [a, b]
                }),
                TestEnum::Struct { x, y } => serde_json::json!({
                    "variant": "Struct",
                    "fields": {"x": x, "y": y}
                }),
            }
        }

        fn deserialize_json(&mut self, _value: &serde_json::Value) -> Result<(), ReflectError> {
            Err(ReflectError::DeserializationError(
                "TestEnum deserialization not implemented".to_string(),
            ))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn enum_variant(&self) -> Option<EnumVariant> {
            match self {
                TestEnum::Unit => Some(EnumVariant {
                    variant_name: "Unit".to_string(),
                    discriminant: Some(0),
                    fields: Vec::new(),
                }),
                TestEnum::Tuple(_, _) => Some(EnumVariant {
                    variant_name: "Tuple".to_string(),
                    discriminant: Some(1),
                    fields: vec![
                        FieldInfo {
                            name: "0".to_string(),
                            type_name: "i32".to_string(),
                            type_id: TypeId::of::<i32>(),
                            description: None,
                            default_value: None,
                        },
                        FieldInfo {
                            name: "1".to_string(),
                            type_name: "String".to_string(),
                            type_id: TypeId::of::<String>(),
                            description: None,
                            default_value: None,
                        },
                    ],
                }),
                TestEnum::Struct { .. } => Some(EnumVariant {
                    variant_name: "Struct".to_string(),
                    discriminant: Some(2),
                    fields: vec![
                        FieldInfo {
                            name: "x".to_string(),
                            type_name: "i32".to_string(),
                            type_id: TypeId::of::<i32>(),
                            description: None,
                            default_value: None,
                        },
                        FieldInfo {
                            name: "y".to_string(),
                            type_name: "String".to_string(),
                            type_id: TypeId::of::<String>(),
                            description: None,
                            default_value: None,
                        },
                    ],
                }),
            }
        }
    }

    #[test]
    fn test_enum_variant_reflection() {
        // Test unit variant
        let unit = TestEnum::Unit;
        let variant = unit.enum_variant().unwrap();
        assert_eq!(variant.variant_name, "Unit");
        assert_eq!(variant.discriminant, Some(0));
        assert_eq!(variant.fields.len(), 0);

        // Test tuple variant
        let tuple = TestEnum::Tuple(42, "hello".to_string());
        let variant = tuple.enum_variant().unwrap();
        assert_eq!(variant.variant_name, "Tuple");
        assert_eq!(variant.discriminant, Some(1));
        assert_eq!(variant.fields.len(), 2);
        assert_eq!(variant.fields[0].name, "0");
        assert_eq!(variant.fields[0].type_name, "i32");
        assert_eq!(variant.fields[1].name, "1");
        assert_eq!(variant.fields[1].type_name, "String");

        // Test struct variant
        let struct_variant = TestEnum::Struct {
            x: 100,
            y: "world".to_string(),
        };
        let variant = struct_variant.enum_variant().unwrap();
        assert_eq!(variant.variant_name, "Struct");
        assert_eq!(variant.discriminant, Some(2));
        assert_eq!(variant.fields.len(), 2);
        assert_eq!(variant.fields[0].name, "x");
        assert_eq!(variant.fields[1].name, "y");
    }

    #[test]
    fn test_enum_type_info() {
        let enum_val = TestEnum::Unit;
        let type_info = enum_val.type_info();

        assert_eq!(type_info.type_name, "TestEnum");
        assert_eq!(type_info.kind, TypeKind::Enum);
    }
}
