use luminara_core::{Component, Entity};
use luminara_math::{Mat4, Quat, Vec3};
use luminara_asset::{Asset, Handle, AssetLoader, AssetLoadError};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
}

impl Asset for AnimationClip {
    fn type_name() -> &'static str {
        "AnimationClip"
    }
}

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub target_node_index: usize,
    pub target_path: AnimationPath,
    pub inputs: Vec<f32>, // Time keyframes
    pub outputs: AnimationOutput, // Value keyframes
}

#[derive(Debug, Clone)]
pub enum AnimationPath {
    Translation,
    Rotation,
    Scale,
    Weights,
}

#[derive(Debug, Clone)]
pub enum AnimationOutput {
    Vector3(Vec<Vec3>),
    Rotation(Vec<Quat>),
    Scalar(Vec<f32>),
}

/// Skeleton extracted from a GLB/glTF file.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    /// Parent index for each bone (None for root bones).
    pub hierarchy: Vec<Option<usize>>,
}

impl Asset for Skeleton {
    fn type_name() -> &'static str {
        "Skeleton"
    }
}

impl Component for Skeleton {
    fn type_name() -> &'static str {
        "Skeleton"
    }
}

/// A single bone in a skeleton.
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub local_transform: Mat4,
    pub inverse_bind_matrix: Mat4,
}

#[derive(Debug, Clone)]
pub struct SkinnedMesh {
    pub mesh: Handle<crate::mesh::Mesh>,
    pub joints: Vec<Entity>, // Entity IDs of bones
    pub inverse_bind_matrices: Vec<Mat4>,
    pub skeleton: Option<Skeleton>,
}

impl Component for SkinnedMesh {
    fn type_name() -> &'static str {
        "SkinnedMesh"
    }
}

/// Result of loading a full GLB scene - contains meshes, skeleton, and animation clips.
#[derive(Debug, Clone)]
pub struct GltfScene {
    pub name: String,
    pub skeleton: Option<Skeleton>,
    pub animation_clips: Vec<AnimationClip>,
    pub node_names: Vec<String>,
}

impl Asset for GltfScene {
    fn type_name() -> &'static str {
        "GltfScene"
    }
}

pub struct GltfLoader;

impl AssetLoader for GltfLoader {
    type Asset = GltfScene;

    fn extensions(&self) -> &[&str] {
        &["glb", "gltf"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        let gltf = gltf::Gltf::from_slice(bytes)
            .map_err(|e| AssetLoadError::Parse(e.to_string()))?;

        let blob = gltf.blob.as_deref();

        // Collect node names
        let node_names: Vec<String> = gltf.nodes()
            .map(|n| n.name().unwrap_or("unnamed").to_string())
            .collect();

        // ── Extract skeleton from first skin ─────────────────
        let skeleton = if let Some(skin) = gltf.skins().next() {
            let joints: Vec<_> = skin.joints().collect();
            let reader = skin.reader(|buf| {
                match buf.source() {
                    gltf::buffer::Source::Bin => blob,
                    gltf::buffer::Source::Uri(_) => None,
                }
            });

            let inverse_bind_matrices: Vec<Mat4> = reader
                .read_inverse_bind_matrices()
                .map(|ibm| {
                    ibm.map(|m| {
                        Mat4::from_cols_array_2d(&m)
                    }).collect()
                })
                .unwrap_or_else(|| vec![Mat4::IDENTITY; joints.len()]);

            // Build bone list
            let mut bones = Vec::with_capacity(joints.len());
            let mut hierarchy = Vec::with_capacity(joints.len());

            // Create a map from node index -> joint index
            let mut node_to_joint: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            for (ji, joint) in joints.iter().enumerate() {
                node_to_joint.insert(joint.index(), ji);
            }

            for (ji, joint) in joints.iter().enumerate() {
                let (t, r, s) = joint.transform().decomposed();
                let translation = Vec3::new(t[0], t[1], t[2]);
                let rotation = Quat::from_xyzw(r[0], r[1], r[2], r[3]);
                let scale = Vec3::new(s[0], s[1], s[2]);
                let local_transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);

                let ibm = if ji < inverse_bind_matrices.len() {
                    inverse_bind_matrices[ji]
                } else {
                    Mat4::IDENTITY
                };

                bones.push(Bone {
                    name: joint.name().unwrap_or("bone").to_string(),
                    local_transform,
                    inverse_bind_matrix: ibm,
                });

                // Find parent joint index
                let parent_idx = joints.iter().enumerate().find(|(_, potential_parent)| {
                    potential_parent.children().any(|child| child.index() == joint.index())
                }).map(|(pi, _)| pi);
                hierarchy.push(parent_idx);
            }

            Some(Skeleton { bones, hierarchy })
        } else {
            None
        };

        // ── Extract all animations ───────────────────────────
        let mut animation_clips = Vec::new();

        for anim in gltf.animations() {
            let mut channels = Vec::new();
            let mut max_time: f32 = 0.0;

            for channel in anim.channels() {
                let target = channel.target();
                let node_index = target.node().index();
                let property = target.property();

                let reader = channel.reader(|buf| {
                    match buf.source() {
                        gltf::buffer::Source::Bin => blob,
                        gltf::buffer::Source::Uri(_) => None,
                    }
                });

                // Read input timestamps
                let inputs: Vec<f32> = reader
                    .read_inputs()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();

                if let Some(&last) = inputs.last() {
                    max_time = max_time.max(last);
                }

                // Read output values
                let (target_path, outputs) = match property {
                    gltf::animation::Property::Translation => {
                        let values: Vec<Vec3> = reader
                            .read_outputs()
                            .map(|out| match out {
                                gltf::animation::util::ReadOutputs::Translations(iter) => {
                                    iter.map(|t| Vec3::new(t[0], t[1], t[2])).collect()
                                }
                                _ => Vec::new(),
                            })
                            .unwrap_or_default();
                        (AnimationPath::Translation, AnimationOutput::Vector3(values))
                    }
                    gltf::animation::Property::Rotation => {
                        let values: Vec<Quat> = reader
                            .read_outputs()
                            .map(|out| match out {
                                gltf::animation::util::ReadOutputs::Rotations(iter) => {
                                    iter.into_f32()
                                        .map(|r| Quat::from_xyzw(r[0], r[1], r[2], r[3]))
                                        .collect()
                                }
                                _ => Vec::new(),
                            })
                            .unwrap_or_default();
                        (AnimationPath::Rotation, AnimationOutput::Rotation(values))
                    }
                    gltf::animation::Property::Scale => {
                        let values: Vec<Vec3> = reader
                            .read_outputs()
                            .map(|out| match out {
                                gltf::animation::util::ReadOutputs::Scales(iter) => {
                                    iter.map(|s| Vec3::new(s[0], s[1], s[2])).collect()
                                }
                                _ => Vec::new(),
                            })
                            .unwrap_or_default();
                        (AnimationPath::Scale, AnimationOutput::Vector3(values))
                    }
                    gltf::animation::Property::MorphTargetWeights => {
                        let values: Vec<f32> = reader
                            .read_outputs()
                            .map(|out| match out {
                                gltf::animation::util::ReadOutputs::MorphTargetWeights(iter) => {
                                    iter.into_f32().collect()
                                }
                                _ => Vec::new(),
                            })
                            .unwrap_or_default();
                        (AnimationPath::Weights, AnimationOutput::Scalar(values))
                    }
                };

                if !inputs.is_empty() {
                    channels.push(AnimationChannel {
                        target_node_index: node_index,
                        target_path,
                        inputs,
                        outputs,
                    });
                }
            }

            animation_clips.push(AnimationClip {
                name: anim.name().unwrap_or("default").to_string(),
                duration: max_time,
                channels,
            });
        }

        Ok(GltfScene {
            name: _path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unnamed".to_string()),
            skeleton,
            animation_clips,
            node_names,
        })
    }
}
