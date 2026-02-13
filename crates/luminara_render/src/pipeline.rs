use crate::shader::Shader;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu;

#[derive(Clone)]
pub struct CachedPipeline {
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
}

pub struct RenderPipelineDescriptor {
    pub shader: Shader,
    pub vertex_layout: Vec<wgpu::VertexBufferLayout<'static>>,
    pub topology: wgpu::PrimitiveTopology,
    pub depth_stencil: bool,
    pub blend: Option<wgpu::BlendState>,
    pub label: String,
}

pub struct PipelineCache {
    pipelines: HashMap<String, CachedPipeline>,
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn get_pipeline(&self, label: &str) -> Option<&CachedPipeline> {
        self.pipelines.get(label)
    }

    pub fn insert_pipeline(
        &mut self,
        label: String,
        pipeline: wgpu::RenderPipeline,
        layouts: Vec<wgpu::BindGroupLayout>,
    ) {
        self.pipelines.insert(
            label,
            CachedPipeline {
                pipeline: Arc::new(pipeline),
                bind_group_layouts: layouts.into_iter().map(Arc::new).collect(),
            },
        );
    }

    // Kept for backward compatibility if needed, but updated to use the new storage
    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        desc: RenderPipelineDescriptor,
    ) -> &CachedPipeline {
        if !self.pipelines.contains_key(&desc.label) {
            let mut shader = desc.shader;
            let module = shader.compile(device);

            // Phase 0: Simple camera bind group layout (legacy fallback)
            let camera_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", desc.label)),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&desc.label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &desc.vertex_layout,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: desc.blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: desc.topology,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: if desc.depth_stencil {
                    Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    })
                } else {
                    None
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            self.pipelines.insert(
                desc.label.clone(),
                CachedPipeline {
                    pipeline: Arc::new(pipeline),
                    bind_group_layouts: vec![Arc::new(camera_bind_group_layout)],
                },
            );
        }
        self.pipelines.get(&desc.label).unwrap()
    }
}

use luminara_core::shared_types::Resource;
impl Resource for PipelineCache {}
