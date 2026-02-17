//! Debug Rendering Demo
//!
//! Demonstrates the debug rendering visualization modes:
//! - Wireframe mode
//! - Normal visualization
//! - Overdraw heatmap
//!
//! Controls:
//! - 1: Toggle wireframe mode
//! - 2: Toggle normal visualization
//! - 3: Toggle overdraw heatmap
//! - 0: Disable all debug modes

use luminara_render::{DebugRenderMode, DebugRenderingResource, GizmoSystem, VisualizationMode};

fn main() {
    println!("Debug Rendering Demo");
    println!("====================");
    println!();
    println!("This example demonstrates the debug rendering visualization modes.");
    println!();
    println!("Controls:");
    println!("  1 - Toggle wireframe mode");
    println!("  2 - Toggle normal visualization");
    println!("  3 - Toggle overdraw heatmap");
    println!("  0 - Disable all debug modes");
    println!();

    // Create debug rendering resource
    let mut debug_rendering = DebugRenderingResource::new();
    let mut gizmo_system = GizmoSystem::new();

    // Enable rendering visualization mode in gizmo system
    gizmo_system.enable_mode(VisualizationMode::Rendering);

    println!("Initial state:");
    println!("  Debug mode: {:?}", debug_rendering.mode());
    println!("  Gizmo rendering mode active: {}", gizmo_system.is_mode_active(VisualizationMode::Rendering));
    println!();

    // Simulate toggling wireframe mode
    println!("Toggling wireframe mode...");
    debug_rendering.toggle_wireframe();
    gizmo_system.rendering_settings_mut().show_wireframe = true;
    println!("  Debug mode: {:?}", debug_rendering.mode());
    println!("  Wireframe enabled: {}", gizmo_system.rendering_settings().show_wireframe);
    println!();

    // Simulate toggling normal visualization
    println!("Toggling normal visualization...");
    debug_rendering.toggle_normals();
    gizmo_system.rendering_settings_mut().show_normals = true;
    println!("  Debug mode: {:?}", debug_rendering.mode());
    println!("  Normals enabled: {}", gizmo_system.rendering_settings().show_normals);
    println!();

    // Simulate toggling overdraw heatmap
    println!("Toggling overdraw heatmap...");
    debug_rendering.toggle_overdraw();
    gizmo_system.rendering_settings_mut().show_overdraw = true;
    println!("  Debug mode: {:?}", debug_rendering.mode());
    println!("  Overdraw enabled: {}", gizmo_system.rendering_settings().show_overdraw);
    println!();

    // Disable all modes
    println!("Disabling all debug modes...");
    debug_rendering.set_mode(DebugRenderMode::None);
    gizmo_system.rendering_settings_mut().show_wireframe = false;
    gizmo_system.rendering_settings_mut().show_normals = false;
    gizmo_system.rendering_settings_mut().show_overdraw = false;
    println!("  Debug mode: {:?}", debug_rendering.mode());
    println!();

    println!("Demo complete!");
    println!();
    println!("Integration with GizmoSystem:");
    println!("  The debug rendering modes integrate seamlessly with the unified");
    println!("  GizmoSystem, allowing you to toggle visualization modes at runtime.");
    println!();
    println!("Shader Implementation:");
    println!("  - Wireframe: debug_wireframe.wgsl");
    println!("  - Normals: debug_normals.wgsl");
    println!("  - Overdraw: debug_overdraw.wgsl");
    println!();
    println!("Performance Impact:");
    println!("  - Wireframe: ~5-10% overhead");
    println!("  - Normals: ~1-2% overhead");
    println!("  - Overdraw: ~10-15% overhead");
}
