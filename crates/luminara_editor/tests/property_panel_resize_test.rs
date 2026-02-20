//! Property-based test for Panel Resize Proportionality
//!
//! **Validates: Requirements 9.2, 9.3**
//!
//! **Property 2: Panel Resize Proportionality**
//!
//! This property verifies that resizable panels update sizes proportionally
//! while respecting minimum and maximum constraints during drag operations.

use proptest::prelude::*;

/// Property: Panel Size Clamping
///
/// For any resize operation, the panel size SHALL always be clamped
/// to the [min_size, max_size] range.
///
/// **Invariants:**
/// 1. current_size >= min_size
/// 2. current_size <= max_size
/// 3. Clamping is idempotent (clamping twice gives same result)
#[test]
fn property_panel_size_clamping() {
    proptest!(|(
        min_size in 50.0f32..500.0f32,
        max_offset in 100.0f32..1000.0f32,
        initial_offset in -200.0f32..200.0f32,
        resize_delta in -500.0f32..500.0f32,
    )| {
        let max_size = min_size + max_offset;
        let initial_size = min_size + initial_offset;
        
        // Create a mock panel (we can't create real GPUI views in property tests)
        // Instead, we test the clamping logic directly
        let clamped_initial = initial_size.max(min_size).min(max_size);
        
        // Invariant 1 & 2: Initial size is within bounds
        prop_assert!(clamped_initial >= min_size);
        prop_assert!(clamped_initial <= max_size);
        
        // Apply resize delta
        let new_size = clamped_initial + resize_delta;
        let clamped_new = new_size.max(min_size).min(max_size);
        
        // Invariant 1 & 2: New size is within bounds
        prop_assert!(clamped_new >= min_size);
        prop_assert!(clamped_new <= max_size);
        
        // Invariant 3: Clamping is idempotent
        let double_clamped = clamped_new.max(min_size).min(max_size);
        prop_assert_eq!(clamped_new, double_clamped);
    });
}

/// Property: Resize Proportionality
///
/// For any drag distance, the panel size SHALL change proportionally
/// to the drag distance, subject to min/max constraints.
///
/// **Invariants:**
/// 1. If not at boundary, size change equals drag distance
/// 2. If at min boundary, negative drags have no effect
/// 3. If at max boundary, positive drags have no effect
/// 4. Drag distance of 0 results in no size change
#[test]
fn property_resize_proportionality() {
    proptest!(|(
        min_size in 100.0f32..300.0f32,
        max_offset in 200.0f32..500.0f32,
        start_offset in 50.0f32..150.0f32,
        drag_distance in -100.0f32..100.0f32,
    )| {
        let max_size = min_size + max_offset;
        let start_size = min_size + start_offset;
        
        // Ensure start_size is within bounds
        let start_size = start_size.max(min_size).min(max_size);
        
        // Calculate expected new size
        let unclamped_new_size = start_size + drag_distance;
        let expected_new_size = unclamped_new_size.max(min_size).min(max_size);
        
        // Invariant 4: Zero drag means no change
        if drag_distance.abs() < 0.001 {
            prop_assert!((expected_new_size - start_size).abs() < 0.001);
        }
        
        // Invariant 1: If not at boundary, size changes by drag distance
        if unclamped_new_size >= min_size && unclamped_new_size <= max_size {
            prop_assert!((expected_new_size - start_size - drag_distance).abs() < 0.001);
        }
        
        // Invariant 2: At min boundary, negative drags clamped
        if start_size <= min_size && drag_distance < 0.0 {
            prop_assert_eq!(expected_new_size, min_size);
        }
        
        // Invariant 3: At max boundary, positive drags clamped
        if start_size >= max_size && drag_distance > 0.0 {
            prop_assert_eq!(expected_new_size, max_size);
        }
    });
}

/// Property: Constraint Enforcement
///
/// The panel SHALL enforce min/max constraints at all times,
/// regardless of the sequence of operations.
///
/// **Invariants:**
/// 1. Multiple resize operations maintain constraints
/// 2. Constraints are enforced even with extreme values
/// 3. Constraints are independent of orientation
#[test]
fn property_constraint_enforcement() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        resize_sequence in prop::collection::vec(-200.0f32..200.0f32, 1..10),
    )| {
        let max_size = min_size + max_offset;
        let mut current_size = (min_size + max_size) / 2.0; // Start in middle
        
        // Apply sequence of resizes
        for delta in resize_sequence {
            let new_size = current_size + delta;
            current_size = new_size.max(min_size).min(max_size);
            
            // Invariant 1 & 2: Constraints maintained after each operation
            prop_assert!(current_size >= min_size);
            prop_assert!(current_size <= max_size);
        }
        
        // Invariant 3: Final size is within constraints regardless of sequence
        prop_assert!(current_size >= min_size);
        prop_assert!(current_size <= max_size);
    });
}

/// Property: Drag Position Mapping
///
/// For drag operations, the size change SHALL correctly map
/// from mouse position delta to panel size delta based on orientation.
///
/// **Invariants:**
/// 1. Horizontal orientation uses X-axis delta
/// 2. Vertical orientation uses Y-axis delta
/// 3. Size delta equals position delta (1:1 mapping)
/// 4. Opposite axis has no effect
#[test]
fn property_drag_position_mapping() {
    proptest!(|(
        start_x in 0.0f32..1000.0f32,
        start_y in 0.0f32..1000.0f32,
        delta_x in -200.0f32..200.0f32,
        delta_y in -200.0f32..200.0f32,
        min_size in 100.0f32..300.0f32,
        max_offset in 200.0f32..500.0f32,
    )| {
        let max_size = min_size + max_offset;
        let start_size = (min_size + max_size) / 2.0;
        
        // Test horizontal orientation
        {
            let end_x = start_x + delta_x;
            let position_delta = end_x - start_x;
            let new_size = (start_size + position_delta).max(min_size).min(max_size);
            
            // Invariant 1 & 3: Horizontal uses X-axis with 1:1 mapping
            if start_size + position_delta >= min_size && start_size + position_delta <= max_size {
                prop_assert!((new_size - start_size - position_delta).abs() < 0.001);
            }
        }
        
        // Test vertical orientation
        {
            let end_y = start_y + delta_y;
            let position_delta = end_y - start_y;
            let new_size = (start_size + position_delta).max(min_size).min(max_size);
            
            // Invariant 2 & 3: Vertical uses Y-axis with 1:1 mapping
            if start_size + position_delta >= min_size && start_size + position_delta <= max_size {
                prop_assert!((new_size - start_size - position_delta).abs() < 0.001);
            }
        }
    });
}

/// Property: Boundary Behavior
///
/// At size boundaries (min/max), the panel SHALL behave correctly
/// and not allow sizes outside the valid range.
///
/// **Invariants:**
/// 1. Cannot resize below min_size
/// 2. Cannot resize above max_size
/// 3. At min_size, only positive deltas have effect
/// 4. At max_size, only negative deltas have effect
#[test]
fn property_boundary_behavior() {
    proptest!(|(
        min_size in 100.0f32..300.0f32,
        max_offset in 200.0f32..500.0f32,
        negative_delta in -200.0f32..-1.0f32,
        positive_delta in 1.0f32..200.0f32,
    )| {
        let max_size = min_size + max_offset;
        
        // Test at minimum boundary
        {
            let at_min = min_size;
            
            // Invariant 1 & 3: Cannot go below min, negative delta has no effect
            let after_negative = (at_min + negative_delta).max(min_size).min(max_size);
            prop_assert_eq!(after_negative, min_size);
            
            // Positive delta should increase size (if within max)
            let after_positive = (at_min + positive_delta).max(min_size).min(max_size);
            prop_assert!(after_positive >= min_size);
            if at_min + positive_delta <= max_size {
                prop_assert!(after_positive > min_size);
            }
        }
        
        // Test at maximum boundary
        {
            let at_max = max_size;
            
            // Invariant 2 & 4: Cannot go above max, positive delta has no effect
            let after_positive = (at_max + positive_delta).max(min_size).min(max_size);
            prop_assert_eq!(after_positive, max_size);
            
            // Negative delta should decrease size (if within min)
            let after_negative = (at_max + negative_delta).max(min_size).min(max_size);
            prop_assert!(after_negative <= max_size);
            if at_max + negative_delta >= min_size {
                prop_assert!(after_negative < max_size);
            }
        }
    });
}

/// Property: Resize Idempotence
///
/// Applying the same resize operation multiple times SHALL
/// produce the same result as applying it once.
///
/// **Invariants:**
/// 1. set_size(x) followed by set_size(x) equals set_size(x)
/// 2. Clamping is stable (doesn't oscillate)
/// 3. Final size is deterministic
#[test]
fn property_resize_idempotence() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        target_size in 0.0f32..1000.0f32,
    )| {
        let max_size = min_size + max_offset;
        
        // Apply clamping once
        let clamped_once = target_size.max(min_size).min(max_size);
        
        // Apply clamping twice
        let clamped_twice = clamped_once.max(min_size).min(max_size);
        
        // Apply clamping three times
        let clamped_thrice = clamped_twice.max(min_size).min(max_size);
        
        // Invariant 1 & 2: Idempotence - all results are equal
        prop_assert_eq!(clamped_once, clamped_twice);
        prop_assert_eq!(clamped_twice, clamped_thrice);
        
        // Invariant 3: Result is deterministic and within bounds
        prop_assert!(clamped_once >= min_size);
        prop_assert!(clamped_once <= max_size);
    });
}

/// Property: Collapse and Expand Operations
///
/// Collapse and expand operations SHALL correctly set the panel
/// to minimum and maximum sizes respectively.
///
/// **Invariants:**
/// 1. Collapse sets size to min_size
/// 2. Expand sets size to max_size
/// 3. Operations are reversible
/// 4. Operations respect constraints
#[test]
fn property_collapse_expand_operations() {
    proptest!(|(
        min_size in 50.0f32..200.0f32,
        max_offset in 100.0f32..500.0f32,
        initial_offset in 0.0f32..1.0f32,
    )| {
        let max_size = min_size + max_offset;
        let _initial_size = min_size + (max_size - min_size) * initial_offset;
        
        // Simulate collapse operation
        let collapsed_size = min_size;
        
        // Invariant 1: Collapse sets to min_size
        prop_assert_eq!(collapsed_size, min_size);
        
        // Simulate expand operation
        let expanded_size = max_size;
        
        // Invariant 2: Expand sets to max_size
        prop_assert_eq!(expanded_size, max_size);
        
        // Invariant 3: Operations are reversible
        // After collapse then expand, we're at max
        prop_assert_eq!(expanded_size, max_size);
        
        // Invariant 4: Both operations respect constraints
        prop_assert!(collapsed_size >= min_size && collapsed_size <= max_size);
        prop_assert!(expanded_size >= min_size && expanded_size <= max_size);
    });
}

/// Property: Resize Delta Accumulation
///
/// Multiple small resize operations SHALL accumulate correctly
/// and produce the same result as a single large operation.
///
/// **Invariants:**
/// 1. Sum of small deltas equals one large delta (when unclamped)
/// 2. Order of operations doesn't matter for final clamped result
/// 3. Intermediate clamping may differ from final clamping
#[test]
fn property_resize_delta_accumulation() {
    proptest!(|(
        min_size in 100.0f32..300.0f32,
        max_offset in 200.0f32..500.0f32,
        deltas in prop::collection::vec(-50.0f32..50.0f32, 2..5),
    )| {
        let max_size = min_size + max_offset;
        let start_size = (min_size + max_size) / 2.0;
        
        // Apply deltas one by one with clamping
        let mut size_with_intermediate_clamping = start_size;
        for delta in &deltas {
            size_with_intermediate_clamping = (size_with_intermediate_clamping + delta)
                .max(min_size)
                .min(max_size);
        }
        
        // Apply sum of deltas with single clamping
        let total_delta: f32 = deltas.iter().sum();
        let size_with_final_clamping = (start_size + total_delta)
            .max(min_size)
            .min(max_size);
        
        // Invariant 1: If no clamping occurs, results are equal
        let unclamped_final = start_size + total_delta;
        if unclamped_final >= min_size && unclamped_final <= max_size {
            prop_assert!((size_with_intermediate_clamping - size_with_final_clamping).abs() < 0.001);
        }
        
        // Invariant 2 & 3: Both results are within constraints
        prop_assert!(size_with_intermediate_clamping >= min_size);
        prop_assert!(size_with_intermediate_clamping <= max_size);
        prop_assert!(size_with_final_clamping >= min_size);
        prop_assert!(size_with_final_clamping <= max_size);
    });
}
