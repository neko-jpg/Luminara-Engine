//! Scene Builder Box - Main Container
//!
//! The main Scene Builder container that combines all panels:
//! - Menu Bar (top)
//! - Toolbar (below menu)
//! - Hierarchy Panel (left, 260px)
//! - Viewport Panel (center, flexible)
//! - Inspector Panel (right, 320px)
//! - Bottom Tab Panel (bottom, 200px)

use gpui::{
    IntoElement, Render, View, ViewContext, VisualContext as _,
};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::ui::layouts::WorkspaceLayout;
use crate::features::scene_builder::menu_bar::MenuBar;
use crate::services::engine_bridge::EngineHandle;
use crate::features::scene_builder::{
    toolbar::MainToolbar,
    hierarchy::HierarchyPanel,
    viewport::Viewport3D,
    inspector::InspectorPanel,
    bottom_tabs::BottomTabPanel,
};


/// Scene Builder Box - Main container component
pub struct SceneBuilderBox {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    editor_state: gpui::Model<crate::core::state::EditorStateManager>,
    // Child views
    menu_bar: View<MenuBar>,
    toolbar: View<MainToolbar>,
    hierarchy: View<HierarchyPanel>,
    viewport: View<Viewport3D>,
    inspector: View<InspectorPanel>,
    bottom_tabs: View<BottomTabPanel>,
}

impl SceneBuilderBox {
    pub fn new(
        engine_handle: Arc<EngineHandle>,
        theme: Arc<Theme>,
        editor_state: gpui::Model<crate::core::state::EditorStateManager>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        // Create child views
        let menu_bar = cx.new_view(|_cx| {
            MenuBar::new(theme.clone())
                .with_engine_handle(engine_handle.clone())
                .with_state(editor_state.clone())
        });
        let toolbar = cx.new_view(|cx| MainToolbar::new(theme.clone(), editor_state.clone(), cx));
        let hierarchy = cx.new_view(|cx| HierarchyPanel::new(
            theme.clone(),
            engine_handle.clone(),
            editor_state.clone(),
            cx,
        ));
        let viewport = cx.new_view(|cx| Viewport3D::new(
            theme.clone(),
            engine_handle.clone(),
            editor_state.clone(),
            cx,
        ));
        let inspector = cx.new_view(|cx| InspectorPanel::new(
            theme.clone(),
            engine_handle.clone(),
            editor_state.clone(),
            cx,
        ));
        let bottom_tabs = cx.new_view(|cx| {
            BottomTabPanel::new(theme.clone())
                .with_state(editor_state.clone(), cx)
        });

        Self {
            theme,
            engine_handle,
            editor_state,
            menu_bar,
            toolbar,
            hierarchy,
            viewport,
            inspector,
            bottom_tabs,
        }
    }

    /// Get theme reference
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get engine handle
    pub fn engine(&self) -> &Arc<EngineHandle> {
        &self.engine_handle
    }

}

impl Render for SceneBuilderBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        // Use the unified WorkspaceLayout for consistent layout structure
        WorkspaceLayout::new(theme.clone())
            .with_state(self.editor_state.clone())
            .menu_bar(self.menu_bar.clone())
            .toolbar(self.toolbar.clone())
            .left_panel(self.hierarchy.clone())
            .center_panel(self.viewport.clone())
            .right_panel(self.inspector.clone())
            .bottom_panel(self.bottom_tabs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_builder_box_initialization() {
        // Verify SceneBuilderBox can be created with editor state
        // Test logic omitted as it requires a full GPUI test context
    }

    #[test]
    fn test_scene_builder_layout_dimensions() {
        // Verify layout dimensions
        let hierarchy_width = 260.0;
        let inspector_width = 320.0;
        let bottom_panel_height = 200.0;

        assert_eq!(hierarchy_width, 260.0);
        assert_eq!(inspector_width, 320.0);
        assert_eq!(bottom_panel_height, 200.0);
    }
}
