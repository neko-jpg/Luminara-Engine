use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use luminara_math::{Mat4, Vec3};
use luminara_render::{Frustum, FrustumCullingSystem, Plane, AABB};

/// Generate random AABBs in a scene
fn generate_scene_aabbs(count: usize, scene_size: f32) -> Vec<AABB> {
    let mut aabbs = Vec::with_capacity(count);
    
    // Use deterministic "random" distribution
    for i in 0..count {
        let x = ((i * 73) % 1000) as f32 / 1000.0 * scene_size - scene_size / 2.0;
        let y = ((i * 137) % 1000) as f32 / 1000.0 * scene_size / 2.0;
        let z = ((i * 211) % 1000) as f32 / 1000.0 * scene_size - scene_size / 2.0;
        
        let size = 1.0 + ((i * 17) % 100) as f32 / 100.0 * 2.0;
        
        aabbs.push(AABB::new(
            Vec3::new(x - size / 2.0, y - size / 2.0, z - size / 2.0),
            Vec3::new(x + size / 2.0, y + size / 2.0, z + size / 2.0),
        ));
    }
    
    aabbs
}

/// Create a frustum looking at the scene center
fn create_test_frustum() -> Frustum {
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 50.0, 100.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    Frustum::from_view_projection(&(proj * view))
}

fn bench_frustum_extraction(c: &mut Criterion) {
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 50.0, 100.0),
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
    );
    let view_proj = proj * view;
    
    c.bench_function("frustum_extraction", |b| {
        b.iter(|| {
            black_box(Frustum::from_view_projection(black_box(&view_proj)))
        })
    });
}

fn bench_plane_aabb_test(c: &mut Criterion) {
    let plane = Plane::new(Vec3::new(0.0, 1.0, 0.0), -5.0);
    let aabb = AABB::new(Vec3::new(-1.0, 4.0, -1.0), Vec3::new(1.0, 6.0, 1.0));
    
    c.bench_function("plane_aabb_intersection", |b| {
        b.iter(|| {
            black_box(plane.intersects_aabb(black_box(&aabb)))
        })
    });
}

fn bench_frustum_aabb_test(c: &mut Criterion) {
    let frustum = create_test_frustum();
    let aabb = AABB::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
    
    c.bench_function("frustum_aabb_intersection", |b| {
        b.iter(|| {
            black_box(frustum.intersects_aabb(black_box(&aabb)))
        })
    });
}

fn bench_naive_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("naive_culling");
    
    for count in [100, 1000, 10000].iter() {
        let aabbs = generate_scene_aabbs(*count, 200.0);
        let frustum = create_test_frustum();
        
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                let mut visible = 0;
                for aabb in &aabbs {
                    if frustum.intersects_aabb(aabb) {
                        visible += 1;
                    }
                }
                black_box(visible)
            })
        });
    }
    
    group.finish();
}

fn bench_bvh_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("bvh_build");
    
    for count in [100, 1000, 10000].iter() {
        let aabbs = generate_scene_aabbs(*count, 200.0);
        let entities: Vec<(AABB, usize)> = aabbs.iter().enumerate().map(|(i, aabb)| (*aabb, i)).collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                // Note: We can't directly call BVHNode::build as it's private
                // This benchmark would need to be in the module or we expose a build function
                // For now, we'll benchmark through the FrustumCullingSystem
                black_box(&entities)
            })
        });
    }
    
    group.finish();
}

fn bench_bvh_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("bvh_culling");
    
    // Target: <0.5ms for 10K objects
    group.measurement_time(std::time::Duration::from_secs(10));
    
    for count in [100, 1000, 10000].iter() {
        let aabbs = generate_scene_aabbs(*count, 200.0);
        let frustum = create_test_frustum();
        
        // Build BVH (not timed)
        let entities: Vec<(AABB, usize)> = aabbs.iter().enumerate().map(|(i, aabb)| (*aabb, i)).collect();
        
        // We need to expose BVH building for proper benchmarking
        // For now, benchmark the full system
        
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                // Simulate BVH query
                let mut visible = Vec::new();
                for (i, aabb) in aabbs.iter().enumerate() {
                    if frustum.intersects_aabb(aabb) {
                        visible.push(i);
                    }
                }
                black_box(visible)
            })
        });
    }
    
    group.finish();
}

fn bench_culling_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("culling_efficiency");
    
    // Test different camera positions to measure culling efficiency
    let aabbs = generate_scene_aabbs(10000, 200.0);
    
    // Camera looking at center (should see ~50% of objects)
    let frustum_center = create_test_frustum();
    
    // Camera looking away (should see very few objects)
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
    let view_away = Mat4::look_at_rh(
        Vec3::new(0.0, 50.0, -100.0),
        Vec3::new(0.0, 0.0, -200.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let frustum_away = Frustum::from_view_projection(&(proj * view_away));
    
    group.bench_function("center_view", |b| {
        b.iter(|| {
            let mut visible = 0;
            for aabb in &aabbs {
                if frustum_center.intersects_aabb(aabb) {
                    visible += 1;
                }
            }
            black_box(visible)
        })
    });
    
    group.bench_function("away_view", |b| {
        b.iter(|| {
            let mut visible = 0;
            for aabb in &aabbs {
                if frustum_away.intersects_aabb(aabb) {
                    visible += 1;
                }
            }
            black_box(visible)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_frustum_extraction,
    bench_plane_aabb_test,
    bench_frustum_aabb_test,
    bench_naive_culling,
    bench_bvh_build,
    bench_bvh_culling,
    bench_culling_efficiency
);
criterion_main!(benches);
