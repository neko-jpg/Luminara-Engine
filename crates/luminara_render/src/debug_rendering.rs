//! # Debug Rendering Modes
//!
//! Provides specialized rendering modes for debugging:
//! - Wireframe mode: Renders mesh edges
//! - Normal visualization: Shows surface normals as colors
//! - Overdraw heatmap: Visualizes pixel overdraw

use crate::{GpuContext, Shader};
use luminara_core::shared_types::Resource;
use std::sync::Arc;
use wgpu;

// ============================================================================
// Debug Rendering Mode
// ============================================================================

/// Debug rendering mode for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugRenderMode {
    /// Normal rendering (no debug)
    None,
    /// Wireframe mode - show mesh edges
    Wireframe,
    /// Normal visualization - show normals as colors
    Normals,
    /// Overdraw heatmap - visualize pixel overdraw
    Overdraw,
}

impl Default for DebugRenderMode {
    fn default() -> Self {
        Self::None
    }
}

// ============================================================================
// Debug Rendering Resource
// ============================================================================

/// Resource for managing debug rendering state
pub struct DebugRenderingResource {
    /// Current debug rendering mode
    pub mode: DebugRenderMode,
    /// Wireframe pipeline
    pub wireframe_pipeline: Option<Arc<wgpu::RenderPipeline>>,
    /// Normal visualization pipeline
    pub normal_pipeline: Option<Arc<wgpu::RenderPipeline>>,
    /// Overdraw heatmap pipeline
    pub overdraw_pipeline: Option<Arc<wgpu::RenderPipeline>>,
    /// Overdraw counter texture
    pub overdraw_texture: Option<wgpu::Texture>,
    /// Overdraw counter texture view
    pub overdraw_view: Option<wgpu::TextureView>,
    /// Overdraw bind group
    pub overdraw_bind_group: Option<wgpu::BindGroup>,
    /// Overdraw bind group layout
    pub overdraw_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Resource for DebugRenderingResource {}

impl Default for DebugRenderingResource {
    fn default() -> Self {
        Self {
            mode: DebugRenderMode::None,
            wireframe_pipeline: None,
            normal_pipeline: None,
            overdraw_pipeline: None,
            overdraw_texture: None,
            overdraw_view: None,
            overdraw_bind_group: None,
            overdraw_bind_group_layout: None,
        }
    }
}

impl DebugRenderingResource {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the debug rendering mode
    pub fn set_mode(&mut self, mode: DebugRenderMode) {
        self.mode = mode;
    }

    /// Get the current debug rendering mode
    pub fn mode(&self) -> DebugRenderMode {
        self.mode
    }

    /// Toggle wireframe mode
    pub fn toggle_wireframe(&mut self) {
        self.mode = if self.mode == DebugRenderMode::Wireframe {
            DebugRenderMode::None
        } else {
            DebugRenderMode::Wireframe
        };
    }

    /// Toggle normal visualization
    pub fn toggle_normals(&mut self) {
        self.mode = if self.mode == DebugRenderMode::Normals {
            DebugRenderMode::None
        } else {
            DebugRenderMode::Normals
        };
    }

    /// Toggle overdraw heatmap
    pub fn toggle_overdraw(&mut self) {
        self.mode = if self.mode == DebugRenderMode::Overdraw {
            DebugRenderMode::None
        } else {
            DebugRenderMode::Overdraw
        };
    }

    /// Initialize debug rendering pipelines
    pub fn initialize(&mut self, gpu: &GpuContext) {
        // Create wireframe pipeline
        self.wireframe_pipeline = Some(Arc::new(self.create_wireframe_pipeline(gpu)));

        // Create normal visualization pipeline
        self.normal_pipeline = Some(Arc::new(self.create_normal_pipeline(gpu)));

        // Create overdraw heatmap pipeline and resources
        self.initialize_overdraw(gpu);
    }

    /// Create wireframe rendering pipeline
    fn create_wireframe_pipeline(&self, gpu: &GpuContext) -> wgpu::RenderPipeline {
        let mut shader = Shader::from_wgsl(include_str!("../shaders/debug_wireframe.wgsl"));
        let shader_module = shader.compile(&gpu.device);

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug Wireframe Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Debug Wireframe Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    }],
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
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Line,
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
            })
    }

    /// Create normal visualization pipeline
    fn create_normal_pipeline(&self, gpu: &GpuContext) -> wgpu::RenderPipeline {
        let mut shader = Shader::from_wgsl(include_str!("../shaders/debug_normals.wgsl"));
        let shader_module = shader.compile(&gpu.device);

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug Normals Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Debug Normals Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[
                        // Position
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                        },
                        // Normal
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x3],
                        },
                    ],
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
            })
    }

    /// Initialize overdraw heatmap resources
    fn initialize_overdraw(&mut self, gpu: &GpuContext) {
        // Create overdraw counter texture (R32Uint for counting)
        let overdraw_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Overdraw Counter Texture"),
            size: wgpu::Extent3d {
                width: gpu.surface_config.width,
                height: gpu.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let overdraw_view = overdraw_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group layout for overdraw texture
        let overdraw_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Overdraw Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }],
                });

        // Create bind group
        let overdraw_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Overdraw Bind Group"),
            layout: &overdraw_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&overdraw_view),
            }],
        });

        // Create overdraw visualization pipeline
        let mut shader = Shader::from_wgsl(include_str!("../shaders/debug_overdraw.wgsl"));
        let shader_module = shader.compile(&gpu.device);

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug Overdraw Pipeline Layout"),
                bind_group_layouts: &[&overdraw_bind_group_layout],
                push_constant_ranges: &[],
            });

        let overdraw_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Debug Overdraw Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[],
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
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        self.overdraw_texture = Some(overdraw_texture);
        self.overdraw_view = Some(overdraw_view);
        self.overdraw_bind_group = Some(overdraw_bind_group);
        self.overdraw_bind_group_layout = Some(overdraw_bind_group_layout);
        self.overdraw_pipeline = Some(Arc::new(overdraw_pipeline));
    }

    /// Resize overdraw texture when window size changes
    pub fn resize_overdraw_texture(&mut self, gpu: &GpuContext) {
        if self.overdraw_texture.is_some() {
            self.initialize_overdraw(gpu);
        }
    }

    /// Clear overdraw counter texture
    pub fn clear_overdraw_counter(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(texture) = &self.overdraw_texture {
            encoder.clear_texture(
                texture,
                &wgpu::ImageSubresourceRange {
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                },
            );
        }
    }
}
