//! Property-based test for drag-and-drop reordering
//!
//! **Validates: Requirements 2.5**
//!
//! **Property 9: Drag-and-Drop Reordering**
//!
//! After dragging an item from position A to position B, the item should be at
//! position B and all other items should maintain their relative order.

use luminara_editor::{ActivityBar, ActivityItem, Badge, BadgeVariant, Theme};
use proptest::prelude::*;
use std::sync::Arc;

/// Helper function to create a test ActivityItem
fn create_test_item(id: usize) -> ActivityItem {
    ActivityItem {
        id: format!("item-{}", id),
        icon: format!("{}", id),
        title: format!("Item {}", id),
        badge: None,
        is_folder: false,
    }
}

/// Helper function to create an ActivityBar with N items
fn create_activity_bar_with_items(count: usize) -> ActivityBar {
    let theme = Arc::new(Theme::default_dark());
    let mut bar = ActivityBar::new(theme);
    
    // Replace default items with test items
    let items: Vec<ActivityItem> = (0..count).map(create_test_item).collect();
    bar.set_items_for_testing(items);
    
    bar
}

/// **Property 9: Drag-and-Drop Reordering**
///
/// This property verifies that after dragging an item from position A to position B:
/// 1. The dragged item is at position B
/// 2. All other items maintain their relative order
/// 3. No items are lost or duplicated
/// 4. The total number of items remains the same
#[test]
fn property_drag_drop_reordering_basic() {
    // Test with a fixed set of items
    let item_count = 7; // Default number of boxes
    
    // Test all possible drag operations
    for from in 0..item_count {
        for to in 0..item_count {
            if from == to {
                continue; // Skip no-op drags
            }
            
            let mut bar = create_activity_bar_with_items(item_count);
            let original_items = bar.items_for_testing().to_vec();
            
            // Perform drag operation
            bar.simulate_drag_for_testing(from, to);
            
            let reordered_items = bar.items_for_testing();
            
            // Property 1: The dragged item is at position B
            assert_eq!(
                &reordered_items[to].id,
                &original_items[from].id,
                "Item from position {} should be at position {} after drag",
                from, to
            );
            
            // Property 2: Total number of items remains the same
            assert_eq!(
                reordered_items.len(),
                original_items.len(),
                "Number of items should not change after drag"
            );
            
            // Property 3: No items are lost or duplicated
            let mut original_ids: Vec<_> = original_items.iter().map(|i| &i.id).collect();
            let mut reordered_ids: Vec<_> = reordered_items.iter().map(|i| &i.id).collect();
            original_ids.sort();
            reordered_ids.sort();
            assert_eq!(
                original_ids, reordered_ids,
                "All items should be preserved after drag (no loss or duplication)"
            );
            
            // Property 4: All other items maintain their relative order
            verify_relative_order_preserved(&original_items, reordered_items, from, to);
        }
    }
}

/// Verify that all items except the dragged one maintain their relative order
fn verify_relative_order_preserved(
    original: &[ActivityItem],
    reordered: &[ActivityItem],
    from: usize,
    to: usize,
) {
    let dragged_id = &original[from].id;
    
    // Extract items that were not dragged, preserving their order
    let original_others: Vec<&String> = original
        .iter()
        .map(|i| &i.id)
        .filter(|id| *id != dragged_id)
        .collect();
    
    let reordered_others: Vec<&String> = reordered
        .iter()
        .map(|i| &i.id)
        .filter(|id| *id != dragged_id)
        .collect();
    
    assert_eq!(
        original_others, reordered_others,
        "Items other than the dragged item should maintain their relative order (from: {}, to: {})",
        from, to
    );
}

/// Property test using proptest for randomized testing
///
/// This test generates random drag operations and verifies the properties hold.
proptest! {
    #[test]
    fn property_drag_drop_preserves_items(
        item_count in 3usize..10,
        from in 0usize..10,
        to in 0usize..10,
    ) {
        // Ensure indices are within bounds
        if from >= item_count || to >= item_count || from == to {
            return Ok(());
        }
        
        let mut bar = create_activity_bar_with_items(item_count);
        let original_items = bar.items_for_testing().to_vec();
        
        // Perform drag operation
        bar.simulate_drag_for_testing(from, to);
        
        let reordered_items = bar.items_for_testing();
        
        // Property: All items are preserved (no loss or duplication)
        let mut original_ids: Vec<_> = original_items.iter().map(|i| &i.id).collect();
        let mut reordered_ids: Vec<_> = reordered_items.iter().map(|i| &i.id).collect();
        original_ids.sort();
        reordered_ids.sort();
        
        prop_assert_eq!(
            original_ids, reordered_ids,
            "All items should be preserved after drag"
        );
        
        // Property: Dragged item is at target position
        prop_assert_eq!(
            &reordered_items[to].id,
            &original_items[from].id,
            "Dragged item should be at target position"
        );
    }
}

/// Property test: Relative order preservation
///
/// Verifies that items not involved in the drag maintain their relative order.
proptest! {
    #[test]
    fn property_drag_drop_relative_order(
        item_count in 3usize..10,
        from in 0usize..10,
        to in 0usize..10,
    ) {
        // Ensure indices are within bounds
        if from >= item_count || to >= item_count || from == to {
            return Ok(());
        }
        
        let mut bar = create_activity_bar_with_items(item_count);
        let original_items = bar.items_for_testing().to_vec();
        
        // Perform drag operation
        bar.simulate_drag_for_testing(from, to);
        
        let reordered_items = bar.items_for_testing();
        
        // Property: Items not involved in drag maintain relative order
        let dragged_id = &original_items[from].id;
        
        let original_others: Vec<&String> = original_items
            .iter()
            .map(|i| &i.id)
            .filter(|id| *id != dragged_id)
            .collect();
        
        let reordered_others: Vec<&String> = reordered_items
            .iter()
            .map(|i| &i.id)
            .filter(|id| *id != dragged_id)
            .collect();
        
        prop_assert_eq!(
            original_others, reordered_others,
            "Non-dragged items should maintain relative order"
        );
    }
}

/// Property test: Drag operation is idempotent for same position
///
/// Dragging an item to its current position should not change the order.
#[test]
fn property_drag_to_same_position_is_noop() {
    let item_count = 7;
    
    for position in 0..item_count {
        let mut bar = create_activity_bar_with_items(item_count);
        let original_items = bar.items_for_testing().to_vec();
        
        // Drag item to its own position
        bar.simulate_drag_for_testing(position, position);
        
        let reordered_items = bar.items_for_testing();
        
        // Property: Order should not change
        for (i, (original, reordered)) in original_items.iter().zip(reordered_items.iter()).enumerate() {
            assert_eq!(
                original.id, reordered.id,
                "Item at position {} should not change when dragging to same position",
                i
            );
        }
    }
}

/// Property test: Multiple consecutive drags
///
/// Verifies that multiple drag operations can be performed sequentially
/// and the properties still hold.
#[test]
fn property_multiple_consecutive_drags() {
    let item_count = 5;
    let mut bar = create_activity_bar_with_items(item_count);
    
    // Perform a sequence of drags
    let drag_sequence = vec![
        (0, 2), // Move first item to position 2
        (1, 3), // Move item at position 1 to position 3
        (4, 0), // Move last item to first position
    ];
    
    let original_items = bar.items_for_testing().to_vec();
    
    for (from, to) in drag_sequence {
        bar.simulate_drag_for_testing(from, to);
    }
    
    let final_items = bar.items_for_testing();
    
    // Property: All items are still present
    let mut original_ids: Vec<_> = original_items.iter().map(|i| &i.id).collect();
    let mut final_ids: Vec<_> = final_items.iter().map(|i| &i.id).collect();
    original_ids.sort();
    final_ids.sort();
    
    assert_eq!(
        original_ids, final_ids,
        "All items should be preserved after multiple drags"
    );
    
    // Property: Count remains the same
    assert_eq!(
        original_items.len(),
        final_items.len(),
        "Item count should not change after multiple drags"
    );
}

/// Property test: Drag with active item tracking
///
/// Verifies that the active item index is correctly updated when items are reordered.
#[test]
fn property_drag_updates_active_index() {
    let item_count = 5;
    
    // Test dragging the active item
    for active_index in 0..item_count {
        for from in 0..item_count {
            for to in 0..item_count {
                if from == to {
                    continue;
                }
                
                let mut bar = create_activity_bar_with_items(item_count);
                bar.set_active_for_testing(active_index);
                
                let original_active_id = bar.items_for_testing()[active_index].id.clone();
                
                // Perform drag
                bar.simulate_drag_for_testing(from, to);
                
                // Find where the originally active item ended up
                let _new_active_index = bar.items_for_testing()
                    .iter()
                    .position(|item| item.id == original_active_id)
                    .unwrap();
                
                // Property: If the active item was dragged, it should still be active
                if from == active_index {
                    assert_eq!(
                        bar.active_index_for_testing(),
                        Some(to),
                        "Active item should move with drag (from: {}, to: {})",
                        from, to
                    );
                } else {
                    // Property: Active item should still be the same item (by ID)
                    let current_active_id = bar.active_item_for_testing()
                        .map(|item| &item.id);
                    assert_eq!(
                        current_active_id,
                        Some(&original_active_id),
                        "Active item ID should not change if it wasn't dragged"
                    );
                }
            }
        }
    }
}

/// Property test: Drag boundaries
///
/// Verifies that drag operations handle boundary conditions correctly.
#[test]
fn property_drag_boundaries() {
    let item_count = 7;
    let mut bar = create_activity_bar_with_items(item_count);
    
    // Test dragging first item to last position
    let original_items = bar.items_for_testing().to_vec();
    bar.simulate_drag_for_testing(0, item_count - 1);
    let reordered = bar.items_for_testing();
    
    assert_eq!(
        reordered[item_count - 1].id,
        original_items[0].id,
        "First item should move to last position"
    );
    
    // Reset and test dragging last item to first position
    let mut bar = create_activity_bar_with_items(item_count);
    let original_items = bar.items_for_testing().to_vec();
    bar.simulate_drag_for_testing(item_count - 1, 0);
    let reordered = bar.items_for_testing();
    
    assert_eq!(
        reordered[0].id,
        original_items[item_count - 1].id,
        "Last item should move to first position"
    );
}

/// Property test: Drag with badges
///
/// Verifies that item properties (like badges) are preserved during drag.
#[test]
fn property_drag_preserves_item_properties() {
    let theme = Arc::new(Theme::default_dark());
    let mut bar = ActivityBar::new(theme);
    
    // Create items with various properties
    let items = vec![
        ActivityItem {
            id: "item-0".to_string(),
            icon: "0".to_string(),
            title: "Item 0".to_string(),
            badge: Some(Badge {
                count: 5,
                variant: BadgeVariant::Error,
            }),
            is_folder: false,
        },
        ActivityItem {
            id: "item-1".to_string(),
            icon: "1".to_string(),
            title: "Item 1".to_string(),
            badge: None,
            is_folder: true,
        },
        ActivityItem {
            id: "item-2".to_string(),
            icon: "2".to_string(),
            title: "Item 2".to_string(),
            badge: Some(Badge {
                count: 3,
                variant: BadgeVariant::Warning,
            }),
            is_folder: false,
        },
    ];
    
    bar.set_items_for_testing(items.clone());
    
    // Drag item with badge from position 0 to position 2
    bar.simulate_drag_for_testing(0, 2);
    
    let reordered = bar.items_for_testing();
    
    // Property: Badge should be preserved
    assert_eq!(
        reordered[2].badge.as_ref().map(|b| b.count),
        Some(5),
        "Badge count should be preserved"
    );
    assert_eq!(
        reordered[2].badge.as_ref().map(|b| b.variant),
        Some(BadgeVariant::Error),
        "Badge variant should be preserved"
    );
    
    // Property: is_folder should be preserved for item-1
    // After dragging item-0 from position 0 to position 2:
    // Original: [item-0, item-1, item-2]
    // Result:   [item-1, item-2, item-0]
    // So item-1 is now at position 0
    let item_1_in_result = reordered.iter().find(|item| item.id == "item-1").unwrap();
    assert_eq!(
        item_1_in_result.is_folder,
        items[1].is_folder,
        "is_folder property should be preserved"
    );
}

/// Property test: Drag operation correctness
///
/// This test verifies the exact behavior of the drag operation by checking
/// the resulting order matches the expected Vec::remove + Vec::insert behavior.
#[test]
fn property_drag_matches_vec_semantics() {
    let item_count = 7;
    
    for from in 0..item_count {
        for to in 0..item_count {
            if from == to {
                continue;
            }
            
            // Test with ActivityBar
            let mut bar = create_activity_bar_with_items(item_count);
            let original_items = bar.items_for_testing().to_vec();
            bar.simulate_drag_for_testing(from, to);
            let bar_result = bar.items_for_testing();
            
            // Test with plain Vec (expected behavior)
            let mut vec_items = original_items.clone();
            let item = vec_items.remove(from);
            vec_items.insert(to, item);
            
            // Property: ActivityBar drag should match Vec remove+insert
            for (i, (bar_item, vec_item)) in bar_result.iter().zip(vec_items.iter()).enumerate() {
                assert_eq!(
                    bar_item.id, vec_item.id,
                    "Item at position {} should match Vec semantics (from: {}, to: {})",
                    i, from, to
                );
            }
        }
    }
}
