use crate::gpu::GpuContext;
use crate::pipeline::PipelineCache;
use crate::render_graph::RenderGraph;
use crate::CameraUniformBuffer;
use luminara_core::shared_types::{App, Plugin, CoreStage, AppInterface, Res, ResMut};
use luminara_window::Window;
use wgpu::util::DeviceExt;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn name(&self) -> &str {
        "RenderPlugin"
    }

    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(PipelineCache::new());
        app.insert_resource(RenderGraph::new());

        // Register startup system to initialize GPU context once Window is available
        app.add_system(CoreStage::Startup, setup_gpu_context);

        // Register mesh upload system
        app.add_system(CoreStage::PreRender, crate::mesh_upload_system);

        // Register render_system to Render stage
        app.add_system(CoreStage::Render, crate::render_system);
    }
}

/// Startup system to initialize GpuContext and basic rendering resources
pub fn setup_gpu_context(window: Res<Window>, mut app: ResMut<App>) {
    let gpu = GpuContext::new(&window);

    // Create camera uniform buffer
    let camera_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Uniform Buffer"),
        contents: bytemuck::cast_slice(&[0.0f32; 16]), // Identity or zero matrix
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Camera Bind Group"),
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });

    app.insert_resource(CameraUniformBuffer {
        buffer: camera_buffer,
        bind_group: camera_bind_group,
    });

    app.insert_resource(gpu);
}
