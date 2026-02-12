pub mod camera;
pub mod error;
pub mod gpu;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod plugin;
pub mod render_graph;
pub mod shader;
pub mod texture;

pub use camera::{Camera, Camera2d, Camera3d, Projection};
pub use error::RenderError;
pub use gpu::GpuContext;
pub use material::Material;
pub use mesh::{Mesh, Vertex};
pub use pipeline::{PipelineCache, RenderPipeline, RenderPipelineDescriptor};
pub use plugin::RenderPlugin;
pub use shader::Shader;
pub use texture::Texture;

use luminara_math::Color;

use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::Transform;

pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Resource for CameraUniformBuffer {}

/// Phase 0: 頂点データのアップロード
pub fn mesh_upload_system(gpu: ResMut<GpuContext>, meshes: Query<&mut Mesh>) {
    let mut count = 0;
    for mesh in meshes.iter() {
        count += 1;
        if mesh.vertex_buffer.is_none() {
            mesh.upload(&gpu.device);
            log::info!("Uploaded mesh: {} vertices, {} indices", mesh.vertices.len(), mesh.indices.len());
        }
    }
    if count == 0 {
        log::warn!("mesh_upload_system: No Mesh entities found by query");
    }
}

pub fn window_resize_system(
    mut gpu: ResMut<GpuContext>,
    _events: luminara_core::event::EventReader<luminara_window::WindowEvent>,
    window: Res<luminara_window::Window>,
) {
    // Single source of truth: query the live inner size from winit.
    // This avoids conflicts between stored event size and live size.
    let (iw, ih) = window.inner_size();
    if iw > 0 && ih > 0
        && (gpu.surface_config.width != iw || gpu.surface_config.height != ih)
    {
        log::info!("window_resize_system: resizing to {}x{}", iw, ih);
        gpu.resize(iw, ih);
    }
}

/// Phase 0: 単純な三角形描画
pub fn render_system(
    mut gpu: ResMut<GpuContext>,
    mut cache: ResMut<PipelineCache>,
    uniform_buffer: ResMut<CameraUniformBuffer>,
    cameras: Query<(&Camera, &Transform)>,
    meshes: Query<(&Mesh, &Transform)>,
    window: Res<luminara_window::Window>,
) {
    // Self-healing sync before acquiring surface texture.
    let (iw, ih) = window.inner_size();
    if iw > 0 && ih > 0
        && (gpu.surface_config.width != iw || gpu.surface_config.height != ih)
    {
        log::info!(
            "render_system: surface size mismatch, resizing {}x{} -> {}x{}",
            gpu.surface_config.width,
            gpu.surface_config.height,
            iw,
            ih
        );
        gpu.resize(iw, ih);
    }

    let camera_count = cameras.iter().count();
    let mesh_count = meshes.iter().count();
    log::debug!("render_system: cameras={}, meshes={}", camera_count, mesh_count);

    let mut frame_opt = None;
    for _ in 0..2 {
        match gpu.surface.get_current_texture() {
            Ok(frame) => {
                frame_opt = Some(frame);
                break;
            }
            Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
                let (rw, rh) = window.inner_size();
                if rw > 0 && rh > 0 {
                    gpu.resize(rw, rh);
                } else {
                    gpu.surface.configure(&gpu.device, &gpu.surface_config);
                }
            }
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface texture checkout timeout — skipping frame");
                return;
            }
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        }
    }

    let frame = match frame_opt {
        Some(frame) => frame,
        None => return,
    };
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Use the actual acquired frame size for viewport/scissor/aspect.
    // This prevents stale config values from causing partially updated regions.
    let frame_size = frame.texture.size();
    let render_width = frame_size.width.max(1);
    let render_height = frame_size.height.max(1);

    // Get clear color from the first active camera, or use default grey
    let clear_color = cameras.iter().next()
        .map(|(cam, _)| cam.clear_color)
        .unwrap_or(Color::rgba(0.1, 0.1, 0.1, 1.0));

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                        r: clear_color.r as f64,
                        g: clear_color.g as f64,
                        b: clear_color.b as f64,
                        a: clear_color.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_viewport(0.0, 0.0, render_width as f32, render_height as f32, 0.0, 1.0);
        render_pass.set_scissor_rect(0, 0, render_width, render_height);

        for (camera, cam_transform) in cameras.iter() {
            let view_matrix = cam_transform.compute_matrix().inverse();
            let aspect = render_width as f32 / render_height as f32;
            let proj_matrix = camera.projection_matrix(aspect);
            let view_proj = proj_matrix * view_matrix;

            // Update uniform buffer
            gpu.queue.write_buffer(
                &uniform_buffer.buffer,
                0,
                bytemuck::cast_slice(view_proj.as_ref()),
            );

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
