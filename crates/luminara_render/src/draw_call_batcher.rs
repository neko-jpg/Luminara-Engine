//! Aggressive Draw Call Batching System
//!
//! Implements state-change minimization through sorting and batching:
//! - Sort by: shader → texture → material
//! - Batch identical materials
//! - Target: <100 draw calls for 1000+ objects (Requirement 19.3)

use crate::{Handle, Mesh, PbrMaterial, Texture};
use luminara_asset::AssetServer;
use luminara_core::Query;
use luminara_math::Transform;
use std::collections::HashMap;

/// Sort key for draw call batching
/// Ordered by: shader → texture → material properties
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrawCallSortKey {
    /// Shader ID (for now, all use same PBR shader, but extensible)
    pub shader_id: u64,
    /// Texture ID (None for untextured)
    pub texture_id: Option<u64>,
    /// Material properties (quantized for sorting)
    pub material_key: MaterialKey,
}

/// Quantized material properties for sorting
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialKey {
    /// Albedo color (quantized to 8-bit per channel)
    pub albedo: [u8; 4],
    /// Metallic (quantized to 8-bit)
    pub metallic: u8,
    /// Roughness (quantized to 8-bit)
    pub roughness: u8,
    /// Emissive (quantized to 8-bit per channel)
    pub emissive: [u8; 3],
}

impl MaterialKey {
    /// Create material key from PBR material
    pub fn from_material(material: &PbrMaterial) -> Self {
        Self {
            albedo: [
                (material.albedo.r * 255.0) as u8,
                (material.albedo.g * 255.0) as u8,
                (material.albedo.b * 255.0) as u8,
                (material.albedo.a * 255.0) as u8,
            ],
            metallic: (material.metallic * 255.0) as u8,
            roughness: (material.roughness * 255.0) as u8,
            emissive: [
                (material.emissive.r * 255.0) as u8,
                (material.emissive.g * 255.0) as u8,
                (material.emissive.b * 255.0) as u8,
            ],
        }
    }
}

/// Batched draw call containing multiple instances
#[derive(Debug)]
pub struct BatchedDrawCall {
    /// Sort key for this batch
    pub sort_key: DrawCallSortKey,
    /// Mesh handle
    pub mesh: Handle<Mesh>,
    /// Texture handle (if any)
    pub texture: Option<Handle<Texture>>,
    /// Material properties
    pub material: PbrMaterial,
    /// Instance transforms
    pub instances: Vec<Transform>,
}

impl BatchedDrawCall {
    fn new(
        sort_key: DrawCallSortKey,
        mesh: Handle<Mesh>,
        texture: Option<Handle<Texture>>,
        material: PbrMaterial,
    ) -> Self {
        Self {
            sort_key,
            mesh,
            texture,
            material,
            instances: Vec::new(),
        }
    }

    fn add_instance(&mut self, transform: Transform) {
        self.instances.push(transform);
    }

    /// Get instance count
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

/// Draw call batcher - sorts and batches draw calls to minimize state changes
pub struct DrawCallBatcher {
    /// Batched draw calls, sorted by shader → texture → material
    batches: Vec<BatchedDrawCall>,
    /// Statistics
    pub total_objects: usize,
    pub total_batches: usize,
    pub batching_ratio: f32,
}

impl DrawCallBatcher {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
            total_objects: 0,
            total_batches: 0,
            batching_ratio: 1.0,
        }
    }

    /// Prepare batched draw calls from mesh query
    /// Sorts by: shader → texture → material
    pub fn prepare(
        &mut self,
        meshes: Query<(&Handle<Mesh>, &Transform, &PbrMaterial)>,
        asset_server: &AssetServer,
    ) {
        self.batches.clear();
        self.total_objects = 0;

        // Group objects by sort key
        let mut batch_map: HashMap<DrawCallSortKey, Vec<(Handle<Mesh>, Transform, PbrMaterial)>> =
            HashMap::new();

        for (mesh_handle, transform, material) in meshes.iter() {
            // Create sort key
            let sort_key = self.create_sort_key(material, asset_server);

            batch_map
                .entry(sort_key)
                .or_insert_with(Vec::new)
                .push((mesh_handle.clone(), *transform, material.clone()));

            self.total_objects += 1;
        }

        // Convert to batches and sort
        for (sort_key, objects) in batch_map {
            // Further group by mesh within same material
            let mut mesh_groups: HashMap<Handle<Mesh>, Vec<(Transform, PbrMaterial)>> =
                HashMap::new();

            for (mesh, transform, material) in objects {
                mesh_groups
                    .entry(mesh)
                    .or_insert_with(Vec::new)
                    .push((transform, material));
            }

            // Create batch for each mesh group
            for (mesh, instances) in mesh_groups {
                if instances.is_empty() {
                    continue;
                }

                // Use first instance's material as representative
                let material = instances[0].1.clone();
                let texture = material.albedo_texture.clone();

                let mut batch = BatchedDrawCall::new(sort_key.clone(), mesh, texture, material);

                for (transform, _) in instances {
                    batch.add_instance(transform);
                }

                self.batches.push(batch);
            }
        }

        // Sort batches by sort key (shader → texture → material)
        self.batches.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));

        // Calculate statistics
        self.total_batches = self.batches.len();
        self.batching_ratio = if self.total_batches > 0 {
            self.total_objects as f32 / self.total_batches as f32
        } else {
            1.0
        };
    }

    /// Create sort key for material
    fn create_sort_key(
        &self,
        material: &PbrMaterial,
        asset_server: &AssetServer,
    ) -> DrawCallSortKey {
        // For now, all use same PBR shader (shader_id = 0)
        // In future, different shaders would have different IDs
        let shader_id = 0;

        // Get texture ID if present
        let texture_id = material.albedo_texture.as_ref().and_then(|handle| {
            // Check if texture is loaded
            if asset_server.get::<Texture>(handle).is_some() {
                // Use handle's asset ID as texture ID
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                handle.id().hash(&mut hasher);
                Some(hasher.finish())
            } else {
                None
            }
        });

        // Create material key
        let material_key = MaterialKey::from_material(material);

        DrawCallSortKey {
            shader_id,
            texture_id,
            material_key,
        }
    }

    /// Get batched draw calls (sorted by shader → texture → material)
    pub fn batches(&self) -> &[BatchedDrawCall] {
        &self.batches
    }

    /// Get statistics
    pub fn stats(&self) -> DrawCallBatcherStats {
        let max_instances = self
            .batches
            .iter()
            .map(|b| b.instance_count())
            .max()
            .unwrap_or(0);

        let min_instances = self
            .batches
            .iter()
            .map(|b| b.instance_count())
            .min()
            .unwrap_or(0);

        DrawCallBatcherStats {
            total_objects: self.total_objects,
            total_batches: self.total_batches,
            batching_ratio: self.batching_ratio,
            max_instances_per_batch: max_instances,
            min_instances_per_batch: min_instances,
            avg_instances_per_batch: if self.total_batches > 0 {
                self.total_objects as f32 / self.total_batches as f32
            } else {
                0.0
            },
        }
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        self.batches.clear();
        self.total_objects = 0;
        self.total_batches = 0;
        self.batching_ratio = 1.0;
    }

    /// Merge adjacent batches with identical materials
    /// This further reduces draw calls when objects happen to be sorted together
    pub fn merge_adjacent_batches(&mut self) {
        if self.batches.len() < 2 {
            return;
        }

        let mut merged = Vec::new();
        let mut current: Option<BatchedDrawCall> = None;

        for batch in self.batches.drain(..) {
            match &mut current {
                Some(curr)
                    if curr.sort_key == batch.sort_key && curr.mesh == batch.mesh =>
                {
                    // Merge instances from compatible batch
                    curr.instances.extend(batch.instances);
                }
                _ => {
                    // Different material or mesh - push current and start new
                    if let Some(prev) = current.take() {
                        merged.push(prev);
                    }
                    current = Some(batch);
                }
            }
        }

        if let Some(last) = current {
            merged.push(last);
        }

        self.batches = merged;
        self.total_batches = self.batches.len();
        self.batching_ratio = if self.total_batches > 0 {
            self.total_objects as f32 / self.total_batches as f32
        } else {
            1.0
        };
    }
}

impl Default for DrawCallBatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from draw call batching
#[derive(Debug, Clone, Copy)]
pub struct DrawCallBatcherStats {
    /// Total number of objects
    pub total_objects: usize,
    /// Total number of batches (draw calls)
    pub total_batches: usize,
    /// Batching ratio (objects per draw call)
    pub batching_ratio: f32,
    /// Maximum instances in a single batch
    pub max_instances_per_batch: usize,
    /// Minimum instances in a single batch
    pub min_instances_per_batch: usize,
    /// Average instances per batch
    pub avg_instances_per_batch: f32,
}

impl DrawCallBatcherStats {
    pub fn print(&self) {
        println!("=== Draw Call Batching Stats ===");
        println!("Total Objects: {}", self.total_objects);
        println!("Draw Calls (Batches): {}", self.total_batches);
        println!("Batching Ratio: {:.2}x", self.batching_ratio);
        println!("Max Instances/Batch: {}", self.max_instances_per_batch);
        println!("Min Instances/Batch: {}", self.min_instances_per_batch);
        println!("Avg Instances/Batch: {:.2}", self.avg_instances_per_batch);

        // Check against target: <100 draw calls for 1000+ objects
        if self.total_objects >= 1000 && self.total_batches > 100 {
            println!(
                "⚠️  Draw calls exceed target of 100 for 1000+ objects (current: {})",
                self.total_batches
            );
        } else if self.total_objects >= 1000 {
            println!("✓ Draw calls within target (<100 for 1000+ objects)");
        }
    }

    /// Check if batching meets performance target
    pub fn meets_target(&self) -> bool {
        // Target: <100 draw calls for 1000+ objects
        if self.total_objects >= 1000 {
            // For 1000+ objects, scale the target proportionally
            // 1000 objects -> <100 batches
            // 2000 objects -> <200 batches
            let max_batches = (self.total_objects as f32 / 10.0).ceil() as usize;
            self.total_batches < max_batches
        } else {
            // For fewer objects, scale proportionally
            let expected_batches = (self.total_objects as f32 / 10.0).ceil() as usize;
            self.total_batches <= expected_batches
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luminara_asset::AssetId;
    use luminara_math::{Color, Vec3};

    #[test]
    fn test_material_key_creation() {
        let material = PbrMaterial {
            albedo: Color::rgba(1.0, 0.5, 0.25, 1.0),
            metallic: 0.8,
            roughness: 0.3,
            emissive: Color::BLACK,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
        };

        let key = MaterialKey::from_material(&material);

        assert_eq!(key.albedo, [255, 127, 63, 255]);
        assert_eq!(key.metallic, 204); // 0.8 * 255
        assert_eq!(key.roughness, 76); // 0.3 * 255
    }

    #[test]
    fn test_sort_key_ordering() {
        // Shader ID takes precedence
        let key1 = DrawCallSortKey {
            shader_id: 0,
            texture_id: Some(1),
            material_key: MaterialKey {
                albedo: [255, 255, 255, 255],
                metallic: 128,
                roughness: 128,
                emissive: [0, 0, 0],
            },
        };

        let key2 = DrawCallSortKey {
            shader_id: 1,
            texture_id: None,
            material_key: MaterialKey {
                albedo: [0, 0, 0, 255],
                metallic: 0,
                roughness: 0,
                emissive: [0, 0, 0],
            },
        };

        assert!(key1 < key2); // Lower shader ID sorts first

        // Texture ID is second priority
        let key3 = DrawCallSortKey {
            shader_id: 0,
            texture_id: None,
            material_key: MaterialKey {
                albedo: [255, 255, 255, 255],
                metallic: 255,
                roughness: 255,
                emissive: [255, 255, 255],
            },
        };

        let key4 = DrawCallSortKey {
            shader_id: 0,
            texture_id: Some(1),
            material_key: MaterialKey {
                albedo: [0, 0, 0, 255],
                metallic: 0,
                roughness: 0,
                emissive: [0, 0, 0],
            },
        };

        assert!(key3 < key4); // None sorts before Some
    }

    #[test]
    fn test_batcher_empty() {
        let batcher = DrawCallBatcher::new();
        let stats = batcher.stats();

        assert_eq!(stats.total_objects, 0);
        assert_eq!(stats.total_batches, 0);
    }

    #[test]
    fn test_batched_draw_call_creation() {
        let sort_key = DrawCallSortKey {
            shader_id: 0,
            texture_id: None,
            material_key: MaterialKey {
                albedo: [255, 255, 255, 255],
                metallic: 128,
                roughness: 128,
                emissive: [0, 0, 0],
            },
        };

        let mesh = Handle::new(AssetId::new(), 0);
        let material = PbrMaterial {
            albedo: Color::WHITE,
            metallic: 0.5,
            roughness: 0.5,
            emissive: Color::BLACK,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
        };

        let mut batch = BatchedDrawCall::new(sort_key, mesh, None, material);

        assert_eq!(batch.instance_count(), 0);

        batch.add_instance(Transform::from_translation(Vec3::ZERO));
        assert_eq!(batch.instance_count(), 1);
    }

    #[test]
    fn test_stats_meets_target() {
        // Test with 1000 objects and 50 batches - should meet target
        let stats = DrawCallBatcherStats {
            total_objects: 1000,
            total_batches: 50,
            batching_ratio: 20.0,
            max_instances_per_batch: 100,
            min_instances_per_batch: 5,
            avg_instances_per_batch: 20.0,
        };

        assert!(stats.meets_target());

        // Test with 1000 objects and 150 batches - should fail target
        let stats_fail = DrawCallBatcherStats {
            total_objects: 1000,
            total_batches: 150,
            batching_ratio: 6.67,
            max_instances_per_batch: 20,
            min_instances_per_batch: 1,
            avg_instances_per_batch: 6.67,
        };

        assert!(!stats_fail.meets_target());
    }
}
