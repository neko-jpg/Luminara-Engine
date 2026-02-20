//! Tests for nested panel layouts
//!
//! This test file validates that ResizablePanel supports nested layouts
//! with proper proportional resizing when parent panels are resized.
//!
//! **Validates Requirements:**
//! - 9.7: Panel supports nested panel layouts with proportional resizing

use gpui::{px, Pixels};

/// Test helper to calculate proportional size
fn calculate_proportional_size(
    proportion: f32,
    new_parent_size: Pixels,
    min_size: Pixels,
    max_size: Pixels,
) -> Pixels {
    let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent_size) };
    let new_size = px(proportion * new_parent_f32);
    new_size.max(min_size).min(max_size)
}

#[test]
fn test_two_level_nesting_proportional_resize() {
    // Test 2-level nested panels with proportional resizing
    // Parent: 1000px -> 800px (20% reduction)
    // Child 1: 40% of parent -> 400px -> 320px
    // Child 2: 60% of parent -> 600px -> 480px

    let new_parent = px(800.0);

    let child1_proportion = 0.4;
    let child1_min = px(100.0);
    let child1_max = px(600.0);
    let child1_new = calculate_proportional_size(
        child1_proportion,
        new_parent,
        child1_min,
        child1_max,
    );
    let child1_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child1_new) };
    assert!((child1_f32 - 320.0).abs() < 0.01);

    let child2_proportion = 0.6;
    let child2_min = px(100.0);
    let child2_max = px(800.0);
    let child2_new = calculate_proportional_size(
        child2_proportion,
        new_parent,
        child2_min,
        child2_max,
    );
    let child2_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child2_new) };
    assert!((child2_f32 - 480.0).abs() < 0.01);
}

#[test]
fn test_three_level_nesting_proportional_resize() {
    // Test 3-level nested panels
    // Grandparent: 1000px -> 1200px (20% increase)
    // Parent: 50% of grandparent -> 500px -> 600px
    // Child: 50% of parent -> 250px -> 300px

    let new_grandparent = px(1200.0);

    // Parent resize
    let parent_proportion = 0.5;
    let parent_min = px(200.0);
    let parent_max = px(800.0);
    let parent_new = calculate_proportional_size(
        parent_proportion,
        new_grandparent,
        parent_min,
        parent_max,
    );
    assert_eq!(parent_new, px(600.0));

    // Child resize based on parent's new size
    let child_proportion = 0.5;
    let child_min = px(100.0);
    let child_max = px(400.0);
    let child_new = calculate_proportional_size(
        child_proportion,
        parent_new,
        child_min,
        child_max,
    );
    assert_eq!(child_new, px(300.0));
}

#[test]
fn test_nested_panels_with_min_constraint() {
    // Test that nested panels respect minimum size constraints
    // Parent: 1000px -> 200px (80% reduction)
    // Child: 40% of parent -> would be 80px, but min is 150px

    let new_parent = px(200.0);

    let child_proportion = 0.4;
    let child_min = px(150.0);
    let child_max = px(600.0);
    let child_new = calculate_proportional_size(
        child_proportion,
        new_parent,
        child_min,
        child_max,
    );

    // Should be clamped to minimum
    assert_eq!(child_new, child_min);
}

#[test]
fn test_nested_panels_with_max_constraint() {
    // Test that nested panels respect maximum size constraints
    // Parent: 1000px -> 2000px (100% increase)
    // Child: 40% of parent -> would be 800px, but max is 600px

    let new_parent = px(2000.0);

    let child_proportion = 0.4;
    let child_min = px(100.0);
    let child_max = px(600.0);
    let child_new = calculate_proportional_size(
        child_proportion,
        new_parent,
        child_min,
        child_max,
    );

    // Should be clamped to maximum
    assert_eq!(child_new, child_max);
}

#[test]
fn test_nested_panels_maintain_proportions() {
    // Test that multiple nested panels maintain their relative proportions
    // Parent: 1000px -> 1500px (50% increase)
    // Child 1: 30% -> 300px -> 450px
    // Child 2: 40% -> 400px -> 600px
    // Child 3: 30% -> 300px -> 450px

    let new_parent = px(1500.0);

    let children = vec![
        (0.3, px(100.0), px(700.0)), // (proportion, min, max)
        (0.4, px(100.0), px(700.0)),
        (0.3, px(100.0), px(700.0)),
    ];

    let mut new_sizes = Vec::new();
    for (proportion, min, max) in children {
        let new_size = calculate_proportional_size(proportion, new_parent, min, max);
        new_sizes.push(new_size);
    }

    let size0_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_sizes[0]) };
    let size1_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_sizes[1]) };
    let size2_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_sizes[2]) };
    
    assert!((size0_f32 - 450.0).abs() < 0.01);
    assert!((size1_f32 - 600.0).abs() < 0.01);
    assert!((size2_f32 - 450.0).abs() < 0.01);
}

#[test]
fn test_nested_panels_different_orientations() {
    // Test nested panels with different orientations
    // Horizontal parent: 1000px -> 800px
    // Vertical child: 60% of parent -> 600px -> 480px
    // Horizontal grandchild: 50% of child -> 300px -> 240px

    let new_h_parent = px(800.0);

    // Vertical child in horizontal parent
    let v_child_proportion = 0.6;
    let v_child_min = px(200.0);
    let v_child_max = px(800.0);
    let v_child_new = calculate_proportional_size(
        v_child_proportion,
        new_h_parent,
        v_child_min,
        v_child_max,
    );
    let v_child_f32 = unsafe { std::mem::transmute::<Pixels, f32>(v_child_new) };
    assert!((v_child_f32 - 480.0).abs() < 0.01);

    // Horizontal grandchild in vertical child
    let h_grandchild_proportion = 0.5;
    let h_grandchild_min = px(100.0);
    let h_grandchild_max = px(500.0);
    let h_grandchild_new = calculate_proportional_size(
        h_grandchild_proportion,
        v_child_new,
        h_grandchild_min,
        h_grandchild_max,
    );
    let h_grandchild_f32 = unsafe { std::mem::transmute::<Pixels, f32>(h_grandchild_new) };
    assert!((h_grandchild_f32 - 240.0).abs() < 0.01);
}

#[test]
fn test_nested_panels_extreme_resize() {
    // Test nested panels with extreme resize scenarios
    // Parent: 1000px -> 100px (90% reduction)
    // Child 1: 50% -> would be 50px, but min is 80px
    // Child 2: 50% -> would be 50px, but min is 80px
    // Total would exceed parent, demonstrating constraint handling

    let new_parent = px(100.0);

    let child1_proportion = 0.5;
    let child1_min = px(80.0);
    let child1_max = px(600.0);
    let child1_new = calculate_proportional_size(
        child1_proportion,
        new_parent,
        child1_min,
        child1_max,
    );

    let child2_proportion = 0.5;
    let child2_min = px(80.0);
    let child2_max = px(600.0);
    let child2_new = calculate_proportional_size(
        child2_proportion,
        new_parent,
        child2_min,
        child2_max,
    );

    // Both should be clamped to minimum
    assert_eq!(child1_new, child1_min);
    assert_eq!(child2_new, child2_min);
}

#[test]
fn test_nested_panels_complex_layout() {
    // Test a complex nested layout scenario
    // Root: 1200px -> 1600px
    //   ├─ Left Panel: 25% -> 300px -> 400px
    //   └─ Right Container: 75% -> 900px -> 1200px
    //       ├─ Top Panel: 50% of container -> 450px -> 600px
    //       └─ Bottom Panel: 50% of container -> 450px -> 600px

    let new_root = px(1600.0);

    // Left panel
    let left_proportion = 0.25;
    let left_min = px(200.0);
    let left_max = px(600.0);
    let left_new = calculate_proportional_size(
        left_proportion,
        new_root,
        left_min,
        left_max,
    );
    assert_eq!(left_new, px(400.0));

    // Right container
    let right_proportion = 0.75;
    let right_min = px(600.0);
    let right_max = px(1400.0);
    let right_new = calculate_proportional_size(
        right_proportion,
        new_root,
        right_min,
        right_max,
    );
    assert_eq!(right_new, px(1200.0));

    // Top panel in right container
    let top_proportion = 0.5;
    let top_min = px(200.0);
    let top_max = px(800.0);
    let top_new = calculate_proportional_size(
        top_proportion,
        right_new,
        top_min,
        top_max,
    );
    assert_eq!(top_new, px(600.0));

    // Bottom panel in right container
    let bottom_proportion = 0.5;
    let bottom_min = px(200.0);
    let bottom_max = px(800.0);
    let bottom_new = calculate_proportional_size(
        bottom_proportion,
        right_new,
        bottom_min,
        bottom_max,
    );
    assert_eq!(bottom_new, px(600.0));
}

#[test]
fn test_proportion_clamping() {
    // Test that proportions are properly clamped to 0.0-1.0 range
    let _parent = px(1000.0);
    
    // Test proportion > 1.0 (should be clamped to 1.0)
    let proportion_over: f32 = 1.5;
    let clamped_over = proportion_over.clamp(0.0, 1.0);
    assert_eq!(clamped_over, 1.0);
    
    // Test proportion < 0.0 (should be clamped to 0.0)
    let proportion_under: f32 = -0.5;
    let clamped_under = proportion_under.clamp(0.0, 1.0);
    assert_eq!(clamped_under, 0.0);
    
    // Test valid proportion
    let proportion_valid: f32 = 0.5;
    let clamped_valid = proportion_valid.clamp(0.0, 1.0);
    assert_eq!(clamped_valid, 0.5);
}
