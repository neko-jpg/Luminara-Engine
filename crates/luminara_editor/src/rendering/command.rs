//! Render Commands
//!
//! Command-based rendering API inspired by Godot's RenderingServer.
//! Commands are batched and executed together for optimal GPU utilization.

use luminara_asset::Handle;
use luminara_math::{Mat4, Vec3, Vec4};
use luminara_render::{Mesh, PbrMaterial};

use super::GizmoType;

/// Render commands that can be submitted to the RenderingServer
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Draw a mesh with transform and material
    DrawMesh {
        mesh: Handle<Mesh>,
        transform: Mat4,
        material: Handle<PbrMaterial>,
    },
    /// Draw a grid on the ground plane
    DrawGrid {
        size: f32,
        divisions: u32,
        color: Vec4,
    },
    /// Draw a gizmo at a specific position
    DrawGizmo {
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        gizmo_type: GizmoType,
    },
    /// Draw lines (for debug visualization)
    DrawLines { points: Vec<Vec3>, color: Vec4 },
    /// Clear the viewport with a color
    Clear { color: Vec4 },
    /// Set the active camera
    SetCamera {
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov: f32,
    },
    /// Set viewport rect (x, y, width, height)
    SetViewport {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Draw debug text
    DrawText {
        text: String,
        position: Vec3,
        color: Vec4,
        size: f32,
    },
}

/// Command queue for batched rendering operations
///
/// This allows multiple render commands to be collected and executed
/// together for better GPU efficiency.
pub struct RenderCommandQueue {
    commands: Vec<RenderCommand>,
    capacity: usize,
}

impl RenderCommandQueue {
    /// Create a new command queue with default capacity
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new command queue with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a command to the queue
    pub fn push(&mut self, command: RenderCommand) {
        if self.commands.len() >= self.capacity {
            // Grow the buffer if needed
            self.capacity *= 2;
            self.commands.reserve(self.capacity - self.commands.len());
        }
        self.commands.push(command);
    }

    /// Take all commands from the queue (clears the queue)
    pub fn take_commands(&mut self) -> Vec<RenderCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Get the number of commands in the queue
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Peek at the commands without removing them
    pub fn peek(&self) -> &[RenderCommand] {
        &self.commands
    }
}

impl Default for RenderCommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Command encoder for building render commands fluently
///
/// Example:
/// ```rust
/// let mut encoder = CommandEncoder::new();
/// encoder
///     .clear(Vec4::new(0.1, 0.1, 0.1, 1.0))
///     .draw_mesh(mesh_handle, transform, material)
///     .draw_grid(100.0, 100, Vec4::new(0.5, 0.5, 0.5, 1.0));
/// let commands = encoder.finish();
/// ```
pub struct CommandEncoder {
    commands: Vec<RenderCommand>,
}

impl CommandEncoder {
    /// Create a new command encoder
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(64),
        }
    }

    /// Clear the viewport
    pub fn clear(mut self, color: Vec4) -> Self {
        self.commands.push(RenderCommand::Clear { color });
        self
    }

    /// Set the camera
    pub fn set_camera(mut self, position: Vec3, target: Vec3, up: Vec3, fov: f32) -> Self {
        self.commands.push(RenderCommand::SetCamera {
            position,
            target,
            up,
            fov,
        });
        self
    }

    /// Set viewport rect
    pub fn set_viewport(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.commands.push(RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
        });
        self
    }

    /// Draw a mesh
    pub fn draw_mesh(
        mut self,
        mesh: Handle<Mesh>,
        transform: Mat4,
        material: Handle<PbrMaterial>,
    ) -> Self {
        self.commands.push(RenderCommand::DrawMesh {
            mesh,
            transform,
            material,
        });
        self
    }

    /// Draw a grid
    pub fn draw_grid(mut self, size: f32, divisions: u32, color: Vec4) -> Self {
        self.commands.push(RenderCommand::DrawGrid {
            size,
            divisions,
            color,
        });
        self
    }

    /// Draw a gizmo
    pub fn draw_gizmo(
        mut self,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        gizmo_type: GizmoType,
    ) -> Self {
        self.commands.push(RenderCommand::DrawGizmo {
            position,
            rotation,
            scale,
            gizmo_type,
        });
        self
    }

    /// Draw lines
    pub fn draw_lines(mut self, points: Vec<Vec3>, color: Vec4) -> Self {
        self.commands
            .push(RenderCommand::DrawLines { points, color });
        self
    }

    /// Draw text
    pub fn draw_text(mut self, text: String, position: Vec3, color: Vec4, size: f32) -> Self {
        self.commands.push(RenderCommand::DrawText {
            text,
            position,
            color,
            size,
        });
        self
    }

    /// Finish encoding and return the commands
    pub fn finish(self) -> Vec<RenderCommand> {
        self.commands
    }
}

impl Default for CommandEncoder {
    fn default() -> Self {
        Self::new()
    }
}
