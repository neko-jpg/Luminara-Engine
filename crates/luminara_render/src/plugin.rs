use crate::camera::Camera;
use crate::command::CommandBuffer;
use crate::forward_plus::{update_lights_system, ForwardPlusRenderer};
use crate::gpu::GpuContext;
use crate::mesh::Mesh;
use crate::pipeline::PipelineCache;
use crate::render_graph::RenderGraph;
use crate::texture::TextureLoader;
use crate::CameraUniformBuffer;
use luminara_asset::{AssetServer, Handle};
use luminara_core::shared_types::{
    App, AppInterface, CoreStage, Plugin, Query, Res, ResMut, World,
};
use luminara_core::system::{ExclusiveMarker, FunctionMarker};
use luminara_math::Transform;
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
        app.insert_resource(CommandBuffer::default());
        app.insert_resource(ForwardPlusRenderer::new());
        app.insert_resource(crate::ShadowMapResources::default());
        app.insert_resource(crate::ShadowCascades::default());
        app.insert_resource(crate::PostProcessResources::default());
        app.insert_resource(crate::overlay::OverlayRenderer::new());
        app.insert_resource(crate::FluidSolverResource::new());
        app.insert_resource(crate::DebugRenderingResource::new());

        // Register startup system to initialize GPU context once Window is available
        app.add_system::<ExclusiveMarker>(CoreStage::Startup, setup_gpu_context);

        // Register window resize system
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, GpuContext>,
            luminara_core::event::EventReader<'static, luminara_window::WindowEvent>,
            Res<'static, luminara_window::Window>,
        )>(CoreStage::PreUpdate, crate::window_resize_system);

        // Register camera resize system to update aspect ratio on window resize
        app.add_system::<(
            FunctionMarker,
            Query<'static, &mut Camera>,
            luminara_core::event::EventReader<'static, luminara_window::WindowEvent>,
            Res<'static, luminara_window::Window>,
        )>(CoreStage::PreUpdate, crate::camera_resize_system);

        // Register camera projection system to update projection matrices
        app.add_system::<(
            FunctionMarker,
            Query<'static, &Camera>,
            Res<'static, luminara_window::Window>,
        )>(CoreStage::PreRender, crate::camera_projection_system);

        // Register mesh upload system
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, GpuContext>,
            Res<'static, AssetServer>,
            Query<'static, &Handle<Mesh>>,
        )>(CoreStage::PreRender, crate::mesh_upload_system);

        // Register fluid systems
        app.add_system::<(
            FunctionMarker,
            Query<'static, (luminara_core::Entity, &crate::FluidRenderer)>,
            ResMut<'static, crate::FluidSolverResource>,
        )>(CoreStage::PreUpdate, crate::init_fluid_solvers_system);

        app.add_system::<(
            FunctionMarker,
            Res<'static, luminara_core::Time>,
            Query<'static, (luminara_core::Entity, &crate::FluidRenderer)>,
            ResMut<'static, crate::FluidSolverResource>,
        )>(CoreStage::Update, crate::update_fluid_simulation_system);

        app.add_system::<(
            FunctionMarker,
            Query<'static, (luminara_core::Entity, &mut crate::FluidRenderer)>,
            Res<'static, crate::FluidSolverResource>,
            ResMut<'static, AssetServer>,
        )>(CoreStage::PreRender, crate::sync_fluid_textures_system);

        app.add_system::<(
            FunctionMarker,
            Query<'static, (luminara_core::Entity, &crate::FluidRenderer)>,
            ResMut<'static, crate::FluidSolverResource>,
        )>(CoreStage::PostUpdate, crate::cleanup_fluid_solvers_system);

        // Register Forward+ light update system
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, ForwardPlusRenderer>,
            Res<'static, GpuContext>,
            Query<'static, (&crate::DirectionalLight, &Transform)>,
            Query<'static, (&crate::PointLight, &Transform)>,
        )>(CoreStage::PreRender, update_lights_system);

        // Register shadow cascade update system
        app.add_system::<(
            FunctionMarker,
            Res<'static, GpuContext>,
            Res<'static, crate::ShadowCascades>,
            ResMut<'static, crate::ShadowMapResources>,
            Query<'static, (&Camera, &Transform)>,
            Query<'static, (&crate::DirectionalLight, &Transform)>,
        )>(CoreStage::PreRender, crate::update_shadow_cascades_system);

        // Register post-process initialization system
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, crate::PostProcessResources>,
            Res<'static, GpuContext>,
        )>(CoreStage::PreRender, crate::init_post_process_system);

        // Register LOD update system
        app.add_system::<(
            FunctionMarker,
            Query<'static, (&mut crate::MeshRenderer, &crate::Lod, &Transform)>,
            Query<'static, (&Camera, &Transform)>,
        )>(CoreStage::PreRender, crate::lod_update_system);

        // Register render_system to Render stage
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, GpuContext>,
            ResMut<'static, PipelineCache>,
            ResMut<'static, CameraUniformBuffer>,
            Res<'static, AssetServer>,
            Query<'static, (&Camera, &Transform)>,
            Query<'static, (&Handle<Mesh>, &Transform, &crate::PbrMaterial)>,
            Res<'static, luminara_window::Window>,
            Res<'static, crate::PostProcessResources>,
        )>(CoreStage::Render, crate::render_system);
    }
}

/// Startup system to initialize GpuContext and basic rendering resources
pub fn setup_gpu_context(world: &mut World) {
    let gpu = {
        let window = world.get_resource::<Window>().expect("Window not found");
        match GpuContext::new(&window) {
            Ok(gpu) => gpu,
            Err(e) => {
                log::error!("Failed to initialize GPU context: {}", e);
                return;
            }
        }
    };

    // Register texture loader
    if let Some(mut asset_server) = world.get_resource_mut::<AssetServer>() {
        asset_server.register_loader(TextureLoader);
    }

    // Create camera uniform buffer
    let camera_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32; 16]), // Identity or zero matrix
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let camera_bind_group_layout =
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    world.insert_resource(CameraUniformBuffer {
        buffer: camera_buffer,
        bind_group: camera_bind_group,
    });

    // Initialize Forward+ renderer
    if let Some(mut forward_plus) = world.get_resource_mut::<ForwardPlusRenderer>() {
        forward_plus.initialize(&gpu.device, gpu.surface_config.format);
    } else {
        log::error!("ForwardPlusRenderer not found during GPU setup");
    }

    // Initialize debug rendering
    if let Some(mut debug_rendering) = world.get_resource_mut::<crate::DebugRenderingResource>() {
        debug_rendering.initialize(&gpu);
    } else {
        log::error!("DebugRenderingResource not found during GPU setup");
    }

    world.insert_resource(gpu);
}
