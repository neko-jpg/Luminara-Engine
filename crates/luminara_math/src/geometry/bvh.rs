//! Bounding Volume Hierarchy with Surface Area Heuristic.
//!
//! Provides efficient spatial acceleration structure for ray tracing and collision queries.

use glam::Vec3;

/// Axis-Aligned Bounding Box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB from min and max points.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an empty AABB (invalid bounds).
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Compute the union of two AABBs.
    pub fn grow(&self, other: &Aabb) -> Aabb {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Grow the AABB to include a point.
    pub fn grow_point(&self, p: Vec3) -> Aabb {
        Self {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    /// Compute the surface area of the AABB.
    pub fn surface_area(&self) -> f32 {
        let d = (self.max - self.min).max(Vec3::ZERO);
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    /// Check if the AABB contains a point.
    pub fn contains(&self, p: Vec3) -> bool {
        p.x >= self.min.x && p.x <= self.max.x &&
        p.y >= self.min.y && p.y <= self.max.y &&
        p.z >= self.min.z && p.z <= self.max.z
    }

    /// Intersect with a ray.
    /// Returns distance to entry point, or None if no intersection.
    /// t_min and t_max define the valid range along the ray.
    pub fn intersect_ray(&self, origin: Vec3, dir_inv: Vec3, t_min: f32, t_max: f32) -> Option<f32> {
        let t1 = (self.min - origin) * dir_inv;
        let t2 = (self.max - origin) * dir_inv;

        let t_near = t1.min(t2);
        let t_far = t1.max(t2);

        let t_enter = t_near.max_element();
        let t_exit = t_far.min_element();

        // Check against provided range
        let t_enter = t_enter.max(t_min);
        let t_exit = t_exit.min(t_max);

        if t_enter <= t_exit {
            Some(t_enter)
        } else {
            None
        }
    }
}

/// Trait for primitives stored in the BVH.
pub trait Primitive: Send + Sync {
    fn aabb(&self) -> Aabb;

    fn center(&self) -> Vec3 {
        let b = self.aabb();
        (b.min + b.max) * 0.5
    }

    /// Intersect with a ray. Returns distance t.
    fn intersect(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<f32>;
}

/// A node in the BVH.
#[derive(Debug)]
pub enum BvhNode {
    Internal {
        aabb: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
    Leaf {
        aabb: Aabb,
        primitives: Vec<usize>, // Indices into the original list
    },
}

impl BvhNode {
    pub fn aabb(&self) -> Aabb {
        match self {
            BvhNode::Internal { aabb, .. } => *aabb,
            BvhNode::Leaf { aabb, .. } => *aabb,
        }
    }
}

/// The BVH structure.
pub struct Bvh<T: Primitive> {
    pub root: BvhNode,
    pub primitives: Vec<T>,
}

impl<T: Primitive> Bvh<T> {
    /// Build a BVH from a list of primitives.
    pub fn build(primitives: Vec<T>) -> Self {
        if primitives.is_empty() {
            return Self {
                root: BvhNode::Leaf { aabb: Aabb::empty(), primitives: vec![] },
                primitives,
            };
        }

        let mut indices: Vec<usize> = (0..primitives.len()).collect();
        let root = Self::build_recursive(&primitives, &mut indices);

        Self {
            root,
            primitives,
        }
    }

    fn build_recursive(primitives: &[T], indices: &mut [usize]) -> BvhNode {
        // Compute AABB for current node
        let mut aabb = Aabb::empty();
        for &idx in indices.iter() {
            aabb = aabb.grow(&primitives[idx].aabb());
        }

        let count = indices.len();
        if count <= 4 { // Small enough to be a leaf
            return BvhNode::Leaf {
                aabb,
                primitives: indices.to_vec(),
            };
        }

        // Surface Area Heuristic
        // Binning approach
        // We evaluate split costs for each axis

        let mut best_cost = f32::INFINITY;
        let mut best_axis = 0;
        let mut best_split_val = 0.0;

        let centroid_bounds = indices.iter().fold(Aabb::empty(), |acc, &idx| {
            acc.grow_point(primitives[idx].center())
        });

        // If centroids are condensed (point), we can't split
        let extent = centroid_bounds.max - centroid_bounds.min;
        if extent.max_element() < 1e-6 {
             return BvhNode::Leaf {
                aabb,
                primitives: indices.to_vec(),
            };
        }

        // Try splitting along each axis
        for axis in 0..3 {
            let axis_len = extent[axis];
            if axis_len < 1e-6 { continue; }

            // Bin centroids
            const NUM_BINS: usize = 12;
            let mut bins = [Aabb::empty(); NUM_BINS];
            let mut bin_counts = [0; NUM_BINS];

            let axis_min = centroid_bounds.min[axis];
            let scale = (NUM_BINS as f32) / axis_len;

            for &idx in indices.iter() {
                let p = primitives[idx].center();
                let bin_idx = ((p[axis] - axis_min) * scale).floor() as usize;
                let bin_idx = bin_idx.min(NUM_BINS - 1);

                bins[bin_idx] = bins[bin_idx].grow(&primitives[idx].aabb());
                bin_counts[bin_idx] += 1;
            }

            // Sweep to find best split plane
            // Split after bin i
            let mut left_area = [0.0; NUM_BINS - 1];
            let mut left_count = [0; NUM_BINS - 1];
            let mut acc_aabb = Aabb::empty();
            let mut acc_count = 0;

            for i in 0..NUM_BINS - 1 {
                acc_aabb = acc_aabb.grow(&bins[i]);
                acc_count += bin_counts[i];
                left_area[i] = acc_aabb.surface_area();
                left_count[i] = acc_count;
            }

            acc_aabb = Aabb::empty();
            acc_count = 0;

            for i in (0..NUM_BINS - 1).rev() {
                acc_aabb = acc_aabb.grow(&bins[i + 1]);
                acc_count += bin_counts[i + 1];

                let count_left = left_count[i];
                let count_right = acc_count;

                if count_left == 0 || count_right == 0 { continue; }

                let area_left = left_area[i];
                let area_right = acc_aabb.surface_area();

                let cost = area_left * (count_left as f32) + area_right * (count_right as f32);

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    // Split is after bin i.
                    // Split value is bin boundary.
                    best_split_val = axis_min + ((i + 1) as f32) / scale;
                }
            }
        }

        // Leaf cost
        let leaf_cost = aabb.surface_area() * (count as f32);

        // If split is not beneficial, make leaf
        if best_cost >= leaf_cost {
             return BvhNode::Leaf {
                aabb,
                primitives: indices.to_vec(),
            };
        }

        // Partition indices based on best split
        // Note: Partitioning logic must match binning logic approximately
        // Actually, we should just partition using the split value.
        // But re-partitioning in place is tricky with binning approx.
        // We use Hoare partition logic.

        let split_idx = partition(indices, |idx| {
            primitives[*idx].center()[best_axis] < best_split_val
        });

        if split_idx == 0 || split_idx == count {
             // Fallback if partition failed to split (e.g. all centers same side due to float precision)
             return BvhNode::Leaf {
                aabb,
                primitives: indices.to_vec(),
            };
        }

        let (left_indices, right_indices) = indices.split_at_mut(split_idx);

        let (left, right) = if count > 1024 {
            rayon::join(
                || Box::new(Self::build_recursive(primitives, left_indices)),
                || Box::new(Self::build_recursive(primitives, right_indices)),
            )
        } else {
            (
                Box::new(Self::build_recursive(primitives, left_indices)),
                Box::new(Self::build_recursive(primitives, right_indices)),
            )
        };

        BvhNode::Internal {
            aabb,
            left,
            right,
        }
    }

    /// Traverse the BVH and find the nearest intersection.
    pub fn intersect_ray(&self, origin: Vec3, dir: Vec3) -> Option<(f32, usize)> {
        // Simple recursive traversal
        // TODO: Stack-based optimization

        // Precompute dir_inv
        let dir_inv = Vec3::new(1.0/dir.x, 1.0/dir.y, 1.0/dir.z);

        self.intersect_recursive(&self.root, origin, dir, dir_inv, f32::MAX)
    }

    fn intersect_recursive(&self, node: &BvhNode, origin: Vec3, dir: Vec3, dir_inv: Vec3, mut nearest_t: f32) -> Option<(f32, usize)> {
        // Check AABB intersection
        let aabb = node.aabb();
        let t_aabb = aabb.intersect_ray(origin, dir_inv, 0.0, nearest_t)?;

        if t_aabb > nearest_t {
            return None;
        }

        match node {
            BvhNode::Leaf { primitives: indices, .. } => {
                let mut hit = None;
                for &idx in indices {
                    if let Some(t) = self.primitives[idx].intersect(origin, dir) {
                        if t < nearest_t && t >= 0.0 {
                            nearest_t = t;
                            hit = Some((t, idx));
                        }
                    }
                }
                hit
            }
            BvhNode::Internal { left, right, .. } => {
                // Visit closer child first?
                // Not strictly necessary for correctness, but good for perf.
                // We just visit both.

                // Note: The order matters for early pruning if we update nearest_t.
                // But since we pass nearest_t recursively...
                // We should update nearest_t between calls.

                // Let's check aabb intersection of children to decide order.
                let t_left = left.aabb().intersect_ray(origin, dir_inv, 0.0, nearest_t);
                let t_right = right.aabb().intersect_ray(origin, dir_inv, 0.0, nearest_t);

                let mut best_hit = None;

                // Logic: visit valid ones, closest first.
                let (first, second) = match (t_left, t_right) {
                    (Some(tl), Some(tr)) => if tl < tr { (left, right) } else { (right, left) },
                    (Some(_), None) => (left, right), // right won't be visited anyway
                    (None, Some(_)) => (right, left),
                    (None, None) => return None,
                };

                if let Some(h) = self.intersect_recursive(first, origin, dir, dir_inv, nearest_t) {
                    nearest_t = h.0;
                    best_hit = Some(h);
                }

                // Must check nearest_t again for second child
                if let Some(h) = self.intersect_recursive(second, origin, dir, dir_inv, nearest_t) {
                    // if h.0 < nearest_t { // Recursive call already checks this internally with updated nearest_t?
                    // No, nearest_t passed is the one at call time.
                    // But if recursive call returns Some, it means it found something closer than passed nearest_t.
                    best_hit = Some(h);
                    //}
                }

                best_hit
            }
        }
    }
}

// Helper for partitioning slice
fn partition<T, F>(data: &mut [T], mut predicate: F) -> usize
where
    F: FnMut(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return 0;
    }
    let mut l = 0;
    let mut r = len - 1;
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return l;
        }
        data.swap(l, r);
        l += 1;
        if r > 0 { r -= 1; } // Check bounds
    }
}
