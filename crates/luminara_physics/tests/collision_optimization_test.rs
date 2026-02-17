//! Tests for collision detection optimization
//!
//! This test suite verifies that spatial acceleration structures
//! (BVH and Octree) correctly detect collisions and provide
//! performance improvements over naive O(n²) approaches.
//!
//! **Validates: Requirements 20.7, 26.1**

use luminara_math::Vec3;
use luminara_physics::spatial_acceleration::{AABB, BVH, Octree};
use std::collections::HashSet;
use std::time::Instant;

/// Generate a grid of AABBs for testing
fn generate_grid_aabbs(grid_size: usize, spacing: f32) -> Vec<(usize, AABB)> {
    let mut aabbs = Vec::new();
    let mut id = 0;

    for x in 0..grid_size {
        for y in 0..grid_size {
            for z in 0..grid_size {
                let center = Vec3::new(
                    x as f32 * spacing,
                    y as f32 * spacing,
                    z as f32 * spacing,
                );
                let half_extents = Vec3::new(0.4, 0.4, 0.4);
                aabbs.push((id, AABB::from_center_half_extents(center, half_extents)));
                id += 1;
            }
        }
    }

    aabbs
}

/// Naive O(n²) collision detection for comparison
fn naive_collision_pairs(aabbs: &[(usize, AABB)]) -> HashSet<(usize, usize)> {
    let mut pairs = HashSet::new();

    for i in 0..aabbs.len() {
        for j in (i + 1)..aabbs.len() {
            if aabbs[i].1.intersects(&aabbs[j].1) {
                let pair = if aabbs[i].0 < aabbs[j].0 {
                    (aabbs[i].0, aabbs[j].0)
                } else {
                    (aabbs[j].0, aabbs[i].0)
                };
                pairs.insert(pair);
            }
        }
    }

    pairs
}

#[test]
fn test_bvh_correctness() {
    // Generate test data
    let aabbs = generate_grid_aabbs(5, 2.0);

    // Build BVH
    let mut bvh = BVH::new();
    bvh.build(aabbs.clone());

    // Get collision pairs from BVH
    let bvh_pairs: HashSet<(usize, usize)> = bvh
        .find_collision_pairs()
        .into_iter()
        .map(|(a, b)| if a < b { (a, b) } else { (b, a) })
        .collect();

    // Get collision pairs from naive approach
    let naive_pairs = naive_collision_pairs(&aabbs);

    // Verify BVH finds all collisions
    assert_eq!(
        bvh_pairs, naive_pairs,
        "BVH should find the same collision pairs as naive approach"
    );

    println!("BVH correctness test passed: {} collision pairs found", bvh_pairs.len());
}

#[test]
fn test_octree_correctness() {
    // Generate test data
    let aabbs = generate_grid_aabbs(5, 2.0);

    // Build Octree
    let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(20.0, 20.0, 20.0));
    let mut octree = Octree::new(bounds, 6, 8);

    for (id, aabb) in &aabbs {
        octree.insert(*id, *aabb);
    }

    // Get collision pairs using octree queries
    let mut octree_pairs = HashSet::new();
    for (id_a, aabb_a) in &aabbs {
        let candidates = octree.query(aabb_a);
        for id_b in candidates {
            if *id_a < id_b {
                octree_pairs.insert((*id_a, id_b));
            }
        }
    }

    // Get collision pairs from naive approach
    let naive_pairs = naive_collision_pairs(&aabbs);

    // Verify Octree finds all collisions
    assert_eq!(
        octree_pairs, naive_pairs,
        "Octree should find the same collision pairs as naive approach"
    );

    println!("Octree correctness test passed: {} collision pairs found", octree_pairs.len());
}

#[test]
fn test_bvh_performance_improvement() {
    // Generate larger dataset
    let aabbs = generate_grid_aabbs(10, 2.0); // 1000 objects
    println!("\nTesting with {} objects", aabbs.len());

    // Measure naive approach
    let start = Instant::now();
    let naive_pairs = naive_collision_pairs(&aabbs);
    let naive_time = start.elapsed();
    println!("Naive O(n²) approach: {:?} ({} pairs)", naive_time, naive_pairs.len());

    // Measure BVH approach
    let start = Instant::now();
    let mut bvh = BVH::new();
    bvh.build(aabbs.clone());
    let build_time = start.elapsed();

    let start = Instant::now();
    let bvh_pairs: HashSet<(usize, usize)> = bvh
        .find_collision_pairs()
        .into_iter()
        .map(|(a, b)| if a < b { (a, b) } else { (b, a) })
        .collect();
    let query_time = start.elapsed();
    let bvh_total_time = build_time + query_time;

    println!("BVH approach: {:?} (build: {:?}, query: {:?}, {} pairs)",
        bvh_total_time, build_time, query_time, bvh_pairs.len());

    // Verify correctness
    assert_eq!(bvh_pairs, naive_pairs, "BVH should find same pairs as naive");

    // Calculate speedup
    let speedup = naive_time.as_secs_f64() / bvh_total_time.as_secs_f64();
    println!("BVH speedup: {:.2}x", speedup);

    // BVH should be faster for 1000 objects
    assert!(
        bvh_total_time < naive_time,
        "BVH should be faster than naive approach for 1000 objects"
    );
}

#[test]
fn test_octree_performance_improvement() {
    // Generate larger dataset
    let aabbs = generate_grid_aabbs(10, 2.0); // 1000 objects
    println!("\nTesting with {} objects", aabbs.len());

    // Measure naive approach
    let start = Instant::now();
    let naive_pairs = naive_collision_pairs(&aabbs);
    let naive_time = start.elapsed();
    println!("Naive O(n²) approach: {:?} ({} pairs)", naive_time, naive_pairs.len());

    // Measure Octree approach
    let start = Instant::now();
    let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(30.0, 30.0, 30.0));
    let mut octree = Octree::new(bounds, 6, 8);

    for (id, aabb) in &aabbs {
        octree.insert(*id, *aabb);
    }
    let build_time = start.elapsed();

    let start = Instant::now();
    let mut octree_pairs = HashSet::new();
    for (id_a, aabb_a) in &aabbs {
        let candidates = octree.query(aabb_a);
        for id_b in candidates {
            if *id_a < id_b {
                octree_pairs.insert((*id_a, id_b));
            }
        }
    }
    let query_time = start.elapsed();
    let octree_total_time = build_time + query_time;

    println!("Octree approach: {:?} (build: {:?}, query: {:?}, {} pairs)",
        octree_total_time, build_time, query_time, octree_pairs.len());

    // Verify correctness
    assert_eq!(octree_pairs, naive_pairs, "Octree should find same pairs as naive");

    // Calculate speedup
    let speedup = naive_time.as_secs_f64() / octree_total_time.as_secs_f64();
    println!("Octree speedup: {:.2}x", speedup);

    // Octree should be faster for 1000 objects
    assert!(
        octree_total_time < naive_time,
        "Octree should be faster than naive approach for 1000 objects"
    );
}

#[test]
fn test_bvh_query() {
    // Generate test data
    let aabbs = generate_grid_aabbs(10, 2.0);

    // Build BVH
    let mut bvh = BVH::new();
    bvh.build(aabbs.clone());

    // Query for objects near origin
    let query_aabb = AABB::from_center_half_extents(Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0));
    let results = bvh.query(&query_aabb);

    // Verify results manually
    let mut expected = Vec::new();
    for (id, aabb) in &aabbs {
        if aabb.intersects(&query_aabb) {
            expected.push(*id);
        }
    }

    assert_eq!(
        results.len(),
        expected.len(),
        "BVH query should find all intersecting objects"
    );

    println!("BVH query test passed: {} objects found", results.len());
}

#[test]
fn test_octree_query() {
    // Generate test data
    let aabbs = generate_grid_aabbs(10, 2.0);

    // Build Octree
    let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(30.0, 30.0, 30.0));
    let mut octree = Octree::new(bounds, 6, 8);

    for (id, aabb) in &aabbs {
        octree.insert(*id, *aabb);
    }

    // Query for objects near origin
    let query_aabb = AABB::from_center_half_extents(Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0));
    let results = octree.query(&query_aabb);

    // Verify results manually
    let mut expected = Vec::new();
    for (id, aabb) in &aabbs {
        if aabb.intersects(&query_aabb) {
            expected.push(*id);
        }
    }

    assert_eq!(
        results.len(),
        expected.len(),
        "Octree query should find all intersecting objects"
    );

    println!("Octree query test passed: {} objects found", results.len());
}

#[test]
fn test_large_scale_performance() {
    // Test with 5000 objects to verify scalability
    let aabbs = generate_grid_aabbs(17, 2.0); // ~5000 objects
    println!("\nLarge scale test with {} objects", aabbs.len());

    // BVH approach
    let start = Instant::now();
    let mut bvh = BVH::new();
    bvh.build(aabbs.clone());
    let bvh_pairs = bvh.find_collision_pairs();
    let bvh_time = start.elapsed();

    println!("BVH: {:?} ({} pairs)", bvh_time, bvh_pairs.len());

    // Octree approach
    let start = Instant::now();
    let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(50.0, 50.0, 50.0));
    let mut octree = Octree::new(bounds, 8, 8);

    for (id, aabb) in &aabbs {
        octree.insert(*id, *aabb);
    }

    let mut octree_pairs = HashSet::new();
    for (id_a, aabb_a) in &aabbs {
        let candidates = octree.query(aabb_a);
        for id_b in candidates {
            if *id_a < id_b {
                octree_pairs.insert((*id_a, id_b));
            }
        }
    }
    let octree_time = start.elapsed();

    println!("Octree: {:?} ({} pairs)", octree_time, octree_pairs.len());

    // Both should complete in reasonable time (< 1 second for 5000 objects)
    assert!(
        bvh_time.as_secs() < 1,
        "BVH should handle 5000 objects in under 1 second"
    );
    assert!(
        octree_time.as_secs() < 1,
        "Octree should handle 5000 objects in under 1 second"
    );

    println!("Large scale performance test passed!");
}
