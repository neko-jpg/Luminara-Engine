pub mod gpu;
pub mod mesh;
pub mod shader;
pub mod pipeline;
pub mod camera;
pub mod render_graph;
pub mod texture;
pub mod material;
pub mod plugin;

pub use gpu::GpuContext;
pub use mesh::{Vertex, Mesh};
pub use shader::Shader;
pub use pipeline::{RenderPipeline, PipelineCache, RenderPipelineDescriptor};
pub use camera::{Camera, Camera3d, Camera2d, Projection};
pub use texture::Texture;
pub use material::Material;
pub use plugin::RenderPlugin;

use luminara_core::shared_types::{ResMut, Query, Resource};
use luminara_math::Transform;

pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Resource for CameraUniformBuffer {}

/// Phase 0: 頂点データのアップロード
pub fn mesh_upload_system(
    gpu: ResMut<GpuContext>,
    meshes: Query<&mut Mesh>,
) {
    for mesh in meshes.iter() {
        if mesh.vertex_buffer.is_none() {
            mesh.upload(&gpu.device);
        }
    }
}

/// Phase 0: 単純な三角形描画
pub fn render_system(
    gpu: ResMut<GpuContext>,
    mut cache: ResMut<PipelineCache>,
    uniform_buffer: ResMut<CameraUniformBuffer>,
    cameras: Query<(&Camera, &Transform)>,
    meshes: Query<(&Mesh, &Transform)>,
) {
    let (frame, view) = gpu.begin_frame();

    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    let shader = Shader::from_wgsl(include_str!("../shaders/triangle.wgsl"));
    let pipeline_desc = RenderPipelineDescriptor {
        shader,
        vertex_layout: vec![Vertex::desc()],
        topology: wgpu::PrimitiveTopology::TriangleList,
        depth_stencil: false,
        blend: None,
        label: "Triangle Pipeline".to_string(),
    };

    let pipeline = cache.get_or_create(&gpu.device, gpu.surface_config.format, pipeline_desc);

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline.pipeline);

        for (camera, cam_transform) in cameras.iter() {
            let view_matrix = cam_transform.compute_matrix().inverse();
            let aspect = gpu.surface_config.width as f32 / gpu.surface_config.height as f32;
            let proj_matrix = camera.projection_matrix(aspect);
            let view_proj = proj_matrix * view_matrix;

            // Update uniform buffer
            gpu.queue.write_buffer(&uniform_buffer.buffer, 0, bytemuck::cast_slice(view_proj.as_ref()));

            render_pass.set_bind_group(0, &uniform_buffer.bind_group, &[]);

            for (mesh, _mesh_transform) in meshes.iter() {
                if let (Some(vb), Some(ib)) = (&mesh.vertex_buffer, &mesh.index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
                }
            }
        }
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    gpu.end_frame(frame);
}
