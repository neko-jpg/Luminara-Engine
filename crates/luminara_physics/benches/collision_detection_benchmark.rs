//! Collision detection optimization benchmark
//!
//! This benchmark measures the performance improvements from spatial
//! acceleration structures (BVH and Octree) for collision detection.
//!
//! **Validates: Requirements 20.7, 26.1**

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use luminara_math::Vec3;
use luminara_physics::spatial_acceleration::{AABB, BVH, Octree};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Generate random AABBs for testing
fn generate_random_aabbs(count: usize, seed: u64) -> Vec<(usize, AABB)> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut aabbs = Vec::with_capacity(count);

    for i in 0..count {
        let center = Vec3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
        );
        let half_extents = Vec3::new(
            rng.gen_range(0.5..2.0),
            rng.gen_range(0.5..2.0),
            rng.gen_range(0.5..2.0),
        );

        aabbs.push((i, AABB::from_center_half_extents(center, half_extents)));
    }

    aabbs
}

/// Naive O(n²) collision detection
fn naive_collision_detection(aabbs: &[(usize, AABB)]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();

    for i in 0..aabbs.len() {
        for j in (i + 1)..aabbs.len() {
            if aabbs[i].1.intersects(&aabbs[j].1) {
                pairs.push((aabbs[i].0, aabbs[j].0));
            }
        }
    }

    pairs
}

/// BVH-based collision detection
fn bvh_collision_detection(aabbs: &[(usize, AABB)]) -> Vec<(usize, usize)> {
    let mut bvh = BVH::new();
    bvh.build(aabbs.to_vec());

    let pairs = bvh.find_collision_pairs();
    pairs.into_iter().map(|(a, b)| if a < b { (a, b) } else { (b, a) }).collect()
}

/// Octree-based collision detection
fn octree_collision_detection(aabbs: &[(usize, AABB)]) -> Vec<(usize, usize)> {
    let bounds = AABB::new(Vec3::new(-150.0, -150.0, -150.0), Vec3::new(150.0, 150.0, 150.0));
    let mut octree = Octree::new(bounds, 6, 8);

    for (id, aabb) in aabbs {
        octree.insert(*id, *aabb);
    }

    let mut pairs = Vec::new();

    // Query each object against the octree
    for (id_a, aabb_a) in aabbs {
        let candidates = octree.query(aabb_a);
        for id_b in candidates {
            if *id_a < id_b {
                pairs.push((*id_a, id_b));
            }
        }
    }

    pairs
}

fn benchmark_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_detection");

    // Test with different object counts
    for count in [100, 500, 1000, 2000, 5000].iter() {
        let aabbs = generate_random_aabbs(*count, 42);

        // Naive O(n²) approach
        group.bench_with_input(
            BenchmarkId::new("naive", count),
            &aabbs,
            |b, aabbs| {
                b.iter(|| {
                    let pairs = naive_collision_detection(black_box(aabbs));
                    black_box(pairs);
                });
            },
        );

        // BVH approach
        group.bench_with_input(
            BenchmarkId::new("bvh", count),
            &aabbs,
            |b, aabbs| {
                b.iter(|| {
                    let pairs = bvh_collision_detection(black_box(aabbs));
                    black_box(pairs);
                });
            },
        );

        // Octree approach
        group.bench_with_input(
            BenchmarkId::new("octree", count),
            &aabbs,
            |b, aabbs| {
                b.iter(|| {
                    let pairs = octree_collision_detection(black_box(aabbs));
                    black_box(pairs);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_bvh_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("bvh_build");

    for count in [100, 500, 1000, 2000, 5000].iter() {
        let aabbs = generate_random_aabbs(*count, 42);

        group.bench_with_input(
            BenchmarkId::new("build", count),
            &aabbs,
            |b, aabbs| {
                b.iter(|| {
                    let mut bvh = BVH::new();
                    bvh.build(black_box(aabbs.clone()));
                    black_box(bvh);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_octree_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("octree_build");

    for count in [100, 500, 1000, 2000, 5000].iter() {
        let aabbs = generate_random_aabbs(*count, 42);

        group.bench_with_input(
            BenchmarkId::new("build", count),
            &aabbs,
            |b, aabbs| {
                b.iter(|| {
                    let bounds = AABB::new(Vec3::new(-150.0, -150.0, -150.0), Vec3::new(150.0, 150.0, 150.0));
                    let mut octree = Octree::new(bounds, 6, 8);

                    for (id, aabb) in aabbs {
                        octree.insert(*id, *aabb);
                    }

                    black_box(octree);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_query_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_performance");

    let aabbs = generate_random_aabbs(5000, 42);
    let query_aabb = AABB::from_center_half_extents(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));

    // BVH query
    let mut bvh = BVH::new();
    bvh.build(aabbs.clone());

    group.bench_function("bvh_query", |b| {
        b.iter(|| {
            let results = bvh.query(black_box(&query_aabb));
            black_box(results);
        });
    });

    // Octree query
    let bounds = AABB::new(Vec3::new(-150.0, -150.0, -150.0), Vec3::new(150.0, 150.0, 150.0));
    let mut octree = Octree::new(bounds, 6, 8);
    for (id, aabb) in &aabbs {
        octree.insert(*id, *aabb);
    }

    group.bench_function("octree_query", |b| {
        b.iter(|| {
            let results = octree.query(black_box(&query_aabb));
            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_collision_detection,
    benchmark_bvh_build,
    benchmark_octree_build,
    benchmark_query_performance
);
criterion_main!(benches);
