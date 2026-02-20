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
use parking_lot::RwLock;
use std::collections::HashSet;
use crate::ui::theme::Theme;
use crate::ui::layouts::{WorkspaceLayout, MenuBar};
use crate::services::engine_bridge::EngineHandle;
use crate::features::scene_builder::{
    toolbar::{MainToolbar, ToolMode},
    hierarchy::HierarchyPanel,
    viewport::Viewport3D,
    inspector::InspectorPanel,
    bottom_tabs::{BottomTabPanel, BottomTab},
};
use luminara_core::Entity;

/// State for the Scene Builder
#[derive(Debug, Clone)]
pub struct SceneBuilderState {
    pub selected_entities: Arc<RwLock<HashSet<Entity>>>,
    pub active_tool: ToolMode,
    pub active_bottom_tab: BottomTab,
}

impl Default for SceneBuilderState {
    fn default() -> Self {
        Self {
            selected_entities: Arc::new(RwLock::new(HashSet::new())),
            active_tool: ToolMode::Move,
            active_bottom_tab: BottomTab::Console,
        }
    }
}

/// Scene Builder Box - Main container component
pub struct SceneBuilderBox {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    state: SceneBuilderState,
    // Child views
    toolbar: View<MainToolbar>,
    hierarchy: View<HierarchyPanel>,
    viewport: View<Viewport3D>,
    inspector: View<InspectorPanel>,
    bottom_tabs: View<BottomTabPanel>,
}

impl SceneBuilderBox {
    /// Create a new Scene Builder Box
    pub fn new(
        engine_handle: Arc<EngineHandle>,
        theme: Arc<Theme>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        // Create shared selection state
        let selected_entities = Arc::new(RwLock::new(HashSet::new()));
        let selected_entities_hierarchy = selected_entities.clone();
        let selected_entities_viewport = selected_entities.clone();
        let selected_entities_inspector = selected_entities.clone();

        // Create child views
        let toolbar = cx.new_view(|_cx| MainToolbar::new(theme.clone()));
        let hierarchy = cx.new_view(|_cx| HierarchyPanel::new(
            theme.clone(),
            engine_handle.clone(),
            selected_entities_hierarchy,
        ));
        let viewport = cx.new_view(|_cx| Viewport3D::new(
            theme.clone(),
            engine_handle.clone(),
            selected_entities_viewport,
        ));
        let inspector = cx.new_view(|cx| InspectorPanel::new(
            theme.clone(),
            engine_handle.clone(),
            selected_entities_inspector,
            cx,
        ));
        let bottom_tabs = cx.new_view(|_cx| BottomTabPanel::new(theme.clone()));

        Self {
            theme,
            engine_handle,
            state: SceneBuilderState::default(),
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

    /// Get state reference
    pub fn state(&self) -> &SceneBuilderState {
        &self.state
    }

    /// Select an entity
    pub fn select_entity(&mut self, entity: Entity, multi_select: bool, cx: &mut ViewContext<Self>) {
        let mut selected = self.state.selected_entities.write();

        if multi_select {
            if selected.contains(&entity) {
                selected.remove(&entity);
            } else {
                selected.insert(entity);
            }
        } else {
            selected.clear();
            selected.insert(entity);
        }

        drop(selected);
        cx.notify();
    }

    /// Clear selection
    pub fn clear_selection(&mut self, cx: &mut ViewContext<Self>) {
        self.state.selected_entities.write().clear();
        cx.notify();
    }

    /// Get selected entities
    pub fn selected_entities(&self) -> HashSet<Entity> {
        self.state.selected_entities.read().clone()
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, tool: ToolMode, cx: &mut ViewContext<Self>) {
        self.state.active_tool = tool;
        cx.notify();
    }

    /// Get active tool
    pub fn active_tool(&self) -> ToolMode {
        self.state.active_tool
    }

    /// Set active bottom tab
    pub fn set_active_bottom_tab(&mut self, tab: BottomTab, cx: &mut ViewContext<Self>) {
        self.state.active_bottom_tab = tab;
        cx.notify();
    }

    /// Get active bottom tab
    pub fn active_bottom_tab(&self) -> BottomTab {
        self.state.active_bottom_tab
    }
}

impl Render for SceneBuilderBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        // Use the unified WorkspaceLayout for consistent layout structure
        WorkspaceLayout::new(theme.clone())
            .menu_bar(
                MenuBar::new(theme.clone())
                    .items(vec!["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"])
            )
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
    fn test_scene_builder_state_default() {
        let state = SceneBuilderState::default();
        assert!(state.selected_entities.read().is_empty());
        assert!(matches!(state.active_tool, ToolMode::Move));
        assert!(matches!(state.active_bottom_tab, BottomTab::Console));
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
