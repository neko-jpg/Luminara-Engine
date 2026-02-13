use criterion::{black_box, criterion_group, criterion_main, Criterion};
use luminara_math::foundations::{Expansion, orient3d, incircle};
use luminara_math::algebra::{Motor, Bivector, SymplecticEuler, LieGroupIntegrator};
use luminara_math::geometry::bvh::{Bvh, Aabb, Primitive};
use glam::{Vec3, Mat4};
use rayon::ThreadPoolBuilder;

// --- Foundations ---

fn bench_expansion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Expansion");

    // Add two expansions of length 2
    let e1 = Expansion::from_f64(1.0).add(&Expansion::from_f64(1e-16));
    let e2 = Expansion::from_f64(2.0).add(&Expansion::from_f64(2e-16));

    group.bench_function("add_len2", |b| b.iter(|| {
        black_box(&e1).add(black_box(&e2))
    }));

    // Mul two expansions of length 2
    group.bench_function("mul_len2", |b| b.iter(|| {
        black_box(&e1).mul(black_box(&e2))
    }));

    group.finish();
}

fn bench_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("Predicates");

    let pa = [0.0, 0.0, 0.0];
    let pb = [1.0, 0.0, 0.0];
    let pc = [0.0, 1.0, 0.0];
    let pd = [0.0, 0.0, 1.0];
    let pe = [0.5, 0.5, 0.5]; // Inside

    group.bench_function("orient3d_simple", |b| b.iter(|| {
        orient3d(black_box(pa), black_box(pb), black_box(pc), black_box(pd))
    }));

    // Incircle case
    let p2a = [0.0, 0.0];
    let p2b = [1.0, 0.0];
    let p2c = [0.0, 1.0];
    let p2d = [0.5, 0.5];

    group.bench_function("incircle_simple", |b| b.iter(|| {
        incircle(black_box(p2a), black_box(p2b), black_box(p2c), black_box(p2d))
    }));

    group.finish();
}

// --- Algebra ---

fn bench_motor(c: &mut Criterion) {
    let mut group = c.benchmark_group("Motor");

    let m1 = Motor::from_translation(luminara_math::algebra::Vector3::new(1.0f32, 2.0, 3.0));
    let m2 = Motor::from_axis_angle(luminara_math::algebra::Vector3::new(0.0f32, 1.0, 0.0), 1.0);

    group.bench_function("geometric_product", |b| b.iter(|| {
        black_box(m1).geometric_product(black_box(&m2))
    }));

    // Comparison with Mat4
    let mat1 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let mat2 = Mat4::from_axis_angle(Vec3::Y, 1.0);

    group.bench_function("mat4_mul", |b| b.iter(|| {
        black_box(mat1).mul_mat4(black_box(&mat2))
    }));

    group.finish();
}

// --- Dynamics ---

// Dummy primitive for BVH
#[derive(Clone)]
struct BenchmarkPrim {
    aabb: Aabb,
}

impl Primitive for BenchmarkPrim {
    fn aabb(&self) -> Aabb {
        self.aabb
    }
    fn intersect(&self, _origin: Vec3, _dir: Vec3) -> Option<f32> { None }
}

fn bench_bvh(c: &mut Criterion) {
    let mut group = c.benchmark_group("BVH");
    group.sample_size(10); // Reduce sample size for slow benchmarks

    // Generate 100k primitives
    let count = 100_000;
    let primitives: Vec<BenchmarkPrim> = (0..count).map(|i| {
        let x = (i % 100) as f32;
        let y = ((i / 100) % 100) as f32;
        let z = (i / 10000) as f32;
        let min = Vec3::new(x, y, z);
        let max = min + Vec3::new(0.5, 0.5, 0.5);
        BenchmarkPrim { aabb: Aabb::new(min, max) }
    }).collect();

    // Ensure thread pool is set up (global)
    let _ = ThreadPoolBuilder::new().num_threads(16).build_global();

    group.bench_function("build_100k_parallel", |b| b.iter(|| {
        Bvh::build(black_box(primitives.clone()))
    }));

    group.finish();
}

fn bench_integrator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Integrator");

    let m = Motor::IDENTITY;
    let v = Bivector::new(0.1, 0.0, 0.0, 1.0, 0.0, 0.0);
    let dt = 0.016;

    group.bench_function("mk4_step", |b| b.iter(|| {
        LieGroupIntegrator::step(black_box(m), black_box(dt), |_| v)
    }));

    group.bench_function("symplectic_step", |b| b.iter(|| {
        SymplecticEuler::step(black_box(m), black_box(v), black_box(dt), |_| Bivector::ZERO)
    }));

    group.finish();
}

criterion_group!(benches, bench_expansion, bench_predicates, bench_motor, bench_bvh, bench_integrator);
criterion_main!(benches);
