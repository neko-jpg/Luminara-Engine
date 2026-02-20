//! Basic editor example
//!
//! This example demonstrates the initialization of the GPUI-based Luminara Editor
//! with a minimal engine setup, SVG asset loading, and command-based keyboard input.

use luminara_editor::{
    EditorWindow, EngineHandle, EditorState, SharedEditorState,
    EditorAssetSource,
};
use luminara_core::App;
use luminara_asset::AssetServer;
use parking_lot::RwLock;
use std::sync::Arc;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use gpui::{
    App as GpuiApp, Bounds, WindowBounds, WindowOptions, px, size, WindowContext, 
    VisualContext as _, 
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
    let database = Arc::new(luminara_editor::Database::memory().expect("Failed to create database"));
    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));
    
    // Create engine handle
    let engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));
    
    // Create shared editor state with generation counter
    let state = Arc::new(RwLock::new(EditorState::new()));
    let shared_state = SharedEditorState::new(state);
    
    // Spawn input monitoring thread (Windows only)
    // The background thread updates SharedEditorState, which increments
    // the generation counter. The GPUI poller in EditorWindow detects
    // the generation change and triggers a re-render.
    #[cfg(target_os = "windows")]
    {
        let input_shared = shared_state.clone();
        thread::spawn(move || {
            println!("Input: Starting keyboard monitoring thread...");
            
            let mut prev_k = false;
            
            loop {
                unsafe {
                    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_K};
                    
                    // Check key states
                    let ctrl = GetAsyncKeyState(VK_CONTROL as i32) < 0;
                    let k = GetAsyncKeyState(VK_K as i32) < 0;
                    
                    // Detect Ctrl+K press (Ctrl held and K just pressed)
                    if ctrl && k && !prev_k {
                        println!("Input: Ctrl+K detected!");
                        
                        // Toggle via SharedEditorState (auto-increments generation counter)
                        input_shared.toggle_global_search();
                        
                        println!("Input: Global Search toggled to {}", input_shared.read().global_search_visible);
                    }
                    
                    prev_k = k;
                }
                
                // Poll at 60Hz
                thread::sleep(Duration::from_millis(16));
            }
        });
    }
    
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
            
            // Open the main editor window with shared state
            let shared = shared_state.clone();
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
                    cx.new_view(|cx| EditorWindow::with_shared_state(engine_handle.clone(), shared, cx))
                },
            ).expect("Failed to open window");
        });
}
