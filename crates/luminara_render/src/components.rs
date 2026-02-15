use luminara_asset::{Asset, Handle};
use luminara_core::Component;
use luminara_math::Color;
use serde::{Deserialize, Serialize};
use luminara_reflect_derive::Reflect;

use crate::{Mesh, Texture};

/// Mesh renderer component
#[derive(Debug, Clone, Reflect)]
pub struct MeshRenderer {
    pub mesh: Handle<Mesh>,
    pub material: Handle<PbrMaterial>,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

impl Component for MeshRenderer {
    fn type_name() -> &'static str {
        "MeshRenderer"
    }
}

/// PBR Material
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PbrMaterial {
    pub albedo: Color,
    pub albedo_texture: Option<Handle<Texture>>,
    pub normal_texture: Option<Handle<Texture>>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    pub emissive: Color,
}

impl Component for PbrMaterial {
    fn type_name() -> &'static str {
        "PbrMaterial"
    }
}

impl Asset for PbrMaterial {
    fn type_name() -> &'static str {
        "PbrMaterial"
    }
}

/// Directional light component
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DirectionalLight {
    pub color: Color,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub shadow_cascade_count: u32,
}

impl Component for DirectionalLight {
    fn type_name() -> &'static str {
        "DirectionalLight"
    }
}

/// Point light component
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
}

impl Component for PointLight {
    fn type_name() -> &'static str {
        "PointLight"
    }
}

/// Level of Detail component
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Lod {
    pub distances: Vec<f32>,       // Distance thresholds for each LOD level
    pub meshes: Vec<Handle<Mesh>>, // Meshes for each LOD level
}

impl Component for Lod {
    fn type_name() -> &'static str {
        "Lod"
    }
}
