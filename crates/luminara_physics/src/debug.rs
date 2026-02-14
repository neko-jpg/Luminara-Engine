use luminara_core::shared_types::{Resource, ResMut, Res, Query};
use luminara_render::command::CommandBuffer;
use luminara_render::GizmoType;
use luminara_render::command::DrawCommand;
use luminara_math::Color;
use crate::components::{Collider, ColliderShape}; // Assuming these exist
use crate::PhysicsWorld3D;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsDebugConfig {
    pub enabled: bool,
    pub collider_color: Color,
    pub sleeping_body_color: Color,
}

impl Default for PhysicsDebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            collider_color: Color::rgb(0.0, 1.0, 0.0),
            sleeping_body_color: Color::rgb(0.5, 0.5, 0.5),
        }
    }
}

impl Resource for PhysicsDebugConfig {}

pub fn physics_debug_render_system(
    config: Res<PhysicsDebugConfig>,
    _physics_world: Res<PhysicsWorld3D>,
    mut command_buffer: ResMut<CommandBuffer>,
    query: Query<(&Collider, &luminara_math::Transform)>,
) {
    if !config.enabled {
        return;
    }

    for (collider, transform) in query.iter() {
        let color = config.collider_color;

        let gizmo = match &collider.shape {
            ColliderShape::Box { half_extents } => {
                GizmoType::Box { half_extents: [half_extents.x, half_extents.y, half_extents.z] }
            },
            ColliderShape::Sphere { radius } => {
                GizmoType::Sphere { radius: *radius }
            },
            ColliderShape::Capsule { radius, half_height } => {
                GizmoType::Capsule { radius: *radius, height: *half_height * 2.0 }
            },
            _ => continue, // Mesh debug not implemented yet for simplicity
        };

        command_buffer.push(DrawCommand::DrawGizmo {
            gizmo,
            transform: transform.compute_matrix(),
            color,
        });
    }
}
