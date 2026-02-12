pub trait RenderNode: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, context: &mut RenderContext) -> Result<(), RenderError>;
}

pub struct RenderGraph {
    pub nodes: Vec<Box<dyn RenderNode>>,
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: impl RenderNode + 'static) {
        self.nodes.push(Box::new(node));
    }
}

// Placeholder types for RenderContext and RenderError
pub struct RenderContext;
#[derive(Debug)]
pub enum RenderError {
    Other(String),
}

use luminara_core::shared_types::Resource;
impl Resource for RenderGraph {}
impl Resource for RenderContext {}
