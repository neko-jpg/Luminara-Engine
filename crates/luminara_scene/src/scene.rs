use crate::registry::TypeRegistry;
use luminara_core::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Component schema for AI understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub type_name: String,
    pub description: String,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    pub description: String,
}

/// Global component schema registry
static COMPONENT_SCHEMA_REGISTRY: once_cell::sync::Lazy<
    Arc<RwLock<HashMap<String, ComponentSchema>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Register a component schema for AI introspection
pub fn register_component_schema(schema: ComponentSchema) {
    let mut registry = COMPONENT_SCHEMA_REGISTRY.write().unwrap();
    registry.insert(schema.type_name.clone(), schema);
}

/// Get a component schema by type name
pub fn get_component_schema(type_name: &str) -> Option<ComponentSchema> {
    let registry = COMPONENT_SCHEMA_REGISTRY.read().unwrap();
    registry.get(type_name).cloned()
}

/// Get all registered component schemas
pub fn get_all_component_schemas() -> Vec<ComponentSchema> {
    let registry = COMPONENT_SCHEMA_REGISTRY.read().unwrap();
    registry.values().cloned().collect()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityData {
    pub name: String,
    pub id: Option<u64>,
    pub parent: Option<u64>,
    pub components: HashMap<String, serde_json::Value>,
    pub children: Vec<EntityData>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    pub meta: SceneMeta,
    pub entities: Vec<EntityData>,
}

#[derive(Debug)]
pub enum SceneError {
    Io(std::io::Error),
    Parse(String),
    MissingComponent(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::Io(e) => write!(f, "IO error: {}", e),
            SceneError::Parse(e) => write!(f, "Parse error: {}", e),
            SceneError::MissingComponent(e) => write!(f, "Missing component: {}", e),
        }
    }
}

impl std::error::Error for SceneError {}

impl From<std::io::Error> for SceneError {
    fn from(err: std::io::Error) -> Self {
        SceneError::Io(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl luminara_core::Component for Name {
    fn type_name() -> &'static str {
        "Name"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag(pub std::collections::HashSet<String>);

impl Default for Tag {
    fn default() -> Self {
        Self::new()
    }
}

impl Tag {
    pub fn new() -> Self {
        Self(std::collections::HashSet::new())
    }

    pub fn insert(&mut self, tag: impl Into<String>) {
        self.0.insert(tag.into());
    }

    pub fn contains(&self, tag: &str) -> bool {
        self.0.contains(tag)
    }
}

impl luminara_core::Component for Tag {
    fn type_name() -> &'static str {
        "Tag"
    }
}

impl Scene {
    pub fn load_from_file(path: &Path) -> Result<Self, SceneError> {
        crate::serialization::load_from_file(path)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), SceneError> {
        crate::serialization::save_to_file(self, path)
    }

    pub fn from_ron(source: &str) -> Result<Self, SceneError> {
        crate::serialization::from_ron(source)
    }

    pub fn to_ron(&self) -> Result<String, SceneError> {
        crate::serialization::to_ron(self)
    }

    pub fn from_json(source: &str) -> Result<Self, SceneError> {
        crate::serialization::from_json(source)
    }

    pub fn to_json(&self) -> Result<String, SceneError> {
        crate::serialization::to_json(self)
    }

    pub fn spawn_into(&self, world: &mut World) -> Vec<Entity> {
        // Attempt to extract TypeRegistry from world to use it for deserialization
        let registry = world.remove_resource::<TypeRegistry>();

        let mut id_map = HashMap::new();
        let mut spawned_entities = Vec::new();

        for entity_data in &self.entities {
            self.spawn_entity_recursive(
                world,
                registry.as_ref(),
                entity_data,
                None,
                &mut id_map,
                &mut spawned_entities,
            );
        }

        // Put the registry back
        if let Some(reg) = registry {
            world.insert_resource(reg);
        }

        spawned_entities
    }

    pub(crate) fn spawn_entity_recursive(
        &self,
        world: &mut World,
        registry: Option<&TypeRegistry>,
        data: &EntityData,
        parent: Option<Entity>,
        id_map: &mut HashMap<u64, Entity>,
        spawned_entities: &mut Vec<Entity>,
    ) -> Entity {
        let entity = world.spawn();
        spawned_entities.push(entity);

        if let Some(id) = data.id {
            id_map.insert(id, entity);
        }

        // Always add Name component
        world.add_component(entity, Name::new(&data.name));

        // Always add Tag component if tags exist
        if !data.tags.is_empty() {
            let mut tag = Tag::new();
            for tag_str in &data.tags {
                tag.insert(tag_str);
            }
            world.add_component(entity, tag);
        }

        // Handle hierarchy
        if let Some(p) = parent {
            crate::hierarchy::set_parent(world, entity, p);
        }

        // Process other components
        for (type_name, value) in &data.components {
            // Special handling for Transform (optimization/legacy)
            if type_name == "Transform" {
                if let Ok(transform) =
                    serde_json::from_value::<luminara_math::Transform>(value.clone())
                {
                    world.add_component(entity, transform);
                }
                continue;
            }

            // Try to use registry
            if let Some(reg) = registry {
                if let Err(e) = reg.deserialize_and_add(world, entity, type_name, value.clone()) {
                    // We can't use log here easily as it might not be initialized or accessible?
                    // But typically log macros work anywhere.
                    // For now, silently ignore or print to stderr if critical?
                    // Better to rely on the fact that if it's missing, it's just not added.
                    // Ideally, we'd have a warning.
                    eprintln!("Scene warning: {}", e);
                }
            }
        }

        for child_data in &data.children {
            self.spawn_entity_recursive(
                world,
                registry,
                child_data,
                Some(entity),
                id_map,
                spawned_entities,
            );
        }

        entity
    }
}

pub fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    for entity in world.entities() {
        if let Some(n) = world.get_component::<Name>(entity) {
            if n.0 == name {
                return Some(entity);
            }
        }
    }
    None
}

pub fn find_entities_by_tag(world: &World, tag: &str) -> Vec<Entity> {
    let mut results = Vec::new();
    for entity in world.entities() {
        if let Some(t) = world.get_component::<Tag>(entity) {
            if t.contains(tag) {
                results.push(entity);
            }
        }
    }
    results
}

/// Initialize default component schemas
pub fn init_default_component_schemas() {
    // Register Name component schema
    register_component_schema(ComponentSchema {
        type_name: "Name".to_string(),
        description: "Entity name for identification".to_string(),
        fields: vec![FieldSchema {
            name: "name".to_string(),
            type_name: "String".to_string(),
            description: "The name of the entity".to_string(),
        }],
    });

    // Register Tag component schema
    register_component_schema(ComponentSchema {
        type_name: "Tag".to_string(),
        description: "Tags for entity categorization".to_string(),
        fields: vec![FieldSchema {
            name: "tags".to_string(),
            type_name: "HashSet<String>".to_string(),
            description: "Set of tags associated with the entity".to_string(),
        }],
    });

    // Register Transform component schema
    register_component_schema(ComponentSchema {
        type_name: "Transform".to_string(),
        description: "Local transform (position, rotation, scale)".to_string(),
        fields: vec![
            FieldSchema {
                name: "translation".to_string(),
                type_name: "Vec3".to_string(),
                description: "Position in 3D space".to_string(),
            },
            FieldSchema {
                name: "rotation".to_string(),
                type_name: "Quat".to_string(),
                description: "Rotation as a quaternion".to_string(),
            },
            FieldSchema {
                name: "scale".to_string(),
                type_name: "Vec3".to_string(),
                description: "Scale factors for each axis".to_string(),
            },
        ],
    });

    // Register Parent component schema
    register_component_schema(ComponentSchema {
        type_name: "Parent".to_string(),
        description: "Parent entity reference for hierarchy".to_string(),
        fields: vec![FieldSchema {
            name: "parent".to_string(),
            type_name: "Entity".to_string(),
            description: "The parent entity ID".to_string(),
        }],
    });

    // Register Children component schema
    register_component_schema(ComponentSchema {
        type_name: "Children".to_string(),
        description: "Child entities for hierarchy".to_string(),
        fields: vec![FieldSchema {
            name: "children".to_string(),
            type_name: "Vec<Entity>".to_string(),
            description: "List of child entity IDs".to_string(),
        }],
    });

    // Register GlobalTransform component schema
    register_component_schema(ComponentSchema {
        type_name: "GlobalTransform".to_string(),
        description: "World-space transform (computed from hierarchy)".to_string(),
        fields: vec![FieldSchema {
            name: "transform".to_string(),
            type_name: "Transform".to_string(),
            description: "The computed world-space transform".to_string(),
        }],
    });
}
