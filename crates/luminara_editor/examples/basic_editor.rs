//! Basic editor example
//!
//! This example demonstrates the initialization of the GPUI-based Luminara Editor
//! with the Bevy backend integration.

use luminara_editor::{
    EditorWindow, EngineHandle, EditorStateManager,
    EditorAssetSource,
    services::bevy_bridge::BevyBridge,
};
use luminara_editor::core::session::EditorSession;
use luminara_asset::AssetServer;
use luminara_db::LuminaraDatabase as Database;
use std::sync::Arc;
use std::path::PathBuf;
use gpui::{
    App as GpuiApp, Bounds, WindowBounds, WindowOptions, px, size, WindowContext, 
    VisualContext as _, Context,
};

fn main() {
    // Initialize Bevy Bridge (starts Bevy in background thread)
    println!("Starting Bevy backend...");
    let (bridge, _bevy_thread) = BevyBridge::new();
    let bridge = Arc::new(bridge);
    
    // Initialize subsystems
    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));

    // Initialize Database (using Tokio runtime)
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(rt.block_on(Database::new_memory()).expect("Failed to create database"));
    drop(rt);

    // Create engine handle
    let engine_handle = Arc::new(EngineHandle::new(
        bridge.clone(),
        asset_server,
        database,
    ));
    
    // Send a test command to log something in Bevy
    engine_handle.execute_command(|_world| {
        println!("Bevy World accessed from Editor Command!");
    });
    
    // Determine the assets directory path
    let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let asset_source = EditorAssetSource::new(assets_path);
    
    println!("Starting GPUI frontend...");

    // Initialize and run GPUI application
    GpuiApp::new()
        .with_assets(asset_source)
        .run(move |cx| {
            // Create centered window bounds
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            
            // Create the state manager model
            let db_clone = engine_handle.database().clone();
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
                        title: Some("Luminara Editor [Bevy Backend]".into()),
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
