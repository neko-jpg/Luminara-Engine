use serde::{Deserialize, Serialize};
use surrealdb::sql::{Thing, Datetime};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneRecord {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub settings: SceneSettings,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneSettings {
    pub ambient_color: Option<[f32; 4]>,
    pub gravity: Option<[f32; 3]>,
    pub physics_timestep: Option<f32>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: Option<Thing>,
    pub name: String,
    pub scene: Thing, // record<scene>
    pub enabled: bool,
    pub tags: Vec<String>,
    pub layer: i32,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRecord {
    pub id: Option<Thing>,
    pub entity: Thing, // record<entity>
    pub component_type: String,
    pub data: serde_json::Value,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub scene_id: String,
    pub scene: SceneRecord,
    pub entities: Vec<EntityRecord>,
    pub components: Vec<ComponentRecord>,
    pub hierarchy: Vec<HierarchyRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyRelation {
    pub parent_db_id: String,
    pub child_db_id: String,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityWithComponents {
    #[serde(flatten)]
    pub entity: EntityRecord,
    pub components: Vec<ComponentRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub has_component: Option<String>,
    pub enabled_only: bool,
}
