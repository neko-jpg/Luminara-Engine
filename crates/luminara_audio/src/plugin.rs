use luminara_core::{App, AppInterface, CoreStage, Plugin};
use luminara_core::system::ExclusiveMarker;
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use log::{info, warn};

/// Wrapper for kira's AudioManager to implement Resource
pub struct KiraAudioManager(pub AudioManager<DefaultBackend>);

impl luminara_core::Resource for KiraAudioManager {}

/// Audio plugin that initializes the kira audio system
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn name(&self) -> &str {
        "AudioPlugin"
    }

    fn build(&self, app: &mut App) {
        info!("Initializing AudioPlugin");
        
        // Initialize kira audio manager
        // If audio device is not available (e.g., in test environments), log a warning
        // and skip audio initialization rather than panicking
        match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(audio_manager) => {
                app.insert_resource(KiraAudioManager(audio_manager));
                
                // Add audio systems
                app.add_system::<ExclusiveMarker>(CoreStage::Update, crate::systems::audio_system);
                
                info!("AudioPlugin initialized successfully");
            }
            Err(e) => {
                warn!("Failed to create audio manager: {:?}. Audio will be disabled.", e);
                warn!("This is expected in headless environments or when no audio device is available.");
            }
        }
    }
}
