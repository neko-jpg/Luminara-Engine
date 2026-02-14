use luminara_core::{App, AppInterface, Component, CoreStage, Plugin, Query, Res};
use luminara_core::system::FunctionMarker;
use luminara_asset::{AssetServer, Handle};
use luminara_math::{Vec3, Quat};
use crate::animation::{AnimationClip, AnimationOutput, AnimationPath};

pub struct AnimationPlayer {
    pub current_clip: Option<Handle<AnimationClip>>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub playing: bool,
    /// Sampled bone transforms updated each frame by the animation system.
    pub sampled_transforms: Vec<SampledBoneTransform>,
}

/// Sampled bone transform from animation playback.
#[derive(Debug, Clone, Default)]
pub struct SampledBoneTransform {
    pub node_index: usize,
    pub translation: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub scale: Option<Vec3>,
}

impl Component for AnimationPlayer {
    fn type_name() -> &'static str {
        "AnimationPlayer"
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: true,
            sampled_transforms: Vec::new(),
        }
    }
}

impl AnimationPlayer {
    /// Start playing an animation clip.
    pub fn play(&mut self, clip: Handle<AnimationClip>) {
        self.current_clip = Some(clip);
        self.time = 0.0;
        self.playing = true;
    }

    /// Stop playback.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Resume playback.
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Check if animation has finished (only meaningful for non-looping).
    pub fn is_finished(&self) -> bool {
        !self.playing && !self.looping
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn name(&self) -> &str {
        "AnimationPlugin"
    }

    fn build(&self, app: &mut App) {
        app.add_system::<(
            FunctionMarker,
            Query<'static, &mut AnimationPlayer>,
            Res<'static, AssetServer>,
            Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, animation_system);
    }
}

pub fn animation_system(
    mut players: Query<&mut AnimationPlayer>,
    assets: Res<AssetServer>,
    time: Res<luminara_core::Time>,
) {
    let dt = time.delta_seconds();

    for player in players.iter_mut() {
        if !player.playing || player.current_clip.is_none() {
            continue;
        }

        let clip_handle = player.current_clip.as_ref().unwrap();
        if let Some(clip) = assets.get(clip_handle) {
            // Update time
            player.time += dt * player.speed;

            if player.looping {
                if clip.duration > 0.0 {
                    player.time %= clip.duration;
                }
            } else if player.time > clip.duration {
                player.time = clip.duration;
                player.playing = false;
            }

            // Sample all channels and store results
            player.sampled_transforms.clear();

            for channel in &clip.channels {
                let inputs = &channel.inputs;
                if inputs.is_empty() {
                    continue;
                }

                // Binary search for the keyframe bracket
                let mut lo = 0usize;
                let mut hi = inputs.len().saturating_sub(1);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    if inputs[mid] < player.time {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                // lo is now the index of the first keyframe >= player.time
                let k = if lo > 0 { lo - 1 } else { 0 };

                let t = if k + 1 < inputs.len() {
                    let duration = inputs[k + 1] - inputs[k];
                    if duration > 0.0001 {
                        ((player.time - inputs[k]) / duration).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let mut sampled = SampledBoneTransform {
                    node_index: channel.target_node_index,
                    ..Default::default()
                };

                match (&channel.target_path, &channel.outputs) {
                    (AnimationPath::Translation, AnimationOutput::Vector3(values)) => {
                        if k < values.len() {
                            let val = if k + 1 < values.len() {
                                values[k].lerp(values[k + 1], t)
                            } else {
                                values[k]
                            };
                            sampled.translation = Some(val);
                        }
                    }
                    (AnimationPath::Rotation, AnimationOutput::Rotation(values)) => {
                        if k < values.len() {
                            let val = if k + 1 < values.len() {
                                values[k].slerp(values[k + 1], t)
                            } else {
                                values[k]
                            };
                            sampled.rotation = Some(val);
                        }
                    }
                    (AnimationPath::Scale, AnimationOutput::Vector3(values)) => {
                        if k < values.len() {
                            let val = if k + 1 < values.len() {
                                values[k].lerp(values[k + 1], t)
                            } else {
                                values[k]
                            };
                            sampled.scale = Some(val);
                        }
                    }
                    _ => {}
                }

                player.sampled_transforms.push(sampled);
            }
        }
    }
}
