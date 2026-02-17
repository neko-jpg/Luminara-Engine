//! Audio debug visualization systems
//!
//! Systems that integrate AudioSource components with the GizmoSystem
//! to provide visual debugging of audio sources, attenuation ranges, and volume levels.

use crate::{CommandBuffer, GizmoSystem};
use luminara_core::shared_types::{Query, Res, ResMut};
use luminara_math::Transform;

/// Visualize all audio sources in the scene
///
/// This system queries all entities with AudioSource and Transform components
/// and draws debug visualization for each one using the GizmoSystem.
pub fn visualize_audio_sources_system(
    audio_sources: Query<(&luminara_audio::AudioSource, &Transform)>,
    gizmo_system: Res<GizmoSystem>,
    mut command_buffer: ResMut<CommandBuffer>,
) {
    if !gizmo_system.is_mode_active(crate::VisualizationMode::Audio) {
        return;
    }

    for (audio_source, transform) in audio_sources.iter() {
        let position = transform.translation;
        let attenuation_radius = if audio_source.spatial {
            audio_source.max_distance
        } else {
            0.0 // Non-spatial audio sources don't have attenuation
        };

        // Draw audio source with volume visualization
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            position,
            attenuation_radius,
            Some(audio_source.volume),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luminara_audio::{AudioClipHandle, AudioSource};
    use luminara_math::Vec3;

    #[test]
    fn test_audio_source_visualization_with_spatial() {
        let gizmo_system = GizmoSystem::new();
        let mut command_buffer = CommandBuffer::default();

        // Create spatial audio source
        let audio_source = AudioSource {
            clip: AudioClipHandle("spatial.ogg".to_string()),
            volume: 0.8,
            spatial: true,
            max_distance: 50.0,
            ..Default::default()
        };

        let transform = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        };

        // Audio mode disabled - should not draw
        let initial_count = command_buffer.commands.len();
        let position = transform.translation;
        let attenuation_radius = if audio_source.spatial {
            audio_source.max_distance
        } else {
            0.0
        };
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            position,
            attenuation_radius,
            Some(audio_source.volume),
        );
        assert_eq!(command_buffer.commands.len(), initial_count);
    }

    #[test]
    fn test_audio_source_visualization_enabled() {
        let mut gizmo_system = GizmoSystem::new();
        let mut command_buffer = CommandBuffer::default();

        gizmo_system.enable_mode(crate::VisualizationMode::Audio);

        // Create spatial audio source
        let audio_source = AudioSource {
            clip: AudioClipHandle("spatial.ogg".to_string()),
            volume: 1.0,
            spatial: true,
            max_distance: 100.0,
            ..Default::default()
        };

        let transform = Transform {
            translation: Vec3::new(5.0, 0.0, 5.0),
            ..Default::default()
        };

        let initial_count = command_buffer.commands.len();
        let position = transform.translation;
        let attenuation_radius = if audio_source.spatial {
            audio_source.max_distance
        } else {
            0.0
        };
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            position,
            attenuation_radius,
            Some(audio_source.volume),
        );

        // Should draw source sphere, attenuation circle, and volume bar
        assert!(command_buffer.commands.len() > initial_count);
    }

    #[test]
    fn test_non_spatial_audio_source() {
        let mut gizmo_system = GizmoSystem::new();
        let mut command_buffer = CommandBuffer::default();

        gizmo_system.enable_mode(crate::VisualizationMode::Audio);

        // Create non-spatial audio source
        let audio_source = AudioSource {
            clip: AudioClipHandle("music.ogg".to_string()),
            volume: 0.5,
            spatial: false,
            max_distance: 100.0,
            ..Default::default()
        };

        let transform = Transform {
            translation: Vec3::ZERO,
            ..Default::default()
        };

        let initial_count = command_buffer.commands.len();
        let position = transform.translation;
        let attenuation_radius = if audio_source.spatial {
            audio_source.max_distance
        } else {
            0.0
        };
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            position,
            attenuation_radius,
            Some(audio_source.volume),
        );

        // Should draw source sphere and volume bar, but no attenuation (radius = 0)
        assert!(command_buffer.commands.len() > initial_count);
    }

    #[test]
    fn test_volume_levels() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_mode(crate::VisualizationMode::Audio);

        // Test low volume
        let mut command_buffer = CommandBuffer::default();
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            Vec3::ZERO,
            10.0,
            Some(0.1),
        );
        let low_volume_count = command_buffer.commands.len();

        // Test high volume
        command_buffer.commands.clear();
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            Vec3::ZERO,
            10.0,
            Some(1.0),
        );
        let high_volume_count = command_buffer.commands.len();

        // Both should draw the same number of commands
        assert_eq!(low_volume_count, high_volume_count);
        assert!(low_volume_count > 0);
    }

    #[test]
    fn test_volume_visualization_toggle() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_mode(crate::VisualizationMode::Audio);

        // With volume visualization enabled
        let mut command_buffer = CommandBuffer::default();
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        let with_volume = command_buffer.commands.len();

        // Disable volume visualization
        gizmo_system.audio_settings_mut().show_volume = false;
        command_buffer.commands.clear();
        gizmo_system.draw_audio_source_with_volume(
            &mut command_buffer,
            Vec3::ZERO,
            10.0,
            Some(0.5),
        );
        let without_volume = command_buffer.commands.len();

        // Should draw fewer commands without volume bar
        assert!(without_volume < with_volume);
    }
}
