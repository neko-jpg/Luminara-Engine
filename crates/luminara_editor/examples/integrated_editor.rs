//! Integrated Editor Example (Vizia version)
//!
//! This example demonstrates the integration of Luminara Engine's rendering
//! with Vizia for the editor UI.

use luminara_asset::AssetServer;
use luminara_core::{schedule::Schedule, App, AppInterface};
use luminara_editor::core::session::EditorSession;
use luminara_editor::{
    core::window::EditorWindowState, services::engine_bridge::EngineHandle, ui::theme::Theme,
};
use luminara_math::{Color, Transform};
use luminara_render::Camera;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use vizia::prelude::*;

fn main() -> Result<(), ApplicationError> {
    println!("=== Luminara Editor with Integrated Rendering ===");
    println!("Initializing engine and Vizia...");

    let mut engine_app = App::new();

    use luminara_render::RenderPlugin;
    engine_app.add_plugins(RenderPlugin);

    setup_test_scene(&mut engine_app);

    let world = Arc::new(RwLock::new(engine_app.world));
    let schedule = Arc::new(RwLock::new(engine_app.schedule));

    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(
        rt.block_on(luminara_editor::Database::new_memory())
            .expect("Failed to create database"),
    );
    drop(rt);

    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));

    let engine_handle = Arc::new(EngineHandle::new(
        world.clone(),
        asset_server,
        database,
        render_pipeline,
    ));

    let theme = Arc::new(Theme::default_dark());

    let engine_state = Arc::new(RwLock::new(EngineState {
        world: world.clone(),
        schedule: schedule.clone(),
        frame_count: 0,
    }));

    let engine_state_clone = engine_state.clone();
    thread::spawn(move || {
        let mut last_time = std::time::Instant::now();

        loop {
            let now = std::time::Instant::now();
            let _dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            {
                let mut state = engine_state_clone.write();
                let mut world = state.world.write();
                let mut schedule = state.schedule.write();

                let mut time = luminara_core::Time::new();
                time.update();
                world.insert_resource(time);
                schedule.run(&mut world);
                state.frame_count += 1;
            }

            thread::sleep(Duration::from_millis(16));
        }
    });

    Application::new(move |cx| {
        EditorWindowState::new(engine_handle.clone(), theme.clone()).build(cx);
    })
    .title("Luminara Editor - Integrated")
    .inner_size((1200.0, 800.0))
    .run()
}

struct EngineState {
    world: Arc<RwLock<luminara_core::World>>,
    schedule: Arc<RwLock<Schedule>>,
    frame_count: u64,
}

fn setup_test_scene(app: &mut App) {
    use luminara_math::Vec3;
    use luminara_render::{DirectionalLight, PbrMaterial};
    use luminara_scene::scene::Name;

    let world = &mut app.world;

    world.register_component::<Camera>();
    world.register_component::<Transform>();
    world.register_component::<DirectionalLight>();
    world.register_component::<PbrMaterial>();

    let camera = world.spawn();
    world.add_component(camera, Name::new("Main Camera"));
    world.add_component(camera, Transform::from_xyz(0.0, 2.0, 5.0));
    world.add_component(
        camera,
        Camera {
            projection: luminara_render::Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 100.0,
            },
            clear_color: Color::rgb(0.1, 0.1, 0.15),
            is_active: true,
        },
    );

    let light = world.spawn();
    world.add_component(light, Name::new("Sun Light"));
    world.add_component(light, Transform::from_xyz(5.0, 10.0, 5.0));
    world.add_component(
        light,
        DirectionalLight {
            color: Color::rgb(1.0, 0.95, 0.9),
            intensity: 2.0,
            cast_shadows: true,
            shadow_cascade_count: 4,
        },
    );

    println!("Test scene created with camera and light");
}
