//! Backend & AI Box Module
//!
//! The Backend & AI Box provides an integrated development environment for:
//! - Script editing (Rust/WASM scripts)
//! - Database query editing and execution
//! - AI-assisted code generation and assistance
//!
//! Layout:
//! - Toolbar with Run button, Mode selector, and Status bar
//! - Main area: 2-column layout (Script Editor left, AI Assistant right)
//! - Bottom tab panel: Console, DB Explorer, Build Output, Diagnostics

pub mod backend_ai_box;
pub mod toolbar;
pub mod script_editor;
pub mod ai_assistant;
pub mod bottom_tabs;
pub mod file_tree;

pub use backend_ai_box::BackendAIBox;
pub use toolbar::{Toolbar, EditorMode};
pub use script_editor::{ScriptEditor, ScriptTab};
pub use ai_assistant::{AIAssistant, ChatMessage, MessageRole};
pub use bottom_tabs::{BottomTabPanel, BottomTab};
pub use file_tree::{FileTree, FileTreeItem, FileTreeItemType};
