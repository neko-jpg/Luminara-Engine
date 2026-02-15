use serde::{Deserialize, Serialize};
use surrealdb::sql::{Thing, Datetime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub id: Option<Thing>,
    pub group_id: String,
    pub sequence: i32,
    pub command_type: String,
    pub description: String,
    pub forward_data: serde_json::Value,
    pub backward_data: serde_json::Value,
    pub timestamp: Datetime,
}
