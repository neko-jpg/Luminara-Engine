use luminara_math::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pending transform data (position, rotation, scale)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for PendingTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

/// Identifies the type of workspace currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceType {
    SceneBuilder = 0,
    LogicGraph = 1,
}

/// In-memory representation of the Editor Session.
/// This aligns with the `EditorSessionRecord` in `luminara_db`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSession {
    pub name: String,
    pub active_workspace: WorkspaceType,
    pub global_search_visible: bool,
    pub layout_config: serde_json::Value,
    pub selected_entities: Vec<String>,
    pub active_tool: String,
    pub active_bottom_tab: String,
    pub editor_mode: String,
    pub entity_counter: u32,
    #[serde(skip)]
    pub pending_transforms: HashMap<String, PendingTransform>,
    #[serde(skip)]
    pub entity_mesh_types: HashMap<String, String>,
}

impl Default for EditorSession {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            active_workspace: WorkspaceType::SceneBuilder,
            global_search_visible: false,
            layout_config: serde_json::json!({}),
            selected_entities: Vec::new(),
            active_tool: "Move".to_string(),
            active_bottom_tab: "Console".to_string(),
            editor_mode: "Edit".to_string(),
            entity_counter: 1,
            pending_transforms: HashMap::new(),
            entity_mesh_types: HashMap::new(),
        }
    }
}
