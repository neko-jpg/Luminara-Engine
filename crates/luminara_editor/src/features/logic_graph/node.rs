//! Graph Node Component
//!
//! Defines the visual nodes used in the Logic Graph editor.
//! Each node has ports for connections and can represent different
//! types of logic elements (State, Condition, Quest, Action, etc.)

use gpui::{Hsla, Point, Pixels, px, rgb};

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new node ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Unique identifier for a port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(pub u64);

impl PortId {
    /// Create a new port ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Type/kind of logic node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// State node (e.g., START, Village)
    State,
    /// Event trigger node
    Event,
    /// Condition/branch node
    Condition,
    /// Action node
    Action,
    /// Quest node
    Quest,
    /// Dialogue node
    Dialogue,
    /// Sub-graph reference
    SubGraph,
}

impl NodeKind {
    /// Get the display name for this node kind
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeKind::State => "State",
            NodeKind::Event => "Event",
            NodeKind::Condition => "Condition",
            NodeKind::Action => "Action",
            NodeKind::Quest => "Quest",
            NodeKind::Dialogue => "Dialogue",
            NodeKind::SubGraph => "Sub-graph",
        }
    }

    /// Get the default color for this node kind
    pub fn default_color(&self) -> Hsla {
        match self {
            NodeKind::State => rgb(0x5698ff).into(),
            NodeKind::Event => rgb(0xffaa00).into(),
            NodeKind::Condition => rgb(0xffaa00).into(),
            NodeKind::Action => rgb(0x8aff8a).into(),
            NodeKind::Quest => rgb(0xff8a8a).into(),
            NodeKind::Dialogue => rgb(0xcf8aff).into(),
            NodeKind::SubGraph => rgb(0x8ac4ff).into(),
        }
    }
}

/// Type of port connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    /// Standard flow connection
    Flow,
    /// True/Yes branch output
    True,
    /// False/No branch output
    False,
    /// Data input
    DataIn,
    /// Data output
    DataOut,
}

impl PortKind {
    /// Get the port color
    pub fn color(&self) -> Hsla {
        match self {
            PortKind::Flow => rgb(0x8a8aff).into(),
            PortKind::True => rgb(0x4caf50).into(),
            PortKind::False => rgb(0xf44336).into(),
            PortKind::DataIn => rgb(0xffaa8a).into(),
            PortKind::DataOut => rgb(0x8a8aff).into(),
        }
    }

    /// Check if this is an input port
    pub fn is_input(&self) -> bool {
        matches!(self, PortKind::Flow | PortKind::DataIn)
    }

    /// Check if this is an output port
    pub fn is_output(&self) -> bool {
        matches!(self, PortKind::Flow | PortKind::True | PortKind::False | PortKind::DataOut)
    }
}

/// A connection port on a node
#[derive(Debug, Clone)]
pub struct NodePort {
    /// Unique identifier
    pub id: PortId,
    /// Port type
    pub kind: PortKind,
    /// Port position relative to node (0-1, where 0 is top/left, 1 is bottom/right)
    pub position: f32,
}

impl NodePort {
    /// Create a new port
    pub fn new(id: PortId, kind: PortKind) -> Self {
        Self {
            id,
            kind,
            position: 0.5, // Default to center
        }
    }

    /// Set the position
    pub fn with_position(mut self, pos: f32) -> Self {
        self.position = pos.clamp(0.0, 1.0);
        self
    }
}

/// A visual node in the logic graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Unique identifier
    pub id: NodeId,
    /// Node type/kind
    pub kind: NodeKind,
    /// Display title
    pub title: String,
    /// Optional custom label
    pub label: Option<String>,
    /// Position on canvas (x, y)
    pub position: (f32, f32),
    /// Node size (width, height)
    pub size: (f32, f32),
    /// Icon color
    pub icon_color: Hsla,
    /// Input ports
    pub inputs: Vec<NodePort>,
    /// Output ports
    pub outputs: Vec<NodePort>,
    /// Whether node is selected
    pub selected: bool,
    /// Whether node is being dragged
    pub dragging: bool,
}

impl GraphNode {
    /// Minimum node width
    pub const MIN_WIDTH: f32 = 120.0;
    /// Minimum node height
    pub const MIN_HEIGHT: f32 = 50.0;
    /// Node padding
    pub const PADDING: f32 = 10.0;
    /// Port radius
    pub const PORT_RADIUS: f32 = 6.0;

    /// Create a new node
    pub fn new(id: NodeId, kind: NodeKind, title: impl Into<String>, position: (f32, f32)) -> Self {
        let title = title.into();
        let icon_color = kind.default_color();
        
        Self {
            id,
            kind,
            title,
            label: None,
            position,
            size: (Self::MIN_WIDTH, Self::MIN_HEIGHT),
            icon_color,
            inputs: Vec::new(),
            outputs: Vec::new(),
            selected: false,
            dragging: false,
        }
    }

    /// Set the custom label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the icon color
    pub fn with_icon_color(mut self, color: Hsla) -> Self {
        self.icon_color = color;
        self
    }

    /// Add an input port
    pub fn with_input_port(mut self, id: PortId, kind: PortKind) -> Self {
        let position = if self.inputs.is_empty() {
            0.5
        } else {
            0.3 + (self.inputs.len() as f32 * 0.2)
        };
        self.inputs.push(NodePort::new(id, kind).with_position(position));
        self.update_size();
        self
    }

    /// Add an output port
    pub fn with_output_port(mut self, id: PortId, kind: PortKind) -> Self {
        let position = if self.outputs.is_empty() {
            0.5
        } else {
            0.3 + (self.outputs.len() as f32 * 0.2)
        };
        self.outputs.push(NodePort::new(id, kind).with_position(position));
        self.update_size();
        self
    }

    /// Set selected state
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Set dragging state
    pub fn set_dragging(&mut self, dragging: bool) {
        self.dragging = dragging;
    }

    /// Update position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    /// Move by delta
    pub fn move_by(&mut self, dx: f32, dy: f32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }

    /// Get the display title
    pub fn display_title(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.title)
    }

    /// Get the full title with badge
    pub fn full_title(&self) -> String {
        format!("{} ({})", self.title, self.kind.display_name())
    }

    /// Update node size based on content
    fn update_size(&mut self) {
        let port_count = self.inputs.len().max(self.outputs.len());
        let min_height = Self::MIN_HEIGHT + (port_count.saturating_sub(1) as f32 * 20.0);
        self.size.1 = min_height.max(Self::MIN_HEIGHT);
    }

    /// Get the screen position of a port
    pub fn get_port_position(&self, port_id: PortId, is_input: bool) -> Option<Point<Pixels>> {
        let ports = if is_input { &self.inputs } else { &self.outputs };
        
        ports.iter().find(|p| p.id == port_id).map(|port| {
            let x = if is_input {
                self.position.0 - Self::PORT_RADIUS
            } else {
                self.position.0 + self.size.0 + Self::PORT_RADIUS
            };
            let y = self.position.1 + (port.position * self.size.1);
            Point::new(px(x), px(y))
        })
    }

    /// Check if a point is inside the node
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.position.0
            && x <= self.position.0 + self.size.0
            && y >= self.position.1
            && y <= self.position.1 + self.size.1
    }

    /// Get the bounds as (x, y, width, height)
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.position.0, self.position.1, self.size.0, self.size.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = GraphNode::new(NodeId::new(1), NodeKind::State, "START", (100.0, 100.0));
        
        assert_eq!(node.id, NodeId::new(1));
        assert_eq!(node.kind, NodeKind::State);
        assert_eq!(node.title, "START");
        assert_eq!(node.position, (100.0, 100.0));
        assert!(!node.selected);
        assert!(!node.dragging);
    }

    #[test]
    fn test_node_with_ports() {
        let node = GraphNode::new(NodeId::new(1), NodeKind::Condition, "Branch", (100.0, 100.0))
            .with_input_port(PortId::new(1), PortKind::Flow)
            .with_output_port(PortId::new(2), PortKind::True)
            .with_output_port(PortId::new(3), PortKind::False);
        
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 2);
    }

    #[test]
    fn test_node_kind_display() {
        assert_eq!(NodeKind::State.display_name(), "State");
        assert_eq!(NodeKind::Condition.display_name(), "Condition");
        assert_eq!(NodeKind::Quest.display_name(), "Quest");
    }

    #[test]
    fn test_port_kind_color() {
        let true_color = PortKind::True.color();
        let false_color = PortKind::False.color();
        
        assert!(true_color != false_color);
    }

    #[test]
    fn test_contains_point() {
        let node = GraphNode::new(NodeId::new(1), NodeKind::State, "Test", (100.0, 100.0));
        
        assert!(node.contains_point(110.0, 110.0));
        assert!(!node.contains_point(50.0, 50.0));
        assert!(!node.contains_point(300.0, 300.0));
    }

    #[test]
    fn test_node_movement() {
        let mut node = GraphNode::new(NodeId::new(1), NodeKind::State, "Test", (100.0, 100.0));
        
        node.move_by(50.0, 25.0);
        assert_eq!(node.position, (150.0, 125.0));
        
        node.set_position(0.0, 0.0);
        assert_eq!(node.position, (0.0, 0.0));
    }

    #[test]
    fn test_display_title() {
        let node1 = GraphNode::new(NodeId::new(1), NodeKind::State, "START", (0.0, 0.0));
        assert_eq!(node1.display_title(), "START");
        
        let node2 = GraphNode::new(NodeId::new(2), NodeKind::Condition, "Branch", (0.0, 0.0))
            .with_label("Has Sword?");
        assert_eq!(node2.display_title(), "Has Sword?");
    }

    #[test]
    fn test_selection_state() {
        let mut node = GraphNode::new(NodeId::new(1), NodeKind::State, "Test", (0.0, 0.0));
        
        assert!(!node.selected);
        node.set_selected(true);
        assert!(node.selected);
    }
}
