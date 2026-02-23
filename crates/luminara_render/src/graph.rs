//! Render graph for composing render passes

use crate::device::RenderContext;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNode>>,
    edges: Vec<(usize, usize)>,
    node_names: HashMap<String, usize>,
}

pub trait RenderNode: Send + Sync {
    fn name(&self) -> &str;
    fn prepare(&mut self, context: &RenderContext);
    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &RenderContext);
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_names: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: impl RenderNode + 'static) -> usize {
        let name = node.name().to_string();
        let index = self.nodes.len();
        self.node_names.insert(name, index);
        self.nodes.push(Box::new(node));
        index
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
    }

    pub fn prepare(&mut self, context: &RenderContext) {
        for node in &mut self.nodes {
            node.prepare(context);
        }
    }

    pub fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &RenderContext) {
        let mut executed = vec![false; self.nodes.len()];

        fn execute_node(
            graph: &RenderGraph,
            index: usize,
            encoder: &mut wgpu::CommandEncoder,
            context: &RenderContext,
            executed: &mut Vec<bool>,
        ) {
            if executed[index] {
                return;
            }
            executed[index] = true;

            for &(from, to) in &graph.edges {
                if to == index {
                    execute_node(graph, from, encoder, context, executed);
                }
            }

            graph.nodes[index].execute(encoder, context);
        }

        for i in 0..self.nodes.len() {
            execute_node(self, i, encoder, context, &mut executed);
        }
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}
