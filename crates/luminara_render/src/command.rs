use crate::{Mesh, PbrMaterial};
use luminara_asset::Handle;
use luminara_math::{Color, Mat4};

#[derive(Debug, Clone)]
pub enum GizmoType {
    Line { start: [f32; 3], end: [f32; 3] },
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    Capsule { radius: f32, height: f32 },
    Arrow { start: [f32; 3], end: [f32; 3] },
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawMesh {
        mesh: Handle<Mesh>,
        material: Handle<PbrMaterial>,
        transform: Mat4,
    },
    DrawInstanced {
        mesh: Handle<Mesh>,
        material: Handle<PbrMaterial>,
        transforms: Vec<Mat4>,
    },
    DrawGizmo {
        gizmo: GizmoType,
        transform: Mat4,
        color: Color,
    },
    DrawParticles {
        system_id: u64, // Placeholder
        count: u32,
    },
}

use luminara_core::shared_types::Resource;

#[derive(Default)]
pub struct CommandBuffer {
    pub commands: Vec<DrawCommand>,
}

impl Resource for CommandBuffer {}

impl CommandBuffer {
    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
