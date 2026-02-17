//! Spatial acceleration structures for optimized collision detection
//!
//! This module provides spatial partitioning structures (BVH and Octree)
//! to optimize broad-phase collision detection. These structures reduce
//! collision checks from O(n²) to O(n log n) by organizing colliders
//! spatially and quickly culling non-overlapping pairs.
//!
//! **Validates: Requirements 20.7, 26.1**

use luminara_math::Vec3;

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB from min and max points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center point and half extents
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if this AABB intersects another
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the surface area of the AABB
    pub fn surface_area(&self) -> f32 {
        let extent = self.max - self.min;
        2.0 * (extent.x * extent.y + extent.y * extent.z + extent.z * extent.x)
    }

    /// Merge this AABB with another
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Check if this AABB contains a point
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }
}

/// Bounding Volume Hierarchy node
#[derive(Debug)]
enum BVHNode<T> {
    Leaf {
        aabb: AABB,
        data: Vec<(T, AABB)>,
    },
    Internal {
        aabb: AABB,
        left: Box<BVHNode<T>>,
        right: Box<BVHNode<T>>,
    },
}

/// Bounding Volume Hierarchy for efficient collision detection
///
/// BVH organizes objects in a tree structure where each node contains
/// an AABB that bounds all objects in its subtree. This allows for
/// efficient culling of non-overlapping pairs.
pub struct BVH<T> {
    root: Option<Box<BVHNode<T>>>,
    max_leaf_size: usize,
}

impl<T: Clone> BVH<T> {
    /// Create a new empty BVH
    pub fn new() -> Self {
        Self {
            root: None,
            max_leaf_size: 8,
        }
    }

    /// Build the BVH from a list of objects and their AABBs
    pub fn build(&mut self, objects: Vec<(T, AABB)>) {
        if objects.is_empty() {
            self.root = None;
            return;
        }

        self.root = Some(Box::new(self.build_recursive(objects)));
    }

    /// Recursively build the BVH tree
    fn build_recursive(&self, mut objects: Vec<(T, AABB)>) -> BVHNode<T> {
        if objects.len() <= self.max_leaf_size {
            // Create leaf node
            let aabb = objects
                .iter()
                .map(|(_, aabb)| *aabb)
                .reduce(|a, b| a.merge(&b))
                .unwrap();

            return BVHNode::Leaf {
                aabb,
                data: objects,
            };
        }

        // Calculate bounding box for all objects
        let total_aabb = objects
            .iter()
            .map(|(_, aabb)| *aabb)
            .reduce(|a, b| a.merge(&b))
            .unwrap();

        // Find the longest axis
        let extent = total_aabb.max - total_aabb.min;
        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        // Sort objects along the longest axis
        objects.sort_by(|(_, a), (_, b)| {
            let a_center = a.center();
            let b_center = b.center();
            let a_val = match axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };
            let b_val = match axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };
            a_val.partial_cmp(&b_val).unwrap()
        });

        // Split objects in half
        let mid = objects.len() / 2;
        let right_objects = objects.split_off(mid);
        let left_objects = objects;

        // Recursively build left and right subtrees
        let left = Box::new(self.build_recursive(left_objects));
        let right = Box::new(self.build_recursive(right_objects));

        BVHNode::Internal {
            aabb: total_aabb,
            left,
            right,
        }
    }

    /// Query the BVH for all objects that intersect with the given AABB
    pub fn query(&self, query_aabb: &AABB) -> Vec<T> {
        let mut results = Vec::new();

        if let Some(root) = &self.root {
            self.query_recursive(root, query_aabb, &mut results);
        }

        results
    }

    /// Recursively query the BVH tree
    fn query_recursive(&self, node: &BVHNode<T>, query_aabb: &AABB, results: &mut Vec<T>) {
        match node {
            BVHNode::Leaf { aabb, data } => {
                if aabb.intersects(query_aabb) {
                    for (obj, obj_aabb) in data {
                        if obj_aabb.intersects(query_aabb) {
                            results.push(obj.clone());
                        }
                    }
                }
            }
            BVHNode::Internal { aabb, left, right } => {
                if aabb.intersects(query_aabb) {
                    self.query_recursive(left, query_aabb, results);
                    self.query_recursive(right, query_aabb, results);
                }
            }
        }
    }

    /// Find all pairs of potentially colliding objects
    pub fn find_collision_pairs(&self) -> Vec<(T, T)>
    where
        T: PartialEq + Clone,
    {
        let mut pairs = Vec::new();

        if let Some(root) = &self.root {
            self.find_pairs_self_test(root, &mut pairs);
        }

        pairs
    }

    /// Find collision pairs within a single node's subtree
    fn find_pairs_self_test(&self, node: &BVHNode<T>, pairs: &mut Vec<(T, T)>)
    where
        T: PartialEq + Clone,
    {
        match node {
            BVHNode::Leaf { data, .. } => {
                // Check all pairs within this leaf
                for i in 0..data.len() {
                    for j in (i + 1)..data.len() {
                        if data[i].1.intersects(&data[j].1) {
                            pairs.push((data[i].0.clone(), data[j].0.clone()));
                        }
                    }
                }
            }
            BVHNode::Internal { left, right, .. } => {
                // Recursively find pairs in left and right subtrees
                self.find_pairs_self_test(left, pairs);
                self.find_pairs_self_test(right, pairs);

                // Find pairs between left and right subtrees
                self.find_pairs_between(left, right, pairs);
            }
        }
    }

    /// Find collision pairs between two different subtrees
    fn find_pairs_between(&self, node_a: &BVHNode<T>, node_b: &BVHNode<T>, pairs: &mut Vec<(T, T)>)
    where
        T: Clone,
    {
        // Get AABBs for both nodes
        let aabb_a = match node_a {
            BVHNode::Leaf { aabb, .. } => aabb,
            BVHNode::Internal { aabb, .. } => aabb,
        };
        let aabb_b = match node_b {
            BVHNode::Leaf { aabb, .. } => aabb,
            BVHNode::Internal { aabb, .. } => aabb,
        };

        // Early out if AABBs don't intersect
        if !aabb_a.intersects(aabb_b) {
            return;
        }

        match (node_a, node_b) {
            (BVHNode::Leaf { data: data_a, .. }, BVHNode::Leaf { data: data_b, .. }) => {
                // Both are leaves, check all pairs
                for (obj_a, aabb_a) in data_a {
                    for (obj_b, aabb_b) in data_b {
                        if aabb_a.intersects(aabb_b) {
                            pairs.push((obj_a.clone(), obj_b.clone()));
                        }
                    }
                }
            }
            (BVHNode::Internal { left: left_a, right: right_a, .. }, node_b) => {
                // node_a is internal, recurse on its children
                self.find_pairs_between(left_a, node_b, pairs);
                self.find_pairs_between(right_a, node_b, pairs);
            }
            (node_a, BVHNode::Internal { left: left_b, right: right_b, .. }) => {
                // node_b is internal, recurse on its children
                self.find_pairs_between(node_a, left_b, pairs);
                self.find_pairs_between(node_a, right_b, pairs);
            }
        }
    }
}

impl<T: Clone> Default for BVH<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Octree node for spatial partitioning
#[derive(Debug)]
struct OctreeNode<T> {
    aabb: AABB,
    objects: Vec<(T, AABB)>,
    children: Option<Box<[OctreeNode<T>; 8]>>,
}

/// Octree for spatial partitioning
///
/// Octree divides 3D space into eight octants recursively, allowing
/// for efficient spatial queries and collision detection.
pub struct Octree<T> {
    root: Option<OctreeNode<T>>,
    max_depth: usize,
    max_objects_per_node: usize,
}

impl<T: Clone> Octree<T> {
    /// Create a new octree with the given bounds
    pub fn new(bounds: AABB, max_depth: usize, max_objects_per_node: usize) -> Self {
        Self {
            root: Some(OctreeNode {
                aabb: bounds,
                objects: Vec::new(),
                children: None,
            }),
            max_depth,
            max_objects_per_node,
        }
    }

    /// Insert an object into the octree
    pub fn insert(&mut self, object: T, aabb: AABB) {
        if let Some(root) = &mut self.root {
            Self::insert_recursive(root, object, aabb, 0, self.max_depth, self.max_objects_per_node);
        }
    }

    /// Recursively insert an object into the octree
    fn insert_recursive(
        node: &mut OctreeNode<T>,
        object: T,
        aabb: AABB,
        depth: usize,
        max_depth: usize,
        max_objects: usize,
    ) {
        // If this node has children, try to insert into appropriate child
        if let Some(children) = &mut node.children {
            let center = node.aabb.center();
            let octant = Self::get_octant(&center, &aabb.center());
            Self::insert_recursive(&mut children[octant], object, aabb, depth + 1, max_depth, max_objects);
            return;
        }

        // Add object to this node
        node.objects.push((object, aabb));

        // Subdivide if necessary
        if node.objects.len() > max_objects && depth < max_depth {
            Self::subdivide(node, depth, max_depth, max_objects);
        }
    }

    /// Subdivide a node into 8 children
    fn subdivide(node: &mut OctreeNode<T>, depth: usize, max_depth: usize, max_objects: usize) {
        let center = node.aabb.center();
        let _half_extent = (node.aabb.max - node.aabb.min) * 0.5;

        // Create 8 child nodes
        let mut children = Box::new([
            OctreeNode { aabb: AABB::new(node.aabb.min, center), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(center.x, node.aabb.min.y, node.aabb.min.z), Vec3::new(node.aabb.max.x, center.y, center.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(node.aabb.min.x, center.y, node.aabb.min.z), Vec3::new(center.x, node.aabb.max.y, center.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(center.x, center.y, node.aabb.min.z), Vec3::new(node.aabb.max.x, node.aabb.max.y, center.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(node.aabb.min.x, node.aabb.min.y, center.z), Vec3::new(center.x, center.y, node.aabb.max.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(center.x, node.aabb.min.y, center.z), Vec3::new(node.aabb.max.x, center.y, node.aabb.max.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(Vec3::new(node.aabb.min.x, center.y, center.z), Vec3::new(center.x, node.aabb.max.y, node.aabb.max.z)), objects: Vec::new(), children: None },
            OctreeNode { aabb: AABB::new(center, node.aabb.max), objects: Vec::new(), children: None },
        ]);

        // Redistribute objects to children
        for (obj, obj_aabb) in node.objects.drain(..) {
            let octant = Self::get_octant(&center, &obj_aabb.center());
            Self::insert_recursive(&mut children[octant], obj, obj_aabb, depth + 1, max_depth, max_objects);
        }

        node.children = Some(children);
    }

    /// Get the octant index for a point relative to a center
    fn get_octant(center: &Vec3, point: &Vec3) -> usize {
        let mut octant = 0;
        if point.x >= center.x {
            octant |= 1;
        }
        if point.y >= center.y {
            octant |= 2;
        }
        if point.z >= center.z {
            octant |= 4;
        }
        octant
    }

    /// Query the octree for all objects that intersect with the given AABB
    pub fn query(&self, query_aabb: &AABB) -> Vec<T> {
        let mut results = Vec::new();

        if let Some(root) = &self.root {
            Self::query_recursive(root, query_aabb, &mut results);
        }

        results
    }

    /// Recursively query the octree
    fn query_recursive(node: &OctreeNode<T>, query_aabb: &AABB, results: &mut Vec<T>) {
        if !node.aabb.intersects(query_aabb) {
            return;
        }

        // Add objects from this node
        for (obj, obj_aabb) in &node.objects {
            if obj_aabb.intersects(query_aabb) {
                results.push(obj.clone());
            }
        }

        // Recursively query children
        if let Some(children) = &node.children {
            for child in children.iter() {
                Self::query_recursive(child, query_aabb, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let aabb2 = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let aabb3 = AABB::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(aabb2.intersects(&aabb1));
        assert!(!aabb1.intersects(&aabb3));
        assert!(!aabb3.intersects(&aabb1));
    }

    #[test]
    fn test_aabb_merge() {
        let aabb1 = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let aabb2 = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(2.0, 2.0, 2.0));
        let merged = aabb1.merge(&aabb2);

        assert_eq!(merged.min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(merged.max, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_bvh_build_and_query() {
        let mut bvh = BVH::new();

        let objects = vec![
            (0, AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0))),
            (1, AABB::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0))),
            (2, AABB::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(1.0, 3.0, 1.0))),
        ];

        bvh.build(objects);

        // Query for objects near origin
        let query_aabb = AABB::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.5, 1.5, 1.5));
        let results = bvh.query(&query_aabb);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }

    #[test]
    fn test_octree_insert_and_query() {
        let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let mut octree = Octree::new(bounds, 4, 4);

        octree.insert(0, AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)));
        octree.insert(1, AABB::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0)));
        octree.insert(2, AABB::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(1.0, 3.0, 1.0)));

        // Query for objects near origin
        let query_aabb = AABB::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.5, 1.5, 1.5));
        let results = octree.query(&query_aabb);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }
}
