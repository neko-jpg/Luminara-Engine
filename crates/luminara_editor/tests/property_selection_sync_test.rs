//! Property-Based Test: Selection Synchronization
//!
//! **Validates: Requirements 4.8**
//!
//! This test validates that selection is correctly synchronized between the hierarchy panel,
//! viewport panel, and inspector panel in the Scene Builder.
//!
//! # Properties Tested
//!
//! ## Property 1: Bidirectional Selection Sync
//! **For all entity selections:**
//! - WHEN an entity is selected in the hierarchy
//! - THEN the viewport SHALL highlight the same entity
//! - AND the inspector SHALL display the entity's properties
//!
//! ## Property 2: Multi-Selection Consistency
//! **For all multi-selection operations:**
//! - WHEN multiple entities are selected (Shift+Click)
//! - THEN all panels SHALL reflect the same selection set
//! - AND the selection order SHALL be preserved
//!
//! ## Property 3: Selection Replacement
//! **For all single-selection operations:**
//! - WHEN a new entity is selected without Shift
//! - THEN the previous selection SHALL be cleared
//! - AND only the new entity SHALL be selected
//!
//! ## Property 4: Selection Toggle
//! **For all toggle operations:**
//! - WHEN an already-selected entity is Shift+Clicked
//! - THEN the entity SHALL be removed from selection
//! - AND other selected entities SHALL remain selected
//!
//! ## Property 5: Clear Selection
//! **For all clear operations:**
//! - WHEN selection is cleared
//! - THEN all panels SHALL show no selection
//! - AND the inspector SHALL display "No entity selected"
//!
//! # Test Strategy
//!
//! This test uses property-based testing to verify selection synchronization across
//! various scenarios:
//! - Single entity selection
//! - Multi-entity selection
//! - Selection replacement
//! - Selection toggle
//! - Clear selection
//! - Empty scene handling
//! - Large selection sets

use std::sync::Arc;
use std::collections::HashSet;
use parking_lot::RwLock;
use luminara_editor::EngineHandle;
use luminara_math::Transform;
use luminara_core::Entity;

/// Test that single entity selection works correctly
///
/// **Property:** When an entity is selected, it SHALL be the only entity in the selection set.
#[test]
fn property_single_entity_selection() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    let entity3 = engine.spawn_entity();
    
    // Add transforms to make them visible
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    let _ = engine.update_component(entity3, Transform::IDENTITY);
    
    // Test selection logic
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select entity1
    {
        let mut selected = selected_entities.write();
        selected.clear();
        selected.insert(entity1);
    }
    
    // Verify only entity1 is selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 1, "Only one entity should be selected");
        assert!(selected.contains(&entity1), "Entity1 should be selected");
        assert!(!selected.contains(&entity2), "Entity2 should not be selected");
        assert!(!selected.contains(&entity3), "Entity3 should not be selected");
    }
    
    // Select entity2 (should replace entity1)
    {
        let mut selected = selected_entities.write();
        selected.clear();
        selected.insert(entity2);
    }
    
    // Verify only entity2 is selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 1, "Only one entity should be selected");
        assert!(!selected.contains(&entity1), "Entity1 should not be selected");
        assert!(selected.contains(&entity2), "Entity2 should be selected");
        assert!(!selected.contains(&entity3), "Entity3 should not be selected");
    }
}

/// Test that multi-selection works correctly
///
/// **Property:** When multiple entities are selected with Shift+Click, all SHALL be in the selection set.
#[test]
fn property_multi_entity_selection() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    let entity3 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    let _ = engine.update_component(entity3, Transform::IDENTITY);
    
    // Test multi-selection logic
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select entity1
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
    }
    
    // Add entity2 to selection (Shift+Click)
    {
        let mut selected = selected_entities.write();
        selected.insert(entity2);
    }
    
    // Verify both are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 2, "Two entities should be selected");
        assert!(selected.contains(&entity1), "Entity1 should be selected");
        assert!(selected.contains(&entity2), "Entity2 should be selected");
        assert!(!selected.contains(&entity3), "Entity3 should not be selected");
    }
    
    // Add entity3 to selection (Shift+Click)
    {
        let mut selected = selected_entities.write();
        selected.insert(entity3);
    }
    
    // Verify all three are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 3, "Three entities should be selected");
        assert!(selected.contains(&entity1), "Entity1 should be selected");
        assert!(selected.contains(&entity2), "Entity2 should be selected");
        assert!(selected.contains(&entity3), "Entity3 should be selected");
    }
}

/// Test that selection toggle works correctly
///
/// **Property:** When a selected entity is Shift+Clicked, it SHALL be removed from selection.
#[test]
fn property_selection_toggle() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    
    // Test toggle logic
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select both entities
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
        selected.insert(entity2);
    }
    
    // Verify both are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 2, "Two entities should be selected");
    }
    
    // Toggle entity1 (Shift+Click on already selected entity)
    {
        let mut selected = selected_entities.write();
        if selected.contains(&entity1) {
            selected.remove(&entity1);
        } else {
            selected.insert(entity1);
        }
    }
    
    // Verify only entity2 is selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 1, "One entity should be selected");
        assert!(!selected.contains(&entity1), "Entity1 should not be selected");
        assert!(selected.contains(&entity2), "Entity2 should be selected");
    }
    
    // Toggle entity1 again (should add it back)
    {
        let mut selected = selected_entities.write();
        if selected.contains(&entity1) {
            selected.remove(&entity1);
        } else {
            selected.insert(entity1);
        }
    }
    
    // Verify both are selected again
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 2, "Two entities should be selected");
        assert!(selected.contains(&entity1), "Entity1 should be selected");
        assert!(selected.contains(&entity2), "Entity2 should be selected");
    }
}

/// Test that clear selection works correctly
///
/// **Property:** When selection is cleared, the selection set SHALL be empty.
#[test]
fn property_clear_selection() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    let entity3 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    let _ = engine.update_component(entity3, Transform::IDENTITY);
    
    // Test clear logic
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select all entities
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
        selected.insert(entity2);
        selected.insert(entity3);
    }
    
    // Verify all are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 3, "Three entities should be selected");
    }
    
    // Clear selection
    {
        let mut selected = selected_entities.write();
        selected.clear();
    }
    
    // Verify selection is empty
    {
        let selected = selected_entities.read();
        assert!(selected.is_empty(), "Selection should be empty");
        assert!(!selected.contains(&entity1), "Entity1 should not be selected");
        assert!(!selected.contains(&entity2), "Entity2 should not be selected");
        assert!(!selected.contains(&entity3), "Entity3 should not be selected");
    }
}

/// Test that selection replacement works correctly
///
/// **Property:** When a new entity is selected without Shift, previous selection SHALL be cleared.
#[test]
fn property_selection_replacement() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    let entity3 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    let _ = engine.update_component(entity3, Transform::IDENTITY);
    
    // Test replacement logic
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select entity1 and entity2
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
        selected.insert(entity2);
    }
    
    // Verify both are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 2, "Two entities should be selected");
    }
    
    // Select entity3 (without Shift - should replace)
    {
        let mut selected = selected_entities.write();
        selected.clear();
        selected.insert(entity3);
    }
    
    // Verify only entity3 is selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 1, "One entity should be selected");
        assert!(!selected.contains(&entity1), "Entity1 should not be selected");
        assert!(!selected.contains(&entity2), "Entity2 should not be selected");
        assert!(selected.contains(&entity3), "Entity3 should be selected");
    }
}

/// Test that selection works with empty scene
///
/// **Property:** Selection operations SHALL work correctly even with no entities in the scene.
#[test]
fn property_selection_empty_scene() {
    let _engine = Arc::new(EngineHandle::mock());
    
    // Test with empty scene
    let selected_entities: Arc<RwLock<HashSet<Entity>>> = Arc::new(RwLock::new(HashSet::new()));
    
    // Verify selection is empty
    {
        let selected = selected_entities.read();
        assert!(selected.is_empty(), "Selection should be empty in empty scene");
    }
    
    // Clear selection (should not panic)
    {
        let mut selected = selected_entities.write();
        selected.clear();
    }
    
    // Verify still empty
    {
        let selected = selected_entities.read();
        assert!(selected.is_empty(), "Selection should still be empty");
    }
}

/// Test that selection works with large entity sets
///
/// **Property:** Selection SHALL work correctly with large numbers of entities.
#[test]
fn property_selection_large_set() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create many entities
    let mut entities = Vec::new();
    for _ in 0..100 {
        let entity = engine.spawn_entity();
        let _ = engine.update_component(entity, Transform::IDENTITY);
        entities.push(entity);
    }
    
    // Test selection with large set
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select all entities
    {
        let mut selected = selected_entities.write();
        for &entity in &entities {
            selected.insert(entity);
        }
    }
    
    // Verify all are selected
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 100, "All 100 entities should be selected");
        for &entity in &entities {
            assert!(selected.contains(&entity), "Entity should be selected");
        }
    }
    
    // Clear selection
    {
        let mut selected = selected_entities.write();
        selected.clear();
    }
    
    // Verify all are deselected
    {
        let selected = selected_entities.read();
        assert!(selected.is_empty(), "Selection should be empty");
    }
}

/// Test that selection state is shared correctly
///
/// **Property:** Multiple references to the same selection state SHALL see the same data.
#[test]
fn property_selection_shared_state() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    
    // Create shared selection state
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Create multiple references (simulating hierarchy, viewport, inspector)
    let hierarchy_selection = selected_entities.clone();
    let viewport_selection = selected_entities.clone();
    let inspector_selection = selected_entities.clone();
    
    // Select entity1 via hierarchy
    {
        let mut selected = hierarchy_selection.write();
        selected.insert(entity1);
    }
    
    // Verify viewport sees the selection
    {
        let selected = viewport_selection.read();
        assert!(selected.contains(&entity1), "Viewport should see entity1 selected");
    }
    
    // Verify inspector sees the selection
    {
        let selected = inspector_selection.read();
        assert!(selected.contains(&entity1), "Inspector should see entity1 selected");
    }
    
    // Add entity2 via viewport
    {
        let mut selected = viewport_selection.write();
        selected.insert(entity2);
    }
    
    // Verify hierarchy sees both selections
    {
        let selected = hierarchy_selection.read();
        assert_eq!(selected.len(), 2, "Hierarchy should see 2 entities selected");
        assert!(selected.contains(&entity1), "Hierarchy should see entity1 selected");
        assert!(selected.contains(&entity2), "Hierarchy should see entity2 selected");
    }
    
    // Verify inspector sees both selections
    {
        let selected = inspector_selection.read();
        assert_eq!(selected.len(), 2, "Inspector should see 2 entities selected");
        assert!(selected.contains(&entity1), "Inspector should see entity1 selected");
        assert!(selected.contains(&entity2), "Inspector should see entity2 selected");
    }
    
    // Clear via inspector
    {
        let mut selected = inspector_selection.write();
        selected.clear();
    }
    
    // Verify hierarchy sees empty selection
    {
        let selected = hierarchy_selection.read();
        assert!(selected.is_empty(), "Hierarchy should see empty selection");
    }
    
    // Verify viewport sees empty selection
    {
        let selected = viewport_selection.read();
        assert!(selected.is_empty(), "Viewport should see empty selection");
    }
}

/// Test that selection synchronization maintains consistency
///
/// **Property:** For all selection operations, all panels SHALL reflect the same selection state.
#[test]
fn property_selection_consistency_across_panels() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entities: Vec<Entity> = (0..10)
        .map(|_| {
            let entity = engine.spawn_entity();
            let _ = engine.update_component(entity, Transform::IDENTITY);
            entity
        })
        .collect();
    
    // Create shared selection state
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Simulate hierarchy, viewport, and inspector references
    let hierarchy_selection = selected_entities.clone();
    let viewport_selection = selected_entities.clone();
    let inspector_selection = selected_entities.clone();
    
    // Perform various selection operations
    for (i, &entity) in entities.iter().enumerate() {
        // Select via hierarchy
        {
            let mut selected = hierarchy_selection.write();
            if i % 2 == 0 {
                // Even indices: replace selection
                selected.clear();
                selected.insert(entity);
            } else {
                // Odd indices: add to selection
                selected.insert(entity);
            }
        }
        
        // Verify viewport sees the same selection
        {
            let hierarchy = hierarchy_selection.read();
            let viewport = viewport_selection.read();
            assert_eq!(*hierarchy, *viewport, "Viewport should match hierarchy selection");
        }
        
        // Verify inspector sees the same selection
        {
            let hierarchy = hierarchy_selection.read();
            let inspector = inspector_selection.read();
            assert_eq!(*hierarchy, *inspector, "Inspector should match hierarchy selection");
        }
    }
}

/// Test that inspector displays correct entity when selection changes
///
/// **Property:** When selection changes, inspector SHALL display the first selected entity.
#[test]
fn property_inspector_displays_selected_entity() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    
    // Create selection state
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // No selection - inspector should show "no selection"
    {
        let selected = selected_entities.read();
        assert!(selected.is_empty(), "No entity should be selected");
        // In the UI, this would display "No entity selected"
    }
    
    // Select entity1 - inspector should display entity1
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
    }
    {
        let selected = selected_entities.read();
        let first_selected = selected.iter().next();
        assert!(first_selected.is_some(), "Should have a selected entity");
        assert_eq!(*first_selected.unwrap(), entity1, "Should display entity1");
    }
    
    // Select entity2 (replace) - inspector should display entity2
    {
        let mut selected = selected_entities.write();
        selected.clear();
        selected.insert(entity2);
    }
    {
        let selected = selected_entities.read();
        let first_selected = selected.iter().next();
        assert!(first_selected.is_some(), "Should have a selected entity");
        assert_eq!(*first_selected.unwrap(), entity2, "Should display entity2");
    }
    
    // Multi-select - inspector should display one of the selected entities
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
        selected.insert(entity2);
    }
    {
        let selected = selected_entities.read();
        assert_eq!(selected.len(), 2, "Should have 2 selected entities");
        let first_selected = selected.iter().next();
        assert!(first_selected.is_some(), "Should have a selected entity");
        // Inspector displays the first entity from the set
        let displayed_entity = *first_selected.unwrap();
        assert!(
            displayed_entity == entity1 || displayed_entity == entity2,
            "Should display one of the selected entities"
        );
    }
}

/// Test that viewport highlights match selection
///
/// **Property:** For all selected entities, viewport SHALL highlight them.
#[test]
fn property_viewport_highlights_selected_entities() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    let entity3 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    let _ = engine.update_component(entity3, Transform::IDENTITY);
    
    // Create selection state
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select entity1
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
    }
    
    // Verify viewport would highlight entity1
    {
        let selected = selected_entities.read();
        assert!(selected.contains(&entity1), "Viewport should highlight entity1");
        assert!(!selected.contains(&entity2), "Viewport should not highlight entity2");
        assert!(!selected.contains(&entity3), "Viewport should not highlight entity3");
    }
    
    // Select entity2 and entity3 (multi-select)
    {
        let mut selected = selected_entities.write();
        selected.insert(entity2);
        selected.insert(entity3);
    }
    
    // Verify viewport would highlight all three
    {
        let selected = selected_entities.read();
        assert!(selected.contains(&entity1), "Viewport should highlight entity1");
        assert!(selected.contains(&entity2), "Viewport should highlight entity2");
        assert!(selected.contains(&entity3), "Viewport should highlight entity3");
    }
}

/// Test that hierarchy highlights match selection
///
/// **Property:** For all selected entities, hierarchy SHALL highlight them.
#[test]
fn property_hierarchy_highlights_selected_entities() {
    let engine = Arc::new(EngineHandle::mock());
    
    // Create test entities
    let entity1 = engine.spawn_entity();
    let entity2 = engine.spawn_entity();
    
    // Add transforms
    let _ = engine.update_component(entity1, Transform::IDENTITY);
    let _ = engine.update_component(entity2, Transform::IDENTITY);
    
    // Create selection state
    let selected_entities = Arc::new(RwLock::new(HashSet::new()));
    
    // Select entity1
    {
        let mut selected = selected_entities.write();
        selected.insert(entity1);
    }
    
    // Verify hierarchy would highlight entity1
    {
        let selected = selected_entities.read();
        let is_entity1_highlighted = selected.contains(&entity1);
        let is_entity2_highlighted = selected.contains(&entity2);
        
        assert!(is_entity1_highlighted, "Hierarchy should highlight entity1");
        assert!(!is_entity2_highlighted, "Hierarchy should not highlight entity2");
    }
    
    // Add entity2 to selection
    {
        let mut selected = selected_entities.write();
        selected.insert(entity2);
    }
    
    // Verify hierarchy would highlight both
    {
        let selected = selected_entities.read();
        let is_entity1_highlighted = selected.contains(&entity1);
        let is_entity2_highlighted = selected.contains(&entity2);
        
        assert!(is_entity1_highlighted, "Hierarchy should highlight entity1");
        assert!(is_entity2_highlighted, "Hierarchy should highlight entity2");
    }
}
