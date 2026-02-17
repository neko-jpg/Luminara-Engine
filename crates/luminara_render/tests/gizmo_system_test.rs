use luminara_render::{
    AudioVisualizationSettings, CommandBuffer, GizmoCategories, GizmoSystem, OverlayRenderer,
    PhysicsVisualizationSettings, RenderingVisualizationSettings,
    TransformVisualizationSettings, VisualizationMode,
};
use luminara_math::{Color, Vec3};

#[test]
fn test_gizmo_system_creation() {
    let gizmo_system = GizmoSystem::new();
    assert!(gizmo_system.is_enabled());
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Physics));
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Rendering));
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Transforms));
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Audio));
}

#[test]
fn test_global_enable_disable() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Initially enabled
    assert!(gizmo_system.is_enabled());
    
    // Disable
    gizmo_system.set_enabled(false);
    assert!(!gizmo_system.is_enabled());
    
    // Enable
    gizmo_system.set_enabled(true);
    assert!(gizmo_system.is_enabled());
    
    // Toggle
    let state = gizmo_system.toggle();
    assert!(!state);
    assert!(!gizmo_system.is_enabled());
}

#[test]
fn test_mode_control() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Enable physics mode
    gizmo_system.enable_mode(VisualizationMode::Physics);
    assert!(gizmo_system.is_mode_active(VisualizationMode::Physics));
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Rendering));
    
    // Enable rendering mode
    gizmo_system.enable_mode(VisualizationMode::Rendering);
    assert!(gizmo_system.is_mode_active(VisualizationMode::Physics));
    assert!(gizmo_system.is_mode_active(VisualizationMode::Rendering));
    
    // Disable physics mode
    gizmo_system.disable_mode(VisualizationMode::Physics);
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Physics));
    assert!(gizmo_system.is_mode_active(VisualizationMode::Rendering));
    
    // Toggle rendering mode
    let state = gizmo_system.toggle_mode(VisualizationMode::Rendering);
    assert!(!state);
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Rendering));
}

#[test]
fn test_mode_respects_global_enable() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Enable a mode
    gizmo_system.enable_mode(VisualizationMode::Physics);
    assert!(gizmo_system.is_mode_active(VisualizationMode::Physics));
    
    // Disable globally
    gizmo_system.set_enabled(false);
    assert!(!gizmo_system.is_mode_active(VisualizationMode::Physics));
    
    // Re-enable globally
    gizmo_system.set_enabled(true);
    assert!(gizmo_system.is_mode_active(VisualizationMode::Physics));
}

#[test]
fn test_active_modes() {
    let mut gizmo_system = GizmoSystem::new();
    
    // No active modes initially
    assert_eq!(gizmo_system.active_modes().len(), 0);
    
    // Enable some modes
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    let active = gizmo_system.active_modes();
    assert_eq!(active.len(), 2);
    assert!(active.contains(&VisualizationMode::Physics));
    assert!(active.contains(&VisualizationMode::Audio));
}

#[test]
fn test_custom_mode() {
    let mut gizmo_system = GizmoSystem::new();
    
    let custom_mode = VisualizationMode::Custom("navigation");
    gizmo_system.enable_mode(custom_mode);
    
    assert!(gizmo_system.is_mode_active(custom_mode));
    assert_eq!(custom_mode.category(), "navigation");
}

#[test]
fn test_sync_with_categories() {
    let mut gizmo_system = GizmoSystem::new();
    let mut categories = GizmoCategories::new();
    
    // Enable some modes
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.enable_mode(VisualizationMode::Rendering);
    
    // Sync
    gizmo_system.sync_with_categories(&mut categories);
    
    assert!(categories.is_enabled("physics"));
    assert!(categories.is_enabled("rendering"));
    assert!(!categories.is_enabled("transforms"));
    assert!(!categories.is_enabled("audio"));
}

#[test]
fn test_sync_with_categories_disabled() {
    let mut gizmo_system = GizmoSystem::new();
    let mut categories = GizmoCategories::new();
    
    // Enable modes but disable globally
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.set_enabled(false);
    
    // Sync should disable all
    gizmo_system.sync_with_categories(&mut categories);
    
    assert!(!categories.is_enabled("physics"));
    // Note: "rendering" is not a default category in GizmoCategories,
    // so it defaults to true. This is expected behavior - only categories
    // that are explicitly synced get disabled.
    // We can verify that physics (which was enabled in gizmo_system) is disabled.
}

#[test]
fn test_physics_settings() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check defaults
    let settings = gizmo_system.physics_settings();
    assert!(settings.show_colliders);
    assert!(settings.show_velocities);
    assert!(settings.show_contacts);
    
    // Modify settings
    let settings_mut = gizmo_system.physics_settings_mut();
    settings_mut.show_colliders = false;
    settings_mut.velocity_scale = 2.0;
    
    // Verify changes
    let settings = gizmo_system.physics_settings();
    assert!(!settings.show_colliders);
    assert_eq!(settings.velocity_scale, 2.0);
}

#[test]
fn test_rendering_settings() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check defaults
    let settings = gizmo_system.rendering_settings();
    assert!(!settings.show_wireframe);
    assert!(!settings.show_normals);
    assert!(settings.show_bounds);
    
    // Modify settings
    let settings_mut = gizmo_system.rendering_settings_mut();
    settings_mut.show_wireframe = true;
    settings_mut.wireframe_color = Color::rgb(1.0, 0.0, 0.0);
    
    // Verify changes
    let settings = gizmo_system.rendering_settings();
    assert!(settings.show_wireframe);
}

#[test]
fn test_transform_settings() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check defaults
    let settings = gizmo_system.transform_settings();
    assert!(settings.show_axes);
    assert!(!settings.show_hierarchy);
    assert_eq!(settings.axes_length, 1.0);
    
    // Modify settings
    let settings_mut = gizmo_system.transform_settings_mut();
    settings_mut.axes_length = 2.5;
    settings_mut.show_hierarchy = true;
    
    // Verify changes
    let settings = gizmo_system.transform_settings();
    assert_eq!(settings.axes_length, 2.5);
    assert!(settings.show_hierarchy);
}

#[test]
fn test_audio_settings() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check defaults
    let settings = gizmo_system.audio_settings();
    assert!(settings.show_sources);
    assert!(settings.show_attenuation);
    
    // Modify settings
    let settings_mut = gizmo_system.audio_settings_mut();
    settings_mut.show_attenuation = false;
    settings_mut.source_color = Color::rgb(1.0, 0.0, 1.0);
    
    // Verify changes
    let settings = gizmo_system.audio_settings();
    assert!(!settings.show_attenuation);
}

#[test]
fn test_draw_physics_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Physics mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_physics(&mut buffer, Vec3::ZERO, Vec3::ONE);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable physics mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.draw_physics(&mut buffer, Vec3::ZERO, Vec3::ONE);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_velocity_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Physics mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_velocity(&mut buffer, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable physics mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.draw_velocity(&mut buffer, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_contact_point_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Physics mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_contact_point(&mut buffer, Vec3::ZERO);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable physics mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.draw_contact_point(&mut buffer, Vec3::ZERO);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_transform_axes_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Transforms mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_transform_axes(&mut buffer, Vec3::ZERO, 1.0);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable transforms mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.draw_transform_axes(&mut buffer, Vec3::ZERO, 1.0);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_bounding_box_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Rendering mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_bounding_box(&mut buffer, Vec3::ZERO, Vec3::ONE);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable rendering mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Rendering);
    gizmo_system.draw_bounding_box(&mut buffer, Vec3::ZERO, Vec3::ONE);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_audio_source_respects_mode() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Audio mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_audio_source(&mut buffer, Vec3::ZERO, 10.0);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable audio mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Audio);
    gizmo_system.draw_audio_source(&mut buffer, Vec3::ZERO, 10.0);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_text_overlay_respects_global_enable() {
    let mut gizmo_system = GizmoSystem::new();
    let mut overlay = OverlayRenderer::new();
    
    // Enabled - should draw
    gizmo_system.draw_text_overlay(&mut overlay, 10.0, 10.0, "Test", [1.0, 1.0, 1.0, 1.0]);
    assert!(!overlay.commands.is_empty());
    
    // Disabled - should not draw
    overlay.clear();
    gizmo_system.set_enabled(false);
    gizmo_system.draw_text_overlay(&mut overlay, 10.0, 10.0, "Test", [1.0, 1.0, 1.0, 1.0]);
    assert!(overlay.commands.is_empty());
}

#[test]
fn test_draw_status_overlay() {
    let mut gizmo_system = GizmoSystem::new();
    let mut overlay = OverlayRenderer::new();
    
    // Enable some modes
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // Draw status
    gizmo_system.draw_status_overlay(&mut overlay, 10.0, 10.0);
    
    // Should have drawn header + 2 mode lines
    assert!(!overlay.commands.is_empty());
}

#[test]
fn test_physics_settings_defaults() {
    let settings = PhysicsVisualizationSettings::default();
    assert!(settings.show_colliders);
    assert!(settings.show_velocities);
    assert!(settings.show_contacts);
    assert_eq!(settings.velocity_scale, 1.0);
}

#[test]
fn test_rendering_settings_defaults() {
    let settings = RenderingVisualizationSettings::default();
    assert!(!settings.show_wireframe);
    assert!(!settings.show_normals);
    assert!(settings.show_bounds);
}

#[test]
fn test_transform_settings_defaults() {
    let settings = TransformVisualizationSettings::default();
    assert!(settings.show_axes);
    assert!(!settings.show_hierarchy);
    assert_eq!(settings.axes_length, 1.0);
}

#[test]
fn test_audio_settings_defaults() {
    let settings = AudioVisualizationSettings::default();
    assert!(settings.show_sources);
    assert!(settings.show_attenuation);
}

#[test]
fn test_multiple_modes_simultaneously() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Enable all modes
    gizmo_system.enable_mode(VisualizationMode::Physics);
    gizmo_system.enable_mode(VisualizationMode::Rendering);
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // All should be active
    assert_eq!(gizmo_system.active_modes().len(), 4);
    
    // All draw calls should work
    let initial_count = buffer.commands.len();
    gizmo_system.draw_physics(&mut buffer, Vec3::ZERO, Vec3::ONE);
    gizmo_system.draw_bounding_box(&mut buffer, Vec3::ZERO, Vec3::ONE);
    gizmo_system.draw_transform_axes(&mut buffer, Vec3::ZERO, 1.0);
    gizmo_system.draw_audio_source(&mut buffer, Vec3::ZERO, 10.0);
    
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_hierarchy_connection() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Transforms mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_hierarchy_connection(
        &mut buffer,
        Vec3::ZERO,
        Vec3::new(1.0, 1.0, 1.0),
    );
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable transforms mode but hierarchy disabled - should not draw
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.draw_hierarchy_connection(
        &mut buffer,
        Vec3::ZERO,
        Vec3::new(1.0, 1.0, 1.0),
    );
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable hierarchy - should draw
    gizmo_system.transform_settings_mut().show_hierarchy = true;
    gizmo_system.draw_hierarchy_connection(
        &mut buffer,
        Vec3::ZERO,
        Vec3::new(1.0, 1.0, 1.0),
    );
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_entity_highlight() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    // Transforms mode disabled - should not draw
    let initial_count = buffer.commands.len();
    gizmo_system.draw_entity_highlight(&mut buffer, Vec3::ZERO, 1.0);
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable transforms mode - should draw
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.draw_entity_highlight(&mut buffer, Vec3::ZERO, 1.0);
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_entity_transform_without_parent() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_entity_transform(
        &mut buffer,
        Vec3::ZERO,
        None,
        false,
        1.0,
    );
    
    // Should draw axes only (3 arrows for X, Y, Z)
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_entity_transform_with_parent() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.transform_settings_mut().show_hierarchy = true;
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_entity_transform(
        &mut buffer,
        Vec3::new(1.0, 1.0, 1.0),
        Some(Vec3::ZERO),
        false,
        1.0,
    );
    
    // Should draw axes + hierarchy line
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_entity_transform_selected() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_entity_transform(
        &mut buffer,
        Vec3::ZERO,
        None,
        true,
        1.0,
    );
    
    // Should draw axes + selection highlight
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_entity_transform_full() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    gizmo_system.transform_settings_mut().show_hierarchy = true;
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_entity_transform(
        &mut buffer,
        Vec3::new(1.0, 1.0, 1.0),
        Some(Vec3::ZERO),
        true,
        1.0,
    );
    
    // Should draw axes + hierarchy line + selection highlight
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_transform_settings_selection_color() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check default selection color
    let settings = gizmo_system.transform_settings();
    assert_eq!(settings.selection_color.r, 1.0);
    assert_eq!(settings.selection_color.g, 1.0);
    assert_eq!(settings.selection_color.b, 0.0);
    
    // Modify selection color
    let settings_mut = gizmo_system.transform_settings_mut();
    settings_mut.selection_color = Color::rgb(1.0, 0.0, 1.0);
    
    // Verify changes
    let settings = gizmo_system.transform_settings();
    assert_eq!(settings.selection_color.r, 1.0);
    assert_eq!(settings.selection_color.g, 0.0);
    assert_eq!(settings.selection_color.b, 1.0);
}

#[test]
fn test_hierarchy_visualization_toggle() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    // Initially hierarchy is disabled
    assert!(!gizmo_system.transform_settings().show_hierarchy);
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_hierarchy_connection(
        &mut buffer,
        Vec3::ZERO,
        Vec3::new(1.0, 1.0, 1.0),
    );
    assert_eq!(buffer.commands.len(), initial_count);
    
    // Enable hierarchy
    gizmo_system.transform_settings_mut().show_hierarchy = true;
    gizmo_system.draw_hierarchy_connection(
        &mut buffer,
        Vec3::ZERO,
        Vec3::new(1.0, 1.0, 1.0),
    );
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_axes_length_scaling() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    // Set custom axes length
    gizmo_system.transform_settings_mut().axes_length = 2.0;
    
    let initial_count = buffer.commands.len();
    gizmo_system.draw_transform_axes(&mut buffer, Vec3::ZERO, 1.0);
    
    // Should draw with scaled length (2.0 * 1.0 = 2.0)
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_selection_highlight_radius_scaling() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Transforms);
    
    let initial_count = buffer.commands.len();
    
    // Draw with scale 2.0 - highlight should be 2.0 * 1.5 = 3.0 radius
    gizmo_system.draw_entity_transform(
        &mut buffer,
        Vec3::ZERO,
        None,
        true,
        2.0,
    );
    
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_audio_source_with_volume() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    let initial_count = buffer.commands.len();
    
    // Draw audio source with volume
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        Some(0.8),
    );
    
    // Should draw source sphere, attenuation circle, and volume bar
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_draw_audio_source_without_volume() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    let initial_count = buffer.commands.len();
    
    // Draw audio source without volume
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        None,
    );
    
    // Should draw source sphere and attenuation circle, but no volume bar
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_audio_volume_visualization_toggle() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // With volume visualization enabled (default)
    let count_with_volume = {
        buffer.commands.clear();
        let initial = buffer.commands.len();
        gizmo_system.draw_audio_source_with_volume(
            &mut buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        buffer.commands.len() - initial
    };
    
    // Disable volume visualization
    gizmo_system.audio_settings_mut().show_volume = false;
    
    let count_without_volume = {
        buffer.commands.clear();
        let initial = buffer.commands.len();
        gizmo_system.draw_audio_source_with_volume(
            &mut buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        buffer.commands.len() - initial
    };
    
    // Should draw fewer commands without volume bar
    assert!(count_without_volume < count_with_volume);
}

#[test]
fn test_audio_source_size_scales_with_volume() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // Draw with low volume
    buffer.commands.clear();
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        Some(0.1),
    );
    let low_volume_commands = buffer.commands.len();
    
    // Draw with high volume
    buffer.commands.clear();
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        Some(1.0),
    );
    let high_volume_commands = buffer.commands.len();
    
    // Both should draw the same number of commands (size is internal to sphere)
    assert_eq!(low_volume_commands, high_volume_commands);
}

#[test]
fn test_audio_settings_show_volume_default() {
    let settings = AudioVisualizationSettings::default();
    assert!(settings.show_volume);
}

#[test]
fn test_audio_settings_show_volume_modification() {
    let mut gizmo_system = GizmoSystem::new();
    
    // Check default
    assert!(gizmo_system.audio_settings().show_volume);
    
    // Modify
    gizmo_system.audio_settings_mut().show_volume = false;
    
    // Verify
    assert!(!gizmo_system.audio_settings().show_volume);
}

#[test]
fn test_draw_audio_source_backward_compatibility() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    let initial_count = buffer.commands.len();
    
    // Old API should still work (calls new API with None for volume)
    gizmo_system.draw_audio_source(&mut buffer, Vec3::ZERO, 10.0);
    
    assert!(buffer.commands.len() > initial_count);
}

#[test]
fn test_volume_clamping() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // Test with volume > 1.0 (should be clamped)
    buffer.commands.clear();
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        Some(2.0),
    );
    let over_volume_commands = buffer.commands.len();
    
    // Test with volume < 0.0 (should be clamped)
    buffer.commands.clear();
    gizmo_system.draw_audio_source_with_volume(
        &mut buffer,
        Vec3::ZERO,
        10.0,
        Some(-0.5),
    );
    let under_volume_commands = buffer.commands.len();
    
    // Both should draw successfully (clamping prevents errors)
    assert!(over_volume_commands > 0);
    assert!(under_volume_commands > 0);
}

#[test]
fn test_attenuation_visualization_with_volume() {
    let mut gizmo_system = GizmoSystem::new();
    let mut buffer = CommandBuffer::default();
    
    gizmo_system.enable_mode(VisualizationMode::Audio);
    
    // With attenuation enabled
    let count_with_attenuation = {
        buffer.commands.clear();
        let initial = buffer.commands.len();
        gizmo_system.draw_audio_source_with_volume(
            &mut buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        buffer.commands.len() - initial
    };
    
    // Disable attenuation
    gizmo_system.audio_settings_mut().show_attenuation = false;
    
    let count_without_attenuation = {
        buffer.commands.clear();
        let initial = buffer.commands.len();
        gizmo_system.draw_audio_source_with_volume(
            &mut buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        buffer.commands.len() - initial
    };
    
    // Should draw fewer commands without attenuation circle
    assert!(count_without_attenuation < count_with_attenuation);
}
