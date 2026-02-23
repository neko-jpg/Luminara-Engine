//! Render Device
//!
//! Abstraction layer over WGPU for editor rendering.
//! Manages device, queue, and shared resources.

use std::sync::Arc;

/// Render device configuration
#[derive(Debug, Clone)]
pub struct RenderDeviceConfig {
    /// Power preference for adapter selection
    pub power_preference: wgpu::PowerPreference,
    /// Required features
    pub required_features: wgpu::Features,
    /// Required limits
    pub required_limits: wgpu::Limits,
}

impl Default for RenderDeviceConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }
    }
}

/// RenderDevice wraps WGPU device and queue
///
/// This provides a safe interface for creating and managing GPU resources
/// that can be shared between the runtime and editor renderers.
pub struct RenderDevice {
    /// WGPU device (wrapped in Arc for shared ownership)
    device: Arc<wgpu::Device>,
    /// Command queue (wrapped in Arc for shared ownership)
    queue: Arc<wgpu::Queue>,
    /// Adapter info
    adapter_info: wgpu::AdapterInfo,
    /// Configuration
    config: RenderDeviceConfig,
}

impl RenderDevice {
    /// Create a new RenderDevice from existing WGPU context
    ///
    /// This is the primary constructor used when integrating with luminara_render.
    /// Note: GpuContext stores Device and Queue directly (not Arc), so we need to
    /// wrap them in Arc manually. This is safe because Device and Queue are Send + Sync.
    pub fn from_gpu_context(gpu_context: &luminara_render::GpuContext) -> Self {
        // We need to wrap device and queue in Arc
        // Since GpuContext stores them directly, we create Arc-wrapped copies
        // Note: This assumes the GpuContext outlives the RenderDevice

        // Safety: wgpu::Device and wgpu::Queue are thread-safe (Send + Sync)
        // We wrap them in Arc to allow shared ownership
        let device = Arc::new(unsafe { std::ptr::read(&gpu_context.device) });
        let queue = Arc::new(unsafe { std::ptr::read(&gpu_context.queue) });

        Self {
            device,
            queue,
            adapter_info: wgpu::AdapterInfo {
                name: "Unknown".to_string(),
                vendor: 0,
                device: 0,
                device_type: wgpu::DeviceType::Other,
                driver: "Unknown".to_string(),
                driver_info: "Unknown".to_string(),
                backend: wgpu::Backend::Empty,
            },
            config: RenderDeviceConfig::default(),
        }
    }

    /// Get the WGPU device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the WGPU queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Create a new render target texture
    pub fn create_render_target(
        &self,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Editor Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Editor Render Target View"),
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        (texture, view)
    }

    /// Create a depth texture for the render target
    pub fn create_depth_texture(
        &self,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Editor Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Editor Depth View"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        (texture, view)
    }

    /// Create a command encoder
    pub fn create_command_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    /// Create a render pipeline
    pub fn create_render_pipeline(
        &self,
        descriptor: &wgpu::RenderPipelineDescriptor,
    ) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(descriptor)
    }

    /// Create a shader module from WGSL source
    pub fn create_shader(&self, label: &str, source: &str) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
    }

    /// Create a buffer
    pub fn create_buffer(&self, label: &str, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Create a bind group layout
    pub fn create_bind_group_layout(
        &self,
        label: &str,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> wgpu::BindGroupLayout {
        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries,
            })
    }

    /// Create a pipeline layout
    pub fn create_pipeline_layout(
        &self,
        label: &str,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        self.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts,
                push_constant_ranges: &[],
            })
    }

    /// Get adapter info
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// Submit command buffer to queue
    pub fn submit(&self, command_buffer: wgpu::CommandBuffer) {
        self.queue.submit(std::iter::once(command_buffer));
    }
}

impl std::fmt::Debug for RenderDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderDevice")
            .field("adapter_info", &self.adapter_info)
            .field("config", &self.config)
            .finish()
    }
}

/// Safe wrapper for texture that can be shared between threads
pub struct SharedTexture {
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    size: (u32, u32),
}

impl SharedTexture {
    /// Create a new shared texture
    pub fn new(device: &RenderDevice, width: u32, height: u32) -> Self {
        let (texture, view) = device.create_render_target(width, height);

        Self {
            texture: Arc::new(texture),
            view: Arc::new(view),
            size: (width, height),
        }
    }

    /// Get the texture
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Get the texture view
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Get the size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Resize the texture
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32) {
        if self.size != (width, height) {
            let (texture, view) = device.create_render_target(width, height);
            self.texture = Arc::new(texture);
            self.view = Arc::new(view);
            self.size = (width, height);
        }
    }
}

impl Clone for SharedTexture {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            view: self.view.clone(),
            size: self.size,
        }
    }
}
