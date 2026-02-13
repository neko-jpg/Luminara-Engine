use luminara_core::Entity;
use serde::{Deserialize, Serialize};

/// Handle to an audio clip asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioClipHandle(pub String);

/// Audio source component for playing sounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource {
    /// Handle to the audio clip to play
    pub clip: AudioClipHandle,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pitch multiplier (1.0 is normal pitch)
    pub pitch: f32,
    /// Whether the audio should loop
    pub looping: bool,
    /// Whether to use spatial audio (3D positioning)
    pub spatial: bool,
    /// Maximum distance for spatial audio attenuation
    pub max_distance: f32,
}

impl luminara_core::Component for AudioSource {
    fn type_name() -> &'static str {
        "AudioSource"
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip: AudioClipHandle(String::new()),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: false,
            max_distance: 100.0,
        }
    }
}

/// Audio listener component (typically attached to the camera)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioListener {
    /// Whether this listener is enabled
    pub enabled: bool,
}

impl luminara_core::Component for AudioListener {
    fn type_name() -> &'static str {
        "AudioListener"
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Commands for controlling audio playback
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Start playing an audio source
    Play(Entity),
    /// Pause an audio source
    Pause(Entity),
    /// Resume a paused audio source
    Resume(Entity),
    /// Stop an audio source
    Stop(Entity),
}
