//! Editor Rendering Server
//!
//! Godot-style RenderingServer implementation for the Luminara Editor.
//! Provides a command-based API for rendering operations that can be executed
//! asynchronously and synchronized with GPUI's render loop.
//!
//! # Architecture
//!
//! ```
//! Editor UI (GPUI) → RenderingServer → Command Queue → RenderThread → WGPU
//!                                          ↓
//!                                   SharedRenderTarget → GPUI Texture
//! ```
//!
//! # Usage
//!
//! ```rust
//! // Initialize the rendering server
//! let server = RenderingServer::new(gpu_context, window);
//!
//! // Create a viewport
//! let viewport = server.viewport_create();
//! server.viewport_set_size(viewport, 1920, 1080);
//!
//! // Submit rendering commands
//! server.draw_mesh(mesh_handle, transform, material);
//! server.viewport_render(viewport);
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use luminara_core::Entity;
use luminara_math::{Mat4, Vec3, Vec4};

pub mod command;
pub mod render_device;
pub mod render_pipeline;
pub mod viewport_renderer;

pub use command::{RenderCommand, RenderCommandQueue};
pub use render_device::{RenderDevice, RenderDeviceConfig};
pub use render_pipeline::{RenderPipeline, ViewportRenderPipeline};
pub use viewport_renderer::{ViewportConfig, ViewportId, ViewportRenderer};

/// RenderingServer - Main entry point for editor rendering
///
/// This is the equivalent of Godot's RenderingServer, providing a thread-safe
/// interface for submitting rendering commands and managing render state.
pub struct RenderingServer {
    /// The render device manages WGPU resources
    device: Arc<RenderDevice>,
    /// Command queue for batched rendering operations
    command_queue: Arc<Mutex<RenderCommandQueue>>,
    /// Active viewports
    viewports: Arc<Mutex<HashMap<ViewportId, ViewportRenderer>>>,
    /// Frame counter for debugging
    frame_count: Arc<Mutex<u64>>,
}

impl RenderingServer {
    /// Create a new RenderingServer
    ///
    /// # Arguments
    /// * `gpu_context` - WGPU context from luminara_render
    /// * `window` - Window handle for surface creation
    pub fn new(gpu_context: &luminara_render::GpuContext) -> Self {
        let device = Arc::new(RenderDevice::from_gpu_context(gpu_context));
        let command_queue = Arc::new(Mutex::new(RenderCommandQueue::new()));
        let viewports = Arc::new(Mutex::new(HashMap::new()));
        let frame_count = Arc::new(Mutex::new(0));

        Self {
            device,
            command_queue,
            viewports,
            frame_count,
        }
    }

    /// Create a new viewport
    ///
    /// Returns a ViewportId that can be used to reference this viewport
    pub fn viewport_create(&self) -> ViewportId {
        let id = ViewportId::new();
        let mut viewports = self.viewports.lock().unwrap();

        let viewport = ViewportRenderer::new(id, self.device.clone(), ViewportConfig::default());

        viewports.insert(id, viewport);
        id
    }

    /// Destroy a viewport
    pub fn viewport_destroy(&self, viewport: ViewportId) {
        let mut viewports = self.viewports.lock().unwrap();
        viewports.remove(&viewport);
    }

    /// Set viewport size
    pub fn viewport_set_size(&self, viewport: ViewportId, width: u32, height: u32) {
        let mut viewports = self.viewports.lock().unwrap();
        if let Some(vp) = viewports.get_mut(&viewport) {
            vp.resize(width, height);
        }
    }

    /// Get viewport texture view for GPUI
    pub fn viewport_get_texture(&self, viewport: ViewportId) -> Option<wgpu::TextureView> {
        let viewports = self.viewports.lock().unwrap();
        viewports.get(&viewport).map(|vp| vp.get_texture_view())
    }

    /// Render a viewport
    ///
    /// This executes all pending commands and renders the viewport
    pub fn viewport_render(&self, viewport: ViewportId) -> Result<(), RenderError> {
        // Increment frame counter
        {
            let mut count = self.frame_count.lock().unwrap();
            *count += 1;
        }

        // Execute pending commands
        let commands = {
            let mut queue = self.command_queue.lock().unwrap();
            queue.take_commands()
        };

        // Get viewport and render
        let mut viewports = self.viewports.lock().unwrap();
        if let Some(vp) = viewports.get_mut(&viewport) {
            vp.render(&commands)?;
        }

        Ok(())
    }

    /// Submit a render command
    pub fn submit_command(&self, command: RenderCommand) {
        let mut queue = self.command_queue.lock().unwrap();
        queue.push(command);
    }

    /// Draw a mesh in the next render pass
    pub fn draw_mesh(
        &self,
        mesh: luminara_asset::Handle<luminara_render::Mesh>,
        transform: Mat4,
        material: luminara_asset::Handle<luminara_render::PbrMaterial>,
    ) {
        self.submit_command(RenderCommand::DrawMesh {
            mesh,
            transform,
            material,
        });
    }

    /// Draw a grid
    pub fn draw_grid(&self, size: f32, divisions: u32, color: Vec4) {
        self.submit_command(RenderCommand::DrawGrid {
            size,
            divisions,
            color,
        });
    }

    /// Draw gizmo at position
    pub fn draw_gizmo(&self, position: Vec3, rotation: Vec3, scale: Vec3, gizmo_type: GizmoType) {
        self.submit_command(RenderCommand::DrawGizmo {
            position,
            rotation,
            scale,
            gizmo_type,
        });
    }

    /// Set camera for a viewport
    pub fn viewport_set_camera(
        &self,
        viewport: ViewportId,
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov: f32,
    ) {
        let mut viewports = self.viewports.lock().unwrap();
        if let Some(vp) = viewports.get_mut(&viewport) {
            vp.set_camera(position, target, up, fov);
        }
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        *self.frame_count.lock().unwrap()
    }

    /// Get device reference
    pub fn device(&self) -> &Arc<RenderDevice> {
        &self.device
    }
}

impl Clone for RenderingServer {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            command_queue: self.command_queue.clone(),
            viewports: self.viewports.clone(),
            frame_count: self.frame_count.clone(),
        }
    }
}

/// Gizmo types for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoType {
    Translate,
    Rotate,
    Scale,
    Light,
    Camera,
}

/// Render errors
#[derive(Debug, Clone)]
pub enum RenderError {
    ViewportNotFound(ViewportId),
    DeviceLost,
    SurfaceError(String),
    CommandError(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::ViewportNotFound(id) => {
                write!(f, "Viewport not found: {:?}", id)
            }
            RenderError::DeviceLost => write!(f, "Render device lost"),
            RenderError::SurfaceError(e) => write!(f, "Surface error: {}", e),
            RenderError::CommandError(e) => write!(f, "Command error: {}", e),
        }
    }
}

impl std::error::Error for RenderError {}
