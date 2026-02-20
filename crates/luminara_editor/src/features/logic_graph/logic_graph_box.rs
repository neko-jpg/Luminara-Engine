//! Logic Graph Box (Main View)
//!
//! The main Logic Graph editor view that combines all components:
//! - Toolbar with simulation controls
//! - Graph canvas (left panel)
//! - Node inspector (right panel)
//! - Bottom tab panel

use crate::ui::theme::Theme;
use crate::ui::layouts::{WorkspaceLayout, MenuBar};
use super::{
    LogicGraph, GraphCanvas, GraphCanvasPanel, NodeInspector,
    BottomTabPanel, LogicGraphToolbar, StatusBarInfo,
};
use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext, InteractiveElement,
    WindowContext, Point,
};
use std::sync::Arc;
use super::NodeId;

/// The main Logic Graph Box component
pub struct LogicGraphBox {
    /// The logic graph being edited
    graph: LogicGraph,
    /// Theme for styling
    theme: Arc<Theme>,
    #[allow(dead_code)]
    toolbar: LogicGraphToolbar,
    /// Bottom tab panel
    bottom_tabs: BottomTabPanel,
    /// Canvas offset for panning
    #[allow(dead_code)]
    canvas_offset: Point<gpui::Pixels>,
    /// Canvas zoom level
    #[allow(dead_code)]
    zoom: f32,
    /// Currently dragged node
    #[allow(dead_code)]
    dragged_node: Option<NodeId>,
    /// Last mouse position
    #[allow(dead_code)]
    last_mouse_pos: Option<Point<gpui::Pixels>>,
    /// Current tool mode
    #[allow(dead_code)]
    tool_mode: ToolMode,
}

/// Tool mode for the canvas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Pan,
}

/// Panel sizes
const INSPECTOR_WIDTH: f32 = 320.0;
const TOOLBAR_HEIGHT: f32 = 44.0;
const MENUBAR_HEIGHT: f32 = 32.0;

impl LogicGraphBox {
    /// Create a new Logic Graph Box with sample data
    pub fn new(theme: Arc<Theme>) -> Self {
        let graph = LogicGraph::sample_main_quest();
        let toolbar = LogicGraphToolbar::new(theme.clone());
        let bottom_tabs = BottomTabPanel::new(theme.clone());

        Self {
            graph,
            theme: theme.clone(),
            toolbar,
            bottom_tabs,
            canvas_offset: Point::new(px(0.0), px(0.0)),
            zoom: 1.0,
            dragged_node: None,
            last_mouse_pos: None,
            tool_mode: ToolMode::Select,
        }
    }

    /// Create with a specific graph
    pub fn with_graph(mut self, graph: LogicGraph) -> Self {
        self.graph = graph;
        self
    }

    /// Get the graph
    pub fn graph(&self) -> &LogicGraph {
        &self.graph
    }

    /// Get mutable graph
    pub fn graph_mut(&mut self) -> &mut LogicGraph {
        &mut self.graph
    }

    /// Handle node selection
    pub fn select_node(&mut self, node_id: Option<NodeId>) {
        // Deselect previous
        if let Some(prev_id) = self.graph.selected_node {
            if let Some(node) = self.graph.nodes.get_mut(&prev_id) {
                node.set_selected(false);
            }
        }

        // Select new
        if let Some(id) = node_id {
            if let Some(node) = self.graph.nodes.get_mut(&id) {
                node.set_selected(true);
            }
        }

        self.graph.select_node(node_id);
    }

    /// Start dragging a node
    #[allow(dead_code)]
    fn start_drag(&mut self, node_id: NodeId, pos: Point<gpui::Pixels>) {
        self.dragged_node = Some(node_id);
        self.last_mouse_pos = Some(pos);
        self.select_node(Some(node_id));
    }

    /// Handle mouse move
    #[allow(dead_code)]
    fn handle_mouse_move(&mut self, pos: Point<gpui::Pixels>) {
        match self.tool_mode {
            ToolMode::Select => {
                if let (Some(node_id), Some(last_pos)) = (self.dragged_node, self.last_mouse_pos) {
                    let dx = (pos.x - last_pos.x).0 / self.zoom;
                    let dy = (pos.y - last_pos.y).0 / self.zoom;

                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.move_by(dx, dy);
                    }

                    self.last_mouse_pos = Some(pos);
                }
            }
            ToolMode::Pan => {
                if let Some(last_pos) = self.last_mouse_pos {
                    let dx = pos.x - last_pos.x;
                    let dy = pos.y - last_pos.y;
                    self.canvas_offset = Point::new(
                        self.canvas_offset.x + dx,
                        self.canvas_offset.y + dy,
                    );
                    self.last_mouse_pos = Some(pos);
                }
            }
        }
    }

    /// End drag
    #[allow(dead_code)]
    fn end_drag(&mut self) {
        self.dragged_node = None;
        self.last_mouse_pos = None;
    }

    /// Find node at screen position
    #[allow(dead_code)]
    fn node_at_screen_pos(&self, x: f32, y: f32) -> Option<NodeId> {
        let canvas_x = (x - self.canvas_offset.x.0) / self.zoom;
        let canvas_y = (y - self.canvas_offset.y.0) / self.zoom;

        self.graph.nodes.values()
            .find(|node| node.contains_point(canvas_x, canvas_y))
            .map(|node| node.id)
    }

    /// Render the menu bar (kept for potential future use)
    #[allow(dead_code)]
    fn render_menubar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let menu_items = vec!["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"];

        div()
            .flex()
            .items_center()
            .gap(px(24.0))
            .h(px(MENUBAR_HEIGHT))
            .px(px(16.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .text_size(px(13.0))
            .text_color(theme.colors.text)
            .children(
                menu_items.into_iter().map(move |item| {
                    let theme = theme.clone();
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .hover(|style| style.bg(theme.colors.surface_hover))
                        .child(item)
                })
            )
    }

    /// Render the toolbar with status bar
    fn render_toolbar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let status = StatusBarInfo::new(
            self.graph.node_count(),
            self.graph.connection_count(),
            self.graph.name.clone(),
        );

        div()
            .flex()
            .items_center()
            .h(px(TOOLBAR_HEIGHT))
            .px(px(12.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            // Simulation controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .bg(theme.colors.surface_active)
                    .rounded(px(6.0))
                    .px(px(4.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .bg(theme.colors.toolbar_active)
                            .text_color(theme.colors.text)
                            .hover(|this| this.bg(theme.colors.accent_hover))
                            .child("▶")
                            .child("Simulate")
                    )
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("⏹")
                    )
            )
            // Separator
            .child(
                div()
                    .w(px(1.0))
                    .h(px(24.0))
                    .mx(px(8.0))
                    .bg(theme.colors.border)
            )
            // Tool buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .bg(theme.colors.surface_active)
                    .rounded(px(6.0))
                    .px(px(4.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .bg(theme.colors.toolbar_active)
                            .text_color(theme.colors.text)
                            .child("↔")
                            .child("Select")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("⟲")
                            .child("Pan")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("🔍")
                            .child("Zoom")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("□")
                            .child("Auto Layout")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("≡")
                            .child("Align")
                    )
            )
            // Status bar (right aligned)
            .child(div().flex_1())
            .child(status.render(theme.clone()))
    }

    /// Render the graph canvas
    fn render_canvas(&self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = self.theme.clone();
        let graph = self.graph.clone();

        GraphCanvasPanel::new(
            GraphCanvas::new(graph, theme.clone()),
            theme,
        ).render(cx)
    }

    /// Render the inspector panel
    fn render_inspector(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let selected_node = self.graph.selected_node.map(|id| self.graph.nodes.get(&id).cloned()).flatten();

        NodeInspector::new(theme.clone()).render(selected_node)
    }

    /// Render the bottom tabs
    fn render_bottom_tabs(&self) -> impl IntoElement {
        self.bottom_tabs.clone().render()
    }
}

impl Render for LogicGraphBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        // Use the unified WorkspaceLayout for consistent layout structure
        WorkspaceLayout::new(theme.clone())
            .menu_bar(
                MenuBar::new(theme.clone())
                    .items(vec!["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"])
            )
            .toolbar(self.render_toolbar())
            .center_panel(self.render_canvas(cx))
            .right_panel(
                div()
                    .w(px(INSPECTOR_WIDTH))
                    .h_full()
                    .child(self.render_inspector())
            )
            .bottom_panel(self.render_bottom_tabs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_creation() {
        let box_component = LogicGraphBox::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(box_component.graph.node_count(), 5);
        assert_eq!(box_component.zoom, 1.0);
    }

    #[test]
    fn test_node_selection() {
        let mut box_component = LogicGraphBox::new(Arc::new(Theme::default_dark()));
        
        // Select a node
        let node_id = NodeId::new(1);
        box_component.select_node(Some(node_id));
        
        assert_eq!(box_component.graph.selected_node, Some(node_id));
    }

    #[test]
    fn test_tool_mode() {
        let mut box_component = LogicGraphBox::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(box_component.tool_mode, ToolMode::Select);
        
        box_component.tool_mode = ToolMode::Pan;
        assert_eq!(box_component.tool_mode, ToolMode::Pan);
    }
}
