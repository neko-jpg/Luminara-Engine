//! Hybrid Editor with Separate 3D Window (Vizia version)
//!
//! This example demonstrates running the engine's 3D rendering in a separate window
//! while Vizia handles the editor UI in another window.

use parking_lot::RwLock;
use std::sync::Arc;

use luminara_asset::AssetServer;
use luminara_core::{App, AppInterface};
use luminara_math::{Color, Transform};
use luminara_render::{Camera, DirectionalLight, GpuContext, RenderPlugin};
use luminara_scene::scene::Name;

fn main() {
    println!("=== Luminara Editor - Hybrid 3D + UI ===\n");

    let _ = env_logger::try_init();

    println!("[1/3] Creating 3D rendering window...");

    let mut app = App::new();
    app.add_plugins(RenderPlugin);

    setup_scene(&mut app);

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

    let window_3d = event_loop
        .create_window(
            winit::window::WindowAttributes::default()
                .with_title("Luminara 3D Viewport")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
        )
        .expect("Failed to create 3D window");

    let gpu_context = match GpuContext::new_from_winit(&window_3d) {
        Ok(ctx) => Arc::new(RwLock::new(ctx)),
        Err(e) => {
            eprintln!("Failed to create GPU context: {:?}", e);
            return;
        }
    };

    println!("[2/3] GPU context created successfully");

    let window_3d = Arc::new(window_3d);
    let gpu_context_3d = gpu_context.clone();
    let clear_color = Color::rgb(0.1, 0.12, 0.15);
    let app_ptr = Arc::new(RwLock::new(app));

    println!("[3/3] Starting Vizia editor UI...");

    std::thread::spawn(move || {
        run_vizia_editor();
    });

    println!("\n=== Running 3D Render Loop ===");
    println!("3D window: Luminara 3D Viewport");
    println!("UI window: Separate Vizia window\n");

    let mut last_time = std::time::Instant::now();
    let app_loop = app_ptr.clone();

    run_event_loop(
        event_loop,
        window_3d,
        gpu_context_3d,
        clear_color,
        app_loop,
        last_time,
    );
}

fn run_event_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    window_3d: Arc<winit::window::Window>,
    gpu_context_3d: Arc<RwLock<GpuContext>>,
    clear_color: Color,
    _app_loop: Arc<RwLock<App>>,
    last_time: std::time::Instant,
) {
    use winit::event::WindowEvent;
    use winit::event_loop::ControlFlow;

    event_loop.set_control_flow(ControlFlow::Poll);

    struct AppState {
        window_3d: Arc<winit::window::Window>,
        gpu_context_3d: Arc<RwLock<GpuContext>>,
        clear_color: Color,
        last_time: std::time::Instant,
    }

    impl winit::application::ApplicationHandler<()> for AppState {
        fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

        fn window_event(
            &mut self,
            _event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            if window_id == self.window_3d.id() {
                if let WindowEvent::CloseRequested = event {
                    println!("3D window closed, exiting...");
                    std::process::exit(0);
                }
            }
        }

        fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
            let now = std::time::Instant::now();
            let dt = now.duration_since(self.last_time).as_secs_f32();
            self.last_time = now;

            render_3d_scene(&self.window_3d, &self.gpu_context_3d, self.clear_color, dt);
        }
    }

    let mut state = AppState {
        window_3d,
        gpu_context_3d,
        clear_color,
        last_time,
    };

    event_loop
        .run_app(&mut state)
        .expect("Failed to run event loop");
}

fn render_3d_scene(
    window: &Arc<winit::window::Window>,
    gpu: &Arc<RwLock<GpuContext>>,
    clear_color: Color,
    _dt: f32,
) {
    let gpu = gpu.read();

    let frame = match gpu.surface.get_current_texture() {
        Ok(f) => f,
        Err(e) => {
            log::warn!("Failed to get frame: {:?}", e);
            return;
        }
    };

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Encoder"),
        });

    let clear_color_wgpu = wgpu::Color {
        r: clear_color.r as f64,
        g: clear_color.g as f64,
        b: clear_color.b as f64,
        a: 1.0,
    };

    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color_wgpu),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    frame.present();
    window.request_redraw();
}

fn run_vizia_editor() {
    use luminara_editor::core::window::EditorWindowState;
    use luminara_editor::services::engine_bridge::EngineHandle;
    use luminara_editor::ui::theme::Theme;
    use std::path::PathBuf;
    use vizia::prelude::*;

    let world = Arc::new(RwLock::new(luminara_core::World::new()));
    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(
        rt.block_on(luminara_editor::Database::new_memory())
            .expect("Failed to create database"),
    );
    drop(rt);

    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));

    let engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));

    let theme = Arc::new(Theme::default_dark());

    Application::new(move |cx| {
        EditorWindowState::new(theme.clone()).build(cx);
    })
    .title("Luminara Editor")
    .inner_size((1200, 800))
    .run()
    .expect("Failed to run Vizia application");
}

fn setup_scene(app: &mut App) {
    let world = &mut app.world;

    world.register_component::<Camera>();
    world.register_component::<Transform>();
    world.register_component::<DirectionalLight>();

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
            clear_color: Color::rgb(0.1, 0.12, 0.15),
            is_active: true,
        },
    );

    let light = world.spawn();
    world.add_component(light, Name::new("Sun Light"));
    world.add_component(light, Transform::from_xyz(5.0, 10.0, 5.0));
    let _ = world.add_component(
        light,
        DirectionalLight {
            color: Color::rgb(1.0, 0.95, 0.9),
            intensity: 2.0,
            cast_shadows: true,
            shadow_cascade_count: 4,
        },
    );

    println!("Scene setup complete: Camera + Directional Light");
}
