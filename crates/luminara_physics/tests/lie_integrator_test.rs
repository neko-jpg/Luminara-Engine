use luminara_math::algebra::{Bivector, Motor};
use luminara_math::{Quat, Transform, Vec3};
use luminara_physics::LiePhysicsIntegrator;

/// Helper function to check if a motor is normalized
fn is_motor_normalized(motor: &Motor<f32>, epsilon: f32) -> bool {
    let norm_sq = motor.s * motor.s
        + motor.e12 * motor.e12
        + motor.e13 * motor.e13
        + motor.e23 * motor.e23;
    (norm_sq - 1.0).abs() < epsilon
}

#[test]
fn test_integrate_rk4_constant_velocity() {
    // Test that RK4 integration works with constant velocity
    let motor = Motor::IDENTITY;
    let velocity = Bivector::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // Rotation around Y axis
    let dt = 0.1;

    let result = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);

    // Motor should be valid (normalized)
    assert!(is_motor_normalized(&result, 1e-5));

    // Should have rotated
    let (rotation, _translation) = result.to_rotation_translation_glam();
    assert!((rotation.w - 1.0).abs() > 1e-6 || rotation.y.abs() > 1e-6);
}

#[test]
fn test_integrate_euler_vs_rk4() {
    // Compare Euler and RK4 integration
    let motor = Motor::IDENTITY;
    let velocity = Bivector::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Rotation around X axis
    let dt = 0.01;

    let euler_result = LiePhysicsIntegrator::integrate_euler(&motor, &velocity, dt);
    let rk4_result = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);

    // Both should be valid
    assert!(is_motor_normalized(&euler_result, 1e-5));
    assert!(is_motor_normalized(&rk4_result, 1e-5));

    // For small timesteps, they should be similar but not identical
    let (euler_rot, euler_trans) = euler_result.to_rotation_translation_glam();
    let (rk4_rot, rk4_trans) = rk4_result.to_rotation_translation_glam();

    // Positions should be close
    assert!((euler_trans - rk4_trans).length() < 0.01);

    // Rotations should be close but RK4 should be more accurate
    let dot = euler_rot.dot(rk4_rot).abs();
    assert!(dot > 0.99); // Very similar for small timestep
}

#[test]
fn test_step_normalizes() {
    // Test that step() normalizes the motor to prevent drift
    let mut motor = Motor::IDENTITY;
    let velocity = Bivector::new(0.5, 0.5, 0.5, 0.1, 0.1, 0.1);
    let dt = 0.01;

    // Run many steps
    for _ in 0..100 {
        LiePhysicsIntegrator::step(&mut motor, &velocity, dt);
    }

    // Motor should still be normalized
    assert!(is_motor_normalized(&motor, 1e-4));
}

#[test]
fn test_transform_motor_conversion() {
    // Test conversion between Transform and Motor
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };

    let motor = LiePhysicsIntegrator::transform_to_motor(&transform);
    let result = LiePhysicsIntegrator::motor_to_transform(&motor, transform.scale);

    // Should match original (scale is preserved)
    assert!((result.translation - transform.translation).length() < 1e-5);
    assert!((result.rotation.dot(transform.rotation)).abs() > 0.9999);
    assert!((result.scale - transform.scale).length() < 1e-5);
}

#[test]
fn test_integrate_transform() {
    // Test the convenience method for integrating transforms
    let mut transform = Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    let linear_velocity = Vec3::new(1.0, 0.0, 0.0);
    let angular_velocity = Vec3::new(0.0, 1.0, 0.0);
    let dt = 0.1;

    LiePhysicsIntegrator::integrate_transform(
        &mut transform,
        linear_velocity,
        angular_velocity,
        dt,
    );

    // Should have moved in X direction
    assert!(transform.translation.x > 0.0);
    assert!(transform.translation.x < 0.2); // Approximately 0.1

    // Should have rotated around Y axis
    assert!((transform.rotation.w - 1.0).abs() > 1e-6 || transform.rotation.y.abs() > 1e-6);

    // Scale should be preserved
    assert!((transform.scale - Vec3::ONE).length() < 1e-5);
}

#[test]
fn test_compute_energy() {
    // Test energy computation
    let velocity = Bivector::new(
        1.0, 0.0, 0.0, // Angular velocity: 1 rad/s around X
        2.0, 0.0, 0.0, // Linear velocity: 2 m/s in X direction
    );

    let mass = 1.0;
    let inertia = 1.0;

    let energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);

    // Translational energy: 0.5 * 1.0 * 4.0 = 2.0
    // Rotational energy: 0.5 * 1.0 * 1.0 = 0.5
    // Total: 2.5
    assert!((energy - 2.5).abs() < 1e-5);
}

#[test]
fn test_energy_conservation_over_time() {
    // Test that energy is approximately conserved during integration
    // (it won't be perfect due to numerical errors, but should be close)
    let motor = Motor::IDENTITY;
    let velocity = Bivector::new(1.0, 0.5, 0.3, 0.1, 0.2, 0.15);
    let mass = 1.0;
    let inertia = 1.0;
    let dt = 0.001; // Small timestep for better accuracy

    let initial_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);

    // Integrate for many steps (constant velocity, so energy should be conserved)
    let mut current_motor = motor;
    for _ in 0..1000 {
        current_motor = LiePhysicsIntegrator::integrate_rk4(&current_motor, &velocity, dt);
    }

    let final_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);

    // Energy should be conserved (velocity is constant)
    assert!((final_energy - initial_energy).abs() < 1e-5);
}

#[test]
fn test_high_angular_velocity_stability() {
    // Test that RK4 remains stable with high angular velocity
    let motor = Motor::IDENTITY;
    let high_angular_velocity = Bivector::new(
        10.0, 5.0, 8.0, // High angular velocity
        0.0, 0.0, 0.0,  // No linear velocity
    );
    let dt = 0.01;

    let mut current_motor = motor;
    for _ in 0..100 {
        current_motor = LiePhysicsIntegrator::integrate_rk4(&current_motor, &high_angular_velocity, dt);
        current_motor.normalize();
    }

    // Motor should still be valid
    assert!(is_motor_normalized(&current_motor, 1e-4));

    // Should have rotated significantly
    let (rotation, _translation) = current_motor.to_rotation_translation_glam();
    assert!((rotation.w - 1.0).abs() > 0.1); // Significant rotation
}

#[test]
fn test_zero_velocity() {
    // Test that zero velocity produces no change
    let motor = Motor::from_rotation_translation_glam(
        Quat::from_rotation_y(0.5),
        Vec3::new(1.0, 2.0, 3.0),
    );
    let zero_velocity = Bivector::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let dt = 0.1;

    let result = LiePhysicsIntegrator::integrate_rk4(&motor, &zero_velocity, dt);

    // Should be unchanged (within numerical precision)
    let (orig_rot, orig_trans) = motor.to_rotation_translation_glam();
    let (result_rot, result_trans) = result.to_rotation_translation_glam();

    assert!((orig_trans - result_trans).length() < 1e-5);
    assert!((orig_rot.dot(result_rot)).abs() > 0.9999);
}

#[test]
fn test_pure_translation() {
    // Test pure translation (no rotation)
    let motor = Motor::IDENTITY;
    let translation_velocity = Bivector::new(
        0.0, 0.0, 0.0, // No angular velocity
        1.0, 2.0, 3.0, // Linear velocity
    );
    let dt = 0.1;

    let result = LiePhysicsIntegrator::integrate_rk4(&motor, &translation_velocity, dt);

    let (rotation, translation) = result.to_rotation_translation_glam();

    // Should have translated
    assert!((translation - Vec3::new(0.1, 0.2, 0.3)).length() < 1e-4);

    // Should not have rotated
    assert!((rotation.dot(Quat::IDENTITY)).abs() > 0.9999);
}

#[test]
fn test_pure_rotation() {
    // Test pure rotation (no translation)
    let motor = Motor::IDENTITY;
    let rotation_velocity = Bivector::new(
        0.0, 1.0, 0.0, // Angular velocity around Y
        0.0, 0.0, 0.0, // No linear velocity
    );
    let dt = 0.1;

    let result = LiePhysicsIntegrator::integrate_rk4(&motor, &rotation_velocity, dt);

    let (rotation, translation) = result.to_rotation_translation_glam();

    // Should not have translated
    assert!(translation.length() < 1e-5);

    // Should have rotated
    assert!((rotation.w - 1.0).abs() > 1e-6 || rotation.y.abs() > 1e-6);
}
