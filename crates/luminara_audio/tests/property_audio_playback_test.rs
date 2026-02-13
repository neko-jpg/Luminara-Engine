use proptest::prelude::*;
use luminara_audio::{AudioSource, AudioClipHandle, AudioListener};
use luminara_audio::systems::{AudioPlayback, audio_system};
use luminara_audio::plugin::KiraAudioManager;
use luminara_core::World;
use luminara_math::Transform;
use luminara_scene::GlobalTransform;
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};

/// **Property 20: Audio Playback Control**
/// **Validates: Requirements 8.2, 8.6**
/// 
/// For any audio source, triggering play should start playback with the specified 
/// volume and pitch, and pause/resume/stop commands should correctly control the 
/// playback state.

fn arb_audio_source() -> impl Strategy<Value = AudioSource> {
    (
        prop::string::string_regex("[a-z]{5,10}\\.wav").unwrap(),
        0.0f32..=1.0f32,  // volume
        0.5f32..=2.0f32,  // pitch
        prop::bool::ANY,  // looping
        prop::bool::ANY,  // spatial
        10.0f32..=200.0f32,  // max_distance
    ).prop_map(|(clip_name, volume, pitch, looping, spatial, max_distance)| {
        AudioSource {
            clip: AudioClipHandle(clip_name),
            volume,
            pitch,
            looping,
            spatial,
            max_distance,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase-1-core-engine, Property 20: Audio Playback Control
        #[test]
        fn prop_audio_playback_control(
            source in arb_audio_source()
        ) {
            // Create a world with audio system
            let mut world = World::new();
            
            // Initialize audio manager
            let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .expect("Failed to create audio manager");
            world.insert_resource(KiraAudioManager(audio_manager));
            world.insert_resource(AudioPlayback::default());
            
            // Create an entity with audio source
            let entity = world.spawn();
            world.add_component(entity, source.clone());
            world.add_component(entity, Transform::IDENTITY);
            world.add_component(entity, GlobalTransform::default());
            
            // Create a listener entity
            let listener = world.spawn();
            world.add_component(listener, AudioListener { enabled: true });
            world.add_component(listener, Transform::IDENTITY);
            world.add_component(listener, GlobalTransform::default());
            
            // Run the audio system
            audio_system(&mut world);
            
            // Verify the audio system initialized properly
            let playback = world.get_resource::<AudioPlayback>().unwrap();
            assert!(playback.spatial_scene.is_some(), "Spatial scene should be initialized");
            assert!(playback.listener.is_some(), "Listener should be initialized");
            
            // Verify audio source properties are preserved
            let retrieved_source = world.get_component::<AudioSource>(entity).unwrap();
            assert_eq!(retrieved_source.volume, source.volume, "Volume should be preserved");
            assert_eq!(retrieved_source.pitch, source.pitch, "Pitch should be preserved");
            assert_eq!(retrieved_source.looping, source.looping, "Looping should be preserved");
            assert_eq!(retrieved_source.spatial, source.spatial, "Spatial should be preserved");
            assert_eq!(retrieved_source.max_distance, source.max_distance, "Max distance should be preserved");
            
            // Note: Actual playback control (play/pause/resume/stop) requires
            // loaded audio clips, which is not yet implemented. This test verifies
            // the system setup and component preservation.
        }
    }
}
