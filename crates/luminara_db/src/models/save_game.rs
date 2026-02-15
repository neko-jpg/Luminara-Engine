use serde::{Deserialize, Serialize};
use surrealdb::sql::{Thing, Datetime};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub scene_name: String,
    pub play_time: Duration,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub screenshot: Option<Vec<u8>>,
    pub custom_data: Option<serde_json::Value>,
}
