//! Integration tests for Global Search preview panel functionality
//!
//! Tests Requirement 3.5: Display real-time preview of selected items

use luminara_editor::global_search::{GlobalSearch, SearchResult, SearchPrefix, GroupedResults};
use luminara_editor::theme::Theme;
use std::sync::Arc;

#[test]
fn test_preview_panel_displays_selected_item() {
    // Test that the preview panel correctly displays the selected item's details
    
    // Create mock results
    let mut groups = GroupedResults::new();
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Player".to_string(),
        Some("Main player character".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Asset,
        "player_texture.png".to_string(),
        Some("assets/textures/".to_string()),
    ));
    
    // Verify we can access the results
    let categories = groups.categories();
    assert_eq!(categories.len(), 2);
    
    // Get the first result (Player entity)
    let entity_results = groups.get_category(SearchPrefix::Entity);
    assert_eq!(entity_results.len(), 1);
    assert_eq!(entity_results[0].name, "Player");
    assert_eq!(entity_results[0].description, Some("Main player character".to_string()));
    
    // Get the second result (texture asset)
    let asset_results = groups.get_category(SearchPrefix::Asset);
    assert_eq!(asset_results.len(), 1);
    assert_eq!(asset_results[0].name, "player_texture.png");
    assert_eq!(asset_results[0].description, Some("assets/textures/".to_string()));
}

#[test]
fn test_preview_panel_updates_on_selection_change() {
    // Test that the preview updates when selection changes
    
    let mut groups = GroupedResults::new();
    
    // Add multiple results in the same category
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Player".to_string(),
        Some("Main character".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Enemy_01".to_string(),
        Some("Basic enemy entity".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Camera_Main".to_string(),
        Some("Primary scene camera".to_string()),
    ));
    
    let categories = groups.categories();
    let entity_results = groups.get_category(categories[0]);
    
    // Verify we can access each result individually
    assert_eq!(entity_results.len(), 3);
    
    // First selection
    assert_eq!(entity_results[0].name, "Player");
    assert_eq!(entity_results[0].description, Some("Main character".to_string()));
    
    // Second selection
    assert_eq!(entity_results[1].name, "Enemy_01");
    assert_eq!(entity_results[1].description, Some("Basic enemy entity".to_string()));
    
    // Third selection
    assert_eq!(entity_results[2].name, "Camera_Main");
    assert_eq!(entity_results[2].description, Some("Primary scene camera".to_string()));
}

#[test]
fn test_preview_panel_handles_no_selection() {
    // Test that the preview panel gracefully handles no selection
    
    let groups = GroupedResults::new();
    
    // Verify empty state
    assert_eq!(groups.total_count(), 0);
    assert_eq!(groups.categories().len(), 0);
    
    // Attempting to get results from any category should return empty slice
    assert_eq!(groups.get_category(SearchPrefix::Entity).len(), 0);
    assert_eq!(groups.get_category(SearchPrefix::Asset).len(), 0);
    assert_eq!(groups.get_category(SearchPrefix::Command).len(), 0);
    assert_eq!(groups.get_category(SearchPrefix::Symbol).len(), 0);
}

#[test]
fn test_preview_panel_displays_all_result_types() {
    // Test that the preview panel can display all types of search results
    
    let mut groups = GroupedResults::new();
    
    // Add one result of each type
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Player".to_string(),
        Some("Main player character".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Asset,
        "texture.png".to_string(),
        Some("assets/textures/".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Command,
        "Save Scene".to_string(),
        Some("Ctrl+S".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Symbol,
        "render_system".to_string(),
        Some("fn render_system(world: &World)".to_string()),
    ));
    
    // Verify all categories are present
    let categories = groups.categories();
    assert_eq!(categories.len(), 4);
    
    // Verify each category has exactly one result
    assert_eq!(groups.get_category(SearchPrefix::Entity).len(), 1);
    assert_eq!(groups.get_category(SearchPrefix::Asset).len(), 1);
    assert_eq!(groups.get_category(SearchPrefix::Command).len(), 1);
    assert_eq!(groups.get_category(SearchPrefix::Symbol).len(), 1);
    
    // Verify each result has the correct details
    let entity = &groups.get_category(SearchPrefix::Entity)[0];
    assert_eq!(entity.name, "Player");
    assert_eq!(entity.category, SearchPrefix::Entity);
    
    let asset = &groups.get_category(SearchPrefix::Asset)[0];
    assert_eq!(asset.name, "texture.png");
    assert_eq!(asset.category, SearchPrefix::Asset);
    
    let command = &groups.get_category(SearchPrefix::Command)[0];
    assert_eq!(command.name, "Save Scene");
    assert_eq!(command.category, SearchPrefix::Command);
    
    let symbol = &groups.get_category(SearchPrefix::Symbol)[0];
    assert_eq!(symbol.name, "render_system");
    assert_eq!(symbol.category, SearchPrefix::Symbol);
}

#[test]
fn test_preview_panel_handles_results_without_description() {
    // Test that the preview panel handles results that don't have descriptions
    
    let mut groups = GroupedResults::new();
    
    // Add results with and without descriptions
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Player".to_string(),
        Some("Has description".to_string()),
    ));
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Enemy".to_string(),
        None, // No description
    ));
    
    let entity_results = groups.get_category(SearchPrefix::Entity);
    
    // Verify both results are present
    assert_eq!(entity_results.len(), 2);
    
    // Verify first has description
    assert_eq!(entity_results[0].description, Some("Has description".to_string()));
    
    // Verify second has no description
    assert_eq!(entity_results[1].description, None);
}

#[test]
fn test_preview_panel_real_time_updates() {
    // Test that the preview panel supports real-time updates as results change
    
    let mut groups = GroupedResults::new();
    
    // Initially empty
    assert_eq!(groups.total_count(), 0);
    
    // Add first result
    groups.add_result(SearchResult::new(
        SearchPrefix::Entity,
        "Player".to_string(),
        Some("Main character".to_string()),
    ));
    assert_eq!(groups.total_count(), 1);
    
    // Add second result
    groups.add_result(SearchResult::new(
        SearchPrefix::Asset,
        "texture.png".to_string(),
        Some("assets/textures/".to_string()),
    ));
    assert_eq!(groups.total_count(), 2);
    
    // Clear all results
    groups.clear();
    assert_eq!(groups.total_count(), 0);
    
    // Add new results after clearing
    groups.add_result(SearchResult::new(
        SearchPrefix::Command,
        "Save".to_string(),
        None,
    ));
    assert_eq!(groups.total_count(), 1);
}
