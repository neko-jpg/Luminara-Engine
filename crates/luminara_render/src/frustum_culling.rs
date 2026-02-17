/// Frustum culling optimization with spatial acceleration
///
/// This module implements efficient frustum culling using a BVH (Bounding Volume Hierarchy)
/// spatial acceleration structure. Target performance: >95% culling efficiency, <0.5ms CPU time for 10K objects.

use luminara_core::shared_types::{Component, Query};
use luminara_math::{Mat4, Vec3, Vec4};
use crate::AABB;
use luminara_asset::Handle;
use crate::Mesh;
use luminara_asset::AssetServer;
use std::sync::Arc;

/// Frustum plane representation (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    /// Create plane from normal and distance
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create plane from 4D vector (normal.xyz, distance)
    pub fn from_vec4(v: Vec4) -> Self {
        let normal = Vec3::new(v.x, v.y, v.z);
        let length = normal.length();
        Self {
            normal: normal / length,
            distance: v.w / length,
        }
    }

    /// Test if point is in front of plane (positive side)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Test if AABB intersects or is in front of plane
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        // Get the positive vertex (furthest in direction of normal)
        let p = Vec3::new(
            if self.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
            if self.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
            if self.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
        );

        // If positive vertex is behind plane, AABB is completely behind
        self.distance_to_point(p) >= 0.0
    }
}

/// View frustum with 6 planes (left, right, bottom, top, near, far)
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix
    pub fn from_view_projection(view_proj: &Mat4) -> Self {
        // Extract planes from view-projection matrix
        // Each plane is a row combination of the matrix
        let m = view_proj.to_cols_array_2d();

        // Left plane: m3 + m0
        let left = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        ));

        // Right plane: m3 - m0
        let right = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        ));

        // Bottom plane: m3 + m1
        let bottom = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        ));

        // Top plane: m3 - m1
        let top = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        ));

        // Near plane: m3 + m2
        let near = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        ));

        // Far plane: m3 - m2
        let far = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        ));

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Test if AABB is visible (intersects or is inside frustum)
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        // AABB must be in front of all 6 planes
        for plane in &self.planes {
            if !plane.intersects_aabb(aabb) {
                return false;
            }
        }
        true
    }

    /// Test if world-space AABB is visible
    pub fn intersects_world_aabb(&self, aabb: &AABB, transform: &Mat4) -> bool {
        // Transform AABB to world space
        let world_aabb = transform_aabb(aabb, transform);
        self.intersects_aabb(&world_aabb)
    }
}

/// Transform AABB by matrix (conservative bounding box)
fn transform_aabb(aabb: &AABB, transform: &Mat4) -> AABB {
    // Transform center
    let center = aabb.center();
    let extents = aabb.extents();
    
    let center_transformed = transform.transform_point3(center);
    
    // Transform extents (conservative approach)
    // Take absolute values of transform to get maximum extents
    let m = transform.to_cols_array_2d();
    let abs_m = [
        [m[0][0].abs(), m[0][1].abs(), m[0][2].abs()],
        [m[1][0].abs(), m[1][1].abs(), m[1][2].abs()],
        [m[2][0].abs(), m[2][1].abs(), m[2][2].abs()],
    ];
    
    let extents_transformed = Vec3::new(
        abs_m[0][0] * extents.x + abs_m[0][1] * extents.y + abs_m[0][2] * extents.z,
        abs_m[1][0] * extents.x + abs_m[1][1] * extents.y + abs_m[1][2] * extents.z,
        abs_m[2][0] * extents.x + abs_m[2][1] * extents.y + abs_m[2][2] * extents.z,
    );
    
    AABB::new(
        center_transformed - extents_transformed,
        center_transformed + extents_transformed,
    )
}

/// BVH node for spatial acceleration
#[derive(Debug, Clone)]
struct BVHNode {
    aabb: AABB,
    children: BVHChildren,
}

#[derive(Debug, Clone)]
enum BVHChildren {
    Leaf { entity_indices: Vec<usize> },
    Internal { left: Box<BVHNode>, right: Box<BVHNode> },
}

impl BVHNode {
    /// Build BVH from entity AABBs
    fn build(entities: &[(AABB, usize)], max_leaf_size: usize) -> Self {
        if entities.is_empty() {
            return Self {
                aabb: AABB::new(Vec3::ZERO, Vec3::ZERO),
                children: BVHChildren::Leaf { entity_indices: Vec::new() },
            };
        }

        // Compute bounding box for all entities
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for (aabb, _) in entities {
            min = min.min(aabb.min);
            max = max.max(aabb.max);
        }
        let aabb = AABB::new(min, max);

        // Leaf node if small enough
        if entities.len() <= max_leaf_size {
            return Self {
                aabb,
                children: BVHChildren::Leaf {
                    entity_indices: entities.iter().map(|(_, idx)| *idx).collect(),
                },
            };
        }

        // Split along longest axis
        let extents = aabb.extents();
        let split_axis = if extents.x > extents.y && extents.x > extents.z {
            0
        } else if extents.y > extents.z {
            1
        } else {
            2
        };

        // Sort entities by center along split axis
        let mut sorted_entities = entities.to_vec();
        sorted_entities.sort_by(|a, b| {
            let a_center = a.0.center();
            let b_center = b.0.center();
            let a_val = match split_axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };
            let b_val = match split_axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };
            a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Split in half
        let mid = sorted_entities.len() / 2;
        let (left_entities, right_entities) = sorted_entities.split_at(mid);

        Self {
            aabb,
            children: BVHChildren::Internal {
                left: Box::new(Self::build(left_entities, max_leaf_size)),
                right: Box::new(Self::build(right_entities, max_leaf_size)),
            },
        }
    }

    /// Query BVH for visible entities
    fn query(&self, frustum: &Frustum, visible: &mut Vec<usize>) {
        // Test node AABB against frustum
        if !frustum.intersects_aabb(&self.aabb) {
            return;
        }

        match &self.children {
            BVHChildren::Leaf { entity_indices } => {
                visible.extend(entity_indices);
            }
            BVHChildren::Internal { left, right } => {
                left.query(frustum, visible);
                right.query(frustum, visible);
            }
        }
    }
}

/// Frustum culling system with BVH acceleration
pub struct FrustumCullingSystem {
    bvh: Option<Arc<BVHNode>>,
    entity_data: Vec<EntityCullData>,
    needs_rebuild: bool,
}

#[derive(Debug, Clone)]
struct EntityCullData {
    aabb: AABB,
    transform: Mat4,
}

impl FrustumCullingSystem {
    pub fn new() -> Self {
        Self {
            bvh: None,
            entity_data: Vec::new(),
            needs_rebuild: true,
        }
    }

    /// Update entity data and mark for rebuild
    pub fn update_entities<T: Component>(
        &mut self,
        query: &Query<(&Handle<Mesh>, &luminara_math::Transform)>,
        asset_server: &AssetServer,
    ) {
        self.entity_data.clear();

        for (mesh_handle, transform) in query.iter() {
            if let Some(mesh) = asset_server.get(mesh_handle) {
                self.entity_data.push(EntityCullData {
                    aabb: mesh.aabb,
                    transform: transform.compute_matrix(),
                });
            }
        }

        self.needs_rebuild = true;
    }

    /// Rebuild BVH if needed
    pub fn rebuild_bvh(&mut self) {
        if !self.needs_rebuild {
            return;
        }

        let entities: Vec<(AABB, usize)> = self.entity_data
            .iter()
            .enumerate()
            .map(|(idx, data)| {
                // Transform AABB to world space for BVH
                let world_aabb = transform_aabb(&data.aabb, &data.transform);
                (world_aabb, idx)
            })
            .collect();

        self.bvh = Some(Arc::new(BVHNode::build(&entities, 16)));
        self.needs_rebuild = false;
    }

    /// Perform frustum culling and return visible entity indices
    pub fn cull(&self, frustum: &Frustum) -> Vec<usize> {
        let mut visible = Vec::new();

        if let Some(bvh) = &self.bvh {
            bvh.query(frustum, &mut visible);
        }

        visible
    }

    /// Get culling statistics
    pub fn stats(&self) -> CullingStats {
        CullingStats {
            total_entities: self.entity_data.len(),
            bvh_built: self.bvh.is_some(),
        }
    }
}

impl Default for FrustumCullingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Culling statistics
#[derive(Debug, Clone, Copy)]
pub struct CullingStats {
    pub total_entities: usize,
    pub bvh_built: bool,
}

/// Component to mark entities for culling
#[derive(Debug, Clone, Copy)]
pub struct Cullable;

impl Component for Cullable {
    fn type_name() -> &'static str {
        "Cullable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance() {
        let plane = Plane::new(Vec3::new(0.0, 1.0, 0.0), -5.0);
        
        // Point above plane
        assert!(plane.distance_to_point(Vec3::new(0.0, 10.0, 0.0)) > 0.0);
        
        // Point below plane
        assert!(plane.distance_to_point(Vec3::new(0.0, 0.0, 0.0)) < 0.0);
        
        // Point on plane
        let dist = plane.distance_to_point(Vec3::new(0.0, 5.0, 0.0));
        assert!(dist.abs() < 0.001);
    }

    #[test]
    fn test_plane_aabb_intersection() {
        let plane = Plane::new(Vec3::new(0.0, 1.0, 0.0), -5.0);
        
        // AABB above plane (visible)
        let aabb_above = AABB::new(Vec3::new(-1.0, 6.0, -1.0), Vec3::new(1.0, 8.0, 1.0));
        assert!(plane.intersects_aabb(&aabb_above));
        
        // AABB below plane (not visible)
        let aabb_below = AABB::new(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 2.0, 1.0));
        assert!(!plane.intersects_aabb(&aabb_below));
        
        // AABB intersecting plane (visible)
        let aabb_intersect = AABB::new(Vec3::new(-1.0, 4.0, -1.0), Vec3::new(1.0, 6.0, 1.0));
        assert!(plane.intersects_aabb(&aabb_intersect));
    }

    #[test]
    fn test_frustum_extraction() {
        // Create a simple perspective projection
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;
        
        let frustum = Frustum::from_view_projection(&view_proj);
        
        // Should have 6 planes
        assert_eq!(frustum.planes.len(), 6);
        
        // AABB at origin should be visible
        let aabb_center = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(frustum.intersects_aabb(&aabb_center));
        
        // AABB far behind camera should not be visible
        let aabb_behind = AABB::new(Vec3::new(-1.0, -1.0, 10.0), Vec3::new(1.0, 1.0, 12.0));
        assert!(!frustum.intersects_aabb(&aabb_behind));
    }

    #[test]
    fn test_bvh_build() {
        let entities = vec![
            (AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)), 0),
            (AABB::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0)), 1),
            (AABB::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)), 2),
        ];
        
        let bvh = BVHNode::build(&entities, 1);
        
        // Root should contain all entities
        assert!(bvh.aabb.min.x <= 0.0);
        assert!(bvh.aabb.max.x >= 11.0);
    }

    #[test]
    fn test_bvh_query() {
        let entities = vec![
            (AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)), 0),
            (AABB::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0)), 1),
            (AABB::new(Vec3::new(100.0, 0.0, 0.0), Vec3::new(101.0, 1.0, 1.0)), 2),
        ];
        
        let bvh = BVHNode::build(&entities, 1);
        
        // Create frustum looking at origin
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 50.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));
        
        let mut visible = Vec::new();
        bvh.query(&frustum, &mut visible);
        
        // Should see entities 0 and 1, but not 2 (too far)
        assert!(visible.contains(&0));
        assert!(!visible.contains(&2));
    }

    #[test]
    fn test_transform_aabb() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let transform = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        
        let transformed = transform_aabb(&aabb, &transform);
        
        // Should be translated
        assert!((transformed.center().x - 5.0).abs() < 0.001);
        assert!(transformed.center().y.abs() < 0.001);
        assert!(transformed.center().z.abs() < 0.001);
    }
}
