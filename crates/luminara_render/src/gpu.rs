use luminara_window::Window;
use wgpu;

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

use crate::error::RenderError;

impl GpuContext {
    pub fn new(window: &Window) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // SAFETY: We transmute the surface to 'static to make it easier to store in a Resource.
        // This is safe as long as the Window (which the surface is tied to) outlives the GpuContext.
        // In Luminara, both are typically long-lived resources managed by the App.
        let surface = instance
            .create_surface(window)
            .map_err(|e| RenderError::SurfaceCreationFailed(e.to_string()))?;
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(RenderError::AdapterRequestFailed)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Luminara Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .map_err(|e: wgpu::RequestDeviceError| RenderError::DeviceRequestFailed(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Fifo)
        {
            wgpu::PresentMode::Fifo
        } else {
            surface_caps.present_modes[0]
        };

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            surface_caps.alpha_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.width,
            height: window.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config: config,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn begin_frame(&mut self) -> Option<(wgpu::SurfaceTexture, wgpu::TextureView)> {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                Some((frame, view))
            }
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface texture checkout timeout — skipping frame");
                None
            }
            Err(wgpu::SurfaceError::Outdated) => {
                log::info!(
                    "Surface outdated, reconfiguring... ({}x{})",
                    self.surface_config.width,
                    self.surface_config.height
                );
                self.surface.configure(&self.device, &self.surface_config);
                // Retry immediately instead of skipping the frame
                match self.surface.get_current_texture() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        Some((frame, view))
                    }
                    Err(e) => {
                        log::error!("Surface still unavailable after reconfigure: {:?}", e);
                        None
                    }
                }
            }
            Err(wgpu::SurfaceError::Lost) => {
                log::warn!("Surface lost, reconfiguring...");
                self.surface.configure(&self.device, &self.surface_config);
                // Retry immediately instead of skipping the frame
                match self.surface.get_current_texture() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        Some((frame, view))
                    }
                    Err(e) => {
                        log::error!("Surface still unavailable after reconfigure: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                None
            }
        }
    }

    pub fn end_frame(&self, frame: wgpu::SurfaceTexture) {
        frame.present();
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        self.device.destroy();
    }
}

use luminara_core::shared_types::Resource;
impl Resource for GpuContext {}
