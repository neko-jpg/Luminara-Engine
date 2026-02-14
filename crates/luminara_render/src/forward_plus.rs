// Forward+ rendering pipeline implementation
use crate::{DirectionalLight, GpuContext, PointLight, Shader};
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::Transform;
use std::mem;

/// Light data structures matching shader layout
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DirectionalLightData {
    direction: [f32; 3],
    _padding1: f32,
    color: [f32; 3],
    intensity: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightData {
    position: [f32; 3],
    range: f32,
    color: [f32; 3],
    intensity: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightBuffer {
    directional_count: u32,
    point_count: u32,
    _padding: [u32; 2],
    directional_lights: [DirectionalLightData; 4],
    point_lights: [PointLightData; 256],
}

impl Default for LightBuffer {
    fn default() -> Self {
        Self {
            directional_count: 0,
            point_count: 0,
            _padding: [0; 2],
            directional_lights: [DirectionalLightData {
                direction: [0.0; 3],
                _padding1: 0.0,
                color: [0.0; 3],
                intensity: 0.0,
            }; 4],
            point_lights: [PointLightData {
                position: [0.0; 3],
                range: 0.0,
                color: [0.0; 3],
                intensity: 0.0,
            }; 256],
        }
    }
}

/// Camera uniform matching shader layout
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

/// Material uniform matching shader layout
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    albedo: [f32; 4],
    metallic: f32,
    roughness: f32,
    emissive: [f32; 3],
    _padding: f32,
}

/// Model transform uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelTransform {
    model: [[f32; 4]; 4],
    normal_matrix: [[f32; 4]; 4],
}

/// Forward+ renderer resources
pub struct ForwardPlusRenderer {
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub camera_buffer: Option<wgpu::Buffer>,
    pub camera_bind_group: Option<wgpu::BindGroup>,
    pub light_buffer: Option<wgpu::Buffer>,
    pub light_bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layouts: Option<BindGroupLayouts>,
}

pub struct BindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
    pub lights: wgpu::BindGroupLayout,
    pub model: wgpu::BindGroupLayout,
}

impl Resource for ForwardPlusRenderer {}

impl Default for ForwardPlusRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ForwardPlusRenderer {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            camera_buffer: None,
            camera_bind_group: None,
            light_buffer: None,
            light_bind_group: None,
            bind_group_layouts: None,
        }
    }

    pub fn initialize(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        // Create bind group layouts
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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

        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                // Material uniform
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
                // Albedo texture
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
                // Albedo sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Normal sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Metallic-roughness texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Metallic-roughness sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let lights_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lights Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
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

        self.bind_group_layouts = Some(BindGroupLayouts {
            camera: camera_layout,
            material: material_layout,
            lights: lights_layout,
            model: model_layout,
        });

        // Create pipeline
        let layouts = self.bind_group_layouts.as_ref().unwrap();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.camera,
                &layouts.material,
                &layouts.lights,
                &layouts.model,
            ],
            push_constant_ranges: &[],
        });

        let mut shader = Shader::from_wgsl(include_str!("../shaders/pbr.wgsl"));
        let shader_module = shader.compile(device);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[crate::Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);

        // Create camera buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layouts.camera,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);

        // Create light buffer
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: mem::size_of::<LightBuffer>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &layouts.lights,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        self.light_buffer = Some(light_buffer);
        self.light_bind_group = Some(light_bind_group);
    }
}

/// System to update light buffer
pub fn update_lights_system(
    renderer: ResMut<ForwardPlusRenderer>,
    gpu: Res<GpuContext>,
    directional_lights: Query<(&DirectionalLight, &Transform)>,
    point_lights: Query<(&PointLight, &Transform)>,
) {
    if renderer.light_buffer.is_none() {
        return;
    }

    let mut light_data = LightBuffer::default();

    // Collect directional lights
    for (light, transform) in directional_lights.iter().take(4) {
        let direction = transform.forward();
        light_data.directional_lights[light_data.directional_count as usize] =
            DirectionalLightData {
                direction: [direction.x, direction.y, direction.z],
                _padding1: 0.0,
                color: [light.color.r, light.color.g, light.color.b],
                intensity: light.intensity,
            };
        light_data.directional_count += 1;
    }

    // Collect point lights
    for (light, transform) in point_lights.iter().take(256) {
        let position = transform.translation;
        light_data.point_lights[light_data.point_count as usize] = PointLightData {
            position: [position.x, position.y, position.z],
            range: light.range,
            color: [light.color.r, light.color.g, light.color.b],
            intensity: light.intensity,
        };
        light_data.point_count += 1;
    }

    // Update buffer
    gpu.queue.write_buffer(
        renderer.light_buffer.as_ref().unwrap(),
        0,
        bytemuck::cast_slice(&[light_data]),
    );
}
