use crate::mesh::Mesh;
use luminara_asset::{AssetLoadError, AssetLoader};
use std::path::Path;

/// Asset loader for GLTF/GLB mesh files
pub struct MeshLoader;

impl AssetLoader for MeshLoader {
    type Asset = Mesh;

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }

    fn load(&self, bytes: &[u8], path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Load meshes from GLTF/GLB
        let meshes = Mesh::from_gltf(bytes).map_err(|e| {
            AssetLoadError::Parse(format!("Failed to parse GLTF file {:?}: {}", path, e))
        })?;

        // For now, return the first mesh
        // TODO: Support loading multiple meshes from a single GLTF file
        meshes
            .into_iter()
            .next()
            .ok_or_else(|| AssetLoadError::Parse("No meshes found in GLTF file".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_loader_extensions() {
        let loader = MeshLoader;
        let extensions = loader.extensions();
        assert_eq!(extensions.len(), 2);
        assert!(extensions.contains(&"gltf"));
        assert!(extensions.contains(&"glb"));
    }

    #[test]
    fn test_mesh_loader_invalid_data() {
        let loader = MeshLoader;
        let invalid_bytes = b"not a gltf file";
        let result = loader.load(invalid_bytes, Path::new("test.gltf"));
        assert!(result.is_err());
    }
}
