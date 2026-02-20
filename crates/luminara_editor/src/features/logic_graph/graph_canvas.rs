//! Graph Canvas Component
//!
//! The main canvas area for the Logic Graph editor.
//! Displays a grid background and renders nodes with drag support.

use crate::ui::theme::Theme;
use super::{GraphNode, NodeId, LogicGraph};
use gpui::{
    div, px, IntoElement, ParentElement, RenderOnce, Styled,
    WindowContext, Point, Pixels, canvas, prelude::FluentBuilder,
};
use std::sync::Arc;

/// The graph canvas component
#[derive(Debug, Clone)]
pub struct GraphCanvas {
    /// The logic graph data
    pub graph: LogicGraph,
    /// Theme for styling
    theme: Arc<Theme>,
    /// Canvas offset for panning
    offset: Point<Pixels>,
    /// Zoom level (1.0 = 100%)
    zoom: f32,
    /// Currently dragged node
    dragged_node: Option<NodeId>,
    /// Last mouse position for drag calculations
    last_mouse_pos: Option<Point<Pixels>>,
}

impl GraphCanvas {
    /// Grid size in pixels
    pub const GRID_SIZE: f32 = 30.0;
    /// Grid dot size
    pub const GRID_DOT_SIZE: f32 = 2.0;

    /// Create a new graph canvas
    pub fn new(graph: LogicGraph, theme: Arc<Theme>) -> Self {
        Self {
            graph,
            theme,
            offset: Point::new(px(0.0), px(0.0)),
            zoom: 1.0,
            dragged_node: None,
            last_mouse_pos: None,
        }
    }

    /// Get the canvas offset
    pub fn offset(&self) -> Point<Pixels> {
        self.offset
    }

    /// Get the zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Set canvas offset
    pub fn set_offset(&mut self, x: Pixels, y: Pixels) {
        self.offset = Point::new(x, y);
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 5.0);
    }

    /// Pan the canvas by delta
    pub fn pan_by(&mut self, dx: Pixels, dy: Pixels) {
        self.offset = Point::new(self.offset.x + dx, self.offset.y + dy);
    }

    /// Start dragging a node
    pub fn start_drag_node(&mut self, node_id: NodeId, pos: Point<Pixels>) {
        self.dragged_node = Some(node_id);
        self.last_mouse_pos = Some(pos);
        
        // Mark node as selected
        self.graph.select_node(Some(node_id));
        
        // Mark node as dragging
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            node.set_dragging(true);
        }
    }

    /// Handle mouse move during drag
    pub fn handle_drag_move(&mut self, pos: Point<Pixels>) {
        if let (Some(node_id), Some(last_pos)) = (self.dragged_node, self.last_mouse_pos) {
            let dx = (pos.x - last_pos.x).0 / self.zoom;
            let dy = (pos.y - last_pos.y).0 / self.zoom;
            
            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                node.move_by(dx, dy);
            }
            
            self.last_mouse_pos = Some(pos);
        }
    }

    /// End drag operation
    pub fn end_drag(&mut self) {
        if let Some(node_id) = self.dragged_node {
            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                node.set_dragging(false);
            }
        }
        self.dragged_node = None;
        self.last_mouse_pos = None;
    }

    /// Find node at position (in canvas coordinates)
    pub fn node_at_position(&self, x: f32, y: f32) -> Option<NodeId> {
        // Convert screen to canvas coordinates
        let canvas_x = (x - self.offset.x.0) / self.zoom;
        let canvas_y = (y - self.offset.y.0) / self.zoom;
        
        self.graph.nodes.values()
            .find(|node| node.contains_point(canvas_x, canvas_y))
            .map(|node| node.id)
    }

    /// Render the canvas background with grid
    fn render_background(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let grid_color = theme.colors.grid_dot;
        let bg_color = theme.colors.canvas_background;
        let grid_size = Self::GRID_SIZE;
        let dot_size = Self::GRID_DOT_SIZE;

        div()
            .absolute()
            .inset_0()
            .bg(bg_color)
            .child(
                canvas(
                    |_, _| {},
                    move |bounds, _, cx| {
                        cx.paint_quad(gpui::fill(bounds, bg_color));
                        
                        // Draw grid dots
                        let width = bounds.size.width.0;
                        let height = bounds.size.height.0;
                        
                        let start_x = (0.0 / grid_size).floor() as i32;
                        let end_x = (width / grid_size).ceil() as i32;
                        let start_y = (0.0 / grid_size).floor() as i32;
                        let end_y = (height / grid_size).ceil() as i32;
                        
                        for x in start_x..=end_x {
                            for y in start_y..=end_y {
                                let px_coord = x as f32 * grid_size + dot_size;
                                let py_coord = y as f32 * grid_size + dot_size;
                                
                                if px_coord >= 0.0 && px_coord <= width && py_coord >= 0.0 && py_coord <= height {
                                    cx.paint_quad(gpui::fill(
                                        gpui::Bounds::new(
                                            Point::new(px(px_coord), px(py_coord)),
                                            gpui::size(px(dot_size * 2.0), px(dot_size * 2.0)),
                                        ),
                                        grid_color,
                                    ));
                                }
                            }
                        }
                    },
                )
                .w_full()
                .h_full()
            )
    }

    /// Render a single node
    fn render_node(&self, node: &GraphNode) -> impl IntoElement {
        let theme = self.theme.clone();
        let (x, y, width, height) = node.bounds();
        let is_selected = node.selected;
        let is_dragging = node.dragging;
        let node_bg = theme.colors.node_background;
        let border_color = if is_selected {
            theme.colors.node_selected
        } else {
            theme.colors.node_border
        };
        
        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(width))
            .min_h(px(height))
            .bg(node_bg)
            .rounded(px(8.0))
            .border_1()
            .border_color(border_color)
            .when(is_selected, |this| {
                this.shadow_lg()
            })
            .when(is_dragging, |this| {
                this.opacity(0.9)
            })
            // Node title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(8.0))
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(node.icon_color)
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.text)
                            .child(node.title.clone())
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded_full()
                            .bg(theme.colors.surface_active)
                            .text_color(theme.colors.text_secondary)
                            .child(node.kind.display_name())
                    )
            )
            // Input ports
            .children(
                node.inputs.iter().map(|port| {
                    let port_y = port.position * height;
                    let port_color = port.kind.color();
                    
                    div()
                        .absolute()
                        .left(px(-6.0))
                        .top(px(port_y - 6.0))
                        .w(px(12.0))
                        .h(px(12.0))
                        .rounded_full()
                        .bg(port_color)
                        .border_2()
                        .border_color(theme.colors.node_background)
                })
            )
            // Output ports
            .children(
                node.outputs.iter().map(|port| {
                    let port_y = port.position * height;
                    let port_color = port.kind.color();
                    
                    div()
                        .absolute()
                        .right(px(-6.0))
                        .top(px(port_y - 6.0))
                        .w(px(12.0))
                        .h(px(12.0))
                        .rounded_full()
                        .bg(port_color)
                        .border_2()
                        .border_color(theme.colors.node_background)
                })
            )
    }

    /// Render the minimap
    fn render_minimap(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let node_count = self.graph.nodes.len();

        div()
            .absolute()
            .bottom(px(20.0))
            .left(px(20.0))
            .px(px(8.0))
            .py(px(6.0))
            .bg(theme.colors.canvas_background) // Semi-transparent background
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.colors.node_border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(11.0))
                    .text_color(theme.colors.text_secondary)
                    .child("🗺")
                    .child("Mini Map")
                    .child(
                        div()
                            .text_color(theme.colors.accent)
                            .child(format!("[▪] {} nodes", node_count))
                    )
            )
    }

    /// Render edge labels
    fn render_edge_labels(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .absolute()
            .children(
                self.graph.connections.iter().filter_map(|conn| {
                    // Find source and target nodes
                    let from_node = self.graph.nodes.get(&conn.from.node_id)?;
                    let to_node = self.graph.nodes.get(&conn.to.node_id)?;
                    
                    // Calculate midpoint for label
                    let (fx, fy, fw, fh) = from_node.bounds();
                    let (tx, ty, _tw, _th) = to_node.bounds();
                    
                    let mid_x = (fx + fw + tx) / 2.0;
                    let mid_y = (fy + fh / 2.0 + ty + 50.0) / 2.0;
                    
                    conn.label.as_ref().map(|label| {
                        div()
                            .absolute()
                            .left(px(mid_x - 30.0))
                            .top(px(mid_y - 10.0))
                            .px(px(4.0))
                            .py(px(2.0))
                            .bg(theme.colors.surface)
                            .rounded(px(4.0))
                            .text_size(px(10.0))
                            .text_color(theme.colors.text_secondary)
                            .child(label.clone())
                    })
                })
            )
    }
}

impl RenderOnce for GraphCanvas {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let _theme = self.theme.clone();
        let nodes: Vec<_> = self.graph.nodes.values().cloned().collect();
        let minimap = self.render_minimap();

        div()
            .relative()
            .w_full()
            .h_full()
            .overflow_hidden()
            // Background with grid
            .child(self.render_background())
            // Nodes container (with transform for pan/zoom)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .children(
                        nodes.into_iter().map(|node| self.render_node(&node))
                    )
            )
            // Edge labels
            .child(self.render_edge_labels())
            // Minimap
            .child(minimap)
    }
}

/// Graph canvas panel with header
#[derive(Debug, Clone)]
pub struct GraphCanvasPanel {
    canvas: GraphCanvas,
    theme: Arc<Theme>,
}

impl GraphCanvasPanel {
    /// Create a new panel
    pub fn new(canvas: GraphCanvas, theme: Arc<Theme>) -> Self {
        Self { canvas, theme }
    }
}

/// Render the canvas content directly
fn render_canvas_content(canvas: &GraphCanvas) -> impl IntoElement {
    let theme = canvas.theme.clone();
    let graph = canvas.graph.clone();
    let nodes: Vec<_> = graph.nodes.values().cloned().collect();
    let minimap = render_minimap(&graph, &theme);

    div()
        .relative()
        .w_full()
        .h_full()
        .bg(theme.colors.background)
        // Background with grid
        .child(render_grid_background(&theme))
        // Nodes container
        .child(
            div()
                .absolute()
                .inset_0()
                .children(
                    nodes.iter().map(|node| render_node(node, &theme))
                )
        )
        // Edge labels
        .child(render_edge_labels(&graph, &theme))
        // Minimap
        .child(minimap)
}

/// Render grid background
fn render_grid_background(theme: &Arc<Theme>) -> impl IntoElement {
    let theme = theme.clone();
    let grid_color = theme.colors.grid_dot;
    let bg_color = theme.colors.canvas_background;
    let grid_size = GraphCanvas::GRID_SIZE;
    let dot_size = GraphCanvas::GRID_DOT_SIZE;

    div()
        .absolute()
        .inset_0()
        .bg(bg_color)
        .child(
            canvas(
                |_, _| {},
                move |bounds, _, cx| {
                    cx.paint_quad(gpui::fill(bounds, bg_color));
                    
                    // Draw grid dots
                    let width = bounds.size.width.0;
                    let height = bounds.size.height.0;
                    
                    let start_x = (0.0 / grid_size).floor() as i32;
                    let end_x = (width / grid_size).ceil() as i32;
                    let start_y = (0.0 / grid_size).floor() as i32;
                    let end_y = (height / grid_size).ceil() as i32;
                    
                    for x in start_x..=end_x {
                        for y in start_y..=end_y {
                            let px_coord = x as f32 * grid_size + dot_size;
                            let py_coord = y as f32 * grid_size + dot_size;
                            
                            if px_coord >= 0.0 && px_coord <= width && py_coord >= 0.0 && py_coord <= height {
                                cx.paint_quad(gpui::fill(
                                    gpui::Bounds::new(
                                        Point::new(px(px_coord), px(py_coord)),
                                        gpui::size(px(dot_size * 2.0), px(dot_size * 2.0)),
                                    ),
                                    grid_color,
                                ));
                            }
                        }
                    }
                },
            )
            .w_full()
            .h_full()
        )
}

/// Render a single node
fn render_node(node: &GraphNode, theme: &Arc<Theme>) -> impl IntoElement {
    let theme = theme.clone();
    let (x, y, width, height) = node.bounds();
    let is_selected = node.selected;
    let is_dragging = node.dragging;
    let node_bg = theme.colors.node_background;
    let border_color = if is_selected {
        theme.colors.node_selected
    } else {
        theme.colors.node_border
    };
    
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(width))
        .min_h(px(height))
        .bg(node_bg)
        .rounded(px(8.0))
        .border_1()
        .border_color(border_color)
        .when(is_selected, |this| {
            this.shadow_lg()
        })
        .when(is_dragging, |this| {
            this.opacity(0.9)
        })
        // Node title
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(10.0))
                .py(px(8.0))
                .child(
                    div()
                        .w(px(8.0))
                        .h(px(8.0))
                        .rounded_full()
                        .bg(node.icon_color)
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text)
                        .child(node.title.clone())
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded_full()
                        .bg(theme.colors.surface_active)
                        .text_color(theme.colors.text_secondary)
                        .child(node.kind.display_name())
                )
        )
        // Input ports
        .children(
            node.inputs.iter().map(|port| {
                let port_y = port.position * height;
                let port_color = port.kind.color();
                
                div()
                    .absolute()
                    .left(px(-6.0))
                    .top(px(port_y - 6.0))
                    .w(px(12.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(port_color)
                    .border_2()
                    .border_color(theme.colors.node_background)
            })
        )
        // Output ports
        .children(
            node.outputs.iter().map(|port| {
                let port_y = port.position * height;
                let port_color = port.kind.color();
                
                div()
                    .absolute()
                    .right(px(-6.0))
                    .top(px(port_y - 6.0))
                    .w(px(12.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(port_color)
                    .border_2()
                    .border_color(theme.colors.node_background)
            })
        )
}

/// Render edge labels
fn render_edge_labels(graph: &LogicGraph, theme: &Arc<Theme>) -> impl IntoElement {
    let theme = theme.clone();

    div()
        .absolute()
        .children(
            graph.connections.iter().filter_map(|conn| {
                // Find source and target nodes
                let from_node = graph.nodes.get(&conn.from.node_id)?;
                let to_node = graph.nodes.get(&conn.to.node_id)?;
                
                // Calculate midpoint for label
                let (fx, fy, fw, fh) = from_node.bounds();
                let (tx, ty, _tw, _th) = to_node.bounds();
                
                let mid_x = (fx + fw + tx) / 2.0;
                let mid_y = (fy + fh / 2.0 + ty + 50.0) / 2.0;
                
                conn.label.as_ref().map(|label| {
                    div()
                        .absolute()
                        .left(px(mid_x - 30.0))
                        .top(px(mid_y - 10.0))
                        .px(px(4.0))
                        .py(px(2.0))
                        .bg(theme.colors.surface)
                        .rounded(px(4.0))
                        .text_size(px(10.0))
                        .text_color(theme.colors.text_secondary)
                        .child(label.clone())
                })
            })
        )
}

/// Render the minimap
fn render_minimap(graph: &LogicGraph, theme: &Arc<Theme>) -> impl IntoElement {
    let node_count = graph.nodes.len();

    div()
        .absolute()
        .bottom(px(20.0))
        .left(px(20.0))
        .px(px(8.0))
        .py(px(6.0))
        .bg(theme.colors.canvas_background) // Semi-transparent background
        .rounded(px(6.0))
        .border_1()
        .border_color(theme.colors.node_border)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .text_size(px(11.0))
                .text_color(theme.colors.text_secondary)
                .child("🗺")
                .child("Mini Map")
                .child(
                    div()
                        .text_color(theme.colors.accent)
                        .child(format!("[▪] {} nodes", node_count))
                )
        )
}

impl GraphCanvasPanel {
    /// Render the panel with header
    pub fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let theme = self.theme.clone();
        let node_count = self.canvas.graph.node_count();
        let edge_count = self.canvas.graph.connection_count();
        let graph_name = self.canvas.graph.name.clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_tl(px(4.0))
            .rounded_tr(px(4.0))
            // Panel header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.text_secondary)
                            .child("◈")
                            .child(format!("Logic Graph [{}]", graph_name))
                    )
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(4.0))
                            .bg(theme.colors.toolbar_active)
                            .rounded(px(4.0))
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .child(format!("✓ Validate ({} nodes, {} edges)", node_count, edge_count))
                    )
            )
            // Canvas content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(render_canvas_content(&self.canvas))
            )
    }
}

impl RenderOnce for GraphCanvasPanel {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let theme = self.theme.clone();
        let node_count = self.canvas.graph.node_count();
        let edge_count = self.canvas.graph.connection_count();
        let graph_name = self.canvas.graph.name.clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_tl(px(4.0))
            .rounded_tr(px(4.0))
            // Panel header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.text_secondary)
                            .child("◈")
                            .child(format!("Logic Graph [{}]", graph_name))
                    )
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(4.0))
                            .bg(theme.colors.toolbar_active)
                            .rounded(px(4.0))
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .child(format!("✓ Validate ({} nodes, {} edges)", node_count, edge_count))
                    )
            )
            // Canvas content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(render_canvas_content(&self.canvas))
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::LogicGraph;

    #[test]
    fn test_canvas_creation() {
        let graph = LogicGraph::sample_main_quest();
        let canvas = GraphCanvas::new(graph, Arc::new(Theme::default_dark()));
        
        assert_eq!(canvas.zoom(), 1.0);
        assert!(canvas.dragged_node.is_none());
    }

    #[test]
    fn test_zoom_limits() {
        let graph = LogicGraph::sample_main_quest();
        let mut canvas = GraphCanvas::new(graph, Arc::new(Theme::default_dark()));
        
        canvas.set_zoom(10.0);
        assert_eq!(canvas.zoom(), 5.0); // Clamped to max
        
        canvas.set_zoom(0.01);
        assert_eq!(canvas.zoom(), 0.1); // Clamped to min
    }

    #[test]
    fn test_pan() {
        let graph = LogicGraph::sample_main_quest();
        let mut canvas = GraphCanvas::new(graph, Arc::new(Theme::default_dark()));
        
        canvas.pan_by(px(100.0), px(50.0));
        assert_eq!(canvas.offset().x, px(100.0));
        assert_eq!(canvas.offset().y, px(50.0));
    }

    #[test]
    fn test_node_selection() {
        let graph = LogicGraph::sample_main_quest();
        let mut canvas = GraphCanvas::new(graph, Arc::new(Theme::default_dark()));
        
        // Select a node
        let node_id = NodeId::new(1);
        canvas.start_drag_node(node_id, Point::new(px(0.0), px(0.0)));
        
        assert_eq!(canvas.dragged_node, Some(node_id));
        assert_eq!(canvas.graph.selected_node, Some(node_id));
    }
}
