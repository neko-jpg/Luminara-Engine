//! Script Editor (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Data)]
pub struct ScriptTab {
    pub id: u64,
    pub name: String,
    pub content: String,
    pub language: String,
}

#[derive(Lens, Clone, Data)]
pub struct ScriptEditorState {
    pub theme: Arc<Theme>,
    pub tabs: Vec<ScriptTab>,
    pub active_tab: Option<u64>,
}

impl ScriptEditorState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            tabs: Vec::new(),
            active_tab: None,
        }
    }
}
