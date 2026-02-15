pub mod animation;
pub mod animation_system;
pub mod buffer_pool;
pub mod camera;
pub mod camera_systems;
pub mod command;
pub mod components;
pub mod error;
pub mod forward_plus;
pub mod gizmo;
pub mod gpu;
pub mod ik;
pub mod material;
pub mod mesh;
pub mod mesh_loader;
pub mod overlay;
pub mod particles;
pub mod pipeline;
pub mod plugin;
pub mod post_process;
pub mod render_graph;
pub mod shader;
pub mod shadow;
pub mod sprite;
pub mod sprite_systems;
pub mod texture;

pub use animation::{AnimationClip, Bone, GltfLoader, GltfScene, Skeleton, SkinnedMesh};
pub use animation_system::{AnimationPlayer, AnimationPlugin, SampledBoneTransform};
pub use camera::{Camera, Camera2d, Camera3d, Projection};
pub use camera_systems::{camera_projection_system, camera_resize_system};
pub use command::{CommandBuffer, DrawCommand, GizmoType};
pub use components::{DirectionalLight, Lod, MeshRenderer, PbrMaterial, PointLight};
pub use error::RenderError;
pub use forward_plus::update_lights_system;
pub use gizmo::{GizmoCategories, Gizmos};
pub use gpu::GpuContext;
pub use ik::{TwoBoneIK, TwoBoneIKSolver};
pub use material::Material;
pub use mesh::{Mesh, Vertex, AABB};
pub use mesh_loader::MeshLoader;
pub use overlay::{OverlayCommand, OverlayRenderer};
pub use particles::{Particle, ParticleEmitter, ParticlePlugin, ParticleSystem};
pub use pipeline::{CachedPipeline, PipelineCache, RenderPipelineDescriptor};
pub use plugin::RenderPlugin;
pub use post_process::{init_post_process_system, PostProcessResources};
pub use shader::Shader;
pub use shadow::{update_shadow_cascades_system, ShadowCascades, ShadowMapResources};
pub use sprite::{Anchor, Rect, Sprite, SpriteBatcher, SpriteRenderResources, ZOrder};
pub use sprite_systems::{init_sprite_system, prepare_sprite_batches, render_sprites};
pub use texture::{Texture, TextureData, TextureFormat};

use luminara_math::Color;

use luminara_asset::{AssetServer, Handle};
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::Transform;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniformData {
    pub albedo: [f32; 4],        // offset 0,  size 16
    pub metallic: f32,           // offset 16, size 4
    pub roughness: f32,          // offset 20, size 4
    pub _pad0: [f32; 2],         // offset 24, size 8 (pad for WGSL vec3 align 16)
    pub emissive: [f32; 3],      // offset 32, size 12
    pub has_albedo_texture: f32, // offset 44, size 4
} // total = 48 bytes, matches WGSL

pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Resource for CameraUniformBuffer {}

/// Phase 0: 頂点データのアップロード
pub fn mesh_upload_system(
    gpu: ResMut<GpuContext>,
    asset_server: Res<AssetServer>,
    meshes: Query<&Handle<Mesh>>,
) {
    let mut count = 0;
    for mesh_handle in meshes.iter() {
        if let Some(mesh) = asset_server.get(mesh_handle) {
            count += 1;
            // Check if buffer exists without locking first? No, need to read.
            let needs_upload = {
                let vb = mesh.vertex_buffer.read().unwrap();
                vb.is_none()
            };

            if needs_upload {
                mesh.upload(&gpu.device);
                log::info!(
                    "Uploaded mesh: {} vertices, {} indices",
                    mesh.vertices.len(),
                    mesh.indices.len()
                );
            }
        }
    }
    if count == 0 {
        // log::warn!("mesh_upload_system: No Mesh entities found by query");
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
    if iw > 0 && ih > 0 && (gpu.surface_config.width != iw || gpu.surface_config.height != ih) {
        log::info!("window_resize_system: resizing to {}x{}", iw, ih);
        gpu.resize(iw, ih);
    }
}

const PBR_SHADER_SOURCE: &str = r#"
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
    emissive: vec3<f32>,
    has_albedo_texture: f32,
};

@group(2) @binding(0)
var<uniform> material: MaterialUniform;

@group(2) @binding(1)
var albedo_texture: texture_2d<f32>;

@group(2) @binding(2)
var albedo_sampler: sampler;

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
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.position = camera.view_proj * world_pos;
    out.world_pos = world_pos.xyz;
    // Use inverse transpose for correct normals under non-uniform scale
    out.normal = normalize((model.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var albedo = material.albedo.rgb;
    if (material.has_albedo_texture > 0.0) {
        albedo = textureSample(albedo_texture, albedo_sampler, in.uv).rgb;
    }
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
    let emissive = material.emissive;

    let final_color = ambient + diffuse + specular + emissive;

    // Simple tone mapping
    let mapped = final_color / (final_color + vec3<f32>(1.0));

    return vec4<f32>(mapped, 1.0);
}
"#;

/// PBR rendering system with Forward+ pipeline
pub fn lod_update_system(
    mut lod_entities: Query<(&mut MeshRenderer, &Lod, &Transform)>,
    cameras: Query<(&Camera, &Transform)>,
) {
    if let Some((_, cam_transform)) = cameras.iter().next() {
        for (mut renderer, lod, transform) in lod_entities.iter_mut() {
            let distance = (transform.translation - cam_transform.translation).length();
            let lod_level = lod
                .distances
                .iter()
                .position(|&d| distance < d)
                .unwrap_or(lod.distances.len());
            if lod_level < lod.meshes.len() {
                renderer.mesh = lod.meshes[lod_level].clone();
            }
        }
    }
}

/// PBR rendering system with Forward+ pipeline
pub fn render_system(
    gpu: ResMut<GpuContext>,
    mut cache: ResMut<PipelineCache>,
    _uniform_buffer: ResMut<CameraUniformBuffer>,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Camera, &Transform)>,
    meshes: Query<(&Handle<Mesh>, &Transform, &PbrMaterial)>,
    _window: Res<luminara_window::Window>,
    post_process: Res<PostProcessResources>,
) {
    let frame = match gpu.surface.get_current_texture() {
        Ok(frame) => frame,
        Err(e) => {
            log::error!("Failed to get surface texture: {:?}", e);
            return;
        }
    };
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let frame_size = frame.texture.size();
    let render_width = frame_size.width.max(1);
    let render_height = frame_size.height.max(1);

    let clear_color = cameras
        .iter()
        .next()
        .map(|(cam, _)| cam.clear_color)
        .unwrap_or(Color::rgba(0.1, 0.1, 0.15, 1.0));

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

    // PBR rendering
    if let Some((camera, cam_transform)) = cameras.iter().next() {
        if cache.get_pipeline("pbr_lite").is_none() {
            // Create pipeline
            let shader_module = gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("PBR Shader"),
                    source: wgpu::ShaderSource::Wgsl(PBR_SHADER_SOURCE.into()),
                });

            // Create bind group layouts
            let camera_layout =
                gpu.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

            let model_layout =
                gpu.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

            let material_layout =
                gpu.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Material Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                    });

            let pipeline_layout =
                gpu.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("PBR Pipeline Layout"),
                        bind_group_layouts: &[&camera_layout, &model_layout, &material_layout],
                        push_constant_ranges: &[],
                    });

            let pipeline = gpu
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("PBR Pipeline"),
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
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
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

            cache.insert_pipeline(
                "pbr_lite".to_string(),
                pipeline,
                vec![camera_layout, model_layout, material_layout],
            );
        }

        let pbr_pipeline = cache.get_pipeline("pbr_lite").unwrap();

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

        // Camera uniform - must match shader CameraUniform: mat4x4 + vec3 + padding = 80 bytes
        let view_matrix = cam_transform.compute_matrix().inverse();
        let aspect = render_width as f32 / render_height as f32;
        let proj_matrix = camera.projection_matrix(aspect);
        let _view_proj = proj_matrix * view_matrix;

        // Serialize camera data: view_proj (64 bytes) + camera_pos (12 bytes) + padding (4 bytes) = 80 bytes
        let vp_cols = _view_proj.to_cols_array();
        let cam_pos = cam_transform.translation;
        let mut camera_data = [0u8; 80];
        camera_data[0..64].copy_from_slice(bytemuck::cast_slice(&vp_cols));
        camera_data[64..76]
            .copy_from_slice(bytemuck::cast_slice(&[cam_pos.x, cam_pos.y, cam_pos.z]));
        // bytes 76..80 remain zero (padding)
        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: &camera_data,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &pbr_pipeline.bind_group_layouts[0],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create default white texture
        let default_texture_data = TextureData {
            width: 1,
            height: 1,
            data: vec![255, 255, 255, 255],
            format: TextureFormat::Rgba8,
        };
        let mut default_tex = Texture::new(default_texture_data);
        default_tex.upload(&gpu.device, &gpu.queue);

        render_pass.set_pipeline(&pbr_pipeline.pipeline);
        render_pass.set_bind_group(0, &camera_bind_group, &[]);

        // Render meshes
        for (mesh_handle, mesh_transform, pbr_mat) in meshes.iter() {
            if let Some(mesh) = asset_server.get(mesh_handle) {
                let vb_guard = mesh.vertex_buffer.read().unwrap();
                let ib_guard = mesh.index_buffer.read().unwrap();

                if let (Some(vb), Some(ib)) = (vb_guard.as_ref(), ib_guard.as_ref()) {
                    let _model_matrix = mesh_transform.compute_matrix();
                    let model_cols = _model_matrix.to_cols_array();
                    let model_data: &[u8] = bytemuck::cast_slice(&model_cols);
                    let model_buffer =
                        gpu.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Model Buffer"),
                                contents: &model_data,
                                usage: wgpu::BufferUsages::UNIFORM,
                            });

                    let model_bind_group =
                        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Model Bind Group"),
                            layout: &pbr_pipeline.bind_group_layouts[1],
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: model_buffer.as_entire_binding(),
                            }],
                        });

                    let has_texture = if let Some(texture_handle) = &pbr_mat.albedo_texture {
                        asset_server.get(texture_handle).is_some()
                    } else {
                        false
                    };
                    let mat_uniform = MaterialUniformData {
                        albedo: [
                            pbr_mat.albedo.r,
                            pbr_mat.albedo.g,
                            pbr_mat.albedo.b,
                            pbr_mat.albedo.a,
                        ],
                        metallic: pbr_mat.metallic,
                        roughness: pbr_mat.roughness,
                        _pad0: [0.0; 2],
                        emissive: [pbr_mat.emissive.r, pbr_mat.emissive.g, pbr_mat.emissive.b],
                        has_albedo_texture: if has_texture { 1.0 } else { 0.0 },
                    };
                    let mat_array = [mat_uniform];
                    let mat_data = bytemuck::cast_slice(&mat_array);
                    let mat_buffer =
                        gpu.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Material Buffer"),
                                contents: mat_data,
                                usage: wgpu::BufferUsages::UNIFORM,
                            });

                    // Texture handling - use default texture for now
                    let tex_view = default_tex.view.as_ref().unwrap();
                    let tex_sampler = default_tex.sampler.as_ref().unwrap();

                    let mat_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Material Bind Group"),
                        layout: &pbr_pipeline.bind_group_layouts[2],
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: mat_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(tex_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(tex_sampler),
                            },
                        ],
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

    // Post-processing (bloom)
    if post_process.bloom_extract_pipeline.is_some() {
        // Create intermediate textures for bloom
        let bloom_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom Texture"),
            size: wgpu::Extent3d {
                width: render_width / 2,
                height: render_height / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let bloom_view = bloom_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Bloom extract pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Extract Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(pipeline) = &post_process.bloom_extract_pipeline {
                render_pass.set_pipeline(pipeline);
                // Bind the main render target as input
                // This would require bind group setup for bloom extract
                // For now, skip detailed implementation
            }
        }

        // Bloom blur and combine would follow...
        // This is a placeholder for full bloom implementation
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    gpu.end_frame(frame);
}
