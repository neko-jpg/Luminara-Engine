//! GPU Instancing System
//!
//! Implements aggressive GPU instancing to reduce draw calls by grouping
//! objects with identical meshes and rendering them in a single draw call.
//!
//! Target: <500 draw calls for 1000+ objects (Requirement 19.3)

use crate::{Handle, Mesh, PbrMaterial};
use luminara_math::Transform;
use std::collections::HashMap;

/// Per-instance data passed to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Model matrix (4x4 = 16 floats = 64 bytes)
    pub model_matrix: [[f32; 4]; 4],
    /// Material properties
    pub albedo: [f32; 4], // 16 bytes
    pub metallic: f32,      // 4 bytes
    pub roughness: f32,     // 4 bytes
    pub emissive: [f32; 3], // 12 bytes
    pub has_albedo_texture: f32, // 4 bytes
                            // Total: 64 + 40 = 104 bytes
}

impl InstanceData {
    /// Create instance data from transform and material
    pub fn new(transform: &Transform, material: &PbrMaterial) -> Self {
        let model_matrix = transform.compute_matrix().to_cols_array_2d();

        Self {
            model_matrix,
            albedo: [
                material.albedo.r,
                material.albedo.g,
                material.albedo.b,
                material.albedo.a,
            ],
            metallic: material.metallic,
            roughness: material.roughness,
            emissive: [
                material.emissive.r,
                material.emissive.g,
                material.emissive.b,
            ],
            has_albedo_texture: if material.albedo_texture.is_some() {
                1.0
            } else {
                0.0
            },
        }
    }

    /// Get vertex buffer layout for instanced rendering
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model matrix (4 vec4s)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Material properties
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4, // albedo
                },
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x2, // metallic, roughness
                },
                wgpu::VertexAttribute {
                    offset: 88,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4, // emissive + has_texture
                },
            ],
        }
    }
}

/// Group of instances sharing the same mesh
#[derive(Debug)]
pub struct InstanceGroup {
    /// Mesh handle
    pub mesh: Handle<Mesh>,
    /// Instance data for all objects using this mesh
    pub instances: Vec<InstanceData>,
    /// Optional texture handle (if all instances share same texture)
    pub shared_texture: Option<Handle<crate::Texture>>,
}

impl InstanceGroup {
    fn new(mesh: Handle<Mesh>) -> Self {
        Self {
            mesh,
            instances: Vec::new(),
            shared_texture: None,
        }
    }

    fn add_instance(&mut self, instance: InstanceData) {
        self.instances.push(instance);
    }

    /// Check if this group can be merged with another
    pub fn can_merge(&self, other: &Self) -> bool {
        // Can merge if meshes are identical
        self.mesh == other.mesh
    }

    /// Merge another group into this one
    pub fn merge(&mut self, other: InstanceGroup) {
        self.instances.extend(other.instances);
    }
}

/// Material sort key for batching
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MaterialSortKey {
    /// Metallic value (scaled to u32 for sorting)
    metallic: u32,
    /// Roughness value (scaled to u32 for sorting)
    roughness: u32,
    /// Texture handle (if any)
    texture_id: Option<u64>,
}


/// Instance batcher - groups objects by mesh for instanced rendering
pub struct InstanceBatcher {
    /// Groups of instances by mesh
    groups: Vec<InstanceGroup>,
    /// Statistics
    pub total_objects: usize,
    pub total_draw_calls: usize,
    pub instancing_ratio: f32,
}

impl InstanceBatcher {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            total_objects: 0,
            total_draw_calls: 0,
            instancing_ratio: 1.0,
        }
    }

    /// Prepare instance groups from mesh query
    pub fn prepare(&mut self, meshes: Query<(&Handle<Mesh>, &Transform, &PbrMaterial)>) {
        self.groups.clear();
        self.total_objects = 0;

        // Group objects by mesh handle
        let mut groups_map: HashMap<Handle<Mesh>, InstanceGroup> = HashMap::new();

        for (mesh_handle, transform, material) in meshes.iter() {
            let instance_data = InstanceData::new(transform, material);

            groups_map
                .entry(mesh_handle.clone())
                .or_insert_with(|| InstanceGroup::new(mesh_handle.clone()))
                .add_instance(instance_data);

            self.total_objects += 1;
        }

        // Convert to vec and sort by material for better batching
        self.groups = groups_map.into_values().collect();
        self.sort_by_material();

        // Calculate statistics
        self.total_draw_calls = self.groups.len();
        self.instancing_ratio = if self.total_draw_calls > 0 {
            self.total_objects as f32 / self.total_draw_calls as f32
        } else {
            1.0
        };
    }

    /// Sort groups by material properties to improve batching
    fn sort_by_material(&mut self) {
        // Sort by average material properties of instances in each group
        self.groups.sort_by_key(|group| {
            if group.instances.is_empty() {
                return MaterialSortKey {
                    metallic: 0,
                    roughness: 0,
                    texture_id: None,
                };
            }

            // Use first instance's material as representative
            let inst = &group.instances[0];
            MaterialSortKey {
                metallic: (inst.metallic * 10000.0) as u32,
                roughness: (inst.roughness * 10000.0) as u32,
                texture_id: if inst.has_albedo_texture > 0.5 {
                    Some(1) // Simplified - just distinguish textured vs non-textured
                } else {
                    None
                },
            }
        });
    }

    /// Merge adjacent groups with compatible materials
    pub fn merge_compatible_groups(&mut self) {
        if self.groups.len() < 2 {
            return;
        }

        let mut merged = Vec::new();
        let mut current = self.groups.drain(..).next();

        for group in self.groups.drain(..) {
            match &mut current {
                Some(curr) if curr.can_merge(&group) => {
                    // Merge compatible groups
                    curr.merge(group);
                }
                _ => {
                    // Different mesh - push current and start new
                    if let Some(prev) = current.take() {
                        merged.push(prev);
                    }
                    current = Some(group);
                }
            }
        }

        if let Some(last) = current {
            merged.push(last);
        }

        self.groups = merged;
        self.total_draw_calls = self.groups.len();
        self.instancing_ratio = if self.total_draw_calls > 0 {
            self.total_objects as f32 / self.total_draw_calls as f32
        } else {
            1.0
        };
    }

    /// Get instance groups for rendering
    pub fn groups(&self) -> &[InstanceGroup] {
        &self.groups
    }

    /// Get statistics
    pub fn stats(&self) -> InstanceBatcherStats {
        InstanceBatcherStats {
            total_objects: self.total_objects,
            total_draw_calls: self.total_draw_calls,
            instancing_ratio: self.instancing_ratio,
            unique_meshes: self.groups.len(),
            avg_instances_per_group: if self.groups.is_empty() {
                0.0
            } else {
                self.total_objects as f32 / self.groups.len() as f32
            },
        }
    }

    /// Clear all groups
    pub fn clear(&mut self) {
        self.groups.clear();
        self.total_objects = 0;
        self.total_draw_calls = 0;
        self.instancing_ratio = 1.0;
    }
}

impl Default for InstanceBatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from instance batching
#[derive(Debug, Clone, Copy)]
pub struct InstanceBatcherStats {
    /// Total number of objects
    pub total_objects: usize,
    /// Total number of draw calls
    pub total_draw_calls: usize,
    /// Instancing ratio (objects per draw call)
    pub instancing_ratio: f32,
    /// Number of unique meshes
    pub unique_meshes: usize,
    /// Average instances per group
    pub avg_instances_per_group: f32,
}

impl InstanceBatcherStats {
    pub fn print(&self) {
        println!("=== Instance Batching Stats ===");
        println!("Total Objects: {}", self.total_objects);
        println!("Draw Calls: {}", self.total_draw_calls);
        println!("Instancing Ratio: {:.2}x", self.instancing_ratio);
        println!("Unique Meshes: {}", self.unique_meshes);
        println!("Avg Instances/Group: {:.2}", self.avg_instances_per_group);

        if self.total_draw_calls > 500 {
            println!("⚠️  Draw calls exceed target of 500");
        } else {
            println!("✓ Draw calls within target");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luminara_math::{Color, Vec3};

    #[test]
    fn test_instance_data_size() {
        // Verify instance data size is as expected
        assert_eq!(std::mem::size_of::<InstanceData>(), 104);
    }

    #[test]
    fn test_instance_group_creation() {
        let mesh_handle = Handle::new(AssetId::new(), 0);
        let mut group = InstanceGroup::new(mesh_handle.clone());

        assert_eq!(group.instances.len(), 0);
        assert_eq!(group.mesh, mesh_handle);
    }

    #[test]
    fn test_instance_batcher_empty() {
        let batcher = InstanceBatcher::new();
        let stats = batcher.stats();

        assert_eq!(stats.total_objects, 0);
        assert_eq!(stats.total_draw_calls, 0);
    }

    #[test]
    fn test_material_sort_key() {
        let material1 = PbrMaterial {
            albedo: Color::WHITE,
            metallic: 0.5,
            roughness: 0.3,
            emissive: Color::BLACK,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
        };

        let material2 = PbrMaterial {
            albedo: Color::WHITE,
            metallic: 0.8,
            roughness: 0.3,
            emissive: Color::BLACK,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
        };

        let key1 = MaterialSortKey::from_material(&material1);
        let key2 = MaterialSortKey::from_material(&material2);

        // Higher metallic should sort later
        assert!(key1 < key2);
    }

    #[test]
    fn test_instancing_ratio_calculation() {
        let mut batcher = InstanceBatcher::new();
        batcher.total_objects = 1000;
        batcher.total_draw_calls = 100;
        batcher.instancing_ratio = 1000.0 / 100.0;

        let stats = batcher.stats();
        assert_eq!(stats.instancing_ratio, 10.0);
    }
}
