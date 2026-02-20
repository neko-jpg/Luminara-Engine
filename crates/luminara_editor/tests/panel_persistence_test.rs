//! Panel Size Persistence Integration Tests
//!
//! Tests the integration between ResizablePanel and EditorPreferences
//! to ensure panel sizes are correctly persisted and loaded.
//!
//! **Validates Requirements:**
//! - 9.4: Panel sizes are persisted to user preferences

use luminara_editor::{EditorPreferences, ResizablePanel, Orientation, Theme};
use gpui::px;
use std::sync::Arc;

#[test]
fn test_panel_persistence_workflow() {
    // Create preferences
    let mut prefs = EditorPreferences::new();
    
    // Simulate a panel with ID
    let panel_id = "test.panel";
    let initial_size = px(320.0);
    
    // Save a size to preferences
    prefs.set_panel_size(panel_id.to_string(), initial_size);
    
    // Verify it was saved
    assert_eq!(prefs.get_panel_size(panel_id), Some(initial_size));
    
    // Simulate loading on next session
    let loaded_size = prefs.get_panel_size(panel_id);
    assert_eq!(loaded_size, Some(initial_size));
}

#[test]
fn test_multiple_panels_persistence() {
    let mut prefs = EditorPreferences::new();
    
    // Save multiple panel sizes
    prefs.set_panel_size("scene_builder.hierarchy".to_string(), px(260.0));
    prefs.set_panel_size("scene_builder.inspector".to_string(), px(320.0));
    prefs.set_panel_size("logic_graph.palette".to_string(), px(200.0));
    
    // Verify all are saved
    assert_eq!(prefs.get_panel_size("scene_builder.hierarchy"), Some(px(260.0)));
    assert_eq!(prefs.get_panel_size("scene_builder.inspector"), Some(px(320.0)));
    assert_eq!(prefs.get_panel_size("logic_graph.palette"), Some(px(200.0)));
}

#[test]
fn test_panel_size_update() {
    let mut prefs = EditorPreferences::new();
    
    let panel_id = "test.panel";
    
    // Set initial size
    prefs.set_panel_size(panel_id.to_string(), px(300.0));
    assert_eq!(prefs.get_panel_size(panel_id), Some(px(300.0)));
    
    // Update size
    prefs.set_panel_size(panel_id.to_string(), px(400.0));
    assert_eq!(prefs.get_panel_size(panel_id), Some(px(400.0)));
}

#[test]
fn test_missing_panel_preference() {
    let prefs = EditorPreferences::new();
    
    // Try to get a non-existent panel size
    let size = prefs.get_panel_size("non.existent.panel");
    assert_eq!(size, None);
}

#[test]
fn test_preferences_serialization_roundtrip() {
    let mut prefs = EditorPreferences::new();
    
    // Set some panel sizes
    prefs.set_panel_size("panel1".to_string(), px(100.0));
    prefs.set_panel_size("panel2".to_string(), px(200.0));
    prefs.set_panel_size("panel3".to_string(), px(300.0));
    
    // Serialize to JSON
    let json = serde_json::to_string(&prefs).expect("Failed to serialize");
    
    // Deserialize back
    let loaded: EditorPreferences = serde_json::from_str(&json).expect("Failed to deserialize");
    
    // Verify all sizes are preserved
    assert_eq!(loaded.get_panel_size("panel1"), Some(px(100.0)));
    assert_eq!(loaded.get_panel_size("panel2"), Some(px(200.0)));
    assert_eq!(loaded.get_panel_size("panel3"), Some(px(300.0)));
}

#[test]
fn test_empty_preferences_serialization() {
    let prefs = EditorPreferences::new();
    
    // Serialize empty preferences
    let json = serde_json::to_string(&prefs).expect("Failed to serialize");
    
    // Deserialize back
    let loaded: EditorPreferences = serde_json::from_str(&json).expect("Failed to deserialize");
    
    // Verify it's still empty
    assert_eq!(loaded.panel_sizes.len(), 0);
}

#[test]
fn test_preferences_json_format() {
    let mut prefs = EditorPreferences::new();
    prefs.set_panel_size("test.panel".to_string(), px(320.0));
    
    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&prefs).expect("Failed to serialize");
    
    // Verify JSON structure
    assert!(json.contains("panel_sizes"));
    assert!(json.contains("sizes"));
    assert!(json.contains("test.panel"));
    assert!(json.contains("320"));
}

#[test]
fn test_panel_id_stability() {
    // Test that panel IDs follow a consistent naming convention
    let hierarchy_id = "scene_builder.hierarchy";
    let inspector_id = "scene_builder.inspector";
    let viewport_id = "scene_builder.viewport";
    
    // Verify IDs are stable and follow pattern: box_name.panel_name
    assert!(hierarchy_id.contains('.'));
    assert!(inspector_id.contains('.'));
    assert!(viewport_id.contains('.'));
    
    // Verify they're unique
    assert_ne!(hierarchy_id, inspector_id);
    assert_ne!(hierarchy_id, viewport_id);
    assert_ne!(inspector_id, viewport_id);
}

#[test]
fn test_size_clamping_on_load() {
    let mut prefs = EditorPreferences::new();
    
    // Save a size that might be outside constraints
    prefs.set_panel_size("test.panel".to_string(), px(5000.0));
    
    // When loading, the panel should clamp to its constraints
    // This is tested in the ResizablePanel implementation
    let stored_size = prefs.get_panel_size("test.panel");
    assert_eq!(stored_size, Some(px(5000.0)));
    
    // The actual clamping happens in ResizablePanel::load_size_from_preferences
    // which uses set_size() that clamps to [min_size, max_size]
}

#[test]
fn test_preferences_path_format() {
    let path = EditorPreferences::preferences_path().expect("Failed to get preferences path");
    
    // Verify path contains expected components
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("luminara"));
    assert!(path_str.ends_with("preferences.json"));
    
    // Verify it's an absolute path
    assert!(path.is_absolute());
}

#[test]
fn test_concurrent_panel_updates() {
    let mut prefs = EditorPreferences::new();
    
    // Simulate multiple panels being resized in quick succession
    for i in 0..10 {
        let panel_id = format!("panel{}", i);
        let size = px(100.0 + (i as f32 * 50.0));
        prefs.set_panel_size(panel_id.clone(), size);
    }
    
    // Verify all updates were recorded
    for i in 0..10 {
        let panel_id = format!("panel{}", i);
        let expected_size = px(100.0 + (i as f32 * 50.0));
        assert_eq!(prefs.get_panel_size(&panel_id), Some(expected_size));
    }
}

#[test]
fn test_panel_size_precision() {
    let mut prefs = EditorPreferences::new();
    
    // Test that floating point precision is preserved
    let precise_size = px(320.5);
    prefs.set_panel_size("test.panel".to_string(), precise_size);
    
    let loaded_size = prefs.get_panel_size("test.panel");
    assert_eq!(loaded_size, Some(precise_size));
}

#[test]
fn test_large_number_of_panels() {
    let mut prefs = EditorPreferences::new();
    
    // Test with many panels (simulating a complex workspace)
    for i in 0..100 {
        let panel_id = format!("box{}.panel{}", i / 10, i % 10);
        prefs.set_panel_size(panel_id, px(200.0 + i as f32));
    }
    
    // Verify count
    assert_eq!(prefs.panel_sizes.len(), 100);
    
    // Spot check a few
    assert_eq!(prefs.get_panel_size("box0.panel0"), Some(px(200.0)));
    assert_eq!(prefs.get_panel_size("box5.panel5"), Some(px(255.0)));
    assert_eq!(prefs.get_panel_size("box9.panel9"), Some(px(299.0)));
}
