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

#[derive(Debug, Clone)]
pub struct SkinnedMesh {
    pub mesh: Handle<crate::mesh::Mesh>,
    pub joints: Vec<Entity>, // Entity IDs of bones
    pub inverse_bind_matrices: Vec<Mat4>,
}

impl Component for SkinnedMesh {
    fn type_name() -> &'static str {
        "SkinnedMesh"
    }
}

pub struct GltfLoader;

impl AssetLoader for GltfLoader {
    type Asset = AnimationClip; // Simplified: Load animation clip directly or a GltfScene asset

    fn extensions(&self) -> &[&str] {
        &["glb", "gltf"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Parsing logic would go here using gltf crate
        // For Phase 1 validation, we return a mock or simple parse result
        // Real implementation requires complex scene graph mapping

        let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| AssetLoadError::Parse(e.to_string()))?;

        // Extract first animation
        if let Some(anim) = gltf.animations().next() {
            let clip = AnimationClip {
                name: anim.name().unwrap_or("default").to_string(),
                duration: 1.0, // Need to compute max input
                channels: Vec::new(), // Populate channels
            };
            Ok(clip)
        } else {
            Err(AssetLoadError::Parse("No animation found".to_string()))
        }
    }
}

// NOTE: This file `src/animation.rs` needs to be added to `lib.rs`
