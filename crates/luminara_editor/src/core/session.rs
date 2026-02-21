use serde::{Deserialize, Serialize};

/// Identifies the type of workspace currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceType {
    SceneBuilder = 0,
    LogicGraph = 1,
    // Future workspaces can be added here
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
        }
    }
}
