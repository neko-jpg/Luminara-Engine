//! Logic Graph Editor (Vizia v0.3)

pub mod bottom_tabs;
pub mod graph_canvas;
pub mod inspector;
pub mod logic_graph_box;
pub mod node;
pub mod toolbar;

pub use bottom_tabs::{LogicGraphBottomTabKind, LogicGraphBottomTabPanelState};
pub use graph_canvas::{GraphCanvas, GraphCanvasPanelState};
pub use inspector::{NodeInspectorState, ViewMode};
pub use logic_graph_box::{LogicGraphState, ToolMode};
pub use node::{GraphNode, NodeId, NodeKind, NodePort, PortId, PortKind};
pub use toolbar::{LogicGraphToolbarState, SimulationState, StatusBarInfo, Tool};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphId(pub u64);

impl GraphId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Placeholder for a complete logic graph structure.
#[derive(Debug, Clone, Default)]
pub struct LogicGraph {
    pub nodes: Vec<GraphNode>,
    pub connections: Vec<(PortId, PortId)>,
}
