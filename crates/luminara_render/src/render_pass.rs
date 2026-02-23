//! Render pass abstractions

use crate::device::RenderContext;
use std::sync::Arc;

pub trait RenderPass: Send + Sync {
    fn name(&self) -> &str;
    fn prepare(&mut self, context: &RenderContext);
    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &RenderContext);
}

pub struct RenderPassManager {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPassManager {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass(&mut self, pass: impl RenderPass + 'static) {
        self.passes.push(Box::new(pass));
    }

    pub fn prepare(&mut self, context: &RenderContext) {
        for pass in &mut self.passes {
            pass.prepare(context);
        }
    }

    pub fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &RenderContext) {
        for pass in &self.passes {
            pass.execute(encoder, context);
        }
    }
}

impl Default for RenderPassManager {
    fn default() -> Self {
        Self::new()
    }
}
