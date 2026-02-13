use luminara::prelude::*;

#[test]
fn test_default_plugins_bundle_initialization() {
    // Validates: Requirements 9.5 - DefaultPlugins bundle registers all core plugins
    let mut app = App::new();
    
    // Add DefaultPlugins bundle
    app.add_plugins(DefaultPlugins);
    
    // Verify all expected plugins are registered in the correct order
    let plugin_order = app.plugin_order();
    
    // Expected plugins in order:
    // 1. PlatformPlugin
    // 2. DiagnosticPlugin
    // 3. WindowPlugin
    // 4. InputPlugin
    // 5. ScenePlugin
    // 6. AssetPlugin
    // 7. RenderPlugin
    // 8. PhysicsPlugin
    // 9. AudioPlugin
    
    assert!(plugin_order.len() >= 9, "DefaultPlugins should register at least 9 plugins");
    
    // Verify specific plugins are present
    assert!(app.has_plugin("PlatformPlugin"), "PlatformPlugin should be registered");
    assert!(app.has_plugin("DiagnosticPlugin"), "DiagnosticPlugin should be registered");
    assert!(app.has_plugin("WindowPlugin"), "WindowPlugin should be registered");
    assert!(app.has_plugin("InputPlugin"), "InputPlugin should be registered");
    assert!(app.has_plugin("ScenePlugin"), "ScenePlugin should be registered");
    assert!(app.has_plugin("AssetPlugin"), "AssetPlugin should be registered");
    assert!(app.has_plugin("RenderPlugin"), "RenderPlugin should be registered");
    assert!(app.has_plugin("PhysicsPlugin"), "PhysicsPlugin should be registered");
    assert!(app.has_plugin("AudioPlugin"), "AudioPlugin should be registered");
}

#[test]
fn test_default_plugins_initialization_order() {
    // Validates: Requirements 9.5 - Plugins are initialized in correct dependency order
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    let plugin_order = app.plugin_order();
    
    // Find indices of key plugins
    let scene_idx = plugin_order.iter().position(|p| p == "ScenePlugin");
    let asset_idx = plugin_order.iter().position(|p| p == "AssetPlugin");
    let render_idx = plugin_order.iter().position(|p| p == "RenderPlugin");
    let physics_idx = plugin_order.iter().position(|p| p == "PhysicsPlugin");
    let audio_idx = plugin_order.iter().position(|p| p == "AudioPlugin");
    
    // Verify all plugins are present
    assert!(scene_idx.is_some(), "ScenePlugin should be registered");
    assert!(asset_idx.is_some(), "AssetPlugin should be registered");
    assert!(render_idx.is_some(), "RenderPlugin should be registered");
    assert!(physics_idx.is_some(), "PhysicsPlugin should be registered");
    assert!(audio_idx.is_some(), "AudioPlugin should be registered");
    
    let scene_idx = scene_idx.unwrap();
    let asset_idx = asset_idx.unwrap();
    let render_idx = render_idx.unwrap();
    let physics_idx = physics_idx.unwrap();
    let audio_idx = audio_idx.unwrap();
    
    // Verify dependency order:
    // - ScenePlugin before RenderPlugin and PhysicsPlugin (provides transform hierarchy)
    // - AssetPlugin before RenderPlugin and AudioPlugin (provides asset loading)
    assert!(scene_idx < render_idx, "ScenePlugin should be initialized before RenderPlugin");
    assert!(scene_idx < physics_idx, "ScenePlugin should be initialized before PhysicsPlugin");
    assert!(asset_idx < render_idx, "AssetPlugin should be initialized before RenderPlugin");
    assert!(asset_idx < audio_idx, "AssetPlugin should be initialized before AudioPlugin");
}

#[test]
fn test_default_plugins_not_duplicated() {
    // Validates: Requirements 9.1, 9.5 - Each plugin in DefaultPlugins is built only once
    let mut app = App::new();
    
    // Add DefaultPlugins twice
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultPlugins);
    
    let plugin_order = app.plugin_order();
    
    // Count occurrences of each plugin
    let scene_count = plugin_order.iter().filter(|p| *p == "ScenePlugin").count();
    let asset_count = plugin_order.iter().filter(|p| *p == "AssetPlugin").count();
    let render_count = plugin_order.iter().filter(|p| *p == "RenderPlugin").count();
    let physics_count = plugin_order.iter().filter(|p| *p == "PhysicsPlugin").count();
    let audio_count = plugin_order.iter().filter(|p| *p == "AudioPlugin").count();
    
    // Each plugin should appear only once
    assert_eq!(scene_count, 1, "ScenePlugin should be registered only once");
    assert_eq!(asset_count, 1, "AssetPlugin should be registered only once");
    assert_eq!(render_count, 1, "RenderPlugin should be registered only once");
    assert_eq!(physics_count, 1, "PhysicsPlugin should be registered only once");
    assert_eq!(audio_count, 1, "AudioPlugin should be registered only once");
}
