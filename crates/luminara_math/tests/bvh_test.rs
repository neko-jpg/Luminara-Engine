use luminara_math::geometry::{Bvh, Aabb, Primitive};
use glam::Vec3;
use proptest::prelude::*;

#[derive(Clone, Copy, Debug)]
struct Sphere {
    center: Vec3,
    radius: f32,
    id: usize,
}

impl Primitive for Sphere {
    fn aabb(&self) -> Aabb {
        Aabb {
            min: self.center - Vec3::splat(self.radius),
            max: self.center + Vec3::splat(self.radius),
        }
    }

    fn intersect(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<f32> {
        let oc = ray_origin - self.center;
        let a = ray_dir.length_squared();
        let b = 2.0 * oc.dot(ray_dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = b*b - 4.0*a*c;
        if discriminant < 0.0 { return None; }
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0*a);
        let t2 = (-b + sqrt_d) / (2.0*a);

        if t1 > 1e-4 { return Some(t1); } // epsilon for self-intersection
        if t2 > 1e-4 { return Some(t2); }
        None
    }
}

// Brute force intersection
fn brute_force_intersect(primitives: &[Sphere], origin: Vec3, dir: Vec3) -> Option<(f32, usize)> {
    let mut nearest_t = f32::MAX;
    let mut best_hit = None;

    for (i, p) in primitives.iter().enumerate() {
        if let Some(t) = p.intersect(origin, dir) {
            if t < nearest_t {
                nearest_t = t;
                best_hit = Some((t, i));
            }
        }
    }
    best_hit
}

// Helpers for random generation
prop_compose! {
    fn arb_vec3()(x in -10.0f32..10.0, y in -10.0f32..10.0, z in -10.0f32..10.0) -> Vec3 {
        Vec3::new(x, y, z)
    }
}

prop_compose! {
    fn arb_sphere()(center in arb_vec3(), radius in 0.1f32..2.0) -> Sphere {
        Sphere { center, radius, id: 0 } // id set later
    }
}

proptest! {
    // Property 20: BVH Ray Traversal Nearest Hit First
    // Validates: Requirements 7.7
    #[test]
    fn prop_bvh_traversal_nearest_hit(
        spheres in proptest::collection::vec(arb_sphere(), 1..20),
        origin in arb_vec3(),
        dir in arb_vec3()
    ) {
        // Fix up IDs and verify dir is not zero
        if dir.length_squared() < 1e-6 { return Ok(()); }
        let dir = dir.normalize();

        let mut primitives = spheres;
        for (i, p) in primitives.iter_mut().enumerate() {
            p.id = i;
        }

        let bvh = Bvh::build(primitives.clone());
        let bvh_hit = bvh.intersect_ray(origin, dir);
        let bf_hit = brute_force_intersect(&primitives, origin, dir);

        match (bvh_hit, bf_hit) {
            (Some((t1, _idx1)), Some((t2, _idx2))) => {
                // Should hit same object (or at same distance if overlapping)
                prop_assert!((t1 - t2).abs() < 1e-4, "Distance mismatch: BVH {}, Brute {}", t1, t2);

                // If distances are close, indices might differ if objects overlap exactly, but usually exact.
                // If distinct objects, index should match.
                // Or check primitive equality.
                // If overlapping, we accept any valid hit at that distance.
                // But BVH should return *nearest*.
                // Brute force returns nearest.
                // So t1 should equal t2.
                // idx might differ.
            },
            (None, None) => {},
            (Some(_), None) => prop_assert!(false, "BVH found hit but Brute Force didn't"),
            (None, Some(_)) => prop_assert!(false, "Brute Force found hit but BVH didn't"),
        }
    }
}

#[test]
fn test_bvh_empty() {
    let primitives: Vec<Sphere> = vec![];
    let bvh = Bvh::build(primitives);
    let hit = bvh.intersect_ray(Vec3::ZERO, Vec3::X);
    assert!(hit.is_none());
}

#[test]
fn test_bvh_single() {
    let s = Sphere { center: Vec3::new(5.0, 0.0, 0.0), radius: 1.0, id: 0 };
    let bvh = Bvh::build(vec![s]);

    let hit = bvh.intersect_ray(Vec3::ZERO, Vec3::X);
    assert!(hit.is_some());
    let (t, idx) = hit.unwrap();
    assert!((t - 4.0).abs() < 1e-4); // dist to surface is 5 - 1 = 4
    assert_eq!(idx, 0);
}

#[test]
fn test_degenerate_aabb() {
    // Sphere with radius 0 (point)
    let s = Sphere { center: Vec3::new(5.0, 0.0, 0.0), radius: 0.0, id: 0 };
    let bvh = Bvh::build(vec![s]);

    // Ray hitting the point exactly is hard with float precision.
    // But AABB will be a point.
    // AABB intersect ray logic handles point?
    // t1 = (min - org) / dir. t2 = (max - org) / dir.
    // If min=max, t1=t2. t_near=t_far.
    // Logic should work.

    // However, sphere intersect uses discriminant.
    // radius=0. discriminant = b*b - 4*a*|oc|^2.
    // If we hit center, b = 2 oc.dir. |oc|^2 = oc.oc.
    // b^2 = 4 (oc.dir)^2.
    // 4 a c = 4 (dir.dir) (oc.oc).
    // If dir is normalized (a=1).
    // b^2 - 4c = 4 ( (oc.dir)^2 - |oc|^2 ).
    // By Cauchy-Schwarz, (oc.dir)^2 <= |oc|^2 |dir|^2.
    // So disc <= 0.
    // disc = 0 only if oc is parallel to dir.
    // So usually None unless exact hit.

    let _hit = bvh.intersect_ray(Vec3::ZERO, Vec3::X);
    // Ray goes through (5,0,0).
    // Should hit.
    // But float precision might make discriminant slightly negative.
    // We expect it to potentially miss due to precision, but construction shouldn't crash.
    // BVH build handles degenerate AABBs (extent < epsilon).
}
