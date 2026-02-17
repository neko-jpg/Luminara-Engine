//! Transform Debug Visualization Demo
//!
//! Demonstrates the transform debug visualization features:
//! - Coordinate axes for entities
//! - Hierarchy connections between parent and child entities
//! - Selection highlighting
//!
//! This example shows how to use the GizmoSystem to visualize entity transforms
//! and their hierarchical relationships.

use luminara_render::{CommandBuffer, GizmoSystem, VisualizationMode};
use luminara_math::Vec3;

fn main() {
    println!("Transform Debug Visualization Demo");
    println!("===================================\n");

    // Create gizmo system
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();

    // Enable transform visualization mode
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    println!("✓ Transform visualization mode enabled");

    // Example 1: Draw coordinate axes for a single entity
    println!("\n1. Drawing coordinate axes at origin:");
    gizmo_system.draw_transform_axes(&mut buffer, Vec3::ZERO, 1.0);
    println!("   - X axis (red) pointing right");
    println!("   - Y axis (green) pointing up");
    println!("   - Z axis (blue) pointing forward");
    println!("   Commands in buffer: {}", buffer.commands.len());

    // Example 2: Draw hierarchy connection
    println!("\n2. Drawing hierarchy connection:");
    gizmo_system.transform_settings_mut().show_hierarchy = true;
    let parent_pos = Vec3::ZERO;
    let child_pos = Vec3::new(2.0, 1.0, 0.0);
    gizmo_system.draw_hierarchy_connection(&mut buffer, parent_pos, child_pos);
    println!("   - Line from parent (0, 0, 0) to child (2, 1, 0)");
    println!("   Commands in buffer: {}", buffer.commands.len());

    // Example 3: Highlight selected entity
    println!("\n3. Highlighting selected entity:");
    let selected_pos = Vec3::new(5.0, 0.0, 0.0);
    gizmo_system.draw_entity_highlight(&mut buffer, selected_pos, 1.5);
    println!("   - Yellow sphere at (5, 0, 0) with radius 1.5");
    println!("   Commands in buffer: {}", buffer.commands.len());

    // Example 4: Complete entity transform visualization
    println!("\n4. Complete entity transform visualization:");
    buffer.clear();
    
    // Root entity (not selected)
    let root_pos = Vec3::ZERO;
    gizmo_system.draw_entity_transform(&mut buffer, root_pos, None, false, 1.0);
    println!("   Root entity at origin:");
    println!("   - Coordinate axes");
    println!("   - No parent connection");
    println!("   - Not selected");
    
    // Child entity (selected)
    let child_pos = Vec3::new(3.0, 2.0, 1.0);
    gizmo_system.draw_entity_transform(&mut buffer, child_pos, Some(root_pos), true, 1.0);
    println!("   Child entity at (3, 2, 1):");
    println!("   - Coordinate axes");
    println!("   - Hierarchy line to parent");
    println!("   - Selection highlight (yellow sphere)");
    
    println!("\n   Total commands in buffer: {}", buffer.commands.len());

    // Example 5: Customize visualization settings
    println!("\n5. Customizing visualization settings:");
    let settings = gizmo_system.transform_settings_mut();
    settings.axes_length = 2.0;
    settings.hierarchy_color = luminara_math::Color::rgb(0.0, 1.0, 1.0);
    settings.selection_color = luminara_math::Color::rgb(1.0, 0.0, 1.0);
    println!("   - Axes length: 2.0");
    println!("   - Hierarchy color: cyan");
    println!("   - Selection color: magenta");

    // Example 6: Complex hierarchy
    println!("\n6. Complex hierarchy visualization:");
    buffer.clear();
    
    let positions = vec![
        (Vec3::ZERO, None, false),                                    // Root
        (Vec3::new(2.0, 0.0, 0.0), Some(Vec3::ZERO), false),         // Child 1
        (Vec3::new(-2.0, 0.0, 0.0), Some(Vec3::ZERO), true),         // Child 2 (selected)
        (Vec3::new(2.0, 2.0, 0.0), Some(Vec3::new(2.0, 0.0, 0.0)), false), // Grandchild
    ];
    
    for (pos, parent_pos, is_selected) in positions {
        gizmo_system.draw_entity_transform(&mut buffer, pos, parent_pos, is_selected, 1.0);
    }
    
    println!("   Hierarchy structure:");
    println!("   Root (0, 0, 0)");
    println!("   ├── Child 1 (2, 0, 0)");
    println!("   │   └── Grandchild (2, 2, 0)");
    println!("   └── Child 2 (-2, 0, 0) [SELECTED]");
    println!("   Total commands: {}", buffer.commands.len());

    // Example 7: Toggle visualization modes
    println!("\n7. Toggling visualization modes:");
    println!("   Initial state:");
    println!("   - show_axes: {}", gizmo_system.transform_settings().show_axes);
    println!("   - show_hierarchy: {}", gizmo_system.transform_settings().show_hierarchy);
    
    gizmo_system.transform_settings_mut().show_axes = false;
    println!("   After disabling axes:");
    println!("   - show_axes: {}", gizmo_system.transform_settings().show_axes);
    
    buffer.clear();
    gizmo_system.draw_entity_transform(&mut buffer, Vec3::ZERO, None, false, 1.0);
    println!("   Commands (axes disabled): {}", buffer.commands.len());
    
    gizmo_system.transform_settings_mut().show_axes = true;
    buffer.clear();
    gizmo_system.draw_entity_transform(&mut buffer, Vec3::ZERO, None, false, 1.0);
    println!("   Commands (axes enabled): {}", buffer.commands.len());

    println!("\n✓ Transform debug visualization demo complete!");
    println!("\nUsage in a real application:");
    println!("1. Create GizmoSystem and add it as a resource");
    println!("2. Enable VisualizationMode::Transforms");
    println!("3. In your render system, query entities with Transform component");
    println!("4. Call draw_entity_transform() for each entity");
    println!("5. Optionally query Parent/Children components for hierarchy");
    println!("6. Pass is_selected flag based on your selection system");
}
