//! GPU Device and Surface management

use crate::shader::ShaderModule;
use crate::Texture;
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;
use wgpu::*;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("WGPU error: {0}")]
    Wgpu(#[from] wgpu::Error),
    #[error("Surface error: {0}")]
    Surface(String),
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    #[error("No compatible GPU found")]
    NoCompatibleGpu,
}

pub type RenderResult<T> = Result<T, RenderError>;

pub struct RenderSurface {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct RenderDevice {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    pub limits: wgpu::Limits,
    pub features: wgpu::Features,
}

impl RenderDevice {
    pub async fn new(window: &winit::window::Window) -> RenderResult<(Self, RenderSurface)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
            interactive: true,
        });

        let surface = instance.create_surface(window).map_err(|e| {
            RenderError::Surface(format!("Failed to create surface: {}", e))
        })?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoCompatibleGpu)?;

        let adapter_info = adapter.get_info();
        log::info!("Using GPU: {}", adapter_info.name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("luminara_render_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        log::info!("Render device initialized successfully");

        Ok((
            Self {
                device,
                queue,
                adapter_info,
                limits: adapter.limits(),
                features: adapter.features(),
            },
            RenderSurface { surface, config },
        ))
    }

    pub fn create_buffer(&self, desc: &BufferDescriptor) -> wgpu::Buffer {
        self.device.create_buffer(desc)
    }

    pub fn create_texture(&self, desc: &TextureDescriptor) -> wgpu::Texture {
        self.device.create_texture(desc)
    }

    pub fn create_shader_module(&self, shader: &ShaderModule) -> wgpu::ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: shader.label,
            source: wgpu::ShaderSource::Wgsl(shader.source.clone().into()),
        })
    }

    pub fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(desc)
    }

    pub fn create_compute_pipeline(
        &self,
        desc: &ComputePipelineDescriptor,
    ) -> wgpu::ComputePipeline {
        self.device.create_compute_pipeline(desc)
    }

    pub fn create_bind_group(&self, desc: &BindGroupDescriptor) -> wgpu::BindGroup {
        self.device.create_bind_group(desc)
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    pub fn create_sampler(&self, desc: &SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(desc)
    }

    pub fn command_encoder(&self, desc: &CommandEncoderDescriptor) -> CommandEncoder {
        self.device.create_command_encoder(desc)
    }
}

pub struct RenderContext {
    pub device: Arc<RenderDevice>,
    pub surface: Arc<RwLock<Option<RenderSurface>>>,
    pub format: wgpu::TextureFormat,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl RenderContext {
    pub fn new(device: RenderDevice, surface: RenderSurface) -> Self {
        let format = surface.config.format;
        let size = winit::dpi::PhysicalSize::new(surface.config.width, surface.config.height);

        Self {
            device: Arc::new(device),
            surface: Arc::new(RwLock::new(Some(surface))),
            format,
            size,
        }
    }

    pub fn resize(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(surface) = self.surface.write().as_mut() {
            surface.config.width = new_size.width;
            surface.config.height = new_size.height;
            surface.surface.configure(&self.device.device, &surface.config);
        }
    }

    pub fn current_surface_texture(&self) -> RenderResult<wgpu::SurfaceTexture> {
        let surface_guard = self.surface.read();
        let surface = surface_guard
            .as_ref()
            .ok_or_else(|| RenderError::Surface("Surface not configured".into()))?;

        surface
            .surface
            .get_current_texture()
            .map_err(|e| RenderError::Surface(format!("Failed to get texture: {}", e)))
    }
}
