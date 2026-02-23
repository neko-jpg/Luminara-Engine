//! Viewport Renderer
//!
//! Renders 3D content to a viewport with camera, grid, and gizmo support.
//! Integrates with GPUI through texture sharing.

use std::sync::Arc;

use luminara_math::{Mat4, Vec3, Vec4};

use super::{command::RenderCommand, RenderDevice, RenderError};

/// Unique identifier for viewports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewportId(u64);

impl ViewportId {
    /// Create a new unique viewport ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ViewportId {
    fn default() -> Self {
        Self::new()
    }
}

/// Viewport configuration
#[derive(Debug, Clone)]
pub struct ViewportConfig {
    /// Background color
    pub background_color: Vec4,
    /// Grid settings
    pub grid_enabled: bool,
    pub grid_size: f32,
    pub grid_divisions: u32,
    pub grid_color: Vec4,
    /// Gizmo settings
    pub gizmo_enabled: bool,
    /// Whether to show debug info
    pub show_debug_info: bool,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            background_color: Vec4::new(0.12, 0.12, 0.12, 1.0), // Dark gray
            grid_enabled: true,
            grid_size: 100.0,
            grid_divisions: 100,
            grid_color: Vec4::new(0.3, 0.3, 0.3, 1.0),
            gizmo_enabled: true,
            show_debug_info: false,
        }
    }
}

/// Camera state for a viewport
#[derive(Debug, Clone)]
pub struct CameraState {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraState {
    /// Calculate the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Calculate the projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), aspect_ratio, self.near, self.far)
    }

    /// Calculate view-projection matrix
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }
}

/// ViewportRenderer manages rendering for a single viewport
pub struct ViewportRenderer {
    /// Viewport ID
    id: ViewportId,
    /// Render device
    device: Arc<RenderDevice>,
    /// Configuration
    config: ViewportConfig,
    /// Camera state
    camera: CameraState,
    /// Viewport size
    size: (u32, u32),
    /// Render target texture
    render_target: Option<Arc<wgpu::Texture>>,
    /// Render target view
    render_target_view: Option<wgpu::TextureView>,
    /// Depth texture
    depth_texture: Option<Arc<wgpu::Texture>>,
    /// Depth view
    depth_view: Option<wgpu::TextureView>,
}

impl ViewportRenderer {
    /// Create a new viewport renderer
    pub fn new(id: ViewportId, device: Arc<RenderDevice>, config: ViewportConfig) -> Self {
        Self {
            id,
            device,
            config,
            camera: CameraState::default(),
            size: (0, 0),
            render_target: None,
            render_target_view: None,
            depth_texture: None,
            depth_view: None,
        }
    }

    /// Get the viewport ID
    pub fn id(&self) -> ViewportId {
        self.id
    }

    /// Get the viewport size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Get the camera state
    pub fn camera(&self) -> &CameraState {
        &self.camera
    }

    /// Set the camera
    pub fn set_camera(&mut self, position: Vec3, target: Vec3, up: Vec3, fov: f32) {
        self.camera.position = position;
        self.camera.target = target;
        self.camera.up = up;
        self.camera.fov = fov;
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        if self.size != (width, height) {
            self.size = (width, height);
            self.recreate_render_targets();
        }
    }

    /// Recreate render targets after resize
    fn recreate_render_targets(&mut self) {
        let (width, height) = self.size;

        // Create color render target
        let (color_texture, color_view) = self.device.create_render_target(width, height);
        self.render_target = Some(Arc::new(color_texture));
        self.render_target_view = Some(color_view);

        // Create depth texture
        let (depth_texture, depth_view) = self.device.create_depth_texture(width, height);
        self.depth_texture = Some(Arc::new(depth_texture));
        self.depth_view = Some(depth_view);
    }

    /// Get the render target texture view (for GPUI integration)
    pub fn get_texture_view(&self) -> wgpu::TextureView {
        // If no render target exists yet, create a temporary one
        if let Some(ref view) = self.render_target_view {
            // Clone the view descriptor and create a new view
            if let Some(ref texture) = self.render_target {
                return texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Editor Viewport Texture View"),
                    format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: Some(1),
                });
            }
        }

        panic!("Render target not initialized. Call resize() first.");
    }

    /// Get the render target texture
    pub fn get_render_target(&self) -> Option<&wgpu::Texture> {
        self.render_target.as_ref().map(|t| t.as_ref())
    }

    /// Render the viewport
    pub fn render(&mut self, commands: &[RenderCommand]) -> Result<(), RenderError> {
        // Ensure render targets are initialized
        if self.render_target.is_none() {
            log::warn!(
                "Viewport {}: Render target not initialized, skipping render",
                self.id.0
            );
            return Ok(());
        }

        // Create command encoder
        let mut encoder = self.device.create_command_encoder("Viewport Render");

        // Get render target view
        let color_view = self
            .render_target_view
            .as_ref()
            .ok_or_else(|| RenderError::SurfaceError("No color view".to_string()))?;

        let depth_view = self
            .depth_view
            .as_ref()
            .ok_or_else(|| RenderError::SurfaceError("No depth view".to_string()))?;

        // Begin render pass
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.config.background_color.x as f64,
                            g: self.config.background_color.y as f64,
                            b: self.config.background_color.z as f64,
                            a: self.config.background_color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // TODO: Execute render commands here
            // For now, we just clear the viewport
        }

        // Submit commands
        self.device.submit(encoder.finish());

        Ok(())
    }

    /// Render grid (placeholder for now)
    fn render_grid(&self, _encoder: &mut wgpu::CommandEncoder) {
        // TODO: Implement grid rendering using line rendering
    }

    /// Render gizmos (placeholder for now)
    fn render_gizmos(&self, _encoder: &mut wgpu::CommandEncoder) {
        // TODO: Implement gizmo rendering
    }
}
