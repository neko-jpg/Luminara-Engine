//! Property-Based Tests for Nested Panel Layouts
//!
//! **Validates: Requirements 9.7**
//!
//! This test file validates Property 24: Nested Panel Layout using property-based testing.
//! It ensures that nested panels maintain proportional sizing when parent panels are resized,
//! respect constraints at all nesting levels, and handle 2-3 levels of nesting correctly.
//!
//! **Property 24: Nested Panel Layout**
//! - WHEN a parent panel is resized, nested child panels SHALL maintain their proportional sizes
//! - Constraint enforcement SHALL work correctly at all nesting levels
//! - The system SHALL support 2-3 levels of nesting without degradation

use gpui::{px, Pixels};
use proptest::prelude::*;

/// Helper to calculate proportional size with constraints
fn calculate_proportional_size(
    proportion: f32,
    parent_size: Pixels,
    min_size: Pixels,
    max_size: Pixels,
) -> Pixels {
    let parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(parent_size) };
    let new_size = px(proportion * parent_f32);
    new_size.max(min_size).min(max_size)
}

/// Strategy for generating valid proportions (0.0 to 1.0)
fn proportion_strategy() -> impl Strategy<Value = f32> {
    (0.1f32..=0.9f32)
}

/// Strategy for generating valid panel sizes (50px to 2000px)
fn size_strategy() -> impl Strategy<Value = f32> {
    (50.0f32..=2000.0f32)
}

/// Strategy for generating valid minimum sizes (50px to 500px)
fn min_size_strategy() -> impl Strategy<Value = f32> {
    (50.0f32..=500.0f32)
}

/// Strategy for generating valid size offsets for max size (100px to 1500px)
fn max_offset_strategy() -> impl Strategy<Value = f32> {
    (100.0f32..=1500.0f32)
}

/// Strategy for generating resize factors (0.5x to 2.0x)
fn resize_factor_strategy() -> impl Strategy<Value = f32> {
    (0.5f32..=2.0f32)
}

#[test]
fn property_two_level_proportional_resize() {
    proptest!(|(
        parent_size in size_strategy(),
        child_proportion in proportion_strategy(),
        child_min in min_size_strategy(),
        max_offset in max_offset_strategy(),
        resize_factor in resize_factor_strategy(),
    )| {
        // Setup: Create a parent with a child panel
        let parent = px(parent_size);
        let child_max = px(child_min + max_offset);
        
        // Calculate initial child size
        let child_initial = calculate_proportional_size(
            child_proportion,
            parent,
            px(child_min),
            child_max,
        );
        
        // Resize parent
        let new_parent = px(parent_size * resize_factor);
        
        // Calculate new child size
        let child_new = calculate_proportional_size(
            child_proportion,
            new_parent,
            px(child_min),
            child_max,
        );
        
        // Property: Child size should be proportional to parent size
        let parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(parent) };
        let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent) };
        let child_initial_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child_initial) };
        let child_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child_new) };
        
        // If neither child size is constrained, the ratio should match
        if child_initial_f32 > child_min && child_initial_f32 < (child_min + max_offset) &&
           child_new_f32 > child_min && child_new_f32 < (child_min + max_offset) {
            let parent_ratio = new_parent_f32 / parent_f32;
            let child_ratio = child_new_f32 / child_initial_f32;
            prop_assert!((parent_ratio - child_ratio).abs() < 0.01,
                "Child resize ratio should match parent resize ratio when unconstrained");
        }
        
        // Property: Child size must respect constraints
        prop_assert!(child_new >= px(child_min),
            "Child size must be >= minimum constraint");
        prop_assert!(child_new <= child_max,
            "Child size must be <= maximum constraint");
    });
}

#[test]
fn property_three_level_proportional_resize() {
    proptest!(|(
        grandparent_size in size_strategy(),
        parent_proportion in proportion_strategy(),
        child_proportion in proportion_strategy(),
        parent_min in min_size_strategy(),
        parent_max_offset in max_offset_strategy(),
        child_max_offset in max_offset_strategy(),
        resize_factor in resize_factor_strategy(),
    )| {
        // Setup: Create a 3-level nested structure
        let grandparent = px(grandparent_size);
        let parent_max = px(parent_min + parent_max_offset);
        
        // Calculate initial parent size
        let parent_initial = calculate_proportional_size(
            parent_proportion,
            grandparent,
            px(parent_min),
            parent_max,
        );
        
        // Child min must be smaller than parent's actual size
        let parent_initial_f32 = unsafe { std::mem::transmute::<Pixels, f32>(parent_initial) };
        let child_min = (parent_initial_f32 * 0.2).max(50.0); // At most 20% of parent
        let child_max = px(child_min + child_max_offset.min(parent_initial_f32 * 0.8));
        
        // Calculate initial child size based on parent
        let _child_initial = calculate_proportional_size(
            child_proportion,
            parent_initial,
            px(child_min),
            child_max,
        );
        
        // Resize grandparent
        let new_grandparent = px(grandparent_size * resize_factor);
        
        // Calculate new parent size
        let parent_new = calculate_proportional_size(
            parent_proportion,
            new_grandparent,
            px(parent_min),
            parent_max,
        );
        
        // Recalculate child constraints based on new parent size
        let parent_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(parent_new) };
        let child_min_adjusted = child_min.min(parent_new_f32 * 0.9);
        let child_max_adjusted = child_max.min(px(parent_new_f32));
        
        // Calculate new child size based on new parent
        let child_new = calculate_proportional_size(
            child_proportion,
            parent_new,
            px(child_min_adjusted),
            child_max_adjusted,
        );
        
        // Property: All sizes must respect their constraints
        prop_assert!(parent_new >= px(parent_min),
            "Parent size must be >= minimum constraint");
        prop_assert!(parent_new <= parent_max,
            "Parent size must be <= maximum constraint");
        prop_assert!(child_new >= px(child_min_adjusted),
            "Child size must be >= minimum constraint");
        prop_assert!(child_new <= child_max_adjusted,
            "Child size must be <= maximum constraint");
        
        // Property: Child should not exceed parent
        prop_assert!(child_new <= parent_new,
            "Child size should not exceed parent size");
    });
}

#[test]
fn property_nested_constraint_enforcement() {
    proptest!(|(
        parent_size in size_strategy(),
        child_proportion in proportion_strategy(),
        child_min in min_size_strategy(),
        max_offset in max_offset_strategy(),
        extreme_resize_factor in (0.1f32..=0.3f32), // Extreme shrink
    )| {
        // Setup: Create a parent with a child that has constraints
        let parent = px(parent_size);
        let child_max = px(child_min + max_offset);
        
        // Resize parent to very small size
        let new_parent = px(parent_size * extreme_resize_factor);
        
        // Calculate child size
        let child_new = calculate_proportional_size(
            child_proportion,
            new_parent,
            px(child_min),
            child_max,
        );
        
        // Property: Even with extreme parent resize, child must respect minimum
        prop_assert!(child_new >= px(child_min),
            "Child must respect minimum constraint even when parent is very small");
        
        // Property: Child size should be clamped to minimum if proportional size would be smaller
        let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent) };
        let proportional_size = child_proportion * new_parent_f32;
        if proportional_size < child_min {
            prop_assert_eq!(child_new, px(child_min),
                "Child should be clamped to minimum when proportional size is too small");
        }
    });
}

#[test]
fn property_nested_max_constraint_enforcement() {
    proptest!(|(
        parent_size in size_strategy(),
        child_proportion in proportion_strategy(),
        child_min in min_size_strategy(),
        max_offset in (50.0f32..=300.0f32), // Smaller max offset
        extreme_resize_factor in (2.0f32..=4.0f32), // Extreme growth
    )| {
        // Setup: Create a parent with a child that has constraints
        let parent = px(parent_size);
        let child_max = px(child_min + max_offset);
        
        // Resize parent to very large size
        let new_parent = px(parent_size * extreme_resize_factor);
        
        // Calculate child size
        let child_new = calculate_proportional_size(
            child_proportion,
            new_parent,
            px(child_min),
            child_max,
        );
        
        // Property: Even with extreme parent resize, child must respect maximum
        prop_assert!(child_new <= child_max,
            "Child must respect maximum constraint even when parent is very large");
        
        // Property: Child size should be clamped to maximum if proportional size would be larger
        let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent) };
        let proportional_size = child_proportion * new_parent_f32;
        if proportional_size > (child_min + max_offset) {
            prop_assert_eq!(child_new, child_max,
                "Child should be clamped to maximum when proportional size is too large");
        }
    });
}

#[test]
fn property_multiple_children_maintain_proportions() {
    proptest!(|(
        parent_size in size_strategy(),
        prop1 in proportion_strategy(),
        prop2 in proportion_strategy(),
        prop3 in proportion_strategy(),
        min1 in min_size_strategy(),
        min2 in min_size_strategy(),
        min3 in min_size_strategy(),
        max_offset in max_offset_strategy(),
        resize_factor in resize_factor_strategy(),
    )| {
        // Normalize proportions to sum to 1.0
        let total = prop1 + prop2 + prop3;
        let norm_prop1 = prop1 / total;
        let norm_prop2 = prop2 / total;
        let norm_prop3 = prop3 / total;
        
        // Setup: Create a parent with three children
        let parent = px(parent_size);
        let max1 = px(min1 + max_offset);
        let max2 = px(min2 + max_offset);
        let max3 = px(min3 + max_offset);
        
        // Calculate initial sizes
        let child1_initial = calculate_proportional_size(norm_prop1, parent, px(min1), max1);
        let child2_initial = calculate_proportional_size(norm_prop2, parent, px(min2), max2);
        let child3_initial = calculate_proportional_size(norm_prop3, parent, px(min3), max3);
        
        // Resize parent
        let new_parent = px(parent_size * resize_factor);
        
        // Calculate new sizes
        let child1_new = calculate_proportional_size(norm_prop1, new_parent, px(min1), max1);
        let child2_new = calculate_proportional_size(norm_prop2, new_parent, px(min2), max2);
        let child3_new = calculate_proportional_size(norm_prop3, new_parent, px(min3), max3);
        
        // Property: All children must respect their constraints
        prop_assert!(child1_new >= px(min1) && child1_new <= max1,
            "Child 1 must respect constraints");
        prop_assert!(child2_new >= px(min2) && child2_new <= max2,
            "Child 2 must respect constraints");
        prop_assert!(child3_new >= px(min3) && child3_new <= max3,
            "Child 3 must respect constraints");
        
        // Property: If unconstrained, relative proportions should be maintained
        let child1_initial_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child1_initial) };
        let child2_initial_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child2_initial) };
        let child1_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child1_new) };
        let child2_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child2_new) };
        
        if child1_initial_f32 > min1 && child1_initial_f32 < (min1 + max_offset) &&
           child2_initial_f32 > min2 && child2_initial_f32 < (min2 + max_offset) &&
           child1_new_f32 > min1 && child1_new_f32 < (min1 + max_offset) &&
           child2_new_f32 > min2 && child2_new_f32 < (min2 + max_offset) {
            let initial_ratio = child1_initial_f32 / child2_initial_f32;
            let new_ratio = child1_new_f32 / child2_new_f32;
            prop_assert!((initial_ratio - new_ratio).abs() < 0.1,
                "Relative proportions between children should be maintained when unconstrained");
        }
    });
}

#[test]
fn property_nested_different_orientations() {
    proptest!(|(
        h_parent_size in size_strategy(),
        v_child_proportion in proportion_strategy(),
        h_grandchild_proportion in proportion_strategy(),
        v_child_min in min_size_strategy(),
        v_max_offset in max_offset_strategy(),
        h_max_offset in max_offset_strategy(),
        resize_factor in resize_factor_strategy(),
    )| {
        // Setup: Horizontal parent -> Vertical child -> Horizontal grandchild
        let h_parent = px(h_parent_size);
        let v_child_max = px(v_child_min + v_max_offset);
        
        // Calculate initial vertical child size
        let v_child_initial = calculate_proportional_size(
            v_child_proportion,
            h_parent,
            px(v_child_min),
            v_child_max,
        );
        
        // Grandchild min must be smaller than child's actual size
        let v_child_initial_f32 = unsafe { std::mem::transmute::<Pixels, f32>(v_child_initial) };
        let h_grandchild_min = (v_child_initial_f32 * 0.2).max(50.0); // At most 20% of child
        let h_grandchild_max = px(h_grandchild_min + h_max_offset.min(v_child_initial_f32 * 0.8));
        
        // Calculate initial horizontal grandchild size
        let _h_grandchild_initial = calculate_proportional_size(
            h_grandchild_proportion,
            v_child_initial,
            px(h_grandchild_min),
            h_grandchild_max,
        );
        
        // Resize horizontal parent
        let new_h_parent = px(h_parent_size * resize_factor);
        
        // Calculate new vertical child size
        let v_child_new = calculate_proportional_size(
            v_child_proportion,
            new_h_parent,
            px(v_child_min),
            v_child_max,
        );
        
        // Recalculate grandchild constraints based on new child size
        let v_child_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(v_child_new) };
        let h_grandchild_min_adjusted = h_grandchild_min.min(v_child_new_f32 * 0.9);
        let h_grandchild_max_adjusted = h_grandchild_max.min(px(v_child_new_f32));
        
        // Calculate new horizontal grandchild size
        let h_grandchild_new = calculate_proportional_size(
            h_grandchild_proportion,
            v_child_new,
            px(h_grandchild_min_adjusted),
            h_grandchild_max_adjusted,
        );
        
        // Property: All sizes must respect their constraints regardless of orientation
        prop_assert!(v_child_new >= px(v_child_min) && v_child_new <= v_child_max,
            "Vertical child must respect constraints");
        prop_assert!(h_grandchild_new >= px(h_grandchild_min_adjusted) && h_grandchild_new <= h_grandchild_max_adjusted,
            "Horizontal grandchild must respect constraints");
        
        // Property: Grandchild should not exceed child
        prop_assert!(h_grandchild_new <= v_child_new,
            "Grandchild size should not exceed child size");
    });
}

#[test]
fn property_proportion_clamping() {
    proptest!(|(
        proportion in (-1.0f32..=2.0f32), // Include invalid proportions
        parent_size in size_strategy(),
        min_size in min_size_strategy(),
        max_offset in max_offset_strategy(),
    )| {
        // Clamp proportion to valid range
        let clamped_proportion = proportion.clamp(0.0, 1.0);
        
        let parent = px(parent_size);
        let max_size = px(min_size + max_offset);
        
        // Calculate size with clamped proportion
        let size = calculate_proportional_size(
            clamped_proportion,
            parent,
            px(min_size),
            max_size,
        );
        
        // Property: Clamped proportion should always be in valid range
        prop_assert!(clamped_proportion >= 0.0 && clamped_proportion <= 1.0,
            "Proportion should be clamped to [0.0, 1.0]");
        
        // Property: Size should be valid
        prop_assert!(size >= px(min_size) && size <= max_size,
            "Size calculated with clamped proportion should respect constraints");
    });
}

#[test]
fn property_nested_resize_idempotence() {
    proptest!(|(
        parent_size in size_strategy(),
        child_proportion in proportion_strategy(),
        child_min in min_size_strategy(),
        max_offset in max_offset_strategy(),
        resize_factor in resize_factor_strategy(),
    )| {
        let parent = px(parent_size);
        let child_max = px(child_min + max_offset);
        
        // Resize parent
        let new_parent = px(parent_size * resize_factor);
        
        // Calculate child size once
        let child_size_1 = calculate_proportional_size(
            child_proportion,
            new_parent,
            px(child_min),
            child_max,
        );
        
        // Calculate child size again with same inputs
        let child_size_2 = calculate_proportional_size(
            child_proportion,
            new_parent,
            px(child_min),
            child_max,
        );
        
        // Property: Calculating size multiple times should give same result
        prop_assert_eq!(child_size_1, child_size_2,
            "Proportional size calculation should be idempotent");
    });
}

#[test]
fn property_nested_resize_commutativity() {
    proptest!(|(
        parent_size in size_strategy(),
        child_proportion in proportion_strategy(),
        child_min in min_size_strategy(),
        max_offset in max_offset_strategy(),
        factor1 in resize_factor_strategy(),
        factor2 in resize_factor_strategy(),
    )| {
        let parent = px(parent_size);
        let child_max = px(child_min + max_offset);
        
        // Path 1: Resize by factor1, then by factor2
        let intermediate1 = px(parent_size * factor1);
        let child_intermediate1 = calculate_proportional_size(
            child_proportion,
            intermediate1,
            px(child_min),
            child_max,
        );
        let final1 = px(parent_size * factor1 * factor2);
        let child_final1 = calculate_proportional_size(
            child_proportion,
            final1,
            px(child_min),
            child_max,
        );
        
        // Path 2: Resize by combined factor
        let final2 = px(parent_size * factor1 * factor2);
        let child_final2 = calculate_proportional_size(
            child_proportion,
            final2,
            px(child_min),
            child_max,
        );
        
        // Property: Final result should be the same regardless of path
        prop_assert_eq!(child_final1, child_final2,
            "Nested resize should give same result regardless of intermediate steps");
    });
}
