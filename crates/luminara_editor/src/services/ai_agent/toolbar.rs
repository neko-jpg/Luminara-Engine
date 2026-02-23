//! AI Agent Toolbar (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum EditorMode {
    Script,
    Query,
    AI,
}

#[derive(Lens, Clone, Data)]
pub struct ToolbarState {
    pub theme: Arc<Theme>,
    pub mode: EditorMode,
    pub is_running: bool,
}

impl ToolbarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            mode: EditorMode::Script,
            is_running: false,
        }
    }
}
