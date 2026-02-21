//! Basic editor example
//!
//! This example demonstrates the initialization of the GPUI-based Luminara Editor
//! with a minimal engine setup, SVG asset loading, and command-based keyboard input.

use luminara_editor::{
    EditorWindow, EngineHandle, EditorStateManager,
    EditorAssetSource,
};
use luminara_editor::core::session::EditorSession;
use luminara_core::App;
use luminara_asset::AssetServer;
use parking_lot::RwLock;
use std::sync::Arc;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use gpui::{
    App as GpuiApp, Bounds, WindowBounds, WindowOptions, px, size, WindowContext, 
    VisualContext as _, Context,
};

fn main() {
    // Custom initialization of Luminara Engine
    let mut engine_app = App::new();
    
    // Setup some dummy entities to test the Hierarchy
    use luminara_scene::scene::Name;
    use luminara_scene::hierarchy::{Parent, Children};
    
    let camera = engine_app.world.spawn();
    let _ = engine_app.world.add_component(camera, Name::new("Main Camera"));
    
    let light = engine_app.world.spawn();
    let _ = engine_app.world.add_component(light, Name::new("Directional Light"));
    
    let player = engine_app.world.spawn();
    let _ = engine_app.world.add_component(player, Name::new("Player Character"));
    
    let mesh = engine_app.world.spawn();
    let _ = engine_app.world.add_component(mesh, Name::new("Body Mesh"));
    let _ = engine_app.world.add_component(mesh, Parent(player));
    let _ = engine_app.world.add_component(player, Children(vec![mesh]));

    let world = Arc::new(RwLock::new(engine_app.world));
    
    // Initialize subsystems
    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));
    // Use a temporary Tokio runtime to initialize the database since it requires a reactor
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(rt.block_on(luminara_editor::Database::new_memory()).expect("Failed to create database"));
    drop(rt);

    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));
    
    // Create engine handle
    let engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));
    
    // Create editor state manager (will be initialized as a Model inside GPUI app)
    // For now, we just pass the DB handle if available
    let db_handle = engine_handle.database().clone();
    
    // We'll create the model inside the GpuiApp::run block below
    
    // Spawn input monitoring thread (Windows only)
    // The background thread updates SharedEditorState, which increments
    // the generation counter. The GPUI poller in EditorWindow detects
    // the generation change and triggers a re-render.
    // Background monitoring thread disabled during refactor to Local-First Model architecture.
    // Native GPUI actions are now preferred for keyboard shortcuts.
    /*
    #[cfg(target_os = "windows")]
    ...
    */
    
    // Determine the assets directory path
    // When running from the crate directory, assets are at ./assets
    let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let asset_source = EditorAssetSource::new(assets_path);
    
    // Initialize and run GPUI application with asset source for SVG icons
    GpuiApp::new()
        .with_assets(asset_source)
        .run(move |cx| {
            // Create centered window bounds
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            
            // Create the state manager model
            let db_clone = db_handle.clone();
            let engine_clone = engine_handle.clone();
            let state_manager = cx.new_model(|cx| {
                let mut state = EditorStateManager::new(EditorSession::default(), Some(db_clone));
                state.set_engine_handle(engine_clone, cx);
                state
            });

            // Open the main editor window
            let _window = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Luminara Editor".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    window_background: gpui::WindowBackgroundAppearance::Opaque,
                    focus: true,
                    show: true,
                    ..Default::default()
                },
                move |cx: &mut WindowContext| {
                    cx.new_view(|cx| EditorWindow::with_state_manager(engine_handle.clone(), state_manager.clone(), cx))
                },
            ).expect("Failed to open window");
        });
}
