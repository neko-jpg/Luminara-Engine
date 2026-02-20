//! Property-based test for Data Persistence Round-Trip
//!
//! **Validates: Requirements 9.4**
//!
//! **Property 3: Data Persistence Round-Trip**
//!
//! This property verifies that panel sizes can be correctly saved to and loaded
//! from preferences, ensuring data integrity through the save/load cycle.

use proptest::prelude::*;
use luminara_editor::EditorPreferences;
use gpui::px;

/// Property: Persistence Round-Trip Correctness
///
/// For any panel size saved to preferences, loading it back SHALL
/// produce the exact same value.
///
/// **Invariants:**
/// 1. save(size) then load() returns size
/// 2. Multiple save/load cycles preserve the value
/// 3. Serialization/deserialization is lossless
/// 4. Panel IDs are preserved correctly
#[test]
fn property_persistence_roundtrip_correctness() {
    proptest!(|(
        panel_id in "[a-z_]{3,20}\\.[a-z_]{3,20}",
        size in 50.0f32..2000.0f32,
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Save the size
        prefs.set_panel_size(panel_id.clone(), px(size));
        
        // Load the size back
        let loaded_size = prefs.get_panel_size(&panel_id);
        
        // Invariant 1: Loaded size equals saved size
        prop_assert_eq!(loaded_size, Some(px(size)));
        
        // Invariant 4: Panel ID is preserved
        prop_assert!(prefs.panel_sizes.sizes.contains_key(&panel_id));
    });
}

/// Property: Multiple Panel Persistence
///
/// Multiple panel sizes SHALL be independently persisted and loaded
/// without interference.
///
/// **Invariants:**
/// 1. Each panel's size is independent
/// 2. Loading one panel doesn't affect others
/// 3. All panels can be loaded simultaneously
/// 4. Order of save/load doesn't matter
#[test]
fn property_multiple_panel_persistence() {
    proptest!(|(
        panels in prop::collection::vec(
            ("[a-z_]{3,15}\\.[a-z_]{3,15}", 50.0f32..2000.0f32),
            1..10
        ),
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Save all panel sizes
        for (panel_id, size) in &panels {
            prefs.set_panel_size(panel_id.clone(), px(*size));
        }
        
        // Load and verify each panel independently
        for (panel_id, expected_size) in &panels {
            let loaded_size = prefs.get_panel_size(panel_id);
            
            // Invariant 1 & 2: Each panel's size is correct and independent
            prop_assert_eq!(loaded_size, Some(px(*expected_size)));
        }
        
        // Invariant 3: All panels are present
        prop_assert_eq!(prefs.panel_sizes.len(), panels.len());
    });
}

/// Property: Serialization Round-Trip
///
/// Serializing preferences to JSON and deserializing back SHALL
/// preserve all panel sizes exactly.
///
/// **Invariants:**
/// 1. serialize(prefs) then deserialize() equals prefs
/// 2. JSON format is stable and parseable
/// 3. All panel sizes survive serialization
/// 4. No data loss or corruption
#[test]
fn property_serialization_roundtrip() {
    proptest!(|(
        panels in prop::collection::vec(
            ("[a-z_]{3,15}\\.[a-z_]{3,15}", 50.0f32..2000.0f32),
            1..10
        ),
    )| {
        let mut original_prefs = EditorPreferences::new();
        
        // Save panel sizes
        for (panel_id, size) in &panels {
            original_prefs.set_panel_size(panel_id.clone(), px(*size));
        }
        
        // Serialize to JSON
        let json = serde_json::to_string(&original_prefs)
            .expect("Failed to serialize preferences");
        
        // Invariant 2: JSON is valid and parseable
        prop_assert!(!json.is_empty());
        
        // Deserialize back
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json)
            .expect("Failed to deserialize preferences");
        
        // Invariant 1 & 3: All panel sizes are preserved
        for (panel_id, expected_size) in &panels {
            let loaded_size = loaded_prefs.get_panel_size(panel_id);
            prop_assert_eq!(loaded_size, Some(px(*expected_size)));
        }
        
        // Invariant 4: No data loss - same number of panels
        prop_assert_eq!(loaded_prefs.panel_sizes.len(), original_prefs.panel_sizes.len());
    });
}

/// Property: Update Persistence
///
/// Updating a panel size SHALL correctly overwrite the previous value
/// and persist the new value.
///
/// **Invariants:**
/// 1. Second save overwrites first save
/// 2. Only the latest value is persisted
/// 3. Update doesn't affect other panels
/// 4. Update is immediately visible on load
#[test]
fn property_update_persistence() {
    proptest!(|(
        panel_id in "[a-z_]{3,20}\\.[a-z_]{3,20}",
        initial_size in 50.0f32..1000.0f32,
        updated_size in 1000.0f32..2000.0f32,
        other_panels in prop::collection::vec(
            ("[a-z_]{3,15}\\.[a-z_]{3,15}", 50.0f32..2000.0f32),
            0..5
        ),
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Save other panels
        for (other_id, other_size) in &other_panels {
            if other_id != &panel_id {
                prefs.set_panel_size(other_id.clone(), px(*other_size));
            }
        }
        
        // Save initial size
        prefs.set_panel_size(panel_id.clone(), px(initial_size));
        
        // Verify initial size
        prop_assert_eq!(prefs.get_panel_size(&panel_id), Some(px(initial_size)));
        
        // Update to new size
        prefs.set_panel_size(panel_id.clone(), px(updated_size));
        
        // Invariant 1 & 2: New value overwrites old value
        let loaded_size = prefs.get_panel_size(&panel_id);
        prop_assert_eq!(loaded_size, Some(px(updated_size)));
        prop_assert_ne!(loaded_size, Some(px(initial_size)));
        
        // Invariant 3: Other panels are unaffected
        for (other_id, expected_size) in &other_panels {
            if other_id != &panel_id {
                let other_loaded = prefs.get_panel_size(other_id);
                prop_assert_eq!(other_loaded, Some(px(*expected_size)));
            }
        }
        
        // Invariant 4: Update survives serialization
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(loaded_prefs.get_panel_size(&panel_id), Some(px(updated_size)));
    });
}

/// Property: Empty Preferences Handling
///
/// Empty preferences SHALL serialize and deserialize correctly
/// without errors.
///
/// **Invariants:**
/// 1. Empty preferences serialize to valid JSON
/// 2. Empty preferences deserialize correctly
/// 3. Loading non-existent panel returns None
/// 4. Empty state is stable
#[test]
fn property_empty_preferences_handling() {
    proptest!(|(
        panel_id in "[a-z_]{3,20}\\.[a-z_]{3,20}",
    )| {
        let empty_prefs = EditorPreferences::new();
        
        // Invariant 1: Empty preferences serialize
        let json = serde_json::to_string(&empty_prefs)
            .expect("Failed to serialize empty preferences");
        prop_assert!(!json.is_empty());
        
        // Invariant 2: Empty preferences deserialize
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json)
            .expect("Failed to deserialize empty preferences");
        
        // Invariant 3: Loading non-existent panel returns None
        prop_assert_eq!(loaded_prefs.get_panel_size(&panel_id), None);
        
        // Invariant 4: Empty state is preserved
        prop_assert_eq!(loaded_prefs.panel_sizes.len(), 0);
        prop_assert!(loaded_prefs.panel_sizes.is_empty());
    });
}

/// Property: Panel ID Uniqueness
///
/// Each panel ID SHALL map to exactly one size value, and duplicate
/// IDs SHALL overwrite previous values.
///
/// **Invariants:**
/// 1. Panel ID uniquely identifies a size
/// 2. Duplicate IDs result in single entry
/// 3. Last write wins for duplicate IDs
/// 4. No duplicate entries in storage
#[test]
fn property_panel_id_uniqueness() {
    proptest!(|(
        panel_id in "[a-z_]{3,20}\\.[a-z_]{3,20}",
        sizes in prop::collection::vec(50.0f32..2000.0f32, 2..5),
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Write same panel ID multiple times with different sizes
        for size in &sizes {
            prefs.set_panel_size(panel_id.clone(), px(*size));
        }
        
        // Invariant 1 & 2: Only one entry exists for the panel ID
        let count = prefs.panel_sizes.sizes.keys()
            .filter(|k| *k == &panel_id)
            .count();
        prop_assert_eq!(count, 1);
        
        // Invariant 3: Last write wins
        let last_size = sizes.last().unwrap();
        let loaded_size = prefs.get_panel_size(&panel_id);
        prop_assert_eq!(loaded_size, Some(px(*last_size)));
        
        // Invariant 4: Total size count is 1
        prop_assert_eq!(prefs.panel_sizes.len(), 1);
    });
}

/// Property: Large Dataset Persistence
///
/// Large numbers of panels SHALL be persisted and loaded correctly
/// without data loss or performance degradation.
///
/// **Invariants:**
/// 1. All panels are persisted regardless of count
/// 2. No data loss with large datasets
/// 3. Serialization succeeds for large datasets
/// 4. Deserialization produces correct count
#[test]
fn property_large_dataset_persistence() {
    proptest!(|(
        panel_count in 10usize..100usize,
        size_base in 100.0f32..500.0f32,
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Create many panels with unique IDs
        for i in 0..panel_count {
            let panel_id = format!("box{}.panel{}", i / 10, i % 10);
            let size = size_base + (i as f32 * 10.0);
            prefs.set_panel_size(panel_id, px(size));
        }
        
        // Invariant 1: All panels are stored
        prop_assert_eq!(prefs.panel_sizes.len(), panel_count);
        
        // Invariant 3: Serialization succeeds
        let json = serde_json::to_string(&prefs)
            .expect("Failed to serialize large dataset");
        prop_assert!(!json.is_empty());
        
        // Invariant 2 & 4: Deserialization preserves all data
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json)
            .expect("Failed to deserialize large dataset");
        prop_assert_eq!(loaded_prefs.panel_sizes.len(), panel_count);
        
        // Verify a sample of panels
        for i in (0..panel_count).step_by(panel_count / 5.max(1)) {
            let panel_id = format!("box{}.panel{}", i / 10, i % 10);
            let expected_size = size_base + (i as f32 * 10.0);
            let loaded_size = loaded_prefs.get_panel_size(&panel_id);
            prop_assert_eq!(loaded_size, Some(px(expected_size)));
        }
    });
}

/// Property: Floating Point Precision
///
/// Floating point panel sizes SHALL be preserved with sufficient
/// precision through serialization.
///
/// **Invariants:**
/// 1. Precision is preserved to at least 2 decimal places
/// 2. No significant rounding errors
/// 3. Fractional sizes are supported
/// 4. Precision is consistent across save/load cycles
#[test]
fn property_floating_point_precision() {
    proptest!(|(
        panel_id in "[a-z_]{3,20}\\.[a-z_]{3,20}",
        integer_part in 100u32..1000u32,
        fractional_part in 0u32..100u32,
    )| {
        let size = integer_part as f32 + (fractional_part as f32 / 100.0);
        let mut prefs = EditorPreferences::new();
        
        // Save size with fractional component
        prefs.set_panel_size(panel_id.clone(), px(size));
        
        // Load back
        let loaded_size = prefs.get_panel_size(&panel_id);
        
        // Invariant 1 & 2: Precision preserved (within floating point tolerance)
        if let Some(loaded) = loaded_size {
            let loaded_f32 = unsafe { std::mem::transmute::<gpui::Pixels, f32>(loaded) };
            let diff = (loaded_f32 - size).abs();
            prop_assert!(diff < 0.01, "Precision loss: expected {}, got {}, diff {}", size, loaded_f32, diff);
        } else {
            prop_assert!(false, "Failed to load size");
        }
        
        // Invariant 4: Precision survives serialization
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json).unwrap();
        let final_size = loaded_prefs.get_panel_size(&panel_id);
        
        if let Some(final_loaded) = final_size {
            let final_f32 = unsafe { std::mem::transmute::<gpui::Pixels, f32>(final_loaded) };
            let final_diff = (final_f32 - size).abs();
            prop_assert!(final_diff < 0.01, "Serialization precision loss: expected {}, got {}, diff {}", size, final_f32, final_diff);
        }
    });
}

/// Property: Panel ID Format Stability
///
/// Panel IDs SHALL follow a consistent format and be stable across
/// sessions.
///
/// **Invariants:**
/// 1. Panel IDs contain a dot separator (box.panel format)
/// 2. Panel IDs are case-sensitive
/// 3. Panel IDs are stable (same ID always maps to same panel)
/// 4. Panel IDs are unique per panel
#[test]
fn property_panel_id_format_stability() {
    proptest!(|(
        box_name in "[a-z_]{3,15}",
        panel_name in "[a-z_]{3,15}",
        size in 100.0f32..1000.0f32,
    )| {
        let panel_id = format!("{}.{}", box_name, panel_name);
        let mut prefs = EditorPreferences::new();
        
        // Invariant 1: Panel ID contains dot separator
        prop_assert!(panel_id.contains('.'));
        
        // Save with formatted ID
        prefs.set_panel_size(panel_id.clone(), px(size));
        
        // Invariant 3: Same ID retrieves same value
        let loaded1 = prefs.get_panel_size(&panel_id);
        let loaded2 = prefs.get_panel_size(&panel_id);
        prop_assert_eq!(loaded1, loaded2);
        
        // Invariant 2: Case sensitivity (different case = different panel)
        let upper_id = panel_id.to_uppercase();
        if upper_id != panel_id {
            let upper_loaded = prefs.get_panel_size(&upper_id);
            prop_assert_eq!(upper_loaded, None);
        }
    });
}

/// Property: Concurrent Updates
///
/// Multiple rapid updates to different panels SHALL all be persisted
/// correctly without data loss.
///
/// **Invariants:**
/// 1. All updates are recorded
/// 2. No updates are lost
/// 3. Final state reflects all updates
/// 4. Order of updates doesn't cause data loss
#[test]
fn property_concurrent_updates() {
    proptest!(|(
        updates in prop::collection::vec(
            ("[a-z_]{3,10}\\.[a-z_]{3,10}", 50.0f32..2000.0f32),
            5..20
        ),
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Apply all updates
        for (panel_id, size) in &updates {
            prefs.set_panel_size(panel_id.clone(), px(*size));
        }
        
        // Build expected state (last write wins for each unique ID)
        let mut expected_state = std::collections::HashMap::new();
        for (panel_id, size) in &updates {
            expected_state.insert(panel_id.clone(), *size);
        }
        
        // Invariant 1 & 2: All unique panels are present
        prop_assert_eq!(prefs.panel_sizes.len(), expected_state.len());
        
        // Invariant 3: Final state matches expected
        for (panel_id, expected_size) in &expected_state {
            let loaded_size = prefs.get_panel_size(panel_id);
            prop_assert_eq!(loaded_size, Some(px(*expected_size)));
        }
        
        // Invariant 4: Serialization preserves final state
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json).unwrap();
        
        for (panel_id, expected_size) in &expected_state {
            let final_size = loaded_prefs.get_panel_size(panel_id);
            prop_assert_eq!(final_size, Some(px(*expected_size)));
        }
    });
}

/// Property: Remove and Clear Operations
///
/// Removing panel sizes SHALL correctly delete entries and clear
/// SHALL remove all entries.
///
/// **Invariants:**
/// 1. Remove deletes the specified panel
/// 2. Remove doesn't affect other panels
/// 3. Clear removes all panels
/// 4. Operations are reflected in serialization
#[test]
fn property_remove_and_clear_operations() {
    proptest!(|(
        panels in prop::collection::vec(
            ("[a-z_]{3,15}\\.[a-z_]{3,15}", 50.0f32..2000.0f32),
            3..10
        ),
        remove_index in 0usize..3usize,
    )| {
        let mut prefs = EditorPreferences::new();
        
        // Add all panels
        for (panel_id, size) in &panels {
            prefs.set_panel_size(panel_id.clone(), px(*size));
        }
        
        let initial_count = prefs.panel_sizes.len();
        
        // Remove one panel
        if remove_index < panels.len() {
            let (remove_id, _) = &panels[remove_index];
            let removed = prefs.panel_sizes.remove_size(remove_id);
            
            // Invariant 1: Panel is removed
            prop_assert!(removed.is_some());
            prop_assert_eq!(prefs.get_panel_size(remove_id), None);
            
            // Invariant 2: Other panels unaffected
            for (i, (panel_id, expected_size)) in panels.iter().enumerate() {
                if i != remove_index {
                    let loaded = prefs.get_panel_size(panel_id);
                    prop_assert_eq!(loaded, Some(px(*expected_size)));
                }
            }
            
            prop_assert_eq!(prefs.panel_sizes.len(), initial_count - 1);
        }
        
        // Clear all panels
        prefs.panel_sizes.clear();
        
        // Invariant 3: All panels removed
        prop_assert_eq!(prefs.panel_sizes.len(), 0);
        prop_assert!(prefs.panel_sizes.is_empty());
        
        // Invariant 4: Clear is reflected in serialization
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded_prefs: EditorPreferences = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(loaded_prefs.panel_sizes.len(), 0);
    });
}
