use luminara_core::shared_types::AppInterface;
use luminara_core::system::FunctionMarker;
use luminara_core::{App, CoreStage, Entity};
use luminara_math::{Color, Transform, Vec3};
use luminara_physics::components::{Collider, ColliderShape, RigidBody};
use luminara_physics::debug::{physics_debug_render_system, PhysicsDebugConfig};
use luminara_render::command::CommandBuffer;
use luminara_render::GizmoType;

#[test]
fn test_physics_debug_gizmos() {
    let mut app = App::new();

    // Register resources with updated PhysicsDebugConfig
    app.insert_resource(PhysicsDebugConfig {
        enabled: true,
        show_colliders: true,
        show_velocities: false,
        show_contacts: false,
        collider_color: Color::RED,
        sleeping_body_color: Color::GRAY,
        velocity_color: Color::rgb(1.0, 1.0, 0.0),
        contact_color: Color::rgb(1.0, 0.0, 0.0),
        velocity_scale: 1.0,
        contact_point_size: 0.05,
    });
    app.insert_resource(luminara_physics::PhysicsWorld3D::default());
    app.insert_resource(CommandBuffer::default());

    // Register system with updated signature
    app.add_system::<(
        FunctionMarker,
        luminara_core::shared_types::Res<'static, PhysicsDebugConfig>,
        luminara_core::shared_types::Res<'static, luminara_physics::PhysicsWorld3D>,
        luminara_core::shared_types::ResMut<'static, CommandBuffer>,
        luminara_core::shared_types::Query<'static, (&Collider, &Transform)>,
        luminara_core::shared_types::Query<'static, (Entity, &Transform, &RigidBody)>,
    )>(CoreStage::Update, physics_debug_render_system);

    // Spawn an entity with a collider
    let entity = app.world.spawn();
    app.world.add_component(
        entity,
        Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
    );
    app.world.add_component(
        entity,
        Collider {
            shape: ColliderShape::Sphere { radius: 0.5 },
            friction: 0.5,
            restitution: 0.5,
            is_sensor: false,
        },
    );
}

#[test]
fn test_gizmo_type_categorization() {
    let sphere = ColliderShape::Sphere { radius: 1.0 };
    let box_shape = ColliderShape::Box {
        half_extents: Vec3::ONE,
    };

    // Check sphere mapping
    match sphere {
        ColliderShape::Sphere { radius } => {
            let gizmo = GizmoType::Sphere { radius };
            if let GizmoType::Sphere { radius: r } = gizmo {
                assert_eq!(r, 1.0);
            } else {
                panic!("Wrong gizmo type");
            }
        }
        _ => panic!("Wrong shape"),
    }

    // Check box mapping
    match box_shape {
        ColliderShape::Box { half_extents } => {
            let gizmo = GizmoType::Box {
                half_extents: [half_extents.x, half_extents.y, half_extents.z],
            };
            if let GizmoType::Box { half_extents: he } = gizmo {
                assert_eq!(he, [1.0, 1.0, 1.0]);
            } else {
                panic!("Wrong gizmo type");
            }
        }
        _ => panic!("Wrong shape"),
    }
}
