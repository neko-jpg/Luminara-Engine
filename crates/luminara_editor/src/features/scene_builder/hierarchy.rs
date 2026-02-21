//! Hierarchy Panel Component (Stub for Bevy Integration MVP)

use gpui::{div, IntoElement, Render, ViewContext, ParentElement};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use crate::core::state::EditorStateManager;

pub struct HierarchyItem;

pub struct HierarchyPanel;

impl HierarchyPanel {
    pub fn new(
        _theme: Arc<Theme>,
        _engine_handle: Arc<EngineHandle>,
        _state: gpui::Model<EditorStateManager>,
        _cx: &mut ViewContext<Self>,
    ) -> Self {
        Self
    }
}

impl Render for HierarchyPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child("Hierarchy Panel (Under Construction)")
    }
}
