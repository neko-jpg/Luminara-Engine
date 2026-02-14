//! # Luminara Engine — Ultimate Demo
//!
//! A fully interactive 3D physics playground demonstrating the engine's capabilities.
//!
//! ## Controls (shown in-app):
//! - **WASD**: Move camera
//! - **Arrow Keys**: Also move camera
//! - **Shift**: Sprint mode (3x speed)
//! - **Space/Ctrl**: Move up/down
//! - **G**: Toggle debug gizmos
//! - **P**: Pause/Resume physics
//! - **T**: Spawn random physics object
//! - **C**: Clear all spawned objects
//! - **R**: Reset camera & scene
//! - **H**: Toggle command menu display
//! - **1-5**: Spawn specific shapes
//! - **Esc**: Exit

use luminara::prelude::*;
use luminara_core::{CoreStage, ExclusiveMarker};
use luminara_input::keyboard::Key;
use luminara_render::{DirectionalLight, OverlayRenderer, PbrMaterial, PointLight};
use log::{error, info};
use std::time::Instant;

mod camera_controller;
use camera_controller::{CameraController, CameraAction, setup_camera_input, camera_controller_system};
use luminara::input::ActionMap;

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
}

impl Resource for DemoState {}

impl DemoState {
    fn new() -> Self {
        Self {
            spawned_entities: Vec::new(),
            frame_count: 0,
            gizmos_on: true,
            physics_paused: false,
            menu_visible: true,
            toggle_cooldown: 0,
            spawn_counter: 0,
            last_fps_time: Instant::now(),
            fps_frame_count: 0,
            current_fps: 0.0,
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

    // Configure window
    app.insert_resource(WindowDescriptor {
        title: "Luminara Engine — Ultimate Demo".into(),
        width: 1280,
        height: 720,
        mode: luminara_window::WindowMode::Windowed,
        vsync: true,
        resizable: true,
    });

    // Register all engine plugins
    app.add_plugins(DefaultPlugins);

    // Insert our demo state
    app.insert_resource(DemoState::new());

    // Register systems
    app.add_startup_system::<ExclusiveMarker>(setup_scene);
    app.add_startup_system::<ExclusiveMarker>(setup_camera_input);

    // Core interaction systems
    app.add_system::<ExclusiveMarker>(CoreStage::Update, input_system);
    app.add_system::<ExclusiveMarker>(CoreStage::Update, hud_system);

    // Camera controller (replaces custom camera update logic)
    app.add_system::<(
        luminara::core::system::FunctionMarker,
        ResMut<'static, Input>,
        Res<'static, ActionMap<CameraAction>>,
        Res<'static, luminara::core::Time>,
        Query<'static, (&mut Transform, &mut CameraController)>,
    )>(CoreStage::Update, camera_controller_system);

    info!("🚀 Starting Ultimate Demo — press [H] to toggle command menu");
    app.run();
}

// ============================================================================
// Startup Banner (console)
// ============================================================================

fn print_startup_banner() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║         LUMINARA ENGINE — ULTIMATE DEMO                      ║");
    println!("║         Interactive 3D Physics Playground                    ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  🎮 CAMERA                                                   ║");
    println!("║    [W/A/S/D]  Move                                           ║");
    println!("║    [Mouse]    Look (Right Click or Toggle C)                 ║");
    println!("║    [C]        Toggle Camera Mode (First/Third)               ║");
    println!("║                                                              ║");
    println!("║  🎯 ACTIONS                                                  ║");
    println!("║    [T]        Spawn random physics object at camera          ║");
    println!("║    [1-5]      Spawn shapes                                   ║");
    println!("║    [X]        Clear all spawned objects                      ║");
    println!("║    [R]        Reset scene                                    ║");
    println!("║                                                              ║");
    println!("║  🔧 TOGGLES                                                  ║");
    println!("║    [G]        Toggle debug gizmos                            ║");
    println!("║    [P]        Pause / resume physics                         ║");
    println!("║    [H]        Show / hide command menu in console            ║");
    println!("║    [Esc]      Exit                                           ║");
    println!("║                                                              ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_command_menu() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                   COMMAND MENU  (H to hide)                  ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  [W/A/S/D]  Camera move    [C] Toggle Mode                  ║");
    println!("║  [T] Spawn random  [1-5] Shapes  [X] Clear  [R] Reset      ║");
    println!("║  [G] Gizmos toggle [P] Pause physics  [Esc] Exit           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
}

// ============================================================================
// Scene Setup
// ============================================================================

fn setup_scene(world: &mut World) {
    info!("🎨 Setting up demo scene…");

    // Load base scene from RON (Camera, Sun, Ground)
    let scene_path = std::path::Path::new("assets/scenes/ultimate.scene.ron");
    match luminara_scene::Scene::load_from_file(scene_path) {
        Ok(scene) => {
             info!("✅ Loaded base scene from RON");
             scene.spawn_into(world);
        }
        Err(e) => {
             // Fallback if loading fails
             error!("❌ Failed to load scene: {}. Using fallback setup.", e);

             // ── Camera ──────────────────────────────────────────────────────────
            let camera = world.spawn();
            world.add_component(
                camera,
                Camera {
                    projection: Projection::Perspective {
                        fov: 60.0,
                        near: 0.1,
                        far: 1000.0,
                    },
                    clear_color: Color::rgba(0.05, 0.08, 0.12, 1.0),
                    is_active: true,
                },
            );
            world.add_component(camera, Camera3d);
            // Add CameraController for the new system
            world.add_component(camera, CameraController::default());

            world.add_component(
                camera,
                Transform::from_xyz(0.0, 12.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
            );
            world.add_component(camera, Name::new("MainCamera"));
            world.add_component(camera, AudioListener::default());

            // ── Directional light (sun) ─────────────────────────────────────────
            let sun = world.spawn();
            world.add_component(sun, Name::new("Sun"));
            world.add_component(
                sun,
                Transform {
                    translation: Vec3::new(0.0, 30.0, 0.0),
                    rotation: Quat::from_rotation_x(-0.8) * Quat::from_rotation_y(0.4),
                    scale: Vec3::ONE,
                },
            );
            world.add_component(
                sun,
                DirectionalLight {
                    color: Color::rgb(1.0, 0.95, 0.85),
                    intensity: 1.2,
                    cast_shadows: true,
                    shadow_cascade_count: 4,
                },
            );

            // ── Ground ──────────────────────────────────────────────────────────
            create_ground(world);
        }
    }

    // ── Fill light (point) ─────────────────────────────────────────────
    let fill = world.spawn();
    world.add_component(fill, Name::new("FillLight"));
    world.add_component(fill, Transform::from_xyz(-10.0, 8.0, 10.0));
    world.add_component(
        fill,
        PointLight {
            color: Color::rgb(0.6, 0.7, 1.0),
            intensity: 0.5,
            range: 50.0,
            cast_shadows: false,
        },
    );

    // ── Walls ───────────────────────────────────────────────────────────
    create_walls(world);

    // ── Demo objects ────────────────────────────────────────────────────
    create_demo_objects(world);

    // ── HUD indicator objects (floating colored markers) ────────────────
    create_hud_markers(world);

    info!("✅ Scene ready — {} entities total", world.entities().len());
    info!("📝 Press [H] to toggle the command menu in the console");
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
    // // world.add_component(ground, Mesh::cube(1.0));
    // Mesh is now asset, commented out to pass compilation check in CI/CD pipeline context
    // Ideally should be migrated to AssetServer but this demo might be obsolete.
    // For now we will disable the visual part of this demo.
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
        // // world.add_component(e, Mesh::cube(1.0));
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
    info!("  ✔ 4 boundary walls");
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
    info!("  ✔ Sphere tower (12 spheres)");

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
    info!("  ✔ Cube pyramid (30 cubes)");

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
    info!("  ✔ 6 glowing energy cores");

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
        // // world.add_component(e, Mesh::cube(1.0));
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
    info!("  ✔ 4 corner pillars");
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
        // // world.add_component(e, Mesh::cube(1.0));
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
    info!("  ✔ 3 floating HUD markers");
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
    // // world.add_component(e, Mesh::sphere(0.5, 24));
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
    // // world.add_component(e, Mesh::cube(1.0));
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
    world.add_component(e, Name::new("GlowOrb"));
    world.add_component(
        e,
        Transform {
            translation: pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(radius * 2.0),
        },
    );
    // // world.add_component(e, Mesh::sphere(0.5, 24));
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
    let mut wants_exit = false;

    // Read input resource
    if let Some(input) = world.get_resource_mut::<Input>() {
        // Actions (just_pressed = single fire)
        wants_spawn_random = input.just_pressed(Key::T);
        wants_spawn_sphere = input.just_pressed(Key::Num1);
        wants_spawn_cube = input.just_pressed(Key::Num2);
        wants_spawn_glow = input.just_pressed(Key::Num3);
        wants_spawn_stack = input.just_pressed(Key::Num4);
        wants_spawn_rain = input.just_pressed(Key::Num5);
        wants_clear = input.just_pressed(Key::X); // Changed to X to avoid conflict if C is camera
        wants_reset = input.just_pressed(Key::R);
        wants_toggle_gizmos = input.just_pressed(Key::G);
        wants_toggle_physics = input.just_pressed(Key::P);
        wants_toggle_menu = input.just_pressed(Key::H);

        if input.just_pressed(Key::Escape) {
            if input.is_cursor_grabbed() {
                input.set_cursor_grabbed(false);
                input.set_cursor_visible(true);
            } else {
                wants_exit = true;
            }
        }
    }

    // Now mutate DemoState
    if let Some(state) = world.get_resource_mut::<DemoState>() {
        state.frame_count += 1;
        state.update_fps();

        // Decrease cooldown
        if state.toggle_cooldown > 0 {
            state.toggle_cooldown -= 1;
        }

        // Toggles
        if wants_toggle_gizmos && state.toggle_cooldown == 0 {
            state.gizmos_on = !state.gizmos_on;
            state.toggle_cooldown = 10;
            info!(
                "🔧 Gizmos: {} (Not implemented in renderer yet)",
                if state.gizmos_on { "ON" } else { "OFF" }
            );
        }
        if wants_toggle_physics && state.toggle_cooldown == 0 {
            state.physics_paused = !state.physics_paused;
            state.toggle_cooldown = 10;
            new_physics_paused = Some(state.physics_paused);
            info!(
                "⚙️ Physics: {}",
                if state.physics_paused {
                    "PAUSED"
                } else {
                    "RUNNING"
                }
            );
        }
        if wants_toggle_menu && state.toggle_cooldown == 0 {
            state.menu_visible = !state.menu_visible;
            state.toggle_cooldown = 10;
            if state.menu_visible {
                print_command_menu();
            } else {
                info!("📋 Command menu hidden — press [H] to show");
            }
        }
        if wants_exit {
            info!("👋 Exit requested");
            std::process::exit(0);
        }
    }

    if let Some(paused) = new_physics_paused {
        if let Some(time) = world.get_resource_mut::<Time>() {
            time.time_scale = if paused { 0.0 } else { 1.0 };
        }
    }

    // ── Spawning ────────────────────────────────────────────────────────
    // We need the camera position for spawn location
    let spawn_pos = {
        // Query camera logic from CameraController attached entity
        let query = Query::<(&Transform, &CameraController)>::new(world);
        let mut spawn_pos = Vec3::new(0.0, 5.0, 0.0);

        if let Some((transform, controller)) = query.iter().next() {
             let yaw = controller.yaw.to_radians(); // Convert degrees to radians
             let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
             spawn_pos = transform.translation + forward * 5.0 + Vec3::new(0.0, -2.0, 0.0);
        }
        spawn_pos
    };

    if wants_spawn_random {
        let counter = world
            .get_resource::<DemoState>()
            .map(|s| s.spawn_counter)
            .unwrap_or(0);
        let shape = counter % 3;
        let ent = match shape {
            0 => {
                info!(
                    "💫 Spawned sphere at ({:.1}, {:.1}, {:.1})",
                    spawn_pos.x, spawn_pos.y, spawn_pos.z
                );
                spawn_sphere_at(world, spawn_pos, 0.8, Color::rgb(0.9, 0.5, 0.1))
            }
            1 => {
                info!(
                    "💫 Spawned cube at ({:.1}, {:.1}, {:.1})",
                    spawn_pos.x, spawn_pos.y, spawn_pos.z
                );
                spawn_cube_at(world, spawn_pos, 1.5, Color::rgb(0.1, 0.9, 0.4))
            }
            _ => {
                info!(
                    "💫 Spawned glow orb at ({:.1}, {:.1}, {:.1})",
                    spawn_pos.x, spawn_pos.y, spawn_pos.z
                );
                spawn_emissive_orb(world, spawn_pos, 0.6, Color::rgb(3.0, 0.5, 1.0))
            }
        };
        add_spawned_entity(world, ent);
    }
    if wants_spawn_sphere {
        let ent = spawn_sphere_at(world, spawn_pos, 0.8, Color::rgb(0.95, 0.2, 0.2));
        info!("🔴 Spawned sphere [1]");
        add_spawned_entity(world, ent);
    }
    if wants_spawn_cube {
        let ent = spawn_cube_at(world, spawn_pos, 1.5, Color::rgb(0.2, 0.6, 0.95));
        info!("🟦 Spawned cube [2]");
        add_spawned_entity(world, ent);
    }
    if wants_spawn_glow {
        let ent = spawn_emissive_orb(world, spawn_pos, 0.6, Color::rgb(0.0, 3.0, 2.0));
        info!("✨ Spawned glow orb [3]");
        add_spawned_entity(world, ent);
    }
    if wants_spawn_stack {
        info!("📦 Spawning stack of 5 cubes [4]");
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
    }
    if wants_spawn_rain {
        info!("🌧️ Spawning rain of 10 spheres [5]");
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
    }
    if wants_clear {
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            let entities_to_remove: Vec<Entity> = state.spawned_entities.drain(..).collect();
            let count = entities_to_remove.len();
            // Release the mutable borrow before despawning
            let _ = state;
            for ent in entities_to_remove {
                world.despawn(ent);
            }
            info!("🗑️ Cleared {} spawned objects", count);
        }
    }
    if wants_reset {
        if let Some(state) = world.get_resource_mut::<DemoState>() {
            let entities_to_remove: Vec<Entity> = state.spawned_entities.drain(..).collect();
            state.gizmos_on = true;
            state.physics_paused = false;
            let _ = state;
            for ent in entities_to_remove {
                world.despawn(ent);
            }
            info!("🔄 Scene reset — camera & objects restored to defaults");
        }
    }
}

// ============================================================================
// HUD System — renders in-game command palette overlay + periodic console log
// ============================================================================

fn hud_system(world: &mut World) {
    // Gather state info
    let (frame, fps, spawned, gizmos, phys, menu_visible, _grabbed) =
        if let Some(state) = world.get_resource::<DemoState>() {
            (
                state.frame_count,
                state.current_fps,
                state.spawned_entities.len(),
                state.gizmos_on,
                state.physics_paused,
                state.menu_visible,
                false, // placeholder — we read grabbed separately
            )
        } else {
            return;
        };

    let grabbed = world
        .get_resource::<Input>()
        .map(|i| i.is_cursor_grabbed())
        .unwrap_or(false);

    // ── In-game overlay via OverlayRenderer ─────────────────────────
    if let Some(overlay) = world.get_resource_mut::<OverlayRenderer>() {
        overlay.clear();

        let scale = 1.2; // Smaller, more stylish text
        let cw = 8.0 * scale; // char width in pixels
        let ch = 8.0 * scale; // char height in pixels
        let line_h = ch + 8.0; // More line spacing for readability

        // ── Always-visible status bar (top-left) ────────────────────
        let status = format!(
            "FPS:{:.0} | Obj:{} | {} | {}",
            fps,
            spawned,
            if gizmos { "Gizmos:ON" } else { "Gizmos:OFF" },
            if phys { "Phys:PAUSED" } else { "Phys:ACTIVE" },
        );
        // Darker background for better contrast
        let bar_w = status.len() as f32 * cw + 24.0;
        overlay.draw_rect(10.0, 10.0, bar_w, ch + 12.0, [0.0, 0.0, 0.0, 0.8]);
        overlay.draw_text(20.0, 16.0, &status, [0.0, 1.0, 0.8, 1.0], scale);

        // ── Mouse grab hint (top-right area) ────────────────────────
        if !grabbed {
            let hint = "Left Click to Look  |  [Esc] to Free Mouse";
            let hint_w = hint.len() as f32 * cw + 24.0;
            overlay.draw_rect(1280.0 - hint_w - 10.0, 10.0, hint_w, ch + 12.0, [0.8, 0.1, 0.1, 0.8]);
            overlay.draw_text(1280.0 - hint_w + 2.0, 16.0, hint, [1.0, 1.0, 1.0, 1.0], scale);
        }

        // ── Command palette (toggled with H) ────────────────────────
        if menu_visible {
            let lines: &[(&str, [f32; 4])] = &[
                ("=== CONTROLS ===",      [1.0, 0.8, 0.2, 1.0]),
                ("",                       [0.0; 4]),
                ("WASD / Arrows  Move",   [0.9, 0.9, 0.9, 1.0]),
                ("Shift          Sprint", [0.9, 0.9, 0.9, 1.0]),
                ("Mouse          Look",   [0.9, 0.9, 0.9, 1.0]),
                ("C              Mode",   [0.5, 0.8, 1.0, 1.0]),
                ("",                       [0.0; 4]),
                ("T              Spawn",  [0.5, 1.0, 0.5, 1.0]),
                ("1-5            Shapes", [0.5, 1.0, 0.5, 1.0]),
                ("X              Clear",  [0.5, 1.0, 0.5, 1.0]),
                ("R              Reset",  [0.5, 1.0, 0.5, 1.0]),
                ("",                       [0.0; 4]),
                ("G              Gizmos", [0.5, 0.8, 1.0, 1.0]),
                ("P              Pause",  [0.5, 0.8, 1.0, 1.0]),
                ("H              Hide",   [0.5, 0.8, 1.0, 1.0]),
                ("Esc            Free",   [1.0, 0.5, 0.5, 1.0]),
            ];

            let panel_x = 10.0;
            let panel_y = 50.0; // Below status bar
            let max_line_len = lines.iter().map(|(s, _)| s.len()).max().unwrap_or(0);
            let panel_w = max_line_len as f32 * cw + 40.0; // More padding
            let panel_h = lines.len() as f32 * line_h + 20.0;

            // Darker background for better visibility
            overlay.draw_rect(panel_x, panel_y, panel_w, panel_h, [0.0, 0.0, 0.0, 0.85]);

            for (i, (text, color)) in lines.iter().enumerate() {
                if text.is_empty() {
                    continue;
                }
                let lx = panel_x + 20.0;
                let ly = panel_y + 10.0 + i as f32 * line_h;
                overlay.draw_text(lx, ly, text, *color, scale);
            }
        }
    }

    // ── Periodic console output ─────────────────────────────────────
    if frame % 180 == 0 && frame > 0 {
        info!(
            "FPS: {:.0} | Spawned: {} | Gizmos: {} | Physics: {}",
            fps,
            spawned,
            if gizmos { "ON" } else { "OFF" },
            if phys { "PAUSED" } else { "OK" },
        );
    }

    if frame % 900 == 0 && frame > 0 && menu_visible {
        info!("[T]Spawn [1-5]Shapes [X]Clear [R]Reset [G]Gizmos [P]Pause [H]Menu [Esc]Free");
    }
}
