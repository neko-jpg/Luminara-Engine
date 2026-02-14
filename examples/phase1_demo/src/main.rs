//! # Luminara Engine — Phase 1 Demo
//!
//! This demo showcases the Phase 1 core engine features:
//! - Scene loading from RON file
//! - 3D PBR rendering with lighting and shadows
//! - Physics simulation with collision detection
//! - Transform hierarchy system
//! - Asset pipeline with hot-reload support
//! - Audio playback (background music)
//!
//! **Requirements validated:** 10.1, 10.2, 10.3, 10.4, 10.5

use log::info;
use luminara::prelude::*;
use luminara::scene::TypeRegistry;
use luminara::asset::AssetServer;

mod camera_controller;
use camera_controller::{CameraController, camera_controller_system, setup_camera_input, CameraAction};

fn main() {
    // Initialize the engine with all default plugins
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // Setup input actions
    app.add_startup_system::<ExclusiveMarker>(setup_camera_input);

    // Add camera controller system
    app.add_system::<(
        luminara::core::system::FunctionMarker,
        ResMut<'static, Input>,
        Res<'static, luminara::input::ActionMap<CameraAction>>,
        Res<'static, luminara::core::Time>,
        Query<'static, (&mut Transform, &mut CameraController)>,
    )>(CoreStage::Update, camera_controller_system);

    // Register component types for scene deserialization
    // This allows the scene loader to automatically deserialize these components
    let mut registry = TypeRegistry::new();
    registry.register::<Camera>();
    registry.register::<luminara::render::DirectionalLight>();
    registry.register::<luminara::render::PbrMaterial>();
    registry.register::<Collider>();
    registry.register::<RigidBody>();
    registry.register::<AudioListener>();
    // Transform is handled specially/fallback, but good to register if we want full uniformity
    registry.register::<Transform>();

    app.insert_resource(registry);

    // Add startup system to load the demo scene
    app.add_startup_system::<ExclusiveMarker>(setup_demo);

    // Run the application
    info!("Starting Phase 1 Demo...");
    app.run();
}

/// Startup system that loads the Phase 1 demo scene
fn setup_demo(world: &mut World) {
    info!("Loading Phase 1 demo scene...");

    // Load the demo scene from the asset pipeline
    let scene_path = std::path::Path::new("assets/scenes/phase1_demo.scene.ron");

    match luminara::scene::Scene::load_from_file(scene_path) {
        Ok(scene) => {
            info!("Scene loaded successfully: {}", scene.meta.name);
            info!("Scene description: {}", scene.meta.description);
            info!("Scene version: {}", scene.meta.version);
            info!("Entity count: {}", scene.entities.len());

            // Spawn all entities from the scene into the world
            // The TypeRegistry resource in the world will be used to deserialize components automatically
            let spawned_entities = scene.spawn_into(world);
            info!("Spawned {} entities from scene", spawned_entities.len());

            // Post-process: Attach resources that cannot be easily serialized in JSON/RON (like Meshes)
            // or require marker components not in the scene file.
            world.resource_scope::<AssetServer, _, _>(|world, asset_server| {
                attach_runtime_assets(world, &scene, &asset_server);
            });

            // Add background audio if available
            add_background_audio(world);

            info!("Phase 1 demo setup complete!");
            info!("Expected behavior:");
            info!("  - Camera positioned at (0, 5, 10) looking at the scene");
            info!("  - Directional light casting shadows from above");
            info!("  - Ground plane (static collider) at y=0");
            info!("  - Red metallic sphere falling due to gravity");
            info!("  - Sphere should collide with ground and bounce");
        }
        Err(e) => {
            log::error!("Failed to load demo scene: {}", e);
            log::error!("Make sure assets/scenes/phase1_demo.scene.ron exists");

            // Fallback: Create a minimal scene manually
            world.resource_scope::<AssetServer, _, _>(|world, asset_server| {
                create_fallback_scene(world, &asset_server);
            });
        }
    }
}

/// Attach runtime assets (Meshes) and markers that are not serialized
fn attach_runtime_assets(world: &mut World, scene: &luminara::scene::Scene, asset_server: &AssetServer) {
    use luminara::render::Mesh;
    use luminara::scene::find_entity_by_name;

    for entity_data in &scene.entities {
        if let Some(entity) = find_entity_by_name(world, &entity_data.name) {
            // Add Camera3d marker and CameraController if it has a Camera
            if entity_data.components.contains_key("Camera") {
                world.add_component(entity, Camera3d);
                world.add_component(entity, CameraController::default());
            }

            // Add meshes based on entity name (since we don't have a Mesh asset loader from JSON yet)
            match entity_data.name.as_str() {
                "Sphere" => {
                    let mesh = Mesh::sphere(0.5, 32);
                    let handle = asset_server.add(mesh);
                    world.add_component(entity, handle);
                    info!("Added sphere mesh to {}", entity_data.name);
                }
                "Ground" => {
                    let mesh = Mesh::cube(1.0); // Will be scaled by transform
                    let handle = asset_server.add(mesh);
                    world.add_component(entity, handle);
                    info!("Added cube mesh to {}", entity_data.name);
                }
                _ => {}
            }
        }
    }
}

/// Add background audio to the scene (optional)
fn add_background_audio(world: &mut World) {
    // Note: For a real demo, you would need an actual audio file in assets/audio/
    // For now, we create a placeholder audio source that demonstrates the API

    // Check if we have an audio file available
    let audio_path = "assets/audio/background_music.ogg";

    if std::path::Path::new(audio_path).exists() {
        let audio_entity = world.spawn();
        world.add_component(
            audio_entity,
            AudioSource {
                clip: luminara::audio::AudioClipHandle(audio_path.to_string()),
                volume: 0.5,
                pitch: 1.0,
                looping: true,
                spatial: false,
                max_distance: 100.0,
            },
        );
        world.add_component(audio_entity, Name::new("BackgroundMusic"));
        info!("Added background music audio source");
    } else {
        info!("No background music file found at {}", audio_path);
        info!("To add audio, place an audio file (WAV, OGG, MP3, FLAC) at that path");
    }
}

/// Fallback scene creation if the scene file cannot be loaded
fn create_fallback_scene(world: &mut World, asset_server: &AssetServer) {
    use luminara::render::Mesh;

    info!("Creating fallback scene...");

    // Create camera
    let camera = world.spawn();
    world.add_component(camera, Name::new("Camera"));
    world.add_component(
        camera,
        Transform {
            translation: Vec3::new(0.0, 5.0, 10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
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
    world.add_component(camera, CameraController::default());
    world.add_component(camera, AudioListener::default());

    // Create a simple sphere to show something is working
    let sphere = world.spawn();
    world.add_component(sphere, Name::new("FallbackSphere"));
    world.add_component(
        sphere,
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );

    let mesh = Mesh::sphere(1.0, 32);
    let handle = asset_server.add(mesh);
    world.add_component(sphere, handle);

    world.add_component(
        sphere,
        luminara::render::PbrMaterial {
            albedo: Color::rgb(0.8, 0.2, 0.2),
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.5,
            roughness: 0.5,
            metallic_roughness_texture: None,
            emissive: Color::BLACK,
        },
    );

    info!("Fallback scene created with camera and sphere");
}
