pub mod camera;
pub mod camera_systems;
pub mod components;
pub mod error;
pub mod forward_plus;
pub mod gpu;
pub mod material;
pub mod mesh;
pub mod mesh_loader;
pub mod pipeline;
pub mod plugin;
pub mod post_process;
pub mod render_graph;
pub mod shader;
pub mod shadow;
pub mod sprite;
pub mod sprite_systems;
pub mod texture;

pub use camera::{Camera, Camera2d, Camera3d, Projection};
pub use camera_systems::{camera_projection_system, camera_resize_system};
pub use components::{DirectionalLight, MeshRenderer, PbrMaterial, PointLight};
pub use error::RenderError;
pub use forward_plus::{update_lights_system, ForwardPlusRenderer};
pub use gpu::GpuContext;
pub use material::Material;
pub use mesh::{Mesh, Vertex, AABB};
pub use mesh_loader::MeshLoader;
pub use pipeline::{PipelineCache, RenderPipeline, RenderPipelineDescriptor};
pub use plugin::RenderPlugin;
pub use post_process::{init_post_process_system, PostProcessResources};
pub use shader::Shader;
pub use shadow::{update_shadow_cascades_system, ShadowCascades, ShadowMapResources};
pub use sprite::{Anchor, Rect, Sprite, SpriteBatcher, SpriteRenderResources, ZOrder};
pub use sprite_systems::{init_sprite_system, prepare_sprite_batches, render_sprites};
pub use texture::{Texture, TextureData, TextureFormat};

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

/// PBR rendering system with Forward+ pipeline
pub fn render_system(
    gpu: ResMut<GpuContext>,
    _cache: ResMut<PipelineCache>,
    _uniform_buffer: ResMut<CameraUniformBuffer>,
    cameras: Query<(&Camera, &Transform)>,
    meshes: Query<(&Mesh, &Transform, &PbrMaterial)>,
    _window: Res<luminara_window::Window>,
) {
    let mut frame_opt = None;
    for attempt in 0..2 {
        match gpu.surface.get_current_texture() {
            Ok(frame) => {
                frame_opt = Some(frame);
                break;
            }
            Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
                log::warn!(
                    "render_system: surface {:?} on attempt {}, reconfiguring ({}x{})",
                    if attempt == 0 { "outdated/lost" } else { "still failing" },
                    attempt,
                    gpu.surface_config.width,
                    gpu.surface_config.height,
                );
                gpu.surface.configure(&gpu.device, &gpu.surface_config);
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

    let frame_size = frame.texture.size();
    let render_width = frame_size.width.max(1);
    let render_height = frame_size.height.max(1);

    // Get clear color from the first active camera
    let clear_color = cameras.iter().next()
        .map(|(cam, _)| cam.clear_color)
        .unwrap_or(Color::rgba(0.1, 0.1, 0.15, 1.0));

    // Create depth texture for proper 3D rendering
    let depth_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: render_width,
            height: render_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    // PBR-lite rendering pass
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        use wgpu::util::DeviceExt;
        
        // Shader with PBR-lite material support
        let shader_source = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ModelTransform {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> model: ModelTransform;

struct MaterialUniform {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    emissive_r: f32,
    emissive_g: f32,
};

@group(2) @binding(0)
var<uniform> material: MaterialUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.position = camera.view_proj * world_pos;
    out.world_pos = world_pos.xyz;
    // Use inverse transpose for correct normals under non-uniform scale
    out.normal = normalize((model.model * vec4<f32>(in.normal, 0.0)).xyz);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = material.albedo.rgb;
    let metallic = material.metallic;
    let roughness = material.roughness;

    // Simple directional light (sun-like)
    let light_dir = normalize(vec3<f32>(0.3, 0.7, 0.5));
    let light_color = vec3<f32>(1.0, 0.95, 0.9);
    let normal = normalize(in.normal);
    
    // Diffuse (Lambertian)
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let diffuse = albedo * light_color * n_dot_l;
    
    // Specular (Blinn-Phong approximation for PBR feel)
    let view_dir = normalize(camera.camera_pos - in.world_pos);
    let half_dir = normalize(light_dir + view_dir);
    let spec_power = mix(16.0, 256.0, 1.0 - roughness);
    let spec = pow(max(dot(normal, half_dir), 0.0), spec_power);
    let fresnel = metallic + (1.0 - metallic) * pow(1.0 - max(dot(view_dir, half_dir), 0.0), 5.0);
    let specular = light_color * spec * fresnel;
    
    // Ambient
    let ambient = albedo * vec3<f32>(0.15, 0.15, 0.2);
    
    // Emissive
    let emissive = vec3<f32>(material.emissive_r, material.emissive_g, 0.0);
    
    let final_color = ambient + diffuse + specular + emissive;
    
    // Simple tone mapping
    let mapped = final_color / (final_color + vec3<f32>(1.0));
    
    return vec4<f32>(mapped, 1.0);
}
"#;

        let shader_module = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR-Lite Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layouts
        let camera_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let model_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Layout"),
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

        let material_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR-Lite Pipeline Layout"),
            bind_group_layouts: &[&camera_layout, &model_layout, &material_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR-Lite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Camera uniform data
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct CameraUniformData {
            view_proj: [[f32; 4]; 4],
            camera_pos: [f32; 3],
            _padding: f32,
        }

        // Material uniform data
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct MaterialUniformData {
            albedo: [f32; 4],
            metallic: f32,
            roughness: f32,
            emissive_r: f32,
            emissive_g: f32,
        }

        // Update camera uniform
        if let Some((camera, cam_transform)) = cameras.iter().next() {
            let view_matrix = cam_transform.compute_matrix().inverse();
            let aspect = render_width as f32 / render_height as f32;
            let proj_matrix = camera.projection_matrix(aspect);
            let view_proj = proj_matrix * view_matrix;

            let camera_data = CameraUniformData {
                view_proj: view_proj.to_cols_array_2d(),
                camera_pos: cam_transform.translation.into(),
                _padding: 0.0,
            };

            let camera_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_data]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

            render_pass.set_pipeline(&pipeline);
            render_pass.set_viewport(0.0, 0.0, render_width as f32, render_height as f32, 0.0, 1.0);
            render_pass.set_scissor_rect(0, 0, render_width, render_height);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);

            // Render each mesh with PBR material
            for (mesh, mesh_transform, pbr_mat) in meshes.iter() {
                if let (Some(vb), Some(ib)) = (&mesh.vertex_buffer, &mesh.index_buffer) {
                    let model_matrix = mesh_transform.compute_matrix();

                    let model_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Model Buffer"),
                        contents: bytemuck::cast_slice(model_matrix.as_ref()),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });

                    let model_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Model Bind Group"),
                        layout: &model_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: model_buffer.as_entire_binding(),
                        }],
                    });

                    let mat_data = MaterialUniformData {
                        albedo: [pbr_mat.albedo.r, pbr_mat.albedo.g, pbr_mat.albedo.b, pbr_mat.albedo.a],
                        metallic: pbr_mat.metallic,
                        roughness: pbr_mat.roughness,
                        emissive_r: pbr_mat.emissive.r,
                        emissive_g: pbr_mat.emissive.g,
                    };

                    let mat_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Material Buffer"),
                        contents: bytemuck::cast_slice(&[mat_data]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });

                    let mat_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Material Bind Group"),
                        layout: &material_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: mat_buffer.as_entire_binding(),
                        }],
                    });

                    render_pass.set_bind_group(1, &model_bind_group, &[]);
                    render_pass.set_bind_group(2, &mat_bind_group, &[]);
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
