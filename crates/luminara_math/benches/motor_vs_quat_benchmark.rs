//! Benchmark comparing Motor-based transforms vs Quaternion-based transforms.
//!
//! This benchmark validates Requirement 13.5: "THE System SHALL benchmark Motor-based
//! transforms against quaternion-based transforms to verify performance."
//!
//! Measures:
//! - Composition performance (combining transforms)
//! - Interpolation performance (SLERP/LERP)
//! - Memory usage (size of data structures)
//! - Point transformation performance
//!
//! Requirements: 13.5

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Mat4, Quat, Vec3};
use luminara_math::{Transform, TransformMotor};
use std::mem::size_of;

// ============================================================================
// Composition Benchmarks
// ============================================================================

fn bench_composition(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transform Composition");

    // Create test transforms
    let quat_t1 = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(0.5),
        scale: Vec3::ONE,
    };
    let quat_t2 = Transform {
        translation: Vec3::new(4.0, 5.0, 6.0),
        rotation: Quat::from_rotation_x(0.3),
        scale: Vec3::ONE,
    };

    let motor_t1 = TransformMotor::from_transform(&quat_t1);
    let motor_t2 = TransformMotor::from_transform(&quat_t2);

    // Benchmark quaternion-based composition (via matrix multiplication)
    group.bench_function("Quaternion (Matrix)", |b| {
        b.iter(|| {
            let m1 = Mat4::from_scale_rotation_translation(
                black_box(quat_t1.scale),
                black_box(quat_t1.rotation),
                black_box(quat_t1.translation),
            );
            let m2 = Mat4::from_scale_rotation_translation(
                black_box(quat_t2.scale),
                black_box(quat_t2.rotation),
                black_box(quat_t2.translation),
            );
            black_box(m1 * m2)
        })
    });

    // Benchmark quaternion-based composition (direct)
    group.bench_function("Quaternion (Direct)", |b| {
        b.iter(|| {
            let t1 = black_box(quat_t1);
            let t2 = black_box(quat_t2);
            
            // Manual composition: rotate t2's translation by t1's rotation, then add
            let rotated_translation = t1.rotation * t2.translation;
            let combined_translation = t1.translation + rotated_translation;
            let combined_rotation = t1.rotation * t2.rotation;
            let combined_scale = t1.scale * t2.scale;
            
            black_box(Transform {
                translation: combined_translation,
                rotation: combined_rotation,
                scale: combined_scale,
            })
        })
    });

    // Benchmark motor-based composition
    group.bench_function("Motor", |b| {
        b.iter(|| {
            black_box(motor_t1).compose(black_box(&motor_t2))
        })
    });

    group.finish();
}

// ============================================================================
// Interpolation Benchmarks
// ============================================================================

fn bench_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transform Interpolation");

    let quat_start = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    let quat_end = Transform {
        translation: Vec3::new(10.0, 5.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI),
        scale: Vec3::splat(2.0),
    };

    let motor_start = TransformMotor::from_transform(&quat_start);
    let motor_end = TransformMotor::from_transform(&quat_end);

    // Benchmark quaternion-based interpolation
    group.bench_function("Quaternion", |b| {
        b.iter(|| {
            let t = black_box(0.5f32);
            let start = black_box(quat_start);
            let end = black_box(quat_end);
            
            black_box(Transform {
                translation: start.translation.lerp(end.translation, t),
                rotation: start.rotation.slerp(end.rotation, t),
                scale: start.scale.lerp(end.scale, t),
            })
        })
    });

    // Benchmark motor-based interpolation
    group.bench_function("Motor", |b| {
        b.iter(|| {
            black_box(motor_start).interpolate(black_box(&motor_end), black_box(0.5))
        })
    });

    group.finish();
}

// ============================================================================
// Point Transformation Benchmarks
// ============================================================================

fn bench_point_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Point Transformation");

    let quat_transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::splat(2.0),
    };

    let motor_transform = TransformMotor::from_transform(&quat_transform);
    let point = Vec3::new(5.0, 6.0, 7.0);

    // Benchmark quaternion-based point transformation (via matrix)
    group.bench_function("Quaternion (Matrix)", |b| {
        b.iter(|| {
            let matrix = Mat4::from_scale_rotation_translation(
                black_box(quat_transform.scale),
                black_box(quat_transform.rotation),
                black_box(quat_transform.translation),
            );
            black_box(matrix.transform_point3(black_box(point)))
        })
    });

    // Benchmark quaternion-based point transformation (direct)
    group.bench_function("Quaternion (Direct)", |b| {
        b.iter(|| {
            let t = black_box(quat_transform);
            let p = black_box(point);
            
            // Scale, then rotate, then translate
            let scaled = p * t.scale;
            let rotated = t.rotation * scaled;
            let translated = rotated + t.translation;
            
            black_box(translated)
        })
    });

    // Benchmark motor-based point transformation
    group.bench_function("Motor", |b| {
        b.iter(|| {
            black_box(motor_transform).transform_point(black_box(point))
        })
    });

    group.finish();
}

// ============================================================================
// Batch Transformation Benchmarks
// ============================================================================

fn bench_batch_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Point Transformation");
    group.sample_size(20); // Reduce sample size for faster benchmarking
    
    // Test with different batch sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        let quat_transform = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
            scale: Vec3::splat(2.0),
        };

        let motor_transform = TransformMotor::from_transform(&quat_transform);
        
        // Generate random points
        let points: Vec<Vec3> = (0..*size)
            .map(|i| {
                let f = i as f32;
                Vec3::new(f * 0.1, f * 0.2, f * 0.3)
            })
            .collect();

        // Benchmark quaternion-based batch transformation
        group.bench_with_input(
            BenchmarkId::new("Quaternion", size),
            size,
            |b, _| {
                b.iter(|| {
                    let t = black_box(quat_transform);
                    let matrix = Mat4::from_scale_rotation_translation(t.scale, t.rotation, t.translation);
                    
                    black_box(
                        points
                            .iter()
                            .map(|&p| matrix.transform_point3(p))
                            .collect::<Vec<_>>()
                    )
                })
            },
        );

        // Benchmark motor-based batch transformation
        group.bench_with_input(
            BenchmarkId::new("Motor", size),
            size,
            |b, _| {
                b.iter(|| {
                    let t = black_box(motor_transform);
                    
                    black_box(
                        points
                            .iter()
                            .map(|&p| t.transform_point(p))
                            .collect::<Vec<_>>()
                    )
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Deep Hierarchy Composition Benchmarks
// ============================================================================

fn bench_deep_hierarchy(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deep Hierarchy Composition");
    group.sample_size(20); // Reduce sample size for faster benchmarking

    // Test with different hierarchy depths
    for depth in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("Quaternion", depth),
            depth,
            |b, &depth| {
                let transforms: Vec<Transform> = (0..depth)
                    .map(|i| Transform {
                        translation: Vec3::new(1.0, 0.0, 0.0),
                        rotation: Quat::from_rotation_y(0.1 * i as f32),
                        scale: Vec3::ONE,
                    })
                    .collect();

                b.iter(|| {
                    let mut result = Mat4::IDENTITY;
                    for t in black_box(&transforms) {
                        let m = Mat4::from_scale_rotation_translation(t.scale, t.rotation, t.translation);
                        result = result * m;
                    }
                    black_box(result)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Motor", depth),
            depth,
            |b, &depth| {
                let transforms: Vec<TransformMotor> = (0..depth)
                    .map(|i| TransformMotor::from_position_rotation(
                        Vec3::new(1.0, 0.0, 0.0),
                        Quat::from_rotation_y(0.1 * i as f32),
                    ))
                    .collect();

                b.iter(|| {
                    let mut result = TransformMotor::IDENTITY;
                    for t in black_box(&transforms) {
                        result = result.compose(t);
                    }
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Memory Usage Reporting
// ============================================================================

fn report_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Usage");
    
    // Report sizes
    let quat_size = size_of::<Transform>();
    let motor_size = size_of::<TransformMotor>();
    
    println!("\n=== Memory Usage Comparison ===");
    println!("Transform (Quaternion):  {} bytes", quat_size);
    println!("TransformMotor (Motor):  {} bytes", motor_size);
    println!("Difference:              {} bytes ({:.1}% {})",
        (motor_size as i32 - quat_size as i32).abs(),
        ((motor_size as f32 / quat_size as f32) - 1.0) * 100.0,
        if motor_size > quat_size { "larger" } else { "smaller" }
    );
    println!("===============================\n");

    // Dummy benchmark to keep the group valid
    group.bench_function("size_check", |b| {
        b.iter(|| {
            black_box(quat_size + motor_size)
        })
    });

    group.finish();
}

// ============================================================================
// Numerical Stability Test
// ============================================================================

fn bench_numerical_stability(c: &mut Criterion) {
    let mut group = c.benchmark_group("Numerical Stability");
    group.sample_size(10);

    // Test repeated composition (accumulates numerical error)
    let iterations = 1000;

    group.bench_function("Quaternion_repeated_composition", |b| {
        b.iter(|| {
            let mut transform = Transform {
                translation: Vec3::new(0.1, 0.0, 0.0),
                rotation: Quat::from_rotation_y(0.01),
                scale: Vec3::ONE,
            };

            for _ in 0..iterations {
                let matrix = Mat4::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );
                let next_matrix = Mat4::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );
                let result = matrix * next_matrix;
                
                // Extract back to transform (simplified)
                let (scale, rotation, translation) = result.to_scale_rotation_translation();
                transform = Transform {
                    translation,
                    rotation,
                    scale,
                };
            }

            black_box(transform)
        })
    });

    group.bench_function("Motor_repeated_composition", |b| {
        b.iter(|| {
            let mut transform = TransformMotor::from_position_rotation(
                Vec3::new(0.1, 0.0, 0.0),
                Quat::from_rotation_y(0.01),
            );

            for _ in 0..iterations {
                let next = TransformMotor::from_position_rotation(
                    Vec3::new(0.1, 0.0, 0.0),
                    Quat::from_rotation_y(0.01),
                );
                transform = transform.compose(&next);
            }

            black_box(transform)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    report_memory_usage,
    bench_composition,
    bench_interpolation,
    bench_point_transformation,
    bench_batch_transformation,
    bench_deep_hierarchy,
    bench_numerical_stability,
);
criterion_main!(benches);
