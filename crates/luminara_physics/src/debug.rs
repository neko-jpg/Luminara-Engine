use crate::components::{Collider, ColliderShape, RigidBody};
use crate::PhysicsWorld3D;
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_core::Entity;
use luminara_math::{Color, Mat4, Vec3};
use luminara_render::command::CommandBuffer;
use luminara_render::command::DrawCommand;
use luminara_render::GizmoType;

/// Configuration for physics debug visualization
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsDebugConfig {
    pub enabled: bool,
    pub show_colliders: bool,
    pub show_velocities: bool,
    pub show_contacts: bool,
    pub collider_color: Color,
    pub sleeping_body_color: Color,
    pub velocity_color: Color,
    pub contact_color: Color,
    pub velocity_scale: f32,
    pub contact_point_size: f32,
}

impl Default for PhysicsDebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_colliders: true,
            show_velocities: true,
            show_contacts: true,
            collider_color: Color::rgb(0.0, 1.0, 0.0),
            sleeping_body_color: Color::rgb(0.5, 0.5, 0.5),
            velocity_color: Color::rgb(1.0, 1.0, 0.0),
            contact_color: Color::rgb(1.0, 0.0, 0.0),
            velocity_scale: 1.0,
            contact_point_size: 0.05,
        }
    }
}

impl Resource for PhysicsDebugConfig {}

/// System to render physics debug visualization
/// Renders:
/// - Collider shapes (wireframe)
/// - Velocity vectors
/// - Contact points
pub fn physics_debug_render_system(
    config: Res<PhysicsDebugConfig>,
    physics_world: Res<PhysicsWorld3D>,
    mut command_buffer: ResMut<CommandBuffer>,
    collider_query: Query<(&Collider, &luminara_math::Transform)>,
    body_query: Query<(Entity, &luminara_math::Transform, &RigidBody)>,
) {
    if !config.enabled {
        return;
    }

    // Render collider shapes (wireframe)
    if config.show_colliders {
        for (collider, transform) in collider_query.iter() {
            let color = config.collider_color;

            let gizmo = match &collider.shape {
                ColliderShape::Box { half_extents } => GizmoType::Box {
                    half_extents: [half_extents.x, half_extents.y, half_extents.z],
                },
                ColliderShape::Sphere { radius } => GizmoType::Sphere { radius: *radius },
                ColliderShape::Capsule {
                    radius,
                    half_height,
                } => GizmoType::Capsule {
                    radius: *radius,
                    height: *half_height * 2.0,
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

    // Render velocity vectors
    if config.show_velocities {
        for (entity, transform, rigid_body) in body_query.iter() {
            // Only show velocities for dynamic bodies
            if rigid_body.body_type != crate::components::RigidBodyType::Dynamic {
                continue;
            }

            // Get the rigid body from Rapier
            if let Some(&body_handle) = physics_world.entity_to_body.get(&entity) {
                if let Some(rapier_body) = physics_world.rigid_body_set.get(body_handle) {
                    let linear_vel = rapier_body.linvel();
                    let linear_vel_vec = Vec3::new(linear_vel.x, linear_vel.y, linear_vel.z);
                    
                    // Skip if velocity is negligible
                    if linear_vel_vec.length() < 0.01 {
                        continue;
                    }

                    let start = transform.translation;
                    let end = start + linear_vel_vec * config.velocity_scale;

                    // Draw main velocity line using Gizmo
                    command_buffer.push(DrawCommand::DrawGizmo {
                        gizmo: GizmoType::Line {
                            start: [start.x, start.y, start.z],
                            end: [end.x, end.y, end.z],
                        },
                        transform: Mat4::IDENTITY,
                        color: config.velocity_color,
                    });

                    // Draw arrowhead
                    let direction = (end - start).normalize();
                    let arrow_size = 0.1;
                    
                    // Create perpendicular vectors for arrowhead
                    let up = if direction.y.abs() < 0.9 {
                        Vec3::new(0.0, 1.0, 0.0)
                    } else {
                        Vec3::new(1.0, 0.0, 0.0)
                    };
                    let right = direction.cross(up).normalize();
                    let up = right.cross(direction).normalize();

                    // Draw arrowhead lines
                    let arrow_base = end - direction * arrow_size;
                    let arrow_left = arrow_base + right * arrow_size * 0.5;
                    let arrow_right = arrow_base - right * arrow_size * 0.5;
                    let arrow_up = arrow_base + up * arrow_size * 0.5;
                    let arrow_down = arrow_base - up * arrow_size * 0.5;

                    command_buffer.push(DrawCommand::DrawGizmo {
                        gizmo: GizmoType::Line {
                            start: [end.x, end.y, end.z],
                            end: [arrow_left.x, arrow_left.y, arrow_left.z],
                        },
                        transform: Mat4::IDENTITY,
                        color: config.velocity_color,
                    });
                    command_buffer.push(DrawCommand::DrawGizmo {
                        gizmo: GizmoType::Line {
                            start: [end.x, end.y, end.z],
                            end: [arrow_right.x, arrow_right.y, arrow_right.z],
                        },
                        transform: Mat4::IDENTITY,
                        color: config.velocity_color,
                    });
                    command_buffer.push(DrawCommand::DrawGizmo {
                        gizmo: GizmoType::Line {
                            start: [end.x, end.y, end.z],
                            end: [arrow_up.x, arrow_up.y, arrow_up.z],
                        },
                        transform: Mat4::IDENTITY,
                        color: config.velocity_color,
                    });
                    command_buffer.push(DrawCommand::DrawGizmo {
                        gizmo: GizmoType::Line {
                            start: [end.x, end.y, end.z],
                            end: [arrow_down.x, arrow_down.y, arrow_down.z],
                        },
                        transform: Mat4::IDENTITY,
                        color: config.velocity_color,
                    });
                }
            }
        }
    }

    // Render contact points
    if config.show_contacts {
        // Iterate through all contact pairs in the narrow phase
        for contact_pair in physics_world.narrow_phase.contact_pairs() {
            if !contact_pair.has_any_active_contact {
                continue;
            }

            // Get all manifolds for this contact pair
            for manifold in contact_pair.manifolds.iter() {
                // Iterate through all contact points in the manifold
                for contact_point in manifold.points.iter() {
                    // Get the contact point position in world space
                    let point = contact_point.local_p1;
                    
                    // Transform to world space using the first collider's position
                    if let Some(collider) = physics_world.collider_set.get(contact_pair.collider1) {
                        let world_point = collider.position() * point;
                        let pos = Vec3::new(
                            world_point.x,
                            world_point.y,
                            world_point.z,
                        );

                        // Draw contact point as a small sphere
                        command_buffer.push(DrawCommand::DrawGizmo {
                            gizmo: GizmoType::Sphere {
                                radius: config.contact_point_size,
                            },
                            transform: Mat4::from_translation(pos),
                            color: config.contact_color,
                        });

                        // Draw contact normal
                        let normal = manifold.local_n1;
                        let normal_vec = Vec3::new(normal.x, normal.y, normal.z);
                        let normal_end = pos + normal_vec * 0.2;

                        command_buffer.push(DrawCommand::DrawGizmo {
                            gizmo: GizmoType::Line {
                                start: [pos.x, pos.y, pos.z],
                                end: [normal_end.x, normal_end.y, normal_end.z],
                            },
                            transform: Mat4::IDENTITY,
                            color: config.contact_color,
                        });
                    }
                }
            }
        }
    }
}
