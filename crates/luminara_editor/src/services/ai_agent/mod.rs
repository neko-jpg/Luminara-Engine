//! Backend & AI Box Module (Vizia version)

pub mod ai_assistant;
pub mod backend_ai_box;
pub mod bottom_tabs;
pub mod file_tree;
pub mod script_editor;
pub mod toolbar;

pub use ai_assistant::{AIAssistantState, ChatMessage, MessageRole};
pub use backend_ai_box::BackendAIState;
pub use bottom_tabs::{BottomTab, BottomTabPanelState};
pub use file_tree::{FileTreeItem, FileTreeItemType, FileTreeState};
pub use script_editor::{ScriptEditorState, ScriptTab};
pub use toolbar::{EditorMode, ToolbarState};
