/// Performance benchmark comparing Euler vs Lie (RK4) integration
///
/// This benchmark measures the performance overhead of RK4 integration
/// compared to simple Euler integration for physics simulation.
///
/// **Validates: Requirements 13.2**

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use luminara_math::algebra::{Bivector, Motor};
use luminara_physics::LiePhysicsIntegrator;

fn bench_euler_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("euler_integration");

    let velocity = Bivector::new(10.0, 5.0, 8.0, 1.0, 0.5, 0.3);
    let dt = 0.01;

    group.bench_function("single_step", |b| {
        let motor = Motor::IDENTITY;
        b.iter(|| {
            black_box(LiePhysicsIntegrator::integrate_euler(
                black_box(&motor),
                black_box(&velocity),
                black_box(dt),
            ))
        });
    });

    group.bench_function("100_steps", |b| {
        b.iter(|| {
            let mut motor = Motor::IDENTITY;
            for _ in 0..100 {
                motor = LiePhysicsIntegrator::integrate_euler(&motor, &velocity, dt);
                motor.normalize();
            }
            black_box(motor)
        });
    });

    group.bench_function("1000_steps", |b| {
        b.iter(|| {
            let mut motor = Motor::IDENTITY;
            for _ in 0..1000 {
                motor = LiePhysicsIntegrator::integrate_euler(&motor, &velocity, dt);
                motor.normalize();
            }
            black_box(motor)
        });
    });

    group.finish();
}

fn bench_rk4_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("rk4_integration");

    let velocity = Bivector::new(10.0, 5.0, 8.0, 1.0, 0.5, 0.3);
    let dt = 0.01;

    group.bench_function("single_step", |b| {
        let motor = Motor::IDENTITY;
        b.iter(|| {
            black_box(LiePhysicsIntegrator::integrate_rk4(
                black_box(&motor),
                black_box(&velocity),
                black_box(dt),
            ))
        });
    });

    group.bench_function("100_steps", |b| {
        b.iter(|| {
            let mut motor = Motor::IDENTITY;
            for _ in 0..100 {
                motor = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);
                motor.normalize();
            }
            black_box(motor)
        });
    });

    group.bench_function("1000_steps", |b| {
        b.iter(|| {
            let mut motor = Motor::IDENTITY;
            for _ in 0..1000 {
                motor = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);
                motor.normalize();
            }
            black_box(motor)
        });
    });

    group.finish();
}

fn bench_comparison_by_velocity(c: &mut Criterion) {
    let mut group = c.benchmark_group("velocity_comparison");

    let dt = 0.01;
    let steps = 100;

    // Test different angular velocity magnitudes
    let velocities = [
        ("low", 5.0),
        ("medium", 20.0),
        ("high", 50.0),
        ("extreme", 100.0),
    ];

    for (name, angular_vel) in velocities.iter() {
        let velocity = Bivector::new(*angular_vel, 0.0, 0.0, 1.0, 0.0, 0.0);

        group.bench_with_input(
            BenchmarkId::new("euler", name),
            &velocity,
            |b, vel| {
                b.iter(|| {
                    let mut motor = Motor::IDENTITY;
                    for _ in 0..steps {
                        motor = LiePhysicsIntegrator::integrate_euler(&motor, vel, dt);
                        motor.normalize();
                    }
                    black_box(motor)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rk4", name),
            &velocity,
            |b, vel| {
                b.iter(|| {
                    let mut motor = Motor::IDENTITY;
                    for _ in 0..steps {
                        motor = LiePhysicsIntegrator::integrate_rk4(&motor, vel, dt);
                        motor.normalize();
                    }
                    black_box(motor)
                });
            },
        );
    }

    group.finish();
}

fn bench_comparison_by_timestep(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestep_comparison");

    let velocity = Bivector::new(20.0, 15.0, 10.0, 1.0, 0.5, 0.3);
    let total_time = 1.0; // 1 second

    // Test different timesteps
    let timesteps = [
        ("1ms", 0.001),
        ("5ms", 0.005),
        ("10ms", 0.01),
        ("20ms", 0.02),
    ];

    for (name, dt) in timesteps.iter() {
        let steps = (total_time / dt) as usize;

        group.bench_with_input(
            BenchmarkId::new("euler", name),
            &(*dt, steps),
            |b, &(dt, steps)| {
                b.iter(|| {
                    let mut motor = Motor::IDENTITY;
                    for _ in 0..steps {
                        motor = LiePhysicsIntegrator::integrate_euler(&motor, &velocity, dt);
                        motor.normalize();
                    }
                    black_box(motor)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rk4", name),
            &(*dt, steps),
            |b, &(dt, steps)| {
                b.iter(|| {
                    let mut motor = Motor::IDENTITY;
                    for _ in 0..steps {
                        motor = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);
                        motor.normalize();
                    }
                    black_box(motor)
                });
            },
        );
    }

    group.finish();
}

fn bench_many_bodies(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_bodies");

    let dt = 0.01;
    let steps = 10;

    // Simulate different numbers of rigid bodies
    let body_counts = [10, 50, 100, 500];

    for count in body_counts.iter() {
        // Create random velocities for each body
        let velocities: Vec<Bivector<f32>> = (0..*count)
            .map(|i| {
                let phase = i as f32 * 0.1;
                Bivector::new(
                    10.0 * phase.sin(),
                    10.0 * phase.cos(),
                    5.0 * (phase * 2.0).sin(),
                    1.0,
                    0.5,
                    0.3,
                )
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("euler", count),
            &velocities,
            |b, vels| {
                b.iter(|| {
                    let mut motors: Vec<Motor<f32>> = vec![Motor::IDENTITY; vels.len()];
                    for _ in 0..steps {
                        for (motor, vel) in motors.iter_mut().zip(vels.iter()) {
                            *motor = LiePhysicsIntegrator::integrate_euler(motor, vel, dt);
                            motor.normalize();
                        }
                    }
                    black_box(motors)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rk4", count),
            &velocities,
            |b, vels| {
                b.iter(|| {
                    let mut motors: Vec<Motor<f32>> = vec![Motor::IDENTITY; vels.len()];
                    for _ in 0..steps {
                        for (motor, vel) in motors.iter_mut().zip(vels.iter()) {
                            *motor = LiePhysicsIntegrator::integrate_rk4(motor, vel, dt);
                            motor.normalize();
                        }
                    }
                    black_box(motors)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_euler_integration,
    bench_rk4_integration,
    bench_comparison_by_velocity,
    bench_comparison_by_timestep,
    bench_many_bodies
);
criterion_main!(benches);
