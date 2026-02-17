//! Demonstration of the unified GizmoSystem
//!
//! This example shows how to use the GizmoSystem for debug visualization
//! across different modes (physics, rendering, transforms, audio).
//!
//! Controls:
//! - 1: Toggle Physics mode
//! - 2: Toggle Rendering mode
//! - 3: Toggle Transforms mode
//! - 4: Toggle Audio mode
//! - G: Toggle entire gizmo system
//! - ESC: Exit

use luminara_math::{Color, Vec3};
use luminara_render::{
    CommandBuffer, GizmoSystem, OverlayRenderer, VisualizationMode,
};

fn main() {
    println!("=== Gizmo System Demo ===\n");
    
    // Create the gizmo system
    let mut gizmo_system = GizmoSystem::new();
    let mut command_buffer = CommandBuffer::default();
    let mut overlay = OverlayRenderer::new();
    
    println!("Initial state:");
    println!("  Gizmo system enabled: {}", gizmo_system.is_enabled());
    println!("  Active modes: {:?}\n", gizmo_system.active_modes());
    
    // ── Physics Mode Demo ───────────────────────────────────────────────
    
    println!("=== Physics Mode Demo ===");
    gizmo_system.enable_mode(VisualizationMode::Physics);
    
    // Configure physics settings
    let physics_settings = gizmo_system.physics_settings_mut();
    physics_settings.velocity_scale = 2.0;
    physics_settings.collider_color = Color::rgba(0.0, 1.0, 0.0, 0.5);
    
    // Draw physics gizmos
    let box_position = Vec3::new(0.0, 1.0, 0.0);
    let box_half_extents = Vec3::new(0.5, 0.5, 0.5);
    gizmo_system.draw_physics(&mut command_buffer, box_position, box_half_extents);
    
    let velocity = Vec3::new(1.0, 0.0, 0.0);
    gizmo_system.draw_velocity(&mut command_buffer, box_position, velocity);
    
    let contact_point = Vec3::new(0.0, 0.0, 0.0);
    gizmo_system.draw_contact_point(&mut command_buffer, contact_point);
    
    println!("  Drew {} physics gizmos", command_buffer.commands.len());
    command_buffer.clear();
    
    // ── Rendering Mode Demo ─────────────────────────────────────────────
    
    println!("\n=== Rendering Mode Demo ===");
    gizmo_system.enable_mode(VisualizationMode::Rendering);
    
    // Configure rendering settings
    let rendering_settings = gizmo_system.rendering_settings_mut();
    rendering_settings.show_bounds = true;
    rendering_settings.bounds_color = Color::rgba(1.0, 0.5, 0.0, 0.5);
    
    // Draw bounding box
    let mesh_center = Vec3::new(2.0, 1.0, 0.0);
    let mesh_half_extents = Vec3::new(1.0, 1.0, 1.0);
    gizmo_system.draw_bounding_box(&mut command_buffer, mesh_center, mesh_half_extents);
    
    println!("  Drew {} rendering gizmos", command_buffer.commands.len());
    command_buffer.clear();
    
    // ── Transforms Mode Demo ────────────────────────────────────────────
    
    println!("\n=== Transforms Mode Demo ===");
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    // Configure transform settings
    let transform_settings = gizmo_system.transform_settings_mut();
    transform_settings.axes_length = 1.5;
    
    // Draw coordinate axes
    let entity_position = Vec3::new(-2.0, 1.0, 0.0);
    gizmo_system.draw_transform_axes(&mut command_buffer, entity_position, 1.0);
    
    println!("  Drew {} transform gizmos", command_buffer.commands.len());
    command_buffer.clear();
    
    // ── Audio Mode Demo ─────────────────────────────────────────────────
    
    println!("\n=== Audio Mode Demo ===");
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // Configure audio settings
    let audio_settings = gizmo_system.audio_settings_mut();
    audio_settings.show_attenuation = true;
    audio_settings.source_color = Color::rgb(0.0, 1.0, 1.0);
    
    // Draw audio source
    let audio_position = Vec3::new(0.0, 2.0, 0.0);
    let attenuation_radius = 5.0;
    gizmo_system.draw_audio_source(&mut command_buffer, audio_position, attenuation_radius);
    
    println!("  Drew {} audio gizmos", command_buffer.commands.len());
    command_buffer.clear();
    
    // ── Overlay Demo ────────────────────────────────────────────────────
    
    println!("\n=== Overlay Demo ===");
    
    // Draw text overlay
    gizmo_system.draw_text_overlay(
        &mut overlay,
        10.0,
        10.0,
        "Gizmo System Active",
        [1.0, 1.0, 1.0, 1.0],
    );
    
    // Draw status overlay
    gizmo_system.draw_status_overlay(&mut overlay, 10.0, 30.0);
    
    println!("  Drew {} overlay commands", overlay.commands.len());
    overlay.clear();
    
    // ── Mode Control Demo ───────────────────────────────────────────────
    
    println!("\n=== Mode Control Demo ===");
    println!("  Active modes: {:?}", gizmo_system.active_modes());
    
    // Toggle modes
    gizmo_system.toggle_mode(VisualizationMode::Physics);
    println!("  After toggling Physics: {:?}", gizmo_system.active_modes());
    
    gizmo_system.toggle_mode(VisualizationMode::Rendering);
    println!("  After toggling Rendering: {:?}", gizmo_system.active_modes());
    
    // Disable all
    gizmo_system.set_enabled(false);
    println!("  After disabling system: enabled={}", gizmo_system.is_enabled());
    
    // Try to draw (should not add commands)
    gizmo_system.draw_physics(&mut command_buffer, Vec3::ZERO, Vec3::ONE);
    println!("  Commands after draw with disabled system: {}", command_buffer.commands.len());
    
    // Re-enable
    gizmo_system.set_enabled(true);
    println!("  After re-enabling system: enabled={}", gizmo_system.is_enabled());
    
    // ── Custom Mode Demo ────────────────────────────────────────────────
    
    println!("\n=== Custom Mode Demo ===");
    let navigation_mode = VisualizationMode::Custom("navigation");
    gizmo_system.enable_mode(navigation_mode);
    
    println!("  Custom mode '{}' active: {}", 
        navigation_mode.category(),
        gizmo_system.is_mode_active(navigation_mode)
    );
    
    // ── Settings Demo ───────────────────────────────────────────────────
    
    println!("\n=== Settings Demo ===");
    
    // Physics settings
    let physics = gizmo_system.physics_settings();
    println!("  Physics settings:");
    println!("    show_colliders: {}", physics.show_colliders);
    println!("    show_velocities: {}", physics.show_velocities);
    println!("    velocity_scale: {}", physics.velocity_scale);
    
    // Rendering settings
    let rendering = gizmo_system.rendering_settings();
    println!("  Rendering settings:");
    println!("    show_wireframe: {}", rendering.show_wireframe);
    println!("    show_bounds: {}", rendering.show_bounds);
    
    // Transform settings
    let transform = gizmo_system.transform_settings();
    println!("  Transform settings:");
    println!("    show_axes: {}", transform.show_axes);
    println!("    axes_length: {}", transform.axes_length);
    
    // Audio settings
    let audio = gizmo_system.audio_settings();
    println!("  Audio settings:");
    println!("    show_sources: {}", audio.show_sources);
    println!("    show_attenuation: {}", audio.show_attenuation);
    
    println!("\n=== Demo Complete ===");
    println!("\nIn a real application, you would:");
    println!("  1. Add GizmoSystem as a resource to your ECS world");
    println!("  2. Create systems that draw gizmos based on component data");
    println!("  3. Bind keyboard shortcuts to toggle modes");
    println!("  4. Use the overlay to show debug information");
}
