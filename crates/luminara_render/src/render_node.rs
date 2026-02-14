use crate::render_graph::{RenderNode, RenderContext, RenderError};
use crate::command::DrawCommand;
use crate::{Mesh, PbrMaterial};
use luminara_math::Mat4;

pub struct MeshRenderNode;

impl RenderNode for MeshRenderNode {
    fn name(&self) -> &str {
        "MeshRenderNode"
    }

    fn run<'a>(&self, context: &mut RenderContext<'a>) -> Result<(), RenderError> {
        if let (Some(cb), Some(assets)) = (context.command_buffer, context.asset_server) {
            for cmd in &cb.commands {
                match cmd {
                    DrawCommand::DrawMesh { mesh: mesh_handle, material: mat_handle, transform } => {
                        // Retrieve mesh and material
                        // Note: AssetServer::get returns Arc<T>
                        if let (Some(_mesh), Some(_material)) = (assets.get(mesh_handle), assets.get(mat_handle)) {
                            // Rendering logic would go here:
                            // 1. Bind pipeline (from cache in context?)
                            // 2. Bind vertex/index buffers from mesh
                            // 3. Bind material uniforms
                            // 4. Draw
                        }
                    }
                    DrawCommand::DrawInstanced { mesh: mesh_handle, material: mat_handle, transforms } => {
                         if let (Some(_mesh), Some(_material)) = (assets.get(mesh_handle), assets.get(mat_handle)) {
                            // Instanced rendering logic
                         }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
