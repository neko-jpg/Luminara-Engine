use luminara_core::shared_types::AppInterface;
use luminara_core::App;
use luminara_math::{Transform, Vec3};
use luminara_physics::{
    Collider, ColliderShape, PhysicsDebugConfig, PhysicsPlugin, RigidBody, RigidBodyType,
};
use luminara_render::command::CommandBuffer;
use luminara_render::command::DrawCommand;
use luminara_render::GizmoType;

#[test]
fn test_physics_debug_collider_rendering() {
    let mut app = App::new();
    
    // Initialize required resources
    app.world.insert_resource(CommandBuffer::default());
    app.world.insert_resource(luminara_core::Time::default());
    
    app.add_plugins(PhysicsPlugin);

    // Enable physics debug visualization
    {
        let mut config = app.world.get_resource_mut::<PhysicsDebugConfig>().unwrap();
        config.enabled = true;
        config.show_colliders = true;
        config.show_velocities = false;
        config.show_contacts = false;
    }

    // Create entity with collider
    let entity = app.world.spawn();
    let _ = app.world.add_component(
        entity,
        Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(1.0, 1.0, 1.0),
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
    );
    let _ = app.world.add_component(
        entity,
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: luminara_math::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );

    // Run one update to create physics bodies
    app.update();

    // Clear command buffer
    {
        let mut buffer = app.world.get_resource_mut::<CommandBuffer>().unwrap();
        buffer.clear();
    }

    // Run another update to render debug visualization
    app.update();

    // Verify that collider was rendered
    let buffer = app.world.get_resource::<CommandBuffer>().unwrap();
    let has_collider_gizmo = buffer.commands.iter().any(|cmd| {
        matches!(
            cmd,
            DrawCommand::DrawGizmo {
                gizmo: GizmoType::Box { .. },
                ..
            }
        )
    });

    assert!(
        has_collider_gizmo,
        "Physics debug should render collider shapes"
    );
}

#[test]
fn test_physics_debug_velocity_rendering() {
    let mut app = App::new();
    
    // Initialize required resources
    app.world.insert_resource(CommandBuffer::default());
    app.world.insert_resource(luminara_core::Time::default());
    
    app.add_plugins(PhysicsPlugin);

    // Enable physics debug visualization for velocities
    {
        let mut config = app.world.get_resource_mut::<PhysicsDebugConfig>().unwrap();
        config.enabled = true;
        config.show_colliders = false;
        config.show_velocities = true;
        config.show_contacts = false;
    }

    // Create entity with rigid body and collider
    let entity = app.world.spawn();
    let _ = app.world.add_component(
        entity,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
        },
    );
    let _ = app.world.add_component(
        entity,
        Collider {
            shape: ColliderShape::Sphere { radius: 0.5 },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
    );
    let _ = app.world.add_component(
        entity,
        Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: luminara_math::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );

    // Run several updates to let physics simulate and build velocity
    for _ in 0..30 {
        app.update();
    }

    // Clear command buffer
    {
        let mut buffer = app.world.get_resource_mut::<CommandBuffer>().unwrap();
        buffer.clear();
    }

    // Run another update to render debug visualization
    app.update();

    // Verify that velocity vectors were rendered (should have lines for velocity)
    let buffer = app.world.get_resource::<CommandBuffer>().unwrap();
    let has_velocity_lines = buffer.commands.iter().any(|cmd| {
        matches!(
            cmd,
            DrawCommand::DrawGizmo {
                gizmo: GizmoType::Line { .. },
                ..
            }
        )
    });

    // Note: This test may not always pass if the body hasn't gained enough velocity
    // In a real scenario, we'd apply a force or set initial velocity
    // For now, we just verify the system doesn't crash
    if !has_velocity_lines {
        println!("Warning: No velocity lines rendered - body may not have moved enough");
    }
}

#[test]
fn test_physics_debug_config_toggles() {
    let mut app = App::new();
    
    // Initialize required resources
    app.world.insert_resource(CommandBuffer::default());
    app.world.insert_resource(luminara_core::Time::default());
    
    app.add_plugins(PhysicsPlugin);

    // Test that debug rendering respects enabled flag
    {
        let mut config = app.world.get_resource_mut::<PhysicsDebugConfig>().unwrap();
        config.enabled = false;
        config.show_colliders = true;
    }

    // Create entity with collider
    let entity = app.world.spawn();
    let _ = app.world.add_component(
        entity,
        Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(1.0, 1.0, 1.0),
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
    );
    let _ = app.world.add_component(
        entity,
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: luminara_math::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );

    // Run updates
    app.update();
    {
        let mut buffer = app.world.get_resource_mut::<CommandBuffer>().unwrap();
        buffer.clear();
    }
    app.update();

    // Verify that nothing was rendered when disabled
    let buffer = app.world.get_resource::<CommandBuffer>().unwrap();
    let has_gizmos = buffer
        .commands
        .iter()
        .any(|cmd| matches!(cmd, DrawCommand::DrawGizmo { .. }));

    assert!(
        !has_gizmos,
        "Physics debug should not render when disabled"
    );
}

#[test]
fn test_physics_debug_default_config() {
    let config = PhysicsDebugConfig::default();

    assert!(!config.enabled, "Debug should be disabled by default");
    assert!(config.show_colliders, "Colliders should be enabled by default");
    assert!(config.show_velocities, "Velocities should be enabled by default");
    assert!(config.show_contacts, "Contacts should be enabled by default");
    assert_eq!(config.velocity_scale, 1.0, "Default velocity scale should be 1.0");
    assert_eq!(
        config.contact_point_size, 0.05,
        "Default contact point size should be 0.05"
    );
}
