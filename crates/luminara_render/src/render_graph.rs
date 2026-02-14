pub trait RenderNode: Send + Sync {
    fn name(&self) -> &str;
    fn run<'a>(&self, context: &mut RenderContext<'a>) -> Result<(), RenderError>;
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

use crate::command::CommandBuffer;
use luminara_asset::AssetServer;

// Placeholder types for RenderContext and RenderError
pub struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub command_buffer: Option<&'a CommandBuffer>,
    pub asset_server: Option<&'a AssetServer>,
}

#[derive(Debug)]
pub enum RenderError {
    Other(String),
}

use luminara_core::shared_types::Resource;
impl Resource for RenderGraph {}
