use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use luminara_core::{Entity, World};
use std::path::Path;

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
    fn type_name() -> &'static str { "Name" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag(pub std::collections::HashSet<String>);

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
    fn type_name() -> &'static str { "Tag" }
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
        let mut id_map = HashMap::new();
        let mut spawned_entities = Vec::new();

        for entity_data in &self.entities {
            self.spawn_entity_recursive(world, entity_data, None, &mut id_map, &mut spawned_entities);
        }

        spawned_entities
    }

    pub(crate) fn spawn_entity_recursive(
        &self,
        world: &mut World,
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

        world.add_component(entity, Name::new(&data.name));

        if !data.tags.is_empty() {
            let mut tag = Tag::new();
            for tag_str in &data.tags {
                tag.insert(tag_str);
            }
            world.add_component(entity, tag);
        }

        if let Some(p) = parent {
            crate::hierarchy::set_parent(world, entity, p);
        }

        if let Some(transform_val) = data.components.get("Transform") {
            if let Ok(transform) = serde_json::from_value::<luminara_math::Transform>(transform_val.clone()) {
                world.add_component(entity, transform);
            }
        }

        for child_data in &data.children {
            self.spawn_entity_recursive(world, child_data, Some(entity), id_map, spawned_entities);
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
