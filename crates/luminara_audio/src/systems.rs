use crate::plugin::KiraAudioManager;
use crate::{AudioListener, AudioSource};
use kira::sound::static_sound::StaticSoundHandle;
use kira::spatial::emitter::EmitterHandle;
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::Tween;
use log::{error, info};
use luminara_core::{Entity, World};
use luminara_math::{Quat, Vec3};
use luminara_scene::GlobalTransform;
use std::collections::HashMap;

/// Resource to track active audio playback
pub struct AudioPlayback {
    /// Map from entity to sound handle
    pub sounds: HashMap<Entity, StaticSoundHandle>,
    /// Map from entity to spatial emitter handle
    pub emitters: HashMap<Entity, EmitterHandle>,
    /// Spatial scene for 3D audio
    pub spatial_scene: Option<SpatialSceneHandle>,
    /// Listener handle for spatial audio
    pub listener: Option<ListenerHandle>,
}

impl Default for AudioPlayback {
    fn default() -> Self {
        Self {
            sounds: HashMap::new(),
            emitters: HashMap::new(),
            spatial_scene: None,
            listener: None,
        }
    }
}

impl luminara_core::Resource for AudioPlayback {}

/// Main audio system that processes audio commands and updates spatial audio
pub fn audio_system(world: &mut World) {
    // Initialize AudioPlayback resource if it doesn't exist
    if world.get_resource::<AudioPlayback>().is_none() {
        world.insert_resource(AudioPlayback::default());
    }

    // Get audio manager
    let audio_manager = world.get_resource::<KiraAudioManager>();
    if audio_manager.is_none() {
        return;
    }

    // Initialize spatial scene if needed
    {
        let playback = world.get_resource_mut::<AudioPlayback>().unwrap();
        let audio_manager = world.get_resource_mut::<KiraAudioManager>().unwrap();

        if playback.spatial_scene.is_none() {
            match audio_manager
                .0
                .add_spatial_scene(SpatialSceneSettings::default())
            {
                Ok(scene) => {
                    playback.spatial_scene = Some(scene);
                    info!("Initialized spatial audio scene");
                }
                Err(e) => {
                    error!("Failed to create spatial scene: {}", e);
                }
            }
        }

        // Initialize listener if needed and there's an AudioListener component
        if playback.listener.is_none() {
            if let Some(scene) = &mut playback.spatial_scene {
                // Convert Vec3 and Quat to mint types for kira
                let pos: [f32; 3] = Vec3::ZERO.into();
                let rot: [f32; 4] = Quat::IDENTITY.into();
                match scene.add_listener(pos, rot, ListenerSettings::default()) {
                    Ok(listener) => {
                        playback.listener = Some(listener);
                        info!("Initialized audio listener");
                    }
                    Err(e) => {
                        error!("Failed to create listener: {}", e);
                    }
                }
            }
        }
    }

    // Update listener position based on AudioListener component
    update_listener_position(world);

    // Update spatial audio for all sources
    update_spatial_audio(world);
}

fn update_listener_position(world: &mut World) {
    // Find the active listener entity
    let listener_query: Vec<(Entity, GlobalTransform)> = {
        let entities = world.entities();
        entities
            .into_iter()
            .filter_map(|e| {
                if let Some(listener) = world.get_component::<AudioListener>(e) {
                    if listener.enabled {
                        if let Some(transform) = world.get_component::<GlobalTransform>(e) {
                            return Some((e, transform.clone()));
                        }
                    }
                }
                None
            })
            .collect()
    };

    if let Some((_, transform)) = listener_query.first() {
        let playback = world.get_resource_mut::<AudioPlayback>().unwrap();

        if let Some(listener) = &mut playback.listener {
            let pos: [f32; 3] = transform.0.translation.into();
            let rot: [f32; 4] = transform.0.rotation.into();

            // Update listener position and orientation
            // Note: In kira 0.9, these methods don't return Result
            listener.set_position(pos, Tween::default());
            listener.set_orientation(rot, Tween::default());
        }
    }
}

fn update_spatial_audio(world: &mut World) {
    // Collect all audio sources with transforms
    let sources: Vec<(Entity, AudioSource, GlobalTransform)> = {
        let entities = world.entities();
        entities
            .into_iter()
            .filter_map(|e| {
                if let Some(source) = world.get_component::<AudioSource>(e) {
                    if let Some(transform) = world.get_component::<GlobalTransform>(e) {
                        return Some((e, source.clone(), transform.clone()));
                    }
                }
                None
            })
            .collect()
    };

    let playback = world.get_resource_mut::<AudioPlayback>().unwrap();

    for (entity, source, transform) in sources {
        if source.spatial {
            // Update emitter position if it exists
            if let Some(emitter) = playback.emitters.get_mut(&entity) {
                let pos: [f32; 3] = transform.0.translation.into();
                emitter.set_position(pos, Tween::default());
            }
        }
    }
}

/// Helper function to play an audio source
pub fn play_audio(world: &mut World, entity: Entity) {
    let source = match world.get_component::<AudioSource>(entity) {
        Some(s) => s.clone(),
        None => {
            error!("Entity {:?} has no AudioSource component", entity);
            return;
        }
    };

    // For now, we'll just log that we would play the audio
    // In a real implementation, we would load the audio clip from the asset server
    // and play it through the audio manager
    info!("Playing audio for entity {:?}: {:?}", entity, source.clip);

    // TODO: Implement actual audio playback once asset loading is integrated
    // This would involve:
    // 1. Getting the AudioClip from the asset server using source.clip
    // 2. Creating a sound handle from the clip data
    // 3. If spatial, creating an emitter and attaching the sound
    // 4. Storing the handle in AudioPlayback for later control
}

/// Helper function to pause an audio source
pub fn pause_audio(world: &mut World, entity: Entity) {
    let playback = world.get_resource_mut::<AudioPlayback>().unwrap();

    if let Some(sound) = playback.sounds.get_mut(&entity) {
        sound.pause(Tween::default());
    }
}

/// Helper function to resume an audio source
pub fn resume_audio(world: &mut World, entity: Entity) {
    let playback = world.get_resource_mut::<AudioPlayback>().unwrap();

    if let Some(sound) = playback.sounds.get_mut(&entity) {
        sound.resume(Tween::default());
    }
}

/// Helper function to stop an audio source
pub fn stop_audio(world: &mut World, entity: Entity) {
    let playback = world.get_resource_mut::<AudioPlayback>().unwrap();

    if let Some(mut sound) = playback.sounds.remove(&entity) {
        sound.stop(Tween::default());
    }

    // Also remove emitter if it exists
    playback.emitters.remove(&entity);
}
