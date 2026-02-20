//! Tests for Activity Bar click handling
//!
//! These tests verify that clicking activity items correctly updates the active state.

use luminara_editor::activity_bar::ActivityBar;
use luminara_editor::theme::Theme;
use std::sync::Arc;

#[test]
fn test_activity_bar_click_updates_active_index() {
    // Create an activity bar with default theme
    let theme = Arc::new(Theme::default_dark());
    let mut activity_bar = ActivityBar::new(theme);
    
    // Initially, Scene Builder (index 1) should be active
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("scene-builder"));
    
    // Simulate clicking on Global Search (index 0)
    activity_bar.set_active(0);
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("global-search"));
    
    // Simulate clicking on Logic Graph (index 2)
    activity_bar.set_active(2);
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("logic-graph"));
    
    // Simulate clicking on Director (index 3)
    activity_bar.set_active(3);
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("director"));
}

#[test]
fn test_activity_bar_click_out_of_bounds() {
    // Create an activity bar with default theme
    let theme = Arc::new(Theme::default_dark());
    let mut activity_bar = ActivityBar::new(theme);
    
    // Initially, Scene Builder (index 1) should be active
    let initial_active = activity_bar.active_item().map(|i| i.id.clone());
    
    // Try to set an out-of-bounds index (should be ignored)
    activity_bar.set_active(999);
    
    // Active item should remain unchanged
    assert_eq!(activity_bar.active_item().map(|i| i.id.clone()), initial_active);
}

#[test]
fn test_activity_bar_click_same_item() {
    // Create an activity bar with default theme
    let theme = Arc::new(Theme::default_dark());
    let mut activity_bar = ActivityBar::new(theme);
    
    // Set active to index 2
    activity_bar.set_active(2);
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("logic-graph"));
    
    // Click the same item again (should remain active)
    activity_bar.set_active(2);
    assert_eq!(activity_bar.active_item().map(|i| i.id.as_str()), Some("logic-graph"));
}

#[test]
fn test_activity_bar_all_items_clickable() {
    // Create an activity bar with default theme
    let theme = Arc::new(Theme::default_dark());
    let mut activity_bar = ActivityBar::new(theme);
    
    // Expected items in order
    let expected_ids = vec![
        "global-search",
        "scene-builder",
        "logic-graph",
        "director",
        "backend-ai",
        "asset-vault",
        "extensions",
    ];
    
    // Verify all items can be activated by clicking
    for (index, expected_id) in expected_ids.iter().enumerate() {
        activity_bar.set_active(index);
        assert_eq!(
            activity_bar.active_item().map(|i| i.id.as_str()),
            Some(*expected_id),
            "Failed to activate item at index {}",
            index
        );
    }
}

#[test]
fn test_activity_bar_visual_feedback() {
    // Create an activity bar with default theme
    let theme = Arc::new(Theme::default_dark());
    let mut activity_bar = ActivityBar::new(theme);
    
    // Set active to index 0
    activity_bar.set_active(0);
    
    // Verify the active item is returned correctly (visual feedback relies on this)
    let active_item = activity_bar.active_item();
    assert!(active_item.is_some());
    assert_eq!(active_item.unwrap().id, "global-search");
    
    // Change to index 3
    activity_bar.set_active(3);
    
    // Verify the new active item
    let active_item = activity_bar.active_item();
    assert!(active_item.is_some());
    assert_eq!(active_item.unwrap().id, "director");
}
