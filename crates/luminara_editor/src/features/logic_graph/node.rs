//! Graph Node Component (Vizia version)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(pub u64);

impl PortId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    State,
    Condition,
    Quest,
    Action,
    Variable,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    Input,
    Output,
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct NodePort {
    pub id: PortId,
    pub name: String,
    pub kind: PortKind,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub title: String,
    pub position: [f32; 2],
    pub inputs: Vec<NodePort>,
    pub outputs: Vec<NodePort>,
    pub is_selected: bool,
    pub is_expanded: bool,
}

impl GraphNode {
    pub fn new(id: NodeId, kind: NodeKind, title: &str) -> Self {
        Self {
            id,
            kind,
            title: title.to_string(),
            position: [0.0, 0.0],
            inputs: Vec::new(),
            outputs: Vec::new(),
            is_selected: false,
            is_expanded: false,
        }
    }
}
