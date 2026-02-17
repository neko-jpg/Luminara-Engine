//! # Luminara Engine — Fluid Visualization Demo
//!
//! This demo showcases the spectral fluid solver integration with the rendering pipeline:
//! - Real-time fluid simulation using spectral methods
//! - Velocity field visualization with multiple modes
//! - Interactive controls for simulation parameters
//! - GPU-accelerated fluid dynamics
//!
//! **Requirements validated:** 13.3
//!
//! ## Controls:
//! - **1-5**: Switch visualization modes
//!   - 1: Velocity Magnitude
//!   - 2: Velocity Direction
//!   - 3: Vorticity
//!   - 4: Pressure
//!   - 5: Streamlines
//! - **+/-**: Adjust viscosity
//! - **Space**: Pause/Resume simulation
//! - **R**: Reset simulation
//! - **Arrow Keys**: Move camera
//! - **Mouse**: Look around

use log::info;
use luminara::asset::AssetServer;
use luminara::prelude::*;
use luminara::render::{FluidRenderer, FluidVisualizationMode, FluidSolverResource};
use luminara_math::dynamics::BoundaryMethod;
use luminara::core::{Name, ExclusiveMarker, CoreStage, App};
use luminara::input::ActionMap;

mod fluid_controls;
use fluid_controls::{
    fluid_control_system, setup_fluid_input, FluidAction, FluidDemoSettings,
};

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Initialize the engine with required plugins
    let mut app = App::new();
    
    // Add core plugins manually
    app.add_plugins(luminara::platform::PlatformPlugin);
    app.add_plugins(luminara::window::WindowPlugin);
    app.add_plugins(luminara::input::InputPlugin);
    app.add_plugins(luminara::asset::AssetPlugin);
    app.add_plugins(luminara::render::RenderPlugin);

    // Setup input actions for fluid controls
    app.add_startup_system::<ExclusiveMarker>(setup_fluid_input);

    // Initialize fluid demo settings
    app.insert_resource(FluidDemoSettings::default());

    // Initialize fluid solver resource
    app.insert_resource(FluidSolverResource::new());

    // Add fluid control system
    app.add_system::<(
        luminara::core::system::FunctionMarker,
        Res<'static, Input>,
        Res<'static, ActionMap<FluidAction>>,
        ResMut<'static, FluidDemoSettings>,
        Query<'static, &mut FluidRenderer>,
    )>(CoreStage::Update, fluid_control_system);

    // Add fluid simulation systems
    app.add_system::<ExclusiveMarker>(
        CoreStage::Update,
        luminara::render::init_fluid_solvers_system,
    );
    app.add_system::<ExclusiveMarker>(
        CoreStage::Update,
        luminara::render::update_fluid_simulation_system,
    );
    app.add_system::<ExclusiveMarker>(
        CoreStage::Update,
        luminara::render::sync_fluid_textures_system,
    );
    app.add_system::<ExclusiveMarker>(
        CoreStage::Update,
        luminara::render::cleanup_fluid_solvers_system,
    );

    // Add startup system to setup the fluid visualization scene
    app.add_startup_system::<ExclusiveMarker>(setup_fluid_scene);

    // Run the application
    info!("Starting Fluid Visualization Demo...");
    app.run();
}

/// Startup system that creates the fluid visualization scene
fn setup_fluid_scene(world: &mut World) {
    info!("Setting up fluid visualization scene...");

    // Create camera
    let camera = world.spawn();
    world.add_component(camera, Name::new("Camera"));
    world.add_component(
        camera,
        Transform {
            translation: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    );
    world.add_component(
        camera,
        Camera {
            projection: Projection::Orthographic {
                left: -2.0,
                right: 2.0,
                bottom: -2.0,
                top: 2.0,
                near: 0.1,
                far: 100.0,
            },
            clear_color: Color::rgb(0.05, 0.05, 0.1),
            is_active: true,
        },
    );
    world.add_component(camera, Camera3d);

    // Create fluid visualization entity
    let fluid_entity = world.spawn();
    world.add_component(fluid_entity, Name::new("FluidSimulation"));
    world.add_component(
        fluid_entity,
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(4.0, 4.0, 1.0), // Scale to fill view
        },
    );

    // Add fluid renderer component
    let mut fluid_renderer = FluidRenderer::new(128, 128);
    fluid_renderer.viscosity = 0.001;
    fluid_renderer.boundary_method = BoundaryMethod::Periodic;
    fluid_renderer.visualization_mode = FluidVisualizationMode::VelocityMagnitude;

    world.add_component(fluid_entity, fluid_renderer);

    // Create a quad mesh for rendering the fluid
    world.resource_scope::<AssetServer, _, _>(|world, asset_server| {
        let mesh = luminara::render::Mesh::quad();
        let handle = asset_server.add(mesh);
        world.add_component(fluid_entity, handle);
    });

    // Add a simple material (will be overridden by fluid shader)
    world.add_component(
        fluid_entity,
        luminara::render::PbrMaterial {
            albedo: Color::WHITE,
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_texture: None,
            emissive: Color::BLACK,
        },
    );

    info!("Fluid visualization scene setup complete!");
    info!("Controls:");
    info!("  1-5: Switch visualization modes");
    info!("  +/-: Adjust viscosity");
    info!("  Space: Pause/Resume simulation");
    info!("  R: Reset simulation");
}
