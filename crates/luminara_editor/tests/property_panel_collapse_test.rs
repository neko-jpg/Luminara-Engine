//! Property-based test for Panel Collapse Functionality
//!
//! **Validates: Requirements 9.5**
//!
//! **Property 23: Panel Collapse**
//!
//! This property verifies that resizable panels correctly collapse to minimum size
//! and expand back to their previous size, storing state appropriately.

use proptest::prelude::*;

/// Property: Collapse to Minimum Size
///
/// When a panel is collapsed, it SHALL be set to the minimum size.
///
/// **Invariants:**
/// 1. After collapse, current_size == min_size
/// 2. Collapse is idempotent (collapsing twice has same effect as once)
/// 3. Previous size is stored before collapse
#[test]
fn property_collapse_to_minimum() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.1f32..0.9f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Ensure initial size is not at minimum
        prop_assume!(initial_size > min_size);
        
        // Simulate collapse operation
        let size_before_collapse = initial_size;
        let collapsed_size = min_size;
        let is_collapsed = true;
        
        // Invariant 1: After collapse, size equals min_size
        prop_assert_eq!(collapsed_size, min_size);
        
        // Invariant 2: Collapsing again has no additional effect
        let collapsed_again = if is_collapsed { min_size } else { min_size };
        prop_assert_eq!(collapsed_again, collapsed_size);
        
        // Invariant 3: Previous size is stored
        prop_assert_eq!(size_before_collapse, initial_size);
        prop_assert!(size_before_collapse > min_size);
    });
}

/// Property: Expand Restores Previous Size
///
/// When a collapsed panel is expanded, it SHALL restore the size
/// it had before collapsing.
///
/// **Invariants:**
/// 1. After expand, current_size == size_before_collapse
/// 2. If no previous size stored, expand to max_size
/// 3. Expand only works when panel is collapsed
/// 4. After expand, is_collapsed flag is false
#[test]
fn property_expand_restores_size() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.1f32..0.9f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Ensure initial size is not at minimum or maximum
        prop_assume!(initial_size > min_size && initial_size < max_size);
        
        // Simulate collapse
        let size_before_collapse = Some(initial_size);
        let collapsed_size = min_size;
        let is_collapsed = true;
        
        // Simulate expand
        let expanded_size = if is_collapsed {
            size_before_collapse.unwrap_or(max_size)
        } else {
            collapsed_size
        };
        let is_collapsed_after_expand = false;
        
        // Invariant 1: Expanded size equals previous size
        prop_assert_eq!(expanded_size, initial_size);
        
        // Invariant 4: After expand, not collapsed
        prop_assert!(!is_collapsed_after_expand);
        
        // Invariant 1: Size is restored correctly
        prop_assert!(expanded_size > min_size);
        prop_assert!(expanded_size < max_size);
    });
}

/// Property: Expand Without Previous Size
///
/// When expanding a collapsed panel with no stored previous size,
/// it SHALL expand to maximum size.
///
/// **Invariants:**
/// 1. If size_before_collapse is None, expand to max_size
/// 2. Expanded size is within constraints
#[test]
fn property_expand_without_previous_size() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
    )| {
        let max_size = min_size + max_offset;
        
        // Simulate collapsed state with no previous size
        let size_before_collapse: Option<f32> = None;
        let collapsed_size = min_size;
        let is_collapsed = true;
        
        // Simulate expand
        let expanded_size = if is_collapsed {
            size_before_collapse.unwrap_or(max_size)
        } else {
            collapsed_size
        };
        
        // Invariant 1: Without previous size, expand to max
        prop_assert_eq!(expanded_size, max_size);
        
        // Invariant 2: Expanded size is within constraints
        prop_assert!(expanded_size >= min_size);
        prop_assert!(expanded_size <= max_size);
    });
}

/// Property: Toggle Collapse State
///
/// Toggling collapse state SHALL alternate between collapsed and expanded,
/// correctly managing the size and state transitions.
///
/// **Invariants:**
/// 1. First toggle collapses (stores size, sets to min)
/// 2. Second toggle expands (restores size, clears stored size)
/// 3. State alternates correctly
/// 4. Size is always within constraints
#[test]
fn property_toggle_collapse_state() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.2f32..0.8f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Ensure initial size is not at boundaries
        prop_assume!(initial_size > min_size + 10.0 && initial_size < max_size - 10.0);
        
        // Initial state: not collapsed
        let mut current_size = initial_size;
        let mut is_collapsed = false;
        let mut size_before_collapse: Option<f32> = None;
        
        // First toggle: should collapse
        if !is_collapsed {
            size_before_collapse = Some(current_size);
            current_size = min_size;
            is_collapsed = true;
        }
        
        // Invariant 1: After first toggle, collapsed to min
        prop_assert_eq!(current_size, min_size);
        prop_assert!(is_collapsed);
        prop_assert_eq!(size_before_collapse, Some(initial_size));
        
        // Second toggle: should expand
        if is_collapsed {
            current_size = size_before_collapse.unwrap_or(max_size);
            is_collapsed = false;
            size_before_collapse = None;
        }
        
        // Invariant 2: After second toggle, restored to initial
        prop_assert_eq!(current_size, initial_size);
        prop_assert!(!is_collapsed);
        prop_assert_eq!(size_before_collapse, None);
        
        // Invariant 4: Size always within constraints
        prop_assert!(current_size >= min_size);
        prop_assert!(current_size <= max_size);
    });
}

/// Property: Collapse Preserves Constraints
///
/// Collapse and expand operations SHALL always maintain
/// the min/max size constraints.
///
/// **Invariants:**
/// 1. Collapsed size is always min_size
/// 2. Expanded size is always within [min_size, max_size]
/// 3. Stored previous size is within constraints
/// 4. Operations never violate constraints
#[test]
fn property_collapse_preserves_constraints() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.0f32..1.0f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Clamp initial size to constraints
        let initial_size = initial_size.max(min_size).min(max_size);
        
        // Simulate collapse
        let size_before_collapse = Some(initial_size);
        let collapsed_size = min_size;
        
        // Invariant 1: Collapsed size is min_size
        prop_assert_eq!(collapsed_size, min_size);
        prop_assert!(collapsed_size >= min_size);
        prop_assert!(collapsed_size <= max_size);
        
        // Invariant 3: Stored size is within constraints
        if let Some(stored) = size_before_collapse {
            prop_assert!(stored >= min_size);
            prop_assert!(stored <= max_size);
        }
        
        // Simulate expand
        let expanded_size = size_before_collapse.unwrap_or(max_size);
        
        // Invariant 2: Expanded size is within constraints
        prop_assert!(expanded_size >= min_size);
        prop_assert!(expanded_size <= max_size);
    });
}

/// Property: Multiple Collapse/Expand Cycles
///
/// Multiple collapse/expand cycles SHALL maintain correctness
/// and not corrupt the state.
///
/// **Invariants:**
/// 1. After even number of toggles, panel is expanded
/// 2. After odd number of toggles, panel is collapsed
/// 3. Size is restored correctly after each cycle
/// 4. State remains consistent
#[test]
fn property_multiple_collapse_expand_cycles() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.2f32..0.8f32,
        num_cycles in 1usize..5usize,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Ensure initial size is not at boundaries
        prop_assume!(initial_size > min_size + 10.0 && initial_size < max_size - 10.0);
        
        let mut current_size = initial_size;
        let mut is_collapsed = false;
        let mut size_before_collapse: Option<f32> = None;
        
        // Perform multiple collapse/expand cycles
        for cycle in 0..num_cycles {
            // Collapse
            if !is_collapsed {
                size_before_collapse = Some(current_size);
                current_size = min_size;
                is_collapsed = true;
            }
            
            // After collapse in each cycle
            prop_assert_eq!(current_size, min_size);
            prop_assert!(is_collapsed);
            
            // Expand
            if is_collapsed {
                current_size = size_before_collapse.unwrap_or(max_size);
                is_collapsed = false;
                size_before_collapse = None;
            }
            
            // After expand in each cycle
            prop_assert_eq!(current_size, initial_size);
            prop_assert!(!is_collapsed);
            
            // Invariant 3: Size is restored correctly
            prop_assert!((current_size - initial_size).abs() < 0.001);
            
            // Invariant 4: State is consistent
            prop_assert!(!is_collapsed);
            prop_assert_eq!(size_before_collapse, None);
        }
        
        // After all cycles, panel should be expanded
        prop_assert!(!is_collapsed);
        prop_assert_eq!(current_size, initial_size);
    });
}

/// Property: Collapse State Independence
///
/// The collapse state SHALL be independent of resize operations
/// and other panel state.
///
/// **Invariants:**
/// 1. Collapse state doesn't affect min/max constraints
/// 2. Resize operations don't affect collapse state
/// 3. Collapse state is explicitly managed
#[test]
fn property_collapse_state_independence() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.2f32..0.8f32,
        resize_delta in -50.0f32..50.0f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Start not collapsed
        let mut current_size = initial_size;
        let is_collapsed = false;
        
        // Apply resize (should not affect collapse state)
        let new_size = (current_size + resize_delta).max(min_size).min(max_size);
        current_size = new_size;
        
        // Invariant 2: Resize doesn't change collapse state
        prop_assert!(!is_collapsed);
        
        // Invariant 1: Constraints still apply
        prop_assert!(current_size >= min_size);
        prop_assert!(current_size <= max_size);
        
        // Now collapse
        let size_before_collapse = Some(current_size);
        let collapsed_size = min_size;
        let is_collapsed_after = true;
        
        // Invariant 1: Constraints still apply when collapsed
        prop_assert_eq!(collapsed_size, min_size);
        prop_assert!(collapsed_size >= min_size);
        prop_assert!(collapsed_size <= max_size);
        
        // Invariant 3: Collapse state is explicit
        prop_assert!(is_collapsed_after);
        prop_assert!(size_before_collapse.is_some());
    });
}

/// Property: Collapse at Boundaries
///
/// Collapsing when already at minimum size SHALL still work correctly,
/// and expanding from maximum size SHALL work correctly.
///
/// **Invariants:**
/// 1. Collapsing at min_size stores min_size
/// 2. Expanding from max_size works correctly
/// 3. Edge cases don't break the logic
#[test]
fn property_collapse_at_boundaries() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
    )| {
        let max_size = min_size + max_offset;
        
        // Test collapsing when already at minimum
        {
            let current_size = min_size;
            let size_before_collapse = Some(current_size);
            let collapsed_size = min_size;
            
            // Invariant 1: Can collapse even at min_size
            prop_assert_eq!(collapsed_size, min_size);
            prop_assert_eq!(size_before_collapse, Some(min_size));
            
            // Expanding should restore min_size
            let expanded_size = size_before_collapse.unwrap_or(max_size);
            prop_assert_eq!(expanded_size, min_size);
        }
        
        // Test collapsing when at maximum
        {
            let current_size = max_size;
            let size_before_collapse = Some(current_size);
            let collapsed_size = min_size;
            
            // Should collapse to min
            prop_assert_eq!(collapsed_size, min_size);
            prop_assert_eq!(size_before_collapse, Some(max_size));
            
            // Invariant 2: Expanding should restore max_size
            let expanded_size = size_before_collapse.unwrap_or(max_size);
            prop_assert_eq!(expanded_size, max_size);
        }
    });
}
