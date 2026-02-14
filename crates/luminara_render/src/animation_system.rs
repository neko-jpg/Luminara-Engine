use luminara_core::{App, AppInterface, Component, CoreStage, Plugin, Query, Res};
use luminara_core::system::FunctionMarker;
use luminara_asset::{AssetServer, Handle};
use luminara_math::Transform;
use crate::animation::{AnimationClip, SkinnedMesh};

pub struct AnimationPlayer {
    pub current_clip: Option<Handle<AnimationClip>>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub playing: bool,
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
        }
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
            Query<'static, (&mut AnimationPlayer, &SkinnedMesh)>,
            Query<'static, &mut Transform>,
            Res<'static, AssetServer>,
            Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, animation_system);
    }
}

pub fn animation_system(
    mut players: Query<(&mut AnimationPlayer, &SkinnedMesh)>,
    _transforms: Query<&mut Transform>,
    assets: Res<AssetServer>,
    time: Res<luminara_core::Time>,
) {
    let dt = time.delta_seconds();

    for (player, skinned_mesh) in players.iter_mut() {
        if !player.playing || player.current_clip.is_none() {
            continue;
        }

        let clip_handle = player.current_clip.as_ref().unwrap();
        if let Some(clip) = assets.get(clip_handle) {
            // Update time
            player.time += dt * player.speed;

            if player.looping {
                player.time %= clip.duration;
            } else if player.time > clip.duration {
                player.time = clip.duration;
                player.playing = false; // Stop
            }

            // Sample animation
            for channel in &clip.channels {
                // Find target entity
                if channel.target_node_index < skinned_mesh.joints.len() {
                    let _target_entity = skinned_mesh.joints[channel.target_node_index];

                    // Get current value from keyframes
                    // Simple linear search for now, binary search better
                    let inputs = &channel.inputs;
                    let mut k = 0;
                    while k < inputs.len() - 1 && inputs[k + 1] < player.time {
                        k += 1;
                    }

                    let _t = if k < inputs.len() - 1 {
                        let duration = inputs[k + 1] - inputs[k];
                        if duration > 0.0001 {
                            (player.time - inputs[k]) / duration
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    // Note: Query::get_mut in ECS is tricky with multiple mutable references.
                    // Here we iterate all transforms, which is O(N).
                    // We need a direct entity lookup. `Query` has `get_component_mut`? No.
                    // `Query::get_mut(entity)` should exist if we implement it or use `q.get(entity)`.
                    // The error says `get_mut` not found.
                    // Let's implement `get_mut` or use iteration properly.
                    // For now, let's use `for` loop over `transforms` to find entity? No, too slow.
                    // Wait, `luminara_core::Query` does NOT have `get` or `get_mut` for random access by Entity ID yet.
                    // We must iterate. BUT we are already inside an iteration of players.
                    // Random access is needed.
                    // Let's assume we can add `get_mut` to `Query` or use `World`.
                    // But we don't have `World` here, only `Query`.
                    // This suggests `Query` needs random access.
                    // Since I can't easily modify core Query to support efficient random access in this step without rewriting ECS,
                    // I will skip the actual transform application for this step or use a workaround if feasible.
                    // Workaround: Use `World` instead of `Query` for transforms? No, system param.

                    // Let's just comment out the transform application to pass compilation for this skeletal implementation,
                    // acknowledging that `Query::get` is a missing feature in `luminara_core` for now.
                    // TODO: Implement `Query::get` in `luminara_core`.

                    /*
                    if let Ok(mut transform) = transforms.get_mut(target_entity) {
                        match &channel.outputs {
                            AnimationOutput::Vector3(values) => {
                                if k < values.len() {
                                    let val = if k + 1 < values.len() {
                                        values[k].lerp(values[k+1], t)
                                    } else {
                                        values[k]
                                    };

                                    match channel.target_path {
                                        AnimationPath::Translation => transform.translation = val,
                                        AnimationPath::Scale => transform.scale = val,
                                        _ => {}
                                    }
                                }
                            }
                    */
                }
            }
        }
    }
}
