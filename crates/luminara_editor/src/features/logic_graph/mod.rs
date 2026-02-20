//! Logic Graph Editor
//!
//! A node-based visual scripting editor for game logic, quests, and state machines.
//! Matches the HTML prototype design with:
//! - Graph canvas with grid background
//! - Visual nodes (State, Condition, Quest, Action)
//! - Connection ports (input/output)
//! - Node inspector panel
//! - Bottom tabs (DB Query, AI Assistant, Node Palette, Variables)

pub mod bottom_tabs;
pub mod graph_canvas;
pub mod inspector;
pub mod logic_graph_box;
pub mod node;
pub mod toolbar;

pub use bottom_tabs::{BottomTab, BottomTabPanel, TabKind};
pub use graph_canvas::{GraphCanvas, GraphCanvasPanel};
pub use inspector::{NodeInspector, ViewMode};
pub use logic_graph_box::{LogicGraphBox, ToolMode};
pub use node::{GraphNode, NodeId, NodeKind, NodePort, PortId, PortKind};
pub use toolbar::{LogicGraphToolbar, Tool, SimulationState, StatusBarInfo};

use gpui::{Hsla, rgb};
use std::collections::HashMap;

/// Unique identifier for a logic graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphId(pub u64);

impl GraphId {
    /// Create a new graph ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// A complete logic graph containing nodes and connections
#[derive(Debug, Clone)]
pub struct LogicGraph {
    /// Unique identifier
    pub id: GraphId,
    /// Graph name/title
    pub name: String,
    /// All nodes in the graph
    pub nodes: HashMap<NodeId, GraphNode>,
    /// Connections between nodes (from_port -> to_port)
    pub connections: Vec<Connection>,
    /// Currently selected node
    pub selected_node: Option<NodeId>,
    /// Canvas offset for panning
    pub canvas_offset: (f32, f32),
    /// Canvas zoom level
    pub zoom: f32,
}

impl LogicGraph {
    /// Create a new empty logic graph
    pub fn new(id: GraphId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            nodes: HashMap::new(),
            connections: Vec::new(),
            selected_node: None,
            canvas_offset: (0.0, 0.0),
            zoom: 1.0,
        }
    }

    /// Create a sample "Main Quest" graph matching the HTML prototype
    pub fn sample_main_quest() -> Self {
        let mut graph = Self::new(GraphId::new(1), "Main Quest");

        // START node
        let start_id = NodeId::new(1);
        let start_node = GraphNode::new(
            start_id,
            NodeKind::State,
            "START",
            (60.0, 40.0),
        )
        .with_icon_color(rgb(0x8aff8a).into())
        .with_output_port(PortId::new(1), PortKind::Flow);
        graph.nodes.insert(start_id, start_node);

        // Village node
        let village_id = NodeId::new(2);
        let village_node = GraphNode::new(
            village_id,
            NodeKind::State,
            "Village",
            (260.0, 40.0),
        )
        .with_icon_color(rgb(0x8a8aff).into())
        .with_input_port(PortId::new(2), PortKind::Flow)
        .with_output_port(PortId::new(3), PortKind::Flow);
        graph.nodes.insert(village_id, village_node);

        // Branch node (Condition)
        let branch_id = NodeId::new(3);
        let branch_node = GraphNode::new(
            branch_id,
            NodeKind::Condition,
            "Branch",
            (460.0, 40.0),
        )
        .with_icon_color(rgb(0xffaa00).into())
        .with_label("Has Sword?")
        .with_input_port(PortId::new(4), PortKind::Flow)
        .with_output_port(PortId::new(5), PortKind::True)
        .with_output_port(PortId::new(6), PortKind::False);
        graph.nodes.insert(branch_id, branch_node);

        // Dragon Quest node
        let dragon_id = NodeId::new(4);
        let dragon_node = GraphNode::new(
            dragon_id,
            NodeKind::Quest,
            "Dragon",
            (380.0, 160.0),
        )
        .with_icon_color(rgb(0xff8a8a).into())
        .with_input_port(PortId::new(7), PortKind::Flow);
        graph.nodes.insert(dragon_id, dragon_node);

        // Trade Route node
        let trade_id = NodeId::new(5);
        let trade_node = GraphNode::new(
            trade_id,
            NodeKind::Quest,
            "Trade",
            (540.0, 160.0),
        )
        .with_icon_color(rgb(0x8affaa).into())
        .with_input_port(PortId::new(8), PortKind::Flow);
        graph.nodes.insert(trade_id, trade_node);

        // Add connections
        graph.connections.push(Connection::new(
            PortRef::new(start_id, PortId::new(1)),
            PortRef::new(village_id, PortId::new(2)),
        ));
        graph.connections.push(Connection::new(
            PortRef::new(village_id, PortId::new(3)),
            PortRef::new(branch_id, PortId::new(4)),
        ));
        graph.connections.push(Connection::new(
            PortRef::new(branch_id, PortId::new(5)),
            PortRef::new(dragon_id, PortId::new(7)),
        ));

        // Select the branch node by default
        graph.selected_node = Some(branch_id);

        graph
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Select a node
    pub fn select_node(&mut self, node_id: Option<NodeId>) {
        self.selected_node = node_id;
    }

    /// Get the selected node
    pub fn selected_node(&self) -> Option<&GraphNode> {
        self.selected_node.and_then(|id| self.nodes.get(&id))
    }

    /// Get mutable reference to selected node
    pub fn selected_node_mut(&mut self) -> Option<&mut GraphNode> {
        self.selected_node.and_then(|id| self.nodes.get_mut(&id))
    }
}

/// Connection between two ports
#[derive(Debug, Clone)]
pub struct Connection {
    /// Source port
    pub from: PortRef,
    /// Target port
    pub to: PortRef,
    /// Optional label (e.g., "true", "false")
    pub label: Option<String>,
}

impl Connection {
    /// Create a new connection
    pub fn new(from: PortRef, to: PortRef) -> Self {
        Self {
            from,
            to,
            label: None,
        }
    }

    /// Create with label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Reference to a specific port on a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortRef {
    /// Node ID
    pub node_id: NodeId,
    /// Port ID
    pub port_id: PortId,
}

impl PortRef {
    /// Create a new port reference
    pub fn new(node_id: NodeId, port_id: PortId) -> Self {
        Self { node_id, port_id }
    }
}

/// Variable scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableScope {
    Global,
    Scene,
    Local,
}

/// Variable definition for the Variables tab
#[derive(Debug, Clone)]
pub struct Variable {
    pub key: String,
    pub value: String,
    pub scope: VariableScope,
}

impl Variable {
    pub fn new(key: impl Into<String>, value: impl Into<String>, scope: VariableScope) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            scope,
        }
    }
}

/// Condition entry for condition builder
#[derive(Debug, Clone)]
pub struct Condition {
    pub variable: String,
    pub operator: String,
    pub value: String,
    pub is_and: bool, // true = AND, false = OR
}

impl Condition {
    pub fn new(variable: impl Into<String>, operator: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
            operator: operator.into(),
            value: value.into(),
            is_and: true,
        }
    }

    pub fn with_or(mut self) -> Self {
        self.is_and = false;
        self
    }
}

/// Action entry for on_enter actions
#[derive(Debug, Clone)]
pub struct Action {
    pub icon: String,
    pub name: String,
    pub detail: String,
}

impl Action {
    pub fn new(icon: impl Into<String>, name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            name: name.into(),
            detail: detail.into(),
        }
    }
}

/// Node palette item for the palette tab
#[derive(Debug, Clone)]
pub struct NodePaletteItem {
    pub name: String,
    pub color: Hsla,
    pub kind: NodeKind,
}

impl NodePaletteItem {
    pub fn new(name: impl Into<String>, color: Hsla, kind: NodeKind) -> Self {
        Self {
            name: name.into(),
            color,
            kind,
        }
    }

    /// Get default palette items
    pub fn default_palette() -> Vec<Self> {
        vec![
            Self::new("State", rgb(0x5698ff).into(), NodeKind::State),
            Self::new("Event", rgb(0xffaa00).into(), NodeKind::Event),
            Self::new("Condition", rgb(0xff8a8a).into(), NodeKind::Condition),
            Self::new("Action", rgb(0x8aff8a).into(), NodeKind::Action),
            Self::new("Dialogue", rgb(0xcf8aff).into(), NodeKind::Dialogue),
            Self::new("Sub-graph", rgb(0x8ac4ff).into(), NodeKind::SubGraph),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_graph_creation() {
        let graph = LogicGraph::new(GraphId::new(1), "Test Graph");
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.connection_count(), 0);
        assert!(graph.selected_node.is_none());
    }

    #[test]
    fn test_sample_main_quest() {
        let graph = LogicGraph::sample_main_quest();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.connection_count(), 3);
        assert!(graph.selected_node.is_some());
    }

    #[test]
    fn test_node_selection() {
        let mut graph = LogicGraph::sample_main_quest();
        let node_id = NodeId::new(1);
        
        graph.select_node(Some(node_id));
        assert_eq!(graph.selected_node, Some(node_id));
        
        graph.select_node(None);
        assert!(graph.selected_node.is_none());
    }

    #[test]
    fn test_connection_with_label() {
        let conn = Connection::new(
            PortRef::new(NodeId::new(1), PortId::new(1)),
            PortRef::new(NodeId::new(2), PortId::new(2)),
        ).with_label("true");
        
        assert_eq!(conn.label, Some("true".to_string()));
    }

    #[test]
    fn test_variable_creation() {
        let var = Variable::new("player_hp", "100", VariableScope::Global);
        assert_eq!(var.key, "player_hp");
        assert_eq!(var.value, "100");
        assert_eq!(var.scope, VariableScope::Global);
    }

    #[test]
    fn test_condition_builder() {
        let cond = Condition::new("has_sword", "==", "true");
        assert_eq!(cond.variable, "has_sword");
        assert_eq!(cond.operator, "==");
        assert_eq!(cond.value, "true");
        assert!(cond.is_and);
        
        let or_cond = Condition::new("gold", ">=", "100").with_or();
        assert!(!or_cond.is_and);
    }

    #[test]
    fn test_action_creation() {
        let action = Action::new("dragon", "spawn dragon", "prefab: enemy_dragon");
        assert_eq!(action.icon, "dragon");
        assert_eq!(action.name, "spawn dragon");
        assert_eq!(action.detail, "prefab: enemy_dragon");
    }

    #[test]
    fn test_default_palette() {
        let palette = NodePaletteItem::default_palette();
        assert_eq!(palette.len(), 6);
        
        let kinds: Vec<_> = palette.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&NodeKind::State));
        assert!(kinds.contains(&NodeKind::Condition));
        assert!(kinds.contains(&NodeKind::Quest));
    }
}
