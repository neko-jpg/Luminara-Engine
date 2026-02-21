//! Menu Bar Component (Stub for Bevy Integration MVP)

use gpui::{div, IntoElement, Render, ViewContext, ParentElement};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use crate::core::state::EditorStateManager;

pub struct MenuItem;

pub struct MenuBar;

impl MenuBar {
    pub fn new(_theme: Arc<Theme>) -> Self {
        Self
    }

    pub fn with_engine_handle(self, _engine_handle: Arc<EngineHandle>) -> Self {
        self
    }

    pub fn with_state(self, _state: gpui::Model<EditorStateManager>) -> Self {
        self
    }
}

impl Render for MenuBar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child("Menu Bar (Stub)")
    }
}
