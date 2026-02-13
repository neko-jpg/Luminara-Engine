use crate::sprite::{
    create_sprite_quad, Sprite, SpriteBatcher, SpriteInstance, SpriteRenderResources, SpriteVertex,
    ZOrder,
};
use crate::GpuContext;
use luminara_asset::AssetServer;
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::Transform;
use wgpu::util::DeviceExt;

impl Resource for SpriteRenderResources {}
impl Resource for SpriteBatcher {}

/// Initialize sprite rendering resources
pub fn init_sprite_system(
    mut resources: ResMut<SpriteRenderResources>,
    gpu: Res<GpuContext>,
    _camera_bind_group_layout: Res<crate::CameraUniformBuffer>,
) {
    // Skip if already initialized
    if resources.pipeline.is_some() {
        return;
    }

    log::info!("Initializing sprite rendering system");

    // Create sprite quad mesh
    let (vertices, indices) = create_sprite_quad();

    let vertex_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let index_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    // Create texture bind group layout
    let texture_bind_group_layout =
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sprite Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

    // Create sampler
    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Sprite Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // Create camera bind group layout (matching the shader)
    let camera_bind_group_layout =
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sprite Camera Bind Group Layout"),
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

    // Create pipeline layout
    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

    // Load shader
    let shader_source = include_str!("../shaders/sprite.wgsl");
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_source)),
        });

    // Create render pipeline
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[SpriteVertex::desc(), SpriteInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

    resources.pipeline = Some(pipeline);
    resources.vertex_buffer = Some(vertex_buffer);
    resources.index_buffer = Some(index_buffer);
    resources.texture_bind_group_layout = Some(texture_bind_group_layout);
    resources.sampler = Some(sampler);

    log::info!("Sprite rendering system initialized");
}

/// Prepare sprite batches for rendering
pub fn prepare_sprite_batches(
    mut batcher: ResMut<SpriteBatcher>,
    sprites: Query<(&Sprite, &Transform)>,
    _z_orders: Query<&ZOrder>,
) {
    // Collect sprites with their global transforms
    let sprite_data: Vec<_> = sprites
        .iter()
        .map(|(sprite, transform)| {
            let matrix = transform.compute_matrix();
            // Try to get z_order for this entity if it exists
            // For now, we'll use None since we can't easily get the entity ID from Query
            (sprite, matrix, None::<&ZOrder>)
        })
        .collect();

    // Prepare batches
    batcher.prepare(sprite_data.iter().map(|(s, m, z)| (*s, m, *z)));
}

/// Render sprites using batched instancing
pub fn render_sprites(
    mut gpu: ResMut<GpuContext>,
    resources: Res<SpriteRenderResources>,
    batcher: Res<SpriteBatcher>,
    camera_uniform: Res<crate::CameraUniformBuffer>,
    asset_server: Res<AssetServer>,
) {
    // Skip if not initialized
    let pipeline = match &resources.pipeline {
        Some(p) => p,
        None => return,
    };

    let vertex_buffer = resources.vertex_buffer.as_ref().unwrap();
    let index_buffer = resources.index_buffer.as_ref().unwrap();
    let texture_bind_group_layout = resources.texture_bind_group_layout.as_ref().unwrap();
    let sampler = resources.sampler.as_ref().unwrap();

    // Get current frame
    let frame = match gpu.begin_frame() {
        Some((frame, _view)) => frame,
        None => return,
    };

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Sprite Render Encoder"),
        });

    // Render each batch
    for batch in &batcher.batches {
        if batch.instances.is_empty() {
            continue;
        }

        // Get texture
        let texture = match asset_server.get(&batch.texture) {
            Some(t) => t,
            None => continue,
        };

        // Skip if texture not uploaded to GPU
        let texture_view = match &texture.view {
            Some(v) => v,
            None => continue,
        };

        // Create instance buffer for this batch
        let instance_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite Instance Buffer"),
                contents: bytemuck::cast_slice(&batch.instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Create texture bind group
        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Texture Bind Group"),
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &camera_uniform.bind_group, &[]);
            render_pass.set_bind_group(1, &texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..batch.instances.len() as u32);
        }
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    gpu.end_frame(frame);
}
