//! Integration tests for Phase 1 Demo
//!
//! These tests validate that the demo application correctly integrates all Phase 1 features:
//! - Scene loading
//! - Physics simulation
//! - Rendering pipeline
//! - Audio playback
//!
//! **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**

use luminara::prelude::*;
use luminara::scene::GlobalTransform;
use std::path::Path;

/// Test that the demo scene loads without errors
/// **Validates: Requirements 10.1, 10.4**
#[test]
fn test_scene_loads_without_errors() {
    let scene_path = Path::new("assets/scenes/phase1_demo.scene.ron");

    // Skip test if file doesn't exist (CI environment)
    if !scene_path.exists() {
        eprintln!(
            "Skipping test: demo scene file not found at {:?}",
            scene_path
        );
        return;
    }

    // Load the scene
    let scene = Scene::load_from_file(scene_path).expect("Failed to load phase1_demo.scene.ron");

    // Verify scene loaded successfully
    assert_eq!(scene.meta.name, "Phase 1 Demo Scene");
    assert_eq!(
        scene.entities.len(),
        4,
        "Scene should have 4 entities (Camera, Sun, Ground, Sphere)"
    );

    // Verify all required entities are present
    let entity_names: Vec<&str> = scene.entities.iter().map(|e| e.name.as_str()).collect();
    assert!(
        entity_names.contains(&"Camera"),
        "Scene should contain Camera entity"
    );
    assert!(
        entity_names.contains(&"Sun"),
        "Scene should contain Sun entity"
    );
    assert!(
        entity_names.contains(&"Ground"),
        "Scene should contain Ground entity"
    );
    assert!(
        entity_names.contains(&"Sphere"),
        "Scene should contain Sphere entity"
    );

    // Verify Camera has required components
    let camera = scene.entities.iter().find(|e| e.name == "Camera").unwrap();
    assert!(
        camera.components.contains_key("Transform"),
        "Camera should have Transform"
    );
    assert!(
        camera.components.contains_key("Camera"),
        "Camera should have Camera component"
    );

    // Verify Sun has lighting component
    let sun = scene.entities.iter().find(|e| e.name == "Sun").unwrap();
    assert!(
        sun.components.contains_key("DirectionalLight"),
        "Sun should have DirectionalLight"
    );

    // Verify Ground has physics components
    let ground = scene.entities.iter().find(|e| e.name == "Ground").unwrap();
    assert!(
        ground.components.contains_key("Collider"),
        "Ground should have Collider"
    );
    assert!(
        ground.components.contains_key("RigidBody"),
        "Ground should have RigidBody"
    );

    // Verify Sphere has physics components
    let sphere = scene.entities.iter().find(|e| e.name == "Sphere").unwrap();
    assert!(
        sphere.components.contains_key("Collider"),
        "Sphere should have Collider"
    );
    assert!(
        sphere.components.contains_key("RigidBody"),
        "Sphere should have RigidBody"
    );
}

/// Test that the scene can be spawned into a world
/// **Validates: Requirements 10.1**
#[test]
fn test_scene_spawns_into_world() {
    let scene_path = Path::new("assets/scenes/phase1_demo.scene.ron");

    if !scene_path.exists() {
        eprintln!("Skipping test: demo scene file not found");
        return;
    }

    let scene = Scene::load_from_file(scene_path).expect("Failed to load scene");

    // Create a world and spawn the scene
    let mut world = World::new();
    let spawned_entities = scene.spawn_into(&mut world);

    // Verify entities were spawned
    assert_eq!(spawned_entities.len(), 4, "Should spawn 4 entities");

    // Verify entities have Transform components
    for &entity in &spawned_entities {
        assert!(
            world.get_component::<Transform>(entity).is_some(),
            "Spawned entity should have Transform component"
        );
    }
}

/// Test that physics system can be initialized with physics components
/// **Validates: Requirements 10.2**
#[test]
fn test_physics_simulation_runs() {
    let mut app = App::new();

    // Add only the plugins needed for physics testing
    app.add_plugins(luminara::physics::PhysicsPlugin);
    app.add_plugins(luminara::scene::ScenePlugin);

    // Verify physics world resource was created
    assert!(
        app.world
            .get_resource::<luminara::physics::PhysicsWorld3D>()
            .is_some(),
        "PhysicsWorld3D resource should be initialized"
    );

    // Create a simple physics scene
    let world = &mut app.world;

    // Create ground (static)
    let ground = world.spawn();
    world.add_component(
        ground,
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(ground, GlobalTransform::default());
    world.add_component(
        ground,
        RigidBody {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
        },
    );
    world.add_component(
        ground,
        Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(5.0, 0.25, 5.0),
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
    );

    // Create falling sphere (dynamic)
    let sphere = world.spawn();
    world.add_component(
        sphere,
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(sphere, GlobalTransform::default());
    world.add_component(
        sphere,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.1,
            angular_damping: 0.1,
            gravity_scale: 1.0,
        },
    );
    world.add_component(
        sphere,
        Collider {
            shape: ColliderShape::Sphere { radius: 0.5 },
            friction: 0.3,
            restitution: 0.7,
            is_sensor: false,
        },
    );

    // Verify components were added successfully
    assert!(
        world.get_component::<RigidBody>(ground).is_some(),
        "Ground should have RigidBody component"
    );
    assert!(
        world.get_component::<Collider>(ground).is_some(),
        "Ground should have Collider component"
    );
    assert!(
        world.get_component::<RigidBody>(sphere).is_some(),
        "Sphere should have RigidBody component"
    );
    assert!(
        world.get_component::<Collider>(sphere).is_some(),
        "Sphere should have Collider component"
    );

    // Verify rigid body types
    let ground_rb = world.get_component::<RigidBody>(ground).unwrap();
    assert!(
        matches!(ground_rb.body_type, RigidBodyType::Static),
        "Ground should be static"
    );

    let sphere_rb = world.get_component::<RigidBody>(sphere).unwrap();
    assert!(
        matches!(sphere_rb.body_type, RigidBodyType::Dynamic),
        "Sphere should be dynamic"
    );

    // Run one update to ensure systems don't panic
    app.update();
}

/// Test that rendering pipeline can be initialized
/// **Validates: Requirements 10.1, 10.3**
#[test]
fn test_rendering_pipeline_initializes() {
    // Note: Full rendering tests require a GPU context which may not be available in CI
    // This test verifies that the rendering components can be created

    let mut app = App::new();

    // Add scene plugin for transform system
    app.add_plugins(luminara::scene::ScenePlugin);

    let world = &mut app.world;

    // Create a camera entity
    let camera = world.spawn();
    world.add_component(
        camera,
        Transform {
            translation: Vec3::new(0.0, 5.0, 10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(camera, GlobalTransform::default());
    world.add_component(
        camera,
        Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: Color::rgb(0.1, 0.1, 0.15),
            is_active: true,
        },
    );
    world.add_component(camera, Camera3d);

    // Create a light entity
    let light = world.spawn();
    world.add_component(
        light,
        Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(std::f32::consts::PI / 4.0),
            scale: Vec3::ONE,
        },
    );
    world.add_component(light, GlobalTransform::default());
    world.add_component(
        light,
        luminara::render::DirectionalLight {
            color: Color::rgb(1.0, 0.95, 0.9),
            intensity: 3.0,
            cast_shadows: true,
            shadow_cascade_count: 4,
        },
    );

    // Create a mesh entity with PBR material
    let mesh_entity = world.spawn();
    world.add_component(
        mesh_entity,
        Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(mesh_entity, GlobalTransform::default());
    world.add_component(
        mesh_entity,
        luminara::render::PbrMaterial {
            albedo: Color::rgb(0.8, 0.1, 0.1),
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.9,
            roughness: 0.3,
            metallic_roughness_texture: None,
            emissive: Color::BLACK,
        },
    );

    // Verify components were added successfully
    assert!(
        world.get_component::<Camera>(camera).is_some(),
        "Camera component should be added"
    );
    assert!(
        world
            .get_component::<luminara::render::DirectionalLight>(light)
            .is_some(),
        "DirectionalLight component should be added"
    );
    assert!(
        world
            .get_component::<luminara::render::PbrMaterial>(mesh_entity)
            .is_some(),
        "PbrMaterial component should be added"
    );

    // Run one update to ensure systems don't panic
    app.update();
}

/// Test that audio system can be initialized
/// **Validates: Requirements 10.5**
#[test]
fn test_audio_system_initializes() {
    let mut app = App::new();

    // Add audio plugin
    app.add_plugins(luminara::audio::AudioPlugin);

    let world = &mut app.world;

    // Create an audio listener (typically on camera)
    let listener = world.spawn();
    world.add_component(
        listener,
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(listener, GlobalTransform::default());
    world.add_component(listener, AudioListener::default());

    // Create an audio source
    let audio_source = world.spawn();
    world.add_component(
        audio_source,
        Transform {
            translation: Vec3::new(5.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(audio_source, GlobalTransform::default());
    world.add_component(
        audio_source,
        AudioSource {
            clip: luminara::audio::AudioClipHandle("test_audio.ogg".to_string()),
            volume: 0.5,
            pitch: 1.0,
            looping: true,
            spatial: true,
            max_distance: 100.0,
        },
    );

    // Verify components were added
    assert!(
        world.get_component::<AudioListener>(listener).is_some(),
        "AudioListener component should be added"
    );
    assert!(
        world.get_component::<AudioSource>(audio_source).is_some(),
        "AudioSource component should be added"
    );

    // Run one update to ensure audio systems don't panic
    app.update();
}

/// Test that DefaultPlugins bundle includes all required plugins for the demo
/// **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**
#[test]
fn test_default_plugins_for_demo() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // Verify all plugins required for the demo are present
    assert!(
        app.has_plugin("ScenePlugin"),
        "ScenePlugin required for scene loading"
    );
    assert!(
        app.has_plugin("AssetPlugin"),
        "AssetPlugin required for asset loading"
    );
    assert!(
        app.has_plugin("RenderPlugin"),
        "RenderPlugin required for rendering"
    );
    assert!(
        app.has_plugin("PhysicsPlugin"),
        "PhysicsPlugin required for physics simulation"
    );
    assert!(
        app.has_plugin("AudioPlugin"),
        "AudioPlugin required for audio playback"
    );
    assert!(
        app.has_plugin("WindowPlugin"),
        "WindowPlugin required for display"
    );
}

/// Test that the demo can run for multiple frames without crashing
/// **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**
#[test]
fn test_demo_runs_multiple_frames() {
    let mut app = App::new();

    // Add core plugins (excluding window/render which need GPU)
    app.add_plugins(luminara::scene::ScenePlugin);
    app.add_plugins(luminara::physics::PhysicsPlugin);
    app.add_plugins(luminara::audio::AudioPlugin);

    // Create a minimal scene
    let world = &mut app.world;

    // Camera
    let camera = world.spawn();
    world.add_component(camera, Transform::default());
    world.add_component(camera, GlobalTransform::default());
    world.add_component(
        camera,
        Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: Color::rgb(0.1, 0.1, 0.15),
            is_active: true,
        },
    );

    // Physics object
    let sphere = world.spawn();
    world.add_component(
        sphere,
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(sphere, GlobalTransform::default());
    world.add_component(
        sphere,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.1,
            angular_damping: 0.1,
            gravity_scale: 1.0,
        },
    );
    world.add_component(
        sphere,
        Collider {
            shape: ColliderShape::Sphere { radius: 0.5 },
            friction: 0.3,
            restitution: 0.7,
            is_sensor: false,
        },
    );

    // Run for 60 frames (1 second at 60 FPS)
    for frame in 0..60 {
        app.update();

        // Verify sphere still exists and has valid transform
        let transform = app
            .world
            .get_component::<Transform>(sphere)
            .expect(&format!("Sphere should exist at frame {}", frame));

        // Verify no NaN values
        assert!(
            transform.translation.is_finite(),
            "Transform translation should be finite at frame {}",
            frame
        );
        assert!(
            transform.rotation.is_finite(),
            "Transform rotation should be finite at frame {}",
            frame
        );
    }
}
