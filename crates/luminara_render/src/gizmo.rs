use crate::command::{DrawCommand, CommandBuffer, GizmoType};
use luminara_math::{Mat4, Color, Vec3};

pub struct Gizmos;

impl Gizmos {
    pub fn line(buffer: &mut CommandBuffer, start: Vec3, end: Vec3, color: Color) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Line {
                start: [start.x, start.y, start.z],
                end: [end.x, end.y, end.z]
            },
            transform: Mat4::IDENTITY,
            color,
        });
    }

    pub fn sphere(buffer: &mut CommandBuffer, position: Vec3, radius: f32, color: Color) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Sphere { radius },
            transform: Mat4::from_translation(position),
            color,
        });
    }

    pub fn cube(buffer: &mut CommandBuffer, position: Vec3, half_extents: Vec3, color: Color) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Box {
                half_extents: [half_extents.x, half_extents.y, half_extents.z]
            },
            transform: Mat4::from_translation(position),
            color,
        });
    }

    // Add more shapes as needed
}
