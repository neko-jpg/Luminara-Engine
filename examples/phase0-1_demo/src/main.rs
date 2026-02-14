//! # Luminara Engine — Ultimate Demo
//!
//! A fully interactive 3D physics playground demonstrating **every** engine feature.
//!
//! ## Controls (shown in-app HUD):
//! - **WASD / Arrows**: Move camera
//! - **Mouse (RMB)**: Look around
//! - **Shift**: Sprint (3x speed)
//! - **Space / Ctrl**: Move up / down
//! - **C**: Toggle camera mode (1st / 3rd person)
//! - **G**: Toggle debug gizmos (cycle categories)
//! - **P**: Pause / Resume physics
//! - **T**: Spawn random physics object
//! - **1-5**: Spawn specific shapes
//! - **E**: Trigger explosion at crosshair
//! - **F**: Toggle target-shooting mode
//! - **Click (LMB)**: Shoot target (when target mode active)
//! - **V**: Cycle gravity direction (Down/Right/Up/Left)
//! - **+/-**: Increase/Decrease gravity scale
//! - **B**: Toggle slow-motion (0.25x)
//! - **X**: Clear all spawned objects
//! - **R**: Reset camera & scene
//! - **H**: Toggle HUD command menu
//! - **Esc**: Free cursor / Exit

use luminara::prelude::*;
use luminara::asset::AssetServer;
use luminara_core::{CoreStage, ExclusiveMarker};
use luminara_input::keyboard::Key;
use luminara_input::mouse::MouseButton;
use luminara_render::{
    CommandBuffer, DirectionalLight, GizmoCategories, Gizmos, OverlayRenderer,
    ParticleEmitter, PbrMaterial, PointLight, Texture,
};
use luminara_audio::{AudioSource, AudioClipHandle};
use luminara_physics::camera_shake::CameraShake;
use luminara_physics::explosion::Explosion;
use luminara_physics::target_game::draw_crosshair;
use luminara_physics::{Target, TargetGameState, CollisionEvents, physics3d::PhysicsWorld3D};
use rapier3d::prelude::{nalgebra, point, vector, Ray, QueryFilter};
use log::{error, info};
use std::time::Instant;

mod camera_controller;
mod post_effects;
mod scene_manager;
mod lod_system;
mod mouse_interaction;
mod console;
mod advanced_effects;
mod spectacle_scenes;

use camera_controller::{
    camera_controller_system, setup_camera_input, CameraAction, CameraController,
};
use luminara::input::ActionMap;
use post_effects::PostEffects;
use scene_manager::SceneManager;
use lod_system::lod_update_system;
use mouse_interaction::{MouseInteractionState, mouse_interaction_system};
use console::{Console, console_input_system, console_render_system};
use advanced_effects::*;
use spectacle_scenes::*;

// ============================================================================
// Resources
// ============================================================================

/// Tracks the demo's interactive state.
#[derive(Debug)]
struct DemoState {
    /// List of entities dynamically spawned by the player.
    spawned_entities: Vec<Entity>,
    /// Total frames elapsed.
    frame_count: u64,
    /// Whether gizmos are active.
    gizmos_on: bool,
    /// Active gizmo category index (for cycling with G).
    gizmo_category_index: usize,
    /// Whether physics is paused.
    physics_paused: bool,
    /// Whether the console command menu is visible.
    menu_visible: bool,
    /// "cooldown" counter to prevent key repeat spam.
    toggle_cooldown: u32,
    /// Spawn counter for unique names.
    spawn_counter: u32,
    /// FPS tracking.
    last_fps_time: Instant,
    fps_frame_count: u32,
    current_fps: f32,
    /// Slow-motion active flag.
    slow_motion: bool,
    /// Target shooting mode.
    target_mode: bool,
    /// Gravity direction index for cycling.
    gravity_index: usize,
    /// Gravity scale multiplier.
    gravity_scale: f32,
    /// Explosion counter for naming.
    explosion_counter: u32,
    /// Notification messages shown on screen.
    notifications: Vec<(String, f32, [f32; 4])>, // (message, remaining_time, color)
    /// Scene ambient time for animated effects.
    scene_time: f32,
    /// Post-processing effects configuration.
    post_effects: PostEffects,
    /// Time scale control (for advanced slow-motion).
    #[allow(dead_code)]
    time_scale_target: f32,
}

impl Resource for DemoState {}

impl DemoState {
    fn new() -> Self {
        Self {
            spawned_entities: Vec::new(),
            frame_count: 0,
            gizmos_on: true,
            gizmo_category_index: 0,
            physics_paused: false,
            menu_visible: true,
            toggle_cooldown: 0,
            spawn_counter: 0,
            last_fps_time: Instant::now(),
            fps_frame_count: 0,
            current_fps: 0.0,
            slow_motion: false,
            target_mode: false,
            gravity_index: 0,
            gravity_scale: 1.0,
            explosion_counter: 0,
            notifications: Vec::new(),
            scene_time: 0.0,
            post_effects: PostEffects::new(),
            time_scale_target: 1.0,
        }
    }

    fn update_fps(&mut self) {
        self.fps_frame_count += 1;
        let elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.fps_frame_count as f32 / elapsed;
            self.fps_frame_count = 0;
            self.last_fps_time = Instant::now();
        }
    }

    fn push_notification(&mut self, msg: String, color: [f32; 4]) {
        self.notifications.push((msg, 3.0, color));
    }

    fn update_notifications(&mut self, dt: f32) {
        for n in &mut self.notifications {
            n.1 -= dt;
        }
        self.notifications.retain(|n| n.1 > 0.0);
    }
}

// ============================================================================
// Entry Point
// ============================================================================

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .init();

    print_startup_banner();

    let mut app = App::new();

    // Configure window — larger for a showcase demo
    app.insert_resource(WindowDescriptor {
        title: "Luminara Engine — Ultimate Demo v2".into(),
        width: 1920,
        height: 1080,
        mode: luminara_window::WindowMode::Windowed,
        vsync: true,
        resizable: true,
    });

    // Register all engine plugins
    app.add_plugins(DefaultPlugins);
    app.add_plugins(luminara_render::AnimationPlugin);

    // Insert demo resources
    app.insert_resource(DemoState::new());
    app.insert_resource(TargetGameState::new());
    app.insert_resource(GizmoCategories::new());
    app.insert_resource(SceneManager::new());
    app.insert_resource(PostEffects::new());
    app.insert_resource(MouseInteractionState::default());
    app.insert_resource(Console::default());

    // Register startup systems
    app.add_startup_system::<ExclusiveMarker>(setup_scene);
    app.add_startup_system::<ExclusiveMarker>(setup_camera_input);

    // Core interaction systems
    app.add_system::<ExclusiveMarker>(CoreStage::Update, input_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, target_game_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, gizmo_draw_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, hud_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, animate_energy_cores_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, collision_sound_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, lod_update_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, post_effects_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, mouse_interaction_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, console_input_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, console_render_system);
    
    // Advanced effects systems
    app.add_system::<ExclusiveMarker>(CoreStage::Update, rotating_platform_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, pendulum_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, orbital_motion_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, pulsating_light_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, trail_effect_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, magnetic_field_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, chain_reaction_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, domino_system);

    // Camera controller (function system)
    app.add_system::<(
        luminara::core::system::FunctionMarker,
        ResMut<'static, Input>,
        Res<'static, ActionMap<CameraAction>>,
        Res<'static, luminara::core::Time>,
        Query<'static, (&mut Transform, &mut CameraController)>,
    )>(CoreStage::Update, camera_controller_system);

    info!("Starting Ultimate Demo v2 — press [H] to toggle command menu");
    app.run();
}

// ============================================================================
// Startup Banner (console)
// ============================================================================

fn print_startup_banner() {
    println!();
    println!("================================================================");
    println!("  LUMINARA ENGINE — ULTIMATE DEMO v2.5 SPECTACLE EDITION       ");
    println!("  Interactive 3D Physics Playground with Spectacular Effects   ");
    println!("================================================================");
    println!();
    println!("  CAMERA");
    println!("    [W/A/S/D]  Move              [Mouse RMB]  Look");
    println!("    [Space/Q]  Up/Down            [C]          Toggle Mode");
    println!();
    println!("  ACTIONS");
    println!("    [T]        Spawn random       [E]          Explosion");
    println!("    [1-5]      Spawn shapes       [F]          Target mode");
    println!("    [LMB]      Shoot target       [Y]          Spawn target");
    println!("    [B]        Slow-motion        [X]          Clear objects");
    println!("    [R]        Reset scene        [V]          Cycle gravity");
    println!("    [+/-]      Gravity scale");
    println!();
    println!("  SPECTACLE SCENES (NEW!)");
    println!("    [D]        Domino Chain       [N]          Pendulum Array");
    println!("    [M]        Rotating Platforms [O]          Orbital System");
    println!("    [K]        Magnetic Field Demo");
    println!();
    println!("  POST-EFFECTS");
    println!("    [7]        Toggle Bloom       [8]          Toggle DOF");
    println!("    [[/]]      Exposure adjust    [,/.]        Prev/Next Scene");
    println!();
    println!("  TOGGLES");
    println!("    [G]        Cycle gizmos        [P]         Pause physics");
    println!("    [H]        Toggle HUD          [Esc]       Free/Exit");
    println!();
    println!("  FEATURES");
    println!("    • PBR Materials with Textures");
    println!("    • Post-Processing (Bloom, DOF, Tone Mapping)");
    println!("    • Particle Systems (Fountain, Fire, Explosions, Trails)");
    println!("    • Target Shooting with Score System");
    println!("    • Multiple Scenes (Physics Lab, Chaos Arena, Target Range)");
    println!("    • LOD System for Performance");
    println!("    • Advanced Camera Controller");
    println!("    • Real-time Profiling");
    println!("    • Spectacular Physics Demonstrations:");
    println!("      - 100-piece Domino Chain with spiral path");
    println!("      - 15-pendulum Newton's Cradle array");
    println!("      - Rotating platform maze");
    println!("      - Orbital planetary system");
    println!("      - Magnetic field interactions");
    println!();
    println!("================================================================");
    println!();
}

fn print_command_menu() {
    println!();
    println!("================================================================");
    println!("  COMMAND MENU  (H to hide)                                     ");
    println!("  [W/A/S/D] Move  [C] Mode  [T] Spawn  [1-5] Shapes           ");
    println!("  [E] Explode  [F] Targets  [B] SlowMo  [X] Clear  [R] Reset  ");
    println!("  [G] Gizmos  [P] Pause  [V] Gravity Dir  [+/-] Gravity Scale ");
    println!("  [Esc] Exit                                                   ");
    println!("================================================================");
    println!();
}

// ============================================================================
// Scene Setup
// ============================================================================

fn setup_scene(world: &mut World) {
    info!("Setting up demo scene...");

    // Load base scene from RON (Camera, Sun, Ground)
    let scene_path = std::path::Path::new("assets/scenes/ultimate.scene.ron");
    match luminara_scene::Scene::load_from_file(scene_path) {
        Ok(scene) => {
            info!("Loaded base scene from RON");
            scene.spawn_into(world);
        }
        Err(e) => {
            error!("Failed to load scene: {}. Using fallback setup.", e);

            // ── Camera ──────────────────────────────────────────────────
            let camera = world.spawn();
            world.add_component(
                camera,
                Camera {
                    projection: Projection::Perspective {
                        fov: 60.0,
                        near: 0.1,
                        far: 1000.0,
                    },
                    clear_color: Color::rgba(0.02, 0.03, 0.06, 1.0),
                    is_active: true,
                },
            );
            world.add_component(camera, Camera3d);
            world.add_component(camera, CameraController::default());
            world.add_component(
                camera,
                CameraShake {
                    intensity: 0.0,
                    frequency: 12.0,
                    decay: 6.0,
                    elapsed: 0.0,
                    seed: 42,
                },
            );
            world.add_component(
                camera,
                Transform::from_xyz(0.0, 12.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
            );
            world.add_component(camera, Name::new("MainCamera"));
            world.add_component(camera, AudioListener::default());

            // ── Directional light (sun) ─────────────────────────────────
            let sun = world.spawn();
            world.add_component(sun, Name::new("Sun"));
            world.add_component(
                sun,
                Transform {
                    translation: Vec3::new(0.0, 50.0, 0.0),
                    rotation: Quat::from_rotation_x(-0.8) * Quat::from_rotation_y(0.4),
                    scale: Vec3::ONE,
                },
            );
            world.add_component(
                sun,
                DirectionalLight {
                    color: Color::rgb(1.0, 0.95, 0.85),
                    intensity: 1.4,
                    cast_shadows: true,
                    shadow_cascade_count: 4,
                },
            );

            // ── Ground ──────────────────────────────────────────────────
            create_ground(world);
        }
    }

    // ── Fill lights (multiple for richer scene) ────────────────────────
    let fill_lights = [
        ("FillLight_Blue", Vec3::new(-15.0, 8.0, 15.0), Color::rgb(0.4, 0.5, 1.0), 0.6),
        ("FillLight_Warm", Vec3::new(15.0, 6.0, -10.0), Color::rgb(1.0, 0.7, 0.4), 0.5),
        ("FillLight_Green", Vec3::new(0.0, 15.0, 0.0), Color::rgb(0.3, 0.8, 0.3), 0.3),
    ];
    for (name, pos, color, intensity) in &fill_lights {
        let fill = world.spawn();
        world.add_component(fill, Name::new(*name));
        world.add_component(fill, Transform::from_xyz(pos.x, pos.y, pos.z));
        world.add_component(
            fill,
            PointLight {
                color: *color,
                intensity: *intensity,
                range: 60.0,
                cast_shadows: false,
            },
        );
    }
    info!("  * 3 fill lights");

    // ── Walls ───────────────────────────────────────────────────────────
    create_walls(world);

    // ── Demo objects ────────────────────────────────────────────────────
    create_demo_objects(world);

    // ── Shooting targets ────────────────────────────────────────────────
    create_targets(world);

    // ── HUD indicator objects (floating colored markers) ────────────────
    create_hud_markers(world);

    // ── Decorative arches ───────────────────────────────────────────────
    create_arches(world);

    // ── Particle fountain ───────────────────────────────────────────────
    create_particle_emitters(world);

    // ── Dynamic colored lights for atmosphere ───────────────────────────
    create_dynamic_lights(world);

    // ── Background music ────────────────────────────────────────────────
    let bgm_entity = world.spawn();
    world.add_component(bgm_entity, Name::new("BGM"));
    world.add_component(
        bgm_entity,
        AudioSource {
            clip: AudioClipHandle("bgm_loop".to_string()),
            volume: 0.3,
            pitch: 1.0,
            looping: true,
            spatial: false,
            max_distance: 100.0,
        },
    );

    info!("Scene ready — {} entities total", world.entities().len());
    info!("Press [H] to toggle the command menu");
}

// ── Ground ──────────────────────────────────────────────────────────────────

fn create_ground(world: &mut World) {
    let ground = world.spawn();
    world.add_component(ground, Name::new("Ground"));
    world.add_component(
        ground,
        Transform {
            translation: Vec3::new(0.0, -0.5, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(60.0, 1.0, 60.0),
        },
    );
    // Use AssetServer to add a cube mesh
    let ground_assets = world.get_resource::<AssetServer>().map(|asset_server| {
        let mesh_handle = asset_server.add(Mesh::cube(1.0));
        let concrete_texture = asset_server.load::<Texture>("assets/textures/concrete_albedo.png");
        let normal_texture = asset_server.load::<Texture>("assets/textures/concrete_normal.png");
        let roughness_texture = asset_server.load::<Texture>("assets/textures/concrete_roughness.png");
        (mesh_handle, concrete_texture, normal_texture, roughness_texture)
    });
    if let Some((mesh_handle, concrete_texture, normal_texture, roughness_texture)) = ground_assets {
        world.add_component(ground, mesh_handle);
        world.add_component(
            ground,
            PbrMaterial {
                albedo: Color::rgb(0.4, 0.4, 0.45),
                albedo_texture: Some(concrete_texture),
                normal_texture: Some(normal_texture),
                metallic: 0.0,
                roughness: 0.9,
                metallic_roughness_texture: Some(roughness_texture),
                emissive: Color::BLACK,
            },
        );
    } else {
        world.add_component(
            ground,
            PbrMaterial {
                albedo: Color::rgb(0.25, 0.27, 0.30),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.05,
                roughness: 0.95,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );
    }
    world.add_component(
        ground,
        RigidBody {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 0.0,
        },
    );
    world.add_component(
        ground,
        Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(30.0, 0.5, 30.0),
            },
            friction: 0.7,
            restitution: 0.1,
            is_sensor: false,
        },
    );
}

// ── Walls ───────────────────────────────────────────────────────────────────

fn create_walls(world: &mut World) {
    let walls = [
        ("Wall_N", Vec3::new(0.0, 4.0, 31.0), Vec3::new(62.0, 8.0, 1.0)),
        ("Wall_S", Vec3::new(0.0, 4.0, -31.0), Vec3::new(62.0, 8.0, 1.0)),
        ("Wall_E", Vec3::new(31.0, 4.0, 0.0), Vec3::new(1.0, 8.0, 62.0)),
        ("Wall_W", Vec3::new(-31.0, 4.0, 0.0), Vec3::new(1.0, 8.0, 62.0)),
    ];
    for (name, pos, s) in &walls {
        let e = world.spawn();
        world.add_component(e, Name::new(*name));
        world.add_component(
            e,
            Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: *s,
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
        world.add_component(
            e,
            PbrMaterial {
                albedo: Color::rgb(0.35, 0.35, 0.40),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.1,
                roughness: 0.85,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );
        world.add_component(
            e,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
            },
        );
        world.add_component(
            e,
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(s.x * 0.5, s.y * 0.5, s.z * 0.5),
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        );
    }
    info!("  * 4 boundary walls");
}

// ── Demo Objects ────────────────────────────────────────────────────────────

fn create_demo_objects(world: &mut World) {
    // Sphere tower (left side)
    for row in 0..4 {
        for col in 0..3 {
            spawn_sphere_at(
                world,
                Vec3::new(col as f32 * 2.5 - 2.5, 1.5 + row as f32 * 2.2, -12.0),
                1.0,
                Color::rgb(0.85, 0.25, 0.15),
            );
        }
    }
    info!("  * Sphere tower (12 spheres)");

    // Cube pyramid (center-right)
    for layer in 0..4u32 {
        let n = 4 - layer;
        for x in 0..n {
            for z in 0..n {
                spawn_cube_at(
                    world,
                    Vec3::new(
                        (x as f32 - n as f32 * 0.5 + 0.5) * 2.2 + 10.0,
                        1.0 + layer as f32 * 2.1,
                        (z as f32 - n as f32 * 0.5 + 0.5) * 2.2,
                    ),
                    1.8,
                    Color::rgb(0.2, 0.55, 0.85),
                );
            }
        }
    }
    info!("  * Cube pyramid (30 cubes)");

    // Glowing energy cores in a ring
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        spawn_emissive_orb(
            world,
            Vec3::new(a.cos() * 14.0, 2.5, a.sin() * 14.0),
            0.7,
            Color::rgb(0.0, 2.0, 3.5),
        );
    }
    info!("  * 6 glowing energy cores");

    // Decorative pillars at corners
    let pillar_positions = [
        Vec3::new(-20.0, 3.0, -20.0),
        Vec3::new(20.0, 3.0, -20.0),
        Vec3::new(-20.0, 3.0, 20.0),
        Vec3::new(20.0, 3.0, 20.0),
    ];
    for pos in &pillar_positions {
        let e = world.spawn();
        world.add_component(e, Name::new("Pillar"));
        world.add_component(
            e,
            Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.5, 6.0, 1.5),
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
        world.add_component(
            e,
            PbrMaterial {
                albedo: Color::rgb(0.7, 0.65, 0.55),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.3,
                roughness: 0.6,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );
        world.add_component(
            e,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
            },
        );
        world.add_component(
            e,
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(0.75, 3.0, 0.75),
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        );
    }
    info!("  * 4 corner pillars");
}

// ── HUD Markers (floating colored objects that add visual interest) ─────────

fn create_hud_markers(world: &mut World) {
    let markers = [
        (Vec3::new(-5.0, 8.0, -5.0), Color::rgb(1.0, 0.3, 0.3), "Marker_Red"),
        (Vec3::new(5.0, 9.0, -5.0), Color::rgb(0.3, 1.0, 0.3), "Marker_Green"),
        (Vec3::new(0.0, 10.0, -8.0), Color::rgb(0.3, 0.3, 1.0), "Marker_Blue"),
    ];
    for (pos, color, name) in &markers {
        let e = world.spawn();
        world.add_component(e, Name::new(*name));
        world.add_component(
            e,
            Transform {
                translation: *pos,
                rotation: Quat::from_rotation_y(0.785),
                            scale: Vec3::splat(0.5),
                        },
                    );
                        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
                    
                    world.add_component(
                        e,
                        PbrMaterial {
                
                albedo: *color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.8,
                roughness: 0.15,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.5, color.g * 0.5, color.b * 0.5),
            },
        );
        world.add_component(
            e,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
            },
        );
    }
    info!("  * 3 floating HUD markers");
}

// ── Shooting Targets ────────────────────────────────────────────────────────

fn create_targets(world: &mut World) {
    let target_configs = [
        (Vec3::new(-18.0, 3.0, -25.0), 10, Color::rgb(1.0, 0.2, 0.2)),
        (Vec3::new(-12.0, 5.0, -25.0), 20, Color::rgb(1.0, 0.5, 0.1)),
        (Vec3::new(-6.0, 4.0, -25.0), 15, Color::rgb(1.0, 0.8, 0.0)),
        (Vec3::new(0.0, 6.0, -25.0), 50, Color::rgb(0.2, 1.0, 0.2)),     // center = high value
        (Vec3::new(6.0, 4.0, -25.0), 15, Color::rgb(0.0, 0.8, 1.0)),
        (Vec3::new(12.0, 5.0, -25.0), 20, Color::rgb(0.5, 0.3, 1.0)),
        (Vec3::new(18.0, 3.0, -25.0), 10, Color::rgb(1.0, 0.3, 0.7)),
        // Elevated row
        (Vec3::new(-9.0, 9.0, -25.0), 30, Color::rgb(1.0, 1.0, 0.2)),
        (Vec3::new(0.0, 11.0, -25.0), 100, Color::rgb(1.0, 0.0, 0.0)),    // golden target
        (Vec3::new(9.0, 9.0, -25.0), 30, Color::rgb(0.2, 1.0, 1.0)),
    ];
    for (pos, points, color) in &target_configs {
        let e = world.spawn();
        world.add_component(e, Name::new("Target"));
        let target_scale = if *points >= 50 { 1.2 } else { 0.8 };
        world.add_component(
            e,
            Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(target_scale),
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::sphere(0.5, 16)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
        world.add_component(
            e,
            PbrMaterial {
                albedo: *color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.8,
                roughness: 0.15,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.8, color.g * 0.8, color.b * 0.8),
            },
        );
        world.add_component(
            e,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
            },
        );
        world.add_component(
            e,
            Collider {
                shape: ColliderShape::Sphere { radius: target_scale * 0.5 },
                friction: 0.3,
                restitution: 0.0,
                is_sensor: false,
            },
        );
        world.add_component(
            e,
            Target {
                points: *points,
                hit: false,
                hit_flash_timer: 0.0,
                original_color: *color,
            },
        );
    }
    info!("  * 10 shooting targets");
}

// ── Decorative Arches ───────────────────────────────────────────────────────

fn create_arches(world: &mut World) {
    // Stone arches along walkways
    let arch_positions = [
        (Vec3::new(0.0, 0.0, -15.0), 0.0f32),
        (Vec3::new(0.0, 0.0, 0.0), 0.0),
        (Vec3::new(0.0, 0.0, 15.0), 0.0),
        (Vec3::new(-15.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(15.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2),
    ];
    for (base_pos, y_rot) in &arch_positions {
        // Left pillar
        let lp = world.spawn();
        world.add_component(lp, Name::new("ArchPillarL"));
        world.add_component(
            lp,
            Transform {
                translation: *base_pos + Quat::from_rotation_y(*y_rot) * Vec3::new(-3.0, 4.0, 0.0),
                rotation: Quat::from_rotation_y(*y_rot),
                scale: Vec3::new(0.8, 8.0, 0.8),
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(lp, h); }
        world.add_component(
            lp,
            PbrMaterial {
                albedo: Color::rgb(0.55, 0.50, 0.45),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.15,
                roughness: 0.85,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );
        world.add_component(
            lp,
            RigidBody { body_type: RigidBodyType::Static, mass: 0.0, linear_damping: 0.0, angular_damping: 0.0, gravity_scale: 0.0 },
        );
        world.add_component(
            lp,
            Collider { shape: ColliderShape::Box { half_extents: Vec3::new(0.4, 4.0, 0.4) }, friction: 0.5, restitution: 0.0, is_sensor: false },
        );

        // Right pillar
        let rp = world.spawn();
        world.add_component(rp, Name::new("ArchPillarR"));
        world.add_component(
            rp,
            Transform {
                translation: *base_pos + Quat::from_rotation_y(*y_rot) * Vec3::new(3.0, 4.0, 0.0),
                rotation: Quat::from_rotation_y(*y_rot),
                scale: Vec3::new(0.8, 8.0, 0.8),
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(rp, h); }
        world.add_component(rp, PbrMaterial {
            albedo: Color::rgb(0.55, 0.50, 0.45), albedo_texture: None, normal_texture: None,
            metallic: 0.15, roughness: 0.85, metallic_roughness_texture: None, emissive: Color::BLACK,
        });
        world.add_component(rp, RigidBody { body_type: RigidBodyType::Static, mass: 0.0, linear_damping: 0.0, angular_damping: 0.0, gravity_scale: 0.0 });
        world.add_component(rp, Collider { shape: ColliderShape::Box { half_extents: Vec3::new(0.4, 4.0, 0.4) }, friction: 0.5, restitution: 0.0, is_sensor: false });

        // Cross-beam
        let beam = world.spawn();
        world.add_component(beam, Name::new("ArchBeam"));
        world.add_component(
            beam,
            Transform {
                translation: *base_pos + Vec3::new(0.0, 8.5, 0.0),
                rotation: Quat::from_rotation_y(*y_rot),
                scale: Vec3::new(7.0, 0.8, 0.8),
            },
        );
        let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(beam, h); }
        world.add_component(beam, PbrMaterial {
            albedo: Color::rgb(0.6, 0.55, 0.48), albedo_texture: None, normal_texture: None,
            metallic: 0.2, roughness: 0.8, metallic_roughness_texture: None, emissive: Color::BLACK,
        });
        world.add_component(beam, RigidBody { body_type: RigidBodyType::Static, mass: 0.0, linear_damping: 0.0, angular_damping: 0.0, gravity_scale: 0.0 });
        world.add_component(beam, Collider { shape: ColliderShape::Box { half_extents: Vec3::new(3.5, 0.4, 0.4) }, friction: 0.5, restitution: 0.0, is_sensor: false });
    }
    info!("  * 5 decorative arches");
}

// ── Particle Emitters ───────────────────────────────────────────────────────

fn create_particle_emitters(world: &mut World) {
    // Central fountain
    let fountain = world.spawn();
    world.add_component(fountain, Name::new("ParticleFountain"));
    world.add_component(fountain, Transform::from_xyz(0.0, 0.5, 0.0));
    world.add_component(
        fountain,
        ParticleEmitter {
            rate: 30.0,
            accumulator: 0.0,
            direction: Vec3::Y,
            spread: 0.3,
            speed: 8.0,
            color: Color::rgb(0.3, 0.7, 1.0),
            size: 0.08,
            lifetime: 2.5,
        },
    );

    // Fire jets at corners of the arena
    let fire_positions = [
        Vec3::new(-25.0, 0.0, -25.0),
        Vec3::new(25.0, 0.0, -25.0),
        Vec3::new(-25.0, 0.0, 25.0),
        Vec3::new(25.0, 0.0, 25.0),
    ];
    for pos in &fire_positions {
        let fire = world.spawn();
        world.add_component(fire, Name::new("FireJet"));
        world.add_component(fire, Transform::from_xyz(pos.x, 0.5, pos.z));
        world.add_component(
            fire,
            ParticleEmitter {
                rate: 15.0,
                accumulator: 0.0,
                direction: Vec3::Y,
                spread: 0.15,
                speed: 6.0,
                color: Color::rgb(1.0, 0.4, 0.05),
                size: 0.12,
                lifetime: 1.8,
            },
        );
    }
    info!("  * 5 particle emitters (fountain + 4 fire jets)");
}

// ── Dynamic Lights ──────────────────────────────────────────────────────────

fn create_dynamic_lights(world: &mut World) {
    // Colored accent lights around the arena
    let accent_configs = [
        ("AccentLight_Red", Vec3::new(-20.0, 4.0, 0.0), Color::rgb(1.0, 0.1, 0.1)),
        ("AccentLight_Blue", Vec3::new(20.0, 4.0, 0.0), Color::rgb(0.1, 0.3, 1.0)),
        ("AccentLight_Purple", Vec3::new(0.0, 4.0, 20.0), Color::rgb(0.7, 0.1, 1.0)),
        ("AccentLight_Cyan", Vec3::new(0.0, 4.0, -20.0), Color::rgb(0.1, 0.9, 0.9)),
        ("AccentLight_Gold", Vec3::new(0.0, 12.0, -25.0), Color::rgb(1.0, 0.85, 0.3)),
    ];
    for (name, pos, color) in &accent_configs {
        let e = world.spawn();
        world.add_component(e, Name::new(*name));
        world.add_component(e, Transform::from_xyz(pos.x, pos.y, pos.z));
        world.add_component(
            e,
            PointLight {
                color: *color,
                intensity: 0.8,
                range: 25.0,
                cast_shadows: false,
            },
        );
    }
    info!("  * 5 dynamic accent lights");
}

// ============================================================================
// Spawn Helpers
// ============================================================================

fn add_spawned_entity(world: &mut World, entity: Entity) {
    if let Some(state) = world.get_resource_mut::<DemoState>() {
        state.spawned_entities.push(entity);
        state.spawn_counter += 1;
    }
}

fn spawn_sphere_at(world: &mut World, pos: Vec3, radius: f32, color: Color) -> Entity {
    let e = world.spawn();
    world.add_component(e, Name::new("Sphere"));
    world.add_component(
        e,
        Transform {
            translation: pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(radius * 2.0),
        },
    );
    let sphere_assets = world.get_resource::<AssetServer>().map(|asset_server| {
        let mesh_handle = asset_server.add(Mesh::sphere(0.5, 24));
        let metal_texture = asset_server.load::<Texture>("assets/textures/metal_albedo.png");
        let normal_texture = asset_server.load::<Texture>("assets/textures/metal_normal.png");
        let metallic_texture = asset_server.load::<Texture>("assets/textures/metal_metallic.png");
        (mesh_handle, metal_texture, normal_texture, metallic_texture)
    });
    if let Some((mesh_handle, metal_texture, normal_texture, metallic_texture)) = sphere_assets {
        world.add_component(e, mesh_handle);
        world.add_component(
            e,
            PbrMaterial {
                albedo: color,
                albedo_texture: Some(metal_texture),
                normal_texture: Some(normal_texture),
                metallic: 0.9,
                roughness: 0.2,
                metallic_roughness_texture: Some(metallic_texture),
                emissive: Color::BLACK,
            },
        );
    } else {
        world.add_component(
            e,
            PbrMaterial {
                albedo: color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.7,
                roughness: 0.25,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );
    }
    world.add_component(
        e,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.1,
            angular_damping: 0.1,
            gravity_scale: 1.0,
        },
    );
    world.add_component(
        e,
        Collider {
            shape: ColliderShape::Sphere { radius },
            friction: 0.5,
            restitution: 0.4,
            is_sensor: false,
        },
    );
    e
}

fn spawn_target_at(world: &mut World, pos: Vec3, radius: f32, points: u32) -> Entity {
    let e = spawn_sphere_at(world, pos, radius, Color::rgb(1.0, 0.2, 0.2));
    world.add_component(e, Name::new("Target"));
    world.add_component(
        e,
        Target {
            points,
            hit: false,
            hit_flash_timer: 0.0,
            original_color: Color::rgb(1.0, 0.2, 0.2),
        },
    );
    e
}

fn spawn_cube_at(world: &mut World, pos: Vec3, size: f32, color: Color) -> Entity {
    let e = world.spawn();
    world.add_component(e, Name::new("Cube"));
    world.add_component(
        e,
        Transform {
            translation: pos,
            rotation: Quat::from_rotation_y(0.3),
            scale: Vec3::splat(size),
        },
    );
    let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::cube(1.0)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
    world.add_component(
        e,
        PbrMaterial {
            albedo: color,
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.4,
            roughness: 0.5,
            metallic_roughness_texture: None,
            emissive: Color::BLACK,
        },
    );
    world.add_component(
        e,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.1,
            angular_damping: 0.1,
            gravity_scale: 1.0,
        },
    );
    world.add_component(
        e,
        Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::splat(size * 0.5),
            },
            friction: 0.5,
            restitution: 0.2,
            is_sensor: false,
        },
    );
    e
}

fn spawn_emissive_orb(world: &mut World, pos: Vec3, radius: f32, glow: Color) -> Entity {
    let e = world.spawn();
    world.add_component(e, Name::new("EnergyCore"));
    world.add_component(
        e,
        Transform {
            translation: pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(radius * 2.0),
        },
    );
    let mesh_h = world.get_resource::<AssetServer>().map(|a| a.add(Mesh::sphere(0.5, 24)));
        if let Some(h) = mesh_h { world.add_component(e, h); }
    world.add_component(
        e,
        PbrMaterial {
            albedo: Color::rgb(0.05, 0.05, 0.05),
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.0,
            roughness: 0.1,
            metallic_roughness_texture: None,
            emissive: glow,
        },
    );
    world.add_component(
        e,
        RigidBody {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 0.0,
        },
    );
    world.add_component(
        e,
        Collider {
            shape: ColliderShape::Sphere { radius },
            friction: 0.3,
            restitution: 0.0,
            is_sensor: false,
        },
    );
    e
}

// ============================================================================
// Input System — reads keyboard and performs actions
// ============================================================================

/// Gizmo category names for cycling.
const GIZMO_CATEGORIES: &[&str] = &[
    "all", "physics", "collision", "camera", "bounds", "skeleton", "navigation",
];

const GRAVITY_DIRECTIONS: &[Vec3] = &[
    Vec3::new(0.0, -9.8, 0.0), // Down
    Vec3::new(9.8, 0.0, 0.0),  // Right
    Vec3::new(0.0, 9.8, 0.0),  // Up
    Vec3::new(-9.8, 0.0, 0.0), // Left
];

fn input_system(world: &mut World) {
    // Read input first, then get mutable demo state
    let mut new_physics_paused = None;
    let mut wants_spawn_random = false;
    let mut wants_spawn_sphere = false;
    let mut wants_spawn_cube = false;
    let mut wants_spawn_glow = false;
    let mut wants_spawn_stack = false;
    let mut wants_spawn_rain = false;
    let mut wants_clear = false;
    let mut wants_reset = false;
    let mut wants_toggle_gizmos = false;
    let mut wants_toggle_physics = false;
    let mut wants_toggle_menu = false;
    let mut wants_explosion = false;
    let mut wants_slow_motion = false;
    let mut wants_target_mode = false;
    let mut wants_shoot = false;
    let mut wants_spawn_target = false;
    let mut wants_cycle_gravity = false;
    let mut wants_increase_gravity = false;
    let mut wants_decrease_gravity = false;
    let mut wants_exit = false;
    let mut wants_toggle_bloom = false;
    let mut wants_toggle_dof = false;
    let mut wants_increase_exposure = false;
    let mut wants_decrease_exposure = false;
    let mut wants_next_scene = false;
    let mut wants_previous_scene = false;
    let mut wants_spawn_domino_chain = false;
    let mut wants_spawn_pendulum_array = false;
    let mut wants_spawn_rotating_platforms = false;
    let mut wants_spawn_orbital_system = false;
    let mut wants_spawn_magnetic_demo = false;

    // Read input resource
    if let Some(input) = world.get_resource_mut::<Input>() {
        // Actions (just_pressed = single fire)
        wants_spawn_random = input.just_pressed(Key::T);
        wants_spawn_sphere = input.just_pressed(Key::Num1);
        wants_spawn_cube = input.just_pressed(Key::Num2);
        wants_spawn_glow = input.just_pressed(Key::Num3);
        wants_spawn_stack = input.just_pressed(Key::Num4);
        wants_spawn_rain = input.just_pressed(Key::Num5);
        wants_clear = input.just_pressed(Key::X);
        wants_reset = input.just_pressed(Key::R);
        wants_toggle_gizmos = input.just_pressed(Key::G);
        wants_toggle_physics = input.just_pressed(Key::P);
        wants_toggle_menu = input.just_pressed(Key::H);
        wants_explosion = input.just_pressed(Key::E);
        wants_slow_motion = input.just_pressed(Key::B);
        wants_target_mode = input.just_pressed(Key::F);
        wants_spawn_target = input.just_pressed(Key::Y);
        wants_cycle_gravity = input.just_pressed(Key::V);
        wants_increase_gravity = input.just_pressed(Key::Equal); // + key
        wants_decrease_gravity = input.just_pressed(Key::Minus); // - key
        wants_shoot = input.mouse_just_pressed(MouseButton::Left);
        wants_toggle_bloom = input.just_pressed(Key::Num7);
        wants_toggle_dof = input.just_pressed(Key::Num8);
        wants_increase_exposure = input.just_pressed(Key::BracketRight); // ] key
        wants_decrease_exposure = input.just_pressed(Key::BracketLeft); // [ key
        wants_next_scene = input.just_pressed(Key::Period); // > key
        wants_previous_scene = input.just_pressed(Key::Comma); // < key
        
        // Spectacle scene spawning (Num keys with Shift)
        wants_spawn_domino_chain = input.just_pressed(Key::D);
        wants_spawn_pendulum_array = input.just_pressed(Key::N);
        wants_spawn_rotating_platforms = input.just_pressed(Key::M);
        wants_spawn_orbital_system = input.just_pressed(Key::O);
        wants_spawn_magnetic_demo = input.just_pressed(Key::K);

        if input.just_pressed(Key::Escape) {
            if input.is_cursor_grabbed() {
                input.set_cursor_grabbed(false);
                input.set_cursor_visible(true);
            } else {
                wants_exit = true;
            }
        }
    }

    // Get delta time for notifications
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    // Now mutate DemoState
    if let Some(state) = world.get_resource_mut::<DemoState>() {
        state.frame_count += 1;
        state.scene_time += dt;
        state.update_fps();
        state.update_notifications(dt);

        // Decrease cooldown
        if state.toggle_cooldown > 0 {
            state.toggle_cooldown -= 1;
        }

        // ── Gizmo cycling ───────────────────────────────────────────
        if wants_toggle_gizmos && state.toggle_cooldown == 0 {
            state.gizmo_category_index = (state.gizmo_category_index + 1) % (GIZMO_CATEGORIES.len() + 1);
            if state.gizmo_category_index == 0 {
                state.gizmos_on = false;
                state.push_notification("Gizmos: OFF".into(), [1.0, 0.5, 0.5, 1.0]);
            } else {
                state.gizmos_on = true;
                let cat = GIZMO_CATEGORIES[state.gizmo_category_index - 1];
                state.push_notification(format!("Gizmos: {} only", cat), [0.5, 1.0, 0.8, 1.0]);
            }
            state.toggle_cooldown = 10;
        }

        // ── Physics pause ───────────────────────────────────────────
        if wants_toggle_physics && state.toggle_cooldown == 0 {
            state.physics_paused = !state.physics_paused;
            state.toggle_cooldown = 10;
            new_physics_paused = Some(state.physics_paused);
            state.push_notification(
                if state.physics_paused { "Physics: PAUSED".into() } else { "Physics: RUNNING".into() },
                [0.5, 0.8, 1.0, 1.0],
            );
        }

        // ── Menu toggle ─────────────────────────────────────────────
        if wants_toggle_menu && state.toggle_cooldown == 0 {
            state.menu_visible = !state.menu_visible;
            state.toggle_cooldown = 10;
            if state.menu_visible {
                print_command_menu();
            }
        }

        // ── Slow motion toggle ──────────────────────────────────────
        if wants_slow_motion && state.toggle_cooldown == 0 {
            state.slow_motion = !state.slow_motion;
            state.toggle_cooldown = 10;
            state.push_notification(
                if state.slow_motion { "SLOW MOTION: ON (0.25x)".into() } else { "SLOW MOTION: OFF".into() },
                [1.0, 1.0, 0.3, 1.0],
            );
        }

        // ── Target mode toggle ──────────────────────────────────────
        if wants_target_mode && state.toggle_cooldown == 0 {
            state.target_mode = !state.target_mode;
            state.toggle_cooldown = 10;
            state.push_notification(
                if state.target_mode { "TARGET MODE: ON — [LMB] to shoot!".into() } else { "TARGET MODE: OFF".into() },
                [1.0, 0.4, 0.2, 1.0],
            );
        }

        // ── Cycle gravity ───────────────────────────────────────────
        if wants_cycle_gravity && state.toggle_cooldown == 0 {
            state.gravity_index = (state.gravity_index + 1) % GRAVITY_DIRECTIONS.len();
            state.toggle_cooldown = 10;
            let _dir = GRAVITY_DIRECTIONS[state.gravity_index];
            let dir_name = match state.gravity_index {
                0 => "DOWN",
                1 => "RIGHT",
                2 => "UP",
                3 => "LEFT",
                _ => "UNKNOWN",
            };
            state.push_notification(
                format!("Gravity: {}", dir_name),
                [0.5, 0.8, 1.0, 1.0],
            );
        }

        // ── Adjust gravity scale ────────────────────────────────────
        if (wants_increase_gravity || wants_decrease_gravity) && state.toggle_cooldown == 0 {
            if wants_increase_gravity {
                state.gravity_scale = (state.gravity_scale * 1.5).min(10.0);
            } else {
                state.gravity_scale = (state.gravity_scale / 1.5).max(0.1);
            }
            state.toggle_cooldown = 5;
            state.push_notification(
                format!("Gravity Scale: {:.1}x", state.gravity_scale),
                [0.8, 0.5, 1.0, 1.0],
            );
        }

        if wants_exit {
            info!("Exit requested");
            std::process::exit(0);
        }

        // ── Post-effects controls ───────────────────────────────────
        if wants_toggle_bloom && state.toggle_cooldown == 0 {
            state.post_effects.toggle_bloom();
            state.toggle_cooldown = 10;
            state.push_notification(
                format!("Bloom: {}", if state.post_effects.bloom_enabled { "ON" } else { "OFF" }),
                [1.0, 0.8, 0.3, 1.0],
            );
        }

        if wants_toggle_dof && state.toggle_cooldown == 0 {
            state.post_effects.toggle_dof();
            state.toggle_cooldown = 10;
            state.push_notification(
                format!("DOF: {}", if state.post_effects.dof_enabled { "ON" } else { "OFF" }),
                [0.8, 0.8, 1.0, 1.0],
            );
        }

        if (wants_increase_exposure || wants_decrease_exposure) && state.toggle_cooldown == 0 {
            let delta = if wants_increase_exposure { 0.1 } else { -0.1 };
            state.post_effects.adjust_exposure(delta);
            state.toggle_cooldown = 5;
            state.push_notification(
                format!("Exposure: {:.1}", state.post_effects.exposure),
                [1.0, 1.0, 0.5, 1.0],
            );
        }

        // ── Scene switching ─────────────────────────────────────────
        if (wants_next_scene || wants_previous_scene) && state.toggle_cooldown == 0 {
            state.toggle_cooldown = 20;
            // Scene switching will be handled below
        }
    }

    // Handle scene switching (needs mutable access to SceneManager)
    if wants_next_scene || wants_previous_scene {
        if let Some(scene_mgr) = world.get_resource_mut::<SceneManager>() {
            if wants_next_scene {
                scene_mgr.next_scene();
            } else {
                scene_mgr.previous_scene();
            }
            let scene_name = scene_mgr.current_scene().name.clone();
            
            if let Some(state) = world.get_resource_mut::<DemoState>() {
                state.push_notification(
                    format!("Scene: {}", scene_name),
                    [0.5, 1.0, 0.8, 1.0],
                );
            }
        }
    }

    // Now handle exit
    if wants_exit {
        info!("Exit requested");
        std::process::exit(0);
    }

    // ── Apply time scale ────────────────────────────────────────────
    {
        let paused = new_physics_paused;
        let slow_motion = world
            .get_resource::<DemoState>()
            .map(|s| s.slow_motion)
            .unwrap_or(false);

        if let Some(time) = world.get_resource_mut::<Time>() {
            if let Some(p) = paused {
                time.time_scale = if p { 0.0 } else if slow_motion { 0.25 } else { 1.0 };
            } else if slow_motion {
                time.time_scale = 0.25;
            } else {
                // Only reset if not paused
                let is_paused = world
                    .get_resource::<DemoState>()
                    .map(|s| s.physics_paused)
                    .unwrap_or(false);
                if !is_paused {
                    // re-borrow
                }
            }
        }
    }
    // Fix time_scale correctly without double-borrow
    {
        let (paused, slow) = world
            .get_resource::<DemoState>()
            .map(|s| (s.physics_paused, s.slow_motion))
            .unwrap_or((false, false));
        if let Some(time) = world.get_resource_mut::<Time>() {
            time.time_scale = if paused { 0.0 } else if slow { 0.25 } else { 1.0 };
        }
    }

    // ── Apply gravity ───────────────────────────────────────────────
    {
        let (gravity_index, gravity_scale) = world
            .get_resource::<DemoState>()
            .map(|s| (s.gravity_index, s.gravity_scale))
            .unwrap_or((0, 1.0));
        let gravity = GRAVITY_DIRECTIONS[gravity_index % GRAVITY_DIRECTIONS.len()] * gravity_scale;
        if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld3D>() {
            physics_world.gravity = vector![gravity.x, gravity.y, gravity.z];
        }
    }

    // ── Update gizmo categories ─────────────────────────────────────
    {
        let (gizmos_on, cat_idx) = world
            .get_resource::<DemoState>()
            .map(|s| (s.gizmos_on, s.gizmo_category_index))
            .unwrap_or((true, 0));
        if let Some(gizmo_cats) = world.get_resource_mut::<GizmoCategories>() {
            if !gizmos_on {
                gizmo_cats.disable_all();
            } else if cat_idx > 0 && cat_idx <= GIZMO_CATEGORIES.len() {
                let active_cat = GIZMO_CATEGORIES[cat_idx - 1];
                if active_cat == "all" {
                    gizmo_cats.enable_all();
                } else {
                    // Enable only the selected category
                    gizmo_cats.disable_all();
                    gizmo_cats.set_enabled(active_cat, true);
                    gizmo_cats.set_enabled("default", true);
                }
            }
        }
    }

    // ── Spawning ────────────────────────────────────────────────────
    let spawn_pos = {
        let mut sp = Vec3::new(0.0, 5.0, 0.0);
        let query = Query::<(&Transform, &CameraController)>::new(world);
        for (transform, controller) in query.iter() {
            let yaw = controller.yaw.to_radians();
            let pitch = controller.pitch.to_radians();
            let forward = Vec3::new(
                -yaw.sin() * pitch.cos(),
                pitch.sin(),
                -yaw.cos() * pitch.cos(),
            );
            sp = Vec3::new(
                transform.translation.x + forward.x * 5.0,
                transform.translation.y + forward.y * 5.0,
                transform.translation.z + forward.z * 5.0,
            );
            break;
        }
        sp
    };

    // ── Explosion trigger ───────────────────────────────────────────
    if wants_explosion {
        let explosion_entity = world.spawn();
        world.add_component(explosion_entity, Name::new("Explosion"));
        world.add_component(
            explosion_entity,
            Explosion {
                radius: 15.0,
                force: 2000.0,
                center: spawn_pos,
                processed: false,
            },
        );
        // Add particle effect for explosion
        world.add_component(
            explosion_entity,
            ParticleEmitter {
                rate: 100.0,
                accumulator: 0.0,
                direction: Vec3::Y,
                spread: 0.5,
                speed: 10.0,
                color: Color::rgb(1.0, 0.5, 0.0),
                size: 0.1,
                lifetime: 2.0,
            },
        );
        // Trigger camera shake
        {
            let mut query = Query::<&mut CameraShake>::new(world);
            for shake in query.iter_mut() {
                shake.intensity = 1.5;
                shake.elapsed = 0.0;
            }
        }
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.explosion_counter += 1;
            state.push_notification(
                format!("EXPLOSION #{} at ({:.0}, {:.0}, {:.0})!", state.explosion_counter, spawn_pos.x, spawn_pos.y, spawn_pos.z),
                [1.0, 0.3, 0.1, 1.0],
            );
        }
    }

    // ── Target shooting ─────────────────────────────────────────────
    if wants_shoot {
        let target_active = world
            .get_resource::<DemoState>()
            .map(|s| s.target_mode)
            .unwrap_or(false);
        if target_active {
            if let Some(tgs) = world.get_resource_mut::<TargetGameState>() {
                tgs.record_shot();
            }
            
            // Perform raycast for target shooting using camera position
            let (cam_pos, cam_dir) = {
                let mut pos = Vec3::ZERO;
                let mut dir = Vec3::NEG_Z;
                let query = Query::<(&Transform, &CameraController)>::new(world);
                for (transform, controller) in query.iter() {
                    let yaw = controller.yaw.to_radians();
                    let pitch = controller.pitch.to_radians();
                    dir = Vec3::new(
                        -yaw.sin() * pitch.cos(),
                        pitch.sin(),
                        -yaw.cos() * pitch.cos(),
                    ).normalize();
                    pos = Vec3::new(transform.translation.x, transform.translation.y, transform.translation.z);
                    break;
                }
                (pos, dir)
            };

            // Phase 1: Raycast to find hit entity (immutable borrow)
            let hit_result: Option<(Entity, f32)> = {
                if let Some(physics_world) = world.get_resource::<PhysicsWorld3D>() {
                    let ray = Ray::new(
                        point![cam_pos.x, cam_pos.y, cam_pos.z],
                        vector![cam_dir.x, cam_dir.y, cam_dir.z],
                    );
                    
                    let max_toi = 100.0;
                    let solid = true;
                    let query_filter = QueryFilter::default();
                    
                    if let Some((handle, toi)) = physics_world.query_pipeline.cast_ray(
                        &physics_world.rigid_body_set,
                        &physics_world.collider_set,
                        &ray,
                        max_toi,
                        solid,
                        query_filter,
                    ) {
                        let mut result = None;
                        if let Some(collider) = physics_world.collider_set.get(handle) {
                            if let Some(body_handle) = collider.parent() {
                                if let Some(entity) = physics_world.body_to_entity.get(&body_handle) {
                                    result = Some((*entity, toi));
                                }
                            }
                        }
                        result
                    } else {
                        // Miss
                        None
                    }
                } else {
                    None
                }
            };

            // Phase 2: Process hit result (mutable borrows)
            match hit_result {
                Some((entity, toi)) => {
                    // Check if entity is a target that hasn't been hit yet
                    let target_info = world.get_component::<Target>(entity)
                        .filter(|t| !t.hit)
                        .map(|t| t.points);

                    if let Some(points) = target_info {
                        let hit_pos_vec3 = cam_pos + cam_dir * toi;
                        
                        if let Some(tgs) = world.get_resource_mut::<TargetGameState>() {
                            tgs.register_hit(points, hit_pos_vec3);
                        }
                        
                        // Mark target as hit
                        if let Some(target) = world.get_component_mut::<Target>(entity) {
                            target.hit = true;
                            target.hit_flash_timer = 0.5;
                        }
                        
                        // Spawn particle effect
                        let particle_entity = world.spawn();
                        world.add_component(particle_entity, Name::new("HitParticles"));
                        world.add_component(
                            particle_entity,
                            Transform {
                                translation: hit_pos_vec3,
                                rotation: Quat::IDENTITY,
                                scale: Vec3::ONE,
                            },
                        );
                        world.add_component(
                            particle_entity,
                            ParticleEmitter {
                                rate: 0.0,
                                accumulator: 0.0,
                                direction: Vec3::new(0.0, 5.0, 0.0),
                                spread: 0.5,
                                speed: 5.0,
                                color: Color::rgb(1.0, 0.5, 0.0),
                                size: 0.1,
                                lifetime: 1.0,
                            },
                        );
                        
                        info!("Target hit! +{} points", points);
                    }
                }
                None => {
                    // Miss
                    if let Some(tgs) = world.get_resource_mut::<TargetGameState>() {
                        tgs.register_miss();
                    }
                }
            }
        }
    }

    if wants_spawn_random {
        let counter = world
            .get_resource::<DemoState>()
            .map(|s| s.spawn_counter)
            .unwrap_or(0);
        let shape = counter % 3;
        let ent = match shape {
            0 => spawn_sphere_at(world, spawn_pos, 0.8, Color::rgb(0.9, 0.5, 0.1)),
            1 => spawn_cube_at(world, spawn_pos, 1.5, Color::rgb(0.1, 0.9, 0.4)),
            _ => spawn_emissive_orb(world, spawn_pos, 0.6, Color::rgb(3.0, 0.5, 1.0)),
        };
        add_spawned_entity(world, ent);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification(
                format!("Spawned object #{}", state.spawn_counter),
                [0.5, 1.0, 0.5, 1.0],
            );
        }
    }
    if wants_spawn_sphere {
        let ent = spawn_sphere_at(world, spawn_pos, 0.8, Color::rgb(0.95, 0.2, 0.2));
        add_spawned_entity(world, ent);
    }
    if wants_spawn_cube {
        let ent = spawn_cube_at(world, spawn_pos, 1.5, Color::rgb(0.2, 0.6, 0.95));
        add_spawned_entity(world, ent);
    }
    if wants_spawn_glow {
        let ent = spawn_emissive_orb(world, spawn_pos, 0.6, Color::rgb(0.0, 3.0, 2.0));
        add_spawned_entity(world, ent);
    }
    if wants_spawn_target {
        let ent = spawn_target_at(world, spawn_pos, 0.8, 10);
        add_spawned_entity(world, ent);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("Spawned target!".into(), [1.0, 0.4, 0.2, 1.0]);
        }
    }
    if wants_spawn_stack {
        for i in 0..5 {
            let p = spawn_pos + Vec3::new(0.0, i as f32 * 2.2, 0.0);
            let hue = i as f32 / 5.0;
            let color = Color::rgb(
                (hue * 6.28).sin() * 0.5 + 0.5,
                ((hue + 0.33) * 6.28).sin() * 0.5 + 0.5,
                ((hue + 0.66) * 6.28).sin() * 0.5 + 0.5,
            );
            let ent = spawn_cube_at(world, p, 1.5, color);
            add_spawned_entity(world, ent);
        }
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("Spawned stack of 5!".into(), [0.8, 0.8, 0.2, 1.0]);
        }
    }
    if wants_spawn_rain {
        for i in 0..10 {
            let offset = Vec3::new(
                (i as f32 * 1.7).sin() * 5.0,
                10.0 + i as f32 * 1.5,
                (i as f32 * 2.3).cos() * 5.0,
            );
            let color = Color::rgb(
                ((i as f32) * 0.3).sin() * 0.4 + 0.6,
                ((i as f32) * 0.5).cos() * 0.4 + 0.5,
                0.8,
            );
            let ent = spawn_sphere_at(world, spawn_pos + offset, 0.5, color);
            add_spawned_entity(world, ent);
        }
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("Sphere rain! (10)".into(), [0.5, 0.5, 1.0, 1.0]);
        }
    }
    if wants_clear {
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            let entities_to_remove: Vec<Entity> = state.spawned_entities.drain(..).collect();
            let count = entities_to_remove.len();
            let _ = state;
            for ent in entities_to_remove {
                world.despawn(ent);
            }
            if let Some(state) = world.get_resource_mut::<DemoState>() {
                state.push_notification(format!("Cleared {} objects", count), [1.0, 0.8, 0.3, 1.0]);
            }
        }
    }
    if wants_reset {
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            let entities_to_remove: Vec<Entity> = state.spawned_entities.drain(..).collect();
            state.gizmos_on = true;
            state.gizmo_category_index = 0;
            state.physics_paused = false;
            state.slow_motion = false;
            state.target_mode = false;
            let _ = state;
            for ent in entities_to_remove {
                world.despawn(ent);
            }
            if let Some(time) = world.get_resource_mut::<Time>() {
                time.time_scale = 1.0;
            }
            if let Some(tgs) = world.get_resource_mut::<TargetGameState>() {
                tgs.reset();
            }
            if let Some(state) = world.get_resource_mut::<DemoState>() {
                state.push_notification("Scene RESET".into(), [1.0, 1.0, 1.0, 1.0]);
            }
        }
    }
    
    // ── Spectacle scene spawning ────────────────────────────────────
    if wants_spawn_domino_chain {
        create_domino_chain(world);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("🎲 DOMINO CHAIN spawned! Push the first one!".into(), [1.0, 0.5, 0.2, 1.0]);
        }
    }
    
    if wants_spawn_pendulum_array {
        create_pendulum_array(world);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("⚖️ PENDULUM ARRAY spawned!".into(), [0.5, 0.8, 1.0, 1.0]);
        }
    }
    
    if wants_spawn_rotating_platforms {
        create_rotating_platforms(world);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("🔄 ROTATING PLATFORMS spawned!".into(), [0.8, 0.5, 1.0, 1.0]);
        }
    }
    
    if wants_spawn_orbital_system {
        create_orbital_system(world);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("🌌 ORBITAL SYSTEM spawned!".into(), [1.0, 0.8, 0.3, 1.0]);
        }
    }
    
    if wants_spawn_magnetic_demo {
        create_magnetic_field_demo(world);
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            state.push_notification("🧲 MAGNETIC FIELD demo spawned!".into(), [0.3, 1.0, 0.8, 1.0]);
        }
    }
}

// ============================================================================
// Target Game System — updates target state and handles hit detection
// ============================================================================

fn target_game_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let target_active = world
        .get_resource::<DemoState>()
        .map(|s| s.target_mode)
        .unwrap_or(false);

    // Update target flash timers
    {
        let mut query = Query::<&mut Target>::new(world);
        for target in query.iter_mut() {
            if target.hit_flash_timer > 0.0 {
                target.hit_flash_timer -= dt;
            }
        }
    }

    // Update target game state timer
    if let Some(tgs) = world.get_resource_mut::<TargetGameState>() {
        tgs.active = target_active;
        tgs.show_crosshair = target_active;
        tgs.update(dt);
    }
}

// ============================================================================
// Gizmo Draw System — draws debug visualizations
// ============================================================================

fn gizmo_draw_system(world: &mut World) {
    let gizmos_on = world
        .get_resource::<DemoState>()
        .map(|s| s.gizmos_on)
        .unwrap_or(false);

    if !gizmos_on {
        return;
    }

    if let Some(cmd_buf) = world.get_resource_mut::<CommandBuffer>() {
        // Ground grid
        Gizmos::grid(cmd_buf, Vec3::ZERO, 60.0, 30, Color::rgba(0.3, 0.3, 0.3, 0.4));

        // World axes at origin
        Gizmos::axes(cmd_buf, Vec3::new(0.0, 0.01, 0.0), 3.0);

        // Draw circles around energy cores
        for i in 0..6 {
            let a = i as f32 * std::f32::consts::TAU / 6.0;
            let pos = Vec3::new(a.cos() * 14.0, 2.5, a.sin() * 14.0);
            Gizmos::circle(cmd_buf, pos, 1.5, 16, Color::rgba(0.0, 0.8, 1.0, 0.6));
        }

        // Arrow pointing up at each pillar
        let pillar_positions = [
            Vec3::new(-20.0, 6.0, -20.0),
            Vec3::new(20.0, 6.0, -20.0),
            Vec3::new(-20.0, 6.0, 20.0),
            Vec3::new(20.0, 6.0, 20.0),
        ];
        for pos in &pillar_positions {
            Gizmos::arrow(cmd_buf, *pos, *pos + Vec3::new(0.0, 3.0, 0.0), Color::rgb(1.0, 0.8, 0.2));
        }

        // Collision bounds for walls (transparent)
        Gizmos::cube_cat(
            cmd_buf,
            Vec3::new(0.0, 4.0, 31.0),
            Vec3::new(31.0, 4.0, 0.5),
            Color::rgba(1.0, 0.3, 0.3, 0.3),
            "collision",
        );
        Gizmos::cube_cat(
            cmd_buf,
            Vec3::new(0.0, 4.0, -31.0),
            Vec3::new(31.0, 4.0, 0.5),
            Color::rgba(1.0, 0.3, 0.3, 0.3),
            "collision",
        );
        Gizmos::cube_cat(
            cmd_buf,
            Vec3::new(31.0, 4.0, 0.0),
            Vec3::new(0.5, 4.0, 31.0),
            Color::rgba(1.0, 0.3, 0.3, 0.3),
            "collision",
        );
        Gizmos::cube_cat(
            cmd_buf,
            Vec3::new(-31.0, 4.0, 0.0),
            Vec3::new(0.5, 4.0, 31.0),
            Color::rgba(1.0, 0.3, 0.3, 0.3),
            "collision",
        );
    }
}

// ============================================================================
// HUD System — polished in-game overlay with gradient panels + outlined text
// ============================================================================

fn hud_system(world: &mut World) {
    // Gather state info
    let (frame, fps, spawned, gizmos, phys, menu_visible, slow_mo, target_mode, notifications, scene_time, gravity_scale, post_fx_status) =
        if let Some(state) = world.get_resource::<DemoState>() {
            (
                state.frame_count,
                state.current_fps,
                state.spawned_entities.len(),
                state.gizmos_on,
                state.physics_paused,
                state.menu_visible,
                state.slow_motion,
                state.target_mode,
                state.notifications.clone(),
                state.scene_time,
                state.gravity_scale,
                state.post_effects.status_string(),
            )
        } else {
            return;
        };

    // Get frame stats for detailed performance info
    let (avg_fps, p99_frame_time) = if let Some(frame_stats) = world.get_resource::<luminara_diagnostic::frame_stats::FrameStats>() {
        (frame_stats.average_fps(), frame_stats.percentile_frame_time(99.0))
    } else {
        (0.0, 0.0)
    };

    let grabbed = world
        .get_resource::<Input>()
        .map(|i| i.is_cursor_grabbed())
        .unwrap_or(false);

    // Screen dimensions (default to window descriptor)
    let (screen_w, screen_h) = (1440.0f32, 900.0f32);

    // ── In-game overlay via OverlayRenderer ─────────────────────────
    if let Some(overlay) = world.get_resource_mut::<OverlayRenderer>() {
        overlay.clear();

        let scale = 1.0;
        let cw = 8.0 * scale;
        let ch = 8.0 * scale;
        let line_h = ch + 6.0;

        // ── Status bar (top-left) with gradient background and icons ──────────
        let gizmo_icon = if gizmos { "🔧" } else { "🚫" };
        let phys_icon = if phys { "⏸️" } else { "▶️" };
        let slow_icon = if slow_mo { "🐌" } else { "" };
        let target_icon = if target_mode { "🎯" } else { "" };
        let phys_label = if phys { " PAUSED" } else { "" };
        let slow_label = if slow_mo { " SLOW" } else { "" };
        let target_label = if target_mode { " TARGET" } else { "" };
        let status = format!(
            "{} {:.0} FPS (avg {:.0}, P99 {:.1}ms) | 📦 {} | {} Phys{}{}{}{}{} | G {:.1}x | ⏱️ {:.1}s",
            gizmo_icon, fps, avg_fps, p99_frame_time, spawned, phys_icon, phys_label, slow_label, target_label, slow_icon, target_icon, gravity_scale, scene_time,
        );
        let bar_w = status.len() as f32 * cw + 28.0;
        overlay.draw_gradient_rect(
            8.0, 8.0, bar_w, ch + 14.0,
            [0.05, 0.1, 0.2, 0.9],
            [0.02, 0.05, 0.1, 0.9],
        );
        overlay.draw_text_outlined(
            18.0, 14.0, &status,
            [0.0, 1.0, 0.8, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            scale,
        );

        // ── Engine badge (top-right) ────────────────────────────────
        let badge = "LUMINARA ENGINE v0.1";
        let badge_w = badge.len() as f32 * cw + 20.0;
        overlay.draw_gradient_rect(
            screen_w - badge_w - 8.0, 8.0, badge_w, ch + 14.0,
            [0.15, 0.05, 0.2, 0.85],
            [0.08, 0.02, 0.12, 0.85],
        );
        overlay.draw_text_outlined(
            screen_w - badge_w + 2.0, 14.0, badge,
            [0.8, 0.6, 1.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            scale,
        );

        // ── Post-effects status (below badge) ───────────────────────
        let fx_text = format!("FX: {}", post_fx_status);
        let fx_w = fx_text.len() as f32 * cw + 20.0;
        overlay.draw_rect(screen_w - fx_w - 8.0, 30.0, fx_w, ch + 10.0, [0.1, 0.05, 0.15, 0.8]);
        overlay.draw_text(screen_w - fx_w + 2.0, 34.0, &fx_text, [0.8, 0.8, 1.0, 1.0], scale);

        // ── Mouse grab hint (below post-effects status) ─────────────
        if !grabbed {
            let hint = "RMB to Look | [Esc] Free";
            let hint_w = hint.len() as f32 * cw + 20.0;
            overlay.draw_rect(screen_w - hint_w - 8.0, 52.0, hint_w, ch + 10.0, [0.6, 0.1, 0.1, 0.8]);
            overlay.draw_text(screen_w - hint_w + 2.0, 56.0, hint, [1.0, 1.0, 1.0, 1.0], scale);
        }

        // ── Slow Motion indicator (center-top, pulsing) ─────────────
        if slow_mo {
            let pulse = (scene_time * 4.0).sin() * 0.3 + 0.7;
            let label = ">> SLOW MOTION <<";
            let lw = label.len() as f32 * cw * 1.5 + 30.0;
            let lx = (screen_w - lw) * 0.5;
            overlay.draw_gradient_rect(
                lx, 50.0, lw, ch * 1.5 + 14.0,
                [0.5 * pulse, 0.3 * pulse, 0.0, 0.85],
                [0.3 * pulse, 0.15 * pulse, 0.0, 0.85],
            );
            overlay.draw_text_outlined(
                lx + 15.0, 56.0, label,
                [1.0, 0.9 * pulse, 0.3, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                1.5,
            );
        }

        // ── Physics paused indicator ────────────────────────────────
        if phys {
            let label = "|| PHYSICS PAUSED ||";
            let lw = label.len() as f32 * cw * 1.3 + 24.0;
            let lx = (screen_w - lw) * 0.5;
            let y = if slow_mo { 85.0 } else { 50.0 };
            overlay.draw_rect(lx, y, lw, ch * 1.3 + 10.0, [0.1, 0.1, 0.4, 0.85]);
            overlay.draw_text_outlined(
                lx + 12.0, y + 5.0, label,
                [0.7, 0.7, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                1.3,
            );
        }

        // ── Notification feed (right side) with animation ──────────────────────────
        for (i, (msg, remaining, color)) in notifications.iter().enumerate() {
            let alpha = (*remaining / 3.0f32).min(1.0f32);
            let fade_in = if *remaining > 2.5f32 { (3.0f32 - *remaining) / 0.5f32 } else { 1.0f32 };
            let effective_alpha = alpha * fade_in;
            let ny = 60.0f32 + i as f32 * (line_h + 4.0f32) - (1.0f32 - fade_in) * 20.0f32; // Slide in from top
            let nw = msg.len() as f32 * cw + 20.0;
            overlay.draw_gradient_rect(
                screen_w - nw - 12.0, ny, nw, line_h + 2.0,
                [color[0] * 0.2, color[1] * 0.2, color[2] * 0.2, 0.8 * effective_alpha],
                [color[0] * 0.1, color[1] * 0.1, color[2] * 0.1, 0.6 * effective_alpha],
            );
            overlay.draw_text(
                screen_w - nw - 2.0, ny + 2.0, msg,
                [color[0], color[1], color[2], effective_alpha],
                scale,
            );
        }

        // ── Target mode crosshair + HUD ─────────────────────────────
        if target_mode {
            draw_crosshair(overlay, screen_w, screen_h, [0.0, 1.0, 0.3, 0.8]);

            // Draw target game score HUD
            if let Some(tgs) = world.get_resource::<TargetGameState>() {
                // We already have an overlay borrow, so we call draw_target_hud carefully
                // Actually we need to re-borrow. Let's inline the important parts:
                let score_panel_w = 200.0;
                let score_panel_x = 10.0;
                let score_panel_y = screen_h - 120.0;

                overlay.draw_gradient_rect(
                    score_panel_x, score_panel_y, score_panel_w, 110.0,
                    [0.1, 0.02, 0.02, 0.85],
                    [0.05, 0.01, 0.01, 0.85],
                );
                overlay.draw_text_outlined(
                    score_panel_x + 10.0, score_panel_y + 8.0,
                    "TARGET GAME",
                    [1.0, 0.3, 0.2, 1.0],
                    [0.0, 0.0, 0.0, 1.0],
                    1.2,
                );
                let score_text = format!("Score: {}", tgs.score);
                overlay.draw_text(score_panel_x + 10.0, score_panel_y + 30.0, &score_text, [1.0, 1.0, 0.5, 1.0], scale);

                let shots_text = format!("Shots: {} | Hits: {}", tgs.shots_fired, tgs.hits);
                overlay.draw_text(score_panel_x + 10.0, score_panel_y + 48.0, &shots_text, [0.8, 0.8, 0.8, 1.0], scale);

                let acc_text = format!("Accuracy: {:.0}%", tgs.accuracy() * 100.0);
                let acc_color = if tgs.accuracy() > 0.7 {
                    [0.3, 1.0, 0.3, 1.0]
                } else if tgs.accuracy() > 0.4 {
                    [1.0, 1.0, 0.3, 1.0]
                } else {
                    [1.0, 0.3, 0.3, 1.0]
                };
                overlay.draw_text(score_panel_x + 10.0, score_panel_y + 66.0, &acc_text, acc_color, scale);

                if !tgs.feedback_message.is_empty() && tgs.feedback_timer > 0.0 {
                    let fb_alpha = (tgs.feedback_timer / 1.0).min(1.0);
                    overlay.draw_text_outlined(
                        score_panel_x + 10.0, score_panel_y + 88.0,
                        &tgs.feedback_message,
                        [1.0, 1.0, 0.0, fb_alpha],
                        [0.0, 0.0, 0.0, fb_alpha],
                        1.1,
                    );
                }
            }
        }

        // ── Command palette (toggled with H) — now with gradient ────
        if menu_visible {
            let lines: &[(&str, [f32; 4])] = &[
                ("=== CONTROLS ===",           [1.0, 0.85, 0.3, 1.0]),
                ("",                            [0.0; 4]),
                ("[WASD] Move  [RMB] Look",    [0.9, 0.9, 0.9, 1.0]),
                ("[Space/Q] Up/Down",           [0.9, 0.9, 0.9, 1.0]),
                ("[C] Camera Mode",             [0.5, 0.8, 1.0, 1.0]),
                ("",                            [0.0; 4]),
                ("=== SPECTACLE SCENES ===",    [1.0, 0.5, 0.2, 1.0]),
                ("[D] Domino Chain",            [1.0, 0.6, 0.3, 1.0]),
                ("[N] Pendulum Array",          [0.5, 0.8, 1.0, 1.0]),
                ("[M] Rotating Platforms",      [0.8, 0.5, 1.0, 1.0]),
                ("[O] Orbital System",          [1.0, 0.8, 0.3, 1.0]),
                ("[K] Magnetic Field",          [0.3, 1.0, 0.8, 1.0]),
                ("",                            [0.0; 4]),
                ("[T] Spawn Random",            [0.5, 1.0, 0.5, 1.0]),
                ("[1-5] Shapes/Stack/Rain",     [0.5, 1.0, 0.5, 1.0]),
                ("[E] Explosion",               [1.0, 0.4, 0.2, 1.0]),
                ("[F] Target Mode",             [1.0, 0.6, 0.3, 1.0]),
                ("[Y] Spawn Target",            [1.0, 0.6, 0.3, 1.0]),
                ("[V] Cycle Gravity",           [0.5, 0.8, 1.0, 1.0]),
                ("[+/-] Gravity Scale",         [0.5, 0.8, 1.0, 1.0]),
                ("[B] Slow Motion",             [1.0, 1.0, 0.3, 1.0]),
                ("",                            [0.0; 4]),
                ("[7] Toggle Bloom",            [1.0, 0.8, 0.3, 1.0]),
                ("[8] Toggle DOF",              [0.8, 0.8, 1.0, 1.0]),
                ("[[/]] Exposure",              [1.0, 1.0, 0.5, 1.0]),
                ("[,/.] Prev/Next Scene",       [0.5, 1.0, 0.8, 1.0]),
                ("",                            [0.0; 4]),
                ("[X] Clear  [R] Reset",        [0.8, 0.8, 0.8, 1.0]),
                ("[G] Cycle Gizmos",            [0.5, 0.8, 1.0, 1.0]),
                ("[P] Pause Physics",           [0.5, 0.8, 1.0, 1.0]),
                ("[H] Hide Menu",               [0.6, 0.6, 0.6, 1.0]),
                ("[Esc] Free/Exit",             [1.0, 0.5, 0.5, 1.0]),
            ];

            let panel_x = 10.0;
            let panel_y = 36.0;
            let max_line_len = lines.iter().map(|(s, _)| s.len()).max().unwrap_or(0);
            let panel_w = max_line_len as f32 * cw + 36.0;
            let panel_h = lines.len() as f32 * line_h + 20.0;

            // Gradient panel background
            overlay.draw_gradient_rect(
                panel_x, panel_y, panel_w, panel_h,
                [0.02, 0.04, 0.08, 0.92],
                [0.01, 0.02, 0.05, 0.88],
            );

            // Thin colored accent line at top
            overlay.draw_rect(panel_x, panel_y, panel_w, 2.0, [0.3, 0.6, 1.0, 0.8]);

            for (i, (text, color)) in lines.iter().enumerate() {
                if text.is_empty() {
                    continue;
                }
                let lx = panel_x + 16.0;
                let ly = panel_y + 10.0 + i as f32 * line_h;
                overlay.draw_text_outlined(lx, ly, text, *color, [0.0, 0.0, 0.0, 0.8], scale);
            }
        }

        // ── Bottom bar: entity count + frame ────────────────────────
        let bottom_text = format!("Frame: {} | Entities: {}", frame, spawned);
        let bw = bottom_text.len() as f32 * cw + 20.0;
        overlay.draw_rect(8.0, screen_h - 24.0, bw, ch + 10.0, [0.0, 0.0, 0.0, 0.6]);
        overlay.draw_text(16.0, screen_h - 20.0, &bottom_text, [0.5, 0.5, 0.5, 1.0], scale);
    }

    // ── Periodic console output ─────────────────────────────────────
    if frame % 300 == 0 && frame > 0 {
        info!(
            "FPS: {:.0} | Spawned: {} | Gizmos: {} | Physics: {} | SlowMo: {}",
            fps, spawned,
            if gizmos { "ON" } else { "OFF" },
            if phys { "PAUSED" } else { "OK" },
            if slow_mo { "ON" } else { "OFF" },
        );
    }
}

// ============================================================================
// Animation Systems
// ============================================================================

fn animate_energy_cores_system(world: &mut World) {
    let (dt, elapsed) = world
        .get_resource::<luminara_core::Time>()
        .map(|t| (t.delta_seconds(), t.elapsed_seconds()))
        .unwrap_or((1.0 / 60.0, 0.0));

    let mut query = Query::<(&Name, &mut Transform)>::new(world);
    for (name, transform) in query.iter_mut() {
        if name.0 == "EnergyCore" {
            // Rotate around Y axis
            let rotation_speed = 1.0;
            transform.rotation = transform.rotation * Quat::from_rotation_y(rotation_speed * dt);
            
            // Add floating motion
            let float_speed = 2.0;
            let float_amplitude = 0.5;
            let float_offset = (elapsed * float_speed).sin() * float_amplitude;
            transform.translation.y = 2.5 + float_offset;
        }
    }
}

fn collision_sound_system(world: &mut World) {
    // Collect collision events first
    let events: Vec<_> = world
        .get_resource::<CollisionEvents>()
        .map(|ce| ce.0.iter().filter(|e| e.started).cloned().collect())
        .unwrap_or_default();

    for _event in &events {
        // Spawn a sound entity for each collision
        let sound_entity = world.spawn();
        world.add_component(sound_entity, Name::new("CollisionSound"));
        world.add_component(
            sound_entity,
            AudioSource {
                clip: AudioClipHandle("collision".to_string()),
                volume: 0.3,
                pitch: 1.0,
                looping: false,
                spatial: true,
                max_distance: 50.0,
            },
        );
    }
}

// ============================================================================
// Post-Effects System — applies post-processing effects
// ============================================================================

fn post_effects_system(world: &mut World) {
    // Get post-effects configuration
    let post_effects = world
        .get_resource::<DemoState>()
        .map(|s| s.post_effects.clone())
        .unwrap_or_default();

    // Apply post-effects to command buffer
    if let Some(cmd_buf) = world.get_resource_mut::<CommandBuffer>() {
        // In a real implementation, this would apply the effects
        // For now, we just store the configuration
        let _ = (cmd_buf, post_effects);
    }
}
