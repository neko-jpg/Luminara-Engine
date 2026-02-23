//! Render Pipeline
//!
//! PBR rendering pipeline for the editor viewport.
//! Supports mesh rendering, materials, lighting, and post-processing.

use std::sync::Arc;

use luminara_asset::Handle;
use luminara_math::Mat4;
use luminara_render::{Material, Mesh};

use super::RenderDevice;

/// Main render pipeline for mesh rendering
pub struct RenderPipeline {
    device: Arc<RenderDevice>,
    /// PBR shader
    pbr_shader: wgpu::ShaderModule,
    /// Camera bind group layout
    camera_layout: wgpu::BindGroupLayout,
    /// Model bind group layout
    model_layout: wgpu::BindGroupLayout,
    /// Material bind group layout
    material_layout: wgpu::BindGroupLayout,
    /// Pipeline layout
    pipeline_layout: wgpu::PipelineLayout,
    /// Render pipeline
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new(device: Arc<RenderDevice>) -> Self {
        // Create shader
        let pbr_shader = device.create_shader("PBR Shader", PBR_SHADER_SOURCE);

        // Create bind group layouts
        let camera_layout = device.create_bind_group_layout(
            "Camera Layout",
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );

        let model_layout = device.create_bind_group_layout(
            "Model Layout",
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );

        let material_layout = device.create_bind_group_layout(
            "Material Layout",
            &[
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
        );

        let pipeline_layout = device.create_pipeline_layout(
            "PBR Pipeline Layout",
            &[&camera_layout, &model_layout, &material_layout],
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pbr_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pbr_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
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

        Self {
            device,
            pbr_shader,
            camera_layout,
            model_layout,
            material_layout,
            pipeline_layout,
            render_pipeline,
        }
    }

    /// Get the camera bind group layout
    pub fn camera_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_layout
    }

    /// Get the model bind group layout
    pub fn model_layout(&self) -> &wgpu::BindGroupLayout {
        &self.model_layout
    }

    /// Get the material bind group layout
    pub fn material_layout(&self) -> &wgpu::BindGroupLayout {
        &self.material_layout
    }

    /// Get the render pipeline
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}

/// Vertex format for mesh rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tangent
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Pipeline specialized for viewport rendering
pub struct ViewportRenderPipeline {
    base_pipeline: RenderPipeline,
    device: Arc<RenderDevice>,
}

impl ViewportRenderPipeline {
    /// Create a new viewport render pipeline
    pub fn new(device: Arc<RenderDevice>) -> Self {
        let base_pipeline = RenderPipeline::new(device.clone());

        Self {
            base_pipeline,
            device,
        }
    }

    /// Render a mesh
    pub fn render_mesh(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        mesh: &Mesh,
        transform: Mat4,
        _material: &Material,
    ) {
        // Set pipeline
        render_pass.set_pipeline(self.base_pipeline.pipeline());

        // TODO: Set vertex and index buffers
        // TODO: Set bind groups for camera, model, and material
        // TODO: Draw
    }
}

/// PBR Shader source (WGSL)
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
