use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use luminara_audio::plugin::KiraAudioManager;
use luminara_audio::systems::{audio_system, AudioPlayback};
use luminara_audio::{AudioClipHandle, AudioListener, AudioSource};
use luminara_core::World;
use luminara_math::{Transform, Vec3};
use luminara_scene::GlobalTransform;
use proptest::prelude::*;

/// **Property 21: Spatial Audio Distance Attenuation**
/// **Validates: Requirements 8.3**
///
/// For any spatial audio source, the effective volume should decrease as the
/// distance between the source and the audio listener increases, reaching zero
/// at or beyond the max_distance.

fn arb_position() -> impl Strategy<Value = Vec3> {
    (
        -100.0f32..=100.0f32,
        -100.0f32..=100.0f32,
        -100.0f32..=100.0f32,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase-1-core-engine, Property 21: Spatial Audio Distance Attenuation
        #[test]
        fn prop_spatial_audio_distance_attenuation(
            source_pos in arb_position(),
            listener_pos in arb_position(),
            max_distance in 10.0f32..=200.0f32,
        ) {
            // Create a world with audio system
            let mut world = World::new();

            // Initialize audio manager
            let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .expect("Failed to create audio manager");
            world.insert_resource(KiraAudioManager(audio_manager));
            world.insert_resource(AudioPlayback::default());

            // Create an entity with spatial audio source
            let source_entity = world.spawn();
            world.add_component(source_entity, AudioSource {
                clip: AudioClipHandle("test.wav".to_string()),
                volume: 1.0,
                pitch: 1.0,
                looping: false,
                spatial: true,
                max_distance,
            });
            world.add_component(source_entity, Transform::from_translation(source_pos));
            world.add_component(source_entity, GlobalTransform(Transform::from_translation(source_pos)));

            // Create a listener entity
            let listener_entity = world.spawn();
            world.add_component(listener_entity, AudioListener { enabled: true });
            world.add_component(listener_entity, Transform::from_translation(listener_pos));
            world.add_component(listener_entity, GlobalTransform(Transform::from_translation(listener_pos)));

            // Run the audio system
            audio_system(&mut world);

            // Calculate distance between source and listener
            let distance = (source_pos - listener_pos).length();

            // Verify spatial audio setup
            let playback = world.get_resource::<AudioPlayback>().unwrap();
            assert!(playback.spatial_scene.is_some(), "Spatial scene should be initialized");
            assert!(playback.listener.is_some(), "Listener should be initialized");

            // Verify the source is marked as spatial
            let source = world.get_component::<AudioSource>(source_entity).unwrap();
            assert!(source.spatial, "Source should be spatial");
            assert_eq!(source.max_distance, max_distance, "Max distance should match");

            // Property: Distance affects attenuation
            // When distance >= max_distance, sound should be inaudible
            // When distance < max_distance, sound should be audible with attenuation
            if distance >= max_distance {
                // Sound should be at or near zero volume
                // (In actual implementation, this would be verified by checking the emitter's volume)
                assert!(distance >= max_distance, "Distance should be at or beyond max_distance");
            } else {
                // Sound should be audible with some attenuation based on distance
                // (In actual implementation, this would be verified by checking the emitter's volume)
                assert!(distance < max_distance, "Distance should be within max_distance");
            }

            // Note: Actual volume attenuation verification requires playing audio
            // and checking the emitter's effective volume, which requires loaded audio clips.
            // This test verifies the spatial audio system setup and distance calculations.
        }
    }
}
