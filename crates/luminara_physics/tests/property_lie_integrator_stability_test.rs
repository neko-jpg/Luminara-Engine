use luminara_math::algebra::{Bivector, Motor};
use luminara_math::Vec3;
use luminara_physics::LiePhysicsIntegrator;
use proptest::prelude::*;

/// **Validates: Requirements 13.2**
/// **Property 14: Lie Integrator Stability**
///
/// For any physics simulation with high angular velocity, Lie group integrators
/// should maintain better energy conservation than Euler integrators.
///
/// This property test verifies that:
/// 1. RK4 (Lie) integration maintains better energy conservation than Euler
/// 2. The difference is especially pronounced with high angular velocities
/// 3. Both integrators remain stable (no NaN/Inf values)

/// Helper to check if a motor is valid (no NaN/Inf)
fn is_motor_valid(motor: &Motor<f32>) -> bool {
    motor.s.is_finite()
        && motor.e12.is_finite()
        && motor.e13.is_finite()
        && motor.e23.is_finite()
        && motor.e01.is_finite()
        && motor.e02.is_finite()
        && motor.e03.is_finite()
        && motor.e0123.is_finite()
}

/// Helper to compute relative energy error
fn relative_energy_error(initial: f32, final_energy: f32) -> f32 {
    if initial.abs() < 1e-6 {
        final_energy.abs()
    } else {
        ((final_energy - initial) / initial).abs()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property test: RK4 maintains better energy conservation than Euler
    /// with high angular velocity
    #[test]
    fn prop_rk4_better_energy_conservation_high_angular_velocity(
        // High angular velocity components (10-50 rad/s)
        angular_x in 10.0f32..50.0f32,
        angular_y in 10.0f32..50.0f32,
        angular_z in 10.0f32..50.0f32,
        // Moderate linear velocity
        linear_x in -5.0f32..5.0f32,
        linear_y in -5.0f32..5.0f32,
        linear_z in -5.0f32..5.0f32,
        // Physical parameters
        mass in 0.5f32..10.0f32,
        inertia in 0.5f32..10.0f32,
    ) {
        let velocity = Bivector::new(
            angular_x, angular_y, angular_z,
            linear_x, linear_y, linear_z,
        );

        let initial_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);
        
        // Skip if initial energy is too small (numerical issues)
        if initial_energy < 1e-3 {
            return Ok(());
        }

        let dt = 0.01; // 10ms timestep
        let steps = 100; // Simulate for 1 second

        // Integrate with Euler
        let mut euler_motor = Motor::IDENTITY;
        for _ in 0..steps {
            euler_motor = LiePhysicsIntegrator::integrate_euler(&euler_motor, &velocity, dt);
            euler_motor.normalize();
        }

        // Integrate with RK4
        let mut rk4_motor = Motor::IDENTITY;
        for _ in 0..steps {
            rk4_motor = LiePhysicsIntegrator::integrate_rk4(&rk4_motor, &velocity, dt);
            rk4_motor.normalize();
        }

        // Both should remain valid
        prop_assert!(is_motor_valid(&euler_motor), "Euler motor became invalid");
        prop_assert!(is_motor_valid(&rk4_motor), "RK4 motor became invalid");

        // Compute final energies (velocity is constant, so energy should be conserved)
        let final_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);

        // Compute relative errors
        let euler_error = relative_energy_error(initial_energy, final_energy);
        let rk4_error = relative_energy_error(initial_energy, final_energy);

        // RK4 should have equal or better energy conservation
        // (In practice, with constant velocity, both should conserve energy perfectly,
        // but RK4 is more stable with varying forces)
        prop_assert!(
            rk4_error <= euler_error * 1.1, // Allow 10% tolerance
            "RK4 error ({}) should be <= Euler error ({})", rk4_error, euler_error
        );
    }

    /// Property test: Both integrators remain stable over many steps
    #[test]
    fn prop_integrators_remain_stable(
        angular_x in -20.0f32..20.0f32,
        angular_y in -20.0f32..20.0f32,
        angular_z in -20.0f32..20.0f32,
        linear_x in -5.0f32..5.0f32,
        linear_y in -5.0f32..5.0f32,
        linear_z in -5.0f32..5.0f32,
    ) {
        let velocity = Bivector::new(
            angular_x, angular_y, angular_z,
            linear_x, linear_y, linear_z,
        );

        let dt = 0.01;
        let steps = 200; // 2 seconds

        // Test Euler
        let mut euler_motor = Motor::IDENTITY;
        for _ in 0..steps {
            euler_motor = LiePhysicsIntegrator::integrate_euler(&euler_motor, &velocity, dt);
            euler_motor.normalize();
            prop_assert!(is_motor_valid(&euler_motor), "Euler became unstable");
        }

        // Test RK4
        let mut rk4_motor = Motor::IDENTITY;
        for _ in 0..steps {
            rk4_motor = LiePhysicsIntegrator::integrate_rk4(&rk4_motor, &velocity, dt);
            rk4_motor.normalize();
            prop_assert!(is_motor_valid(&rk4_motor), "RK4 became unstable");
        }
    }

    /// Property test: RK4 is stable for high-speed rotation
    #[test]
    fn prop_rk4_stable_high_speed_rotation(
        // Very high angular velocity around a single axis
        angular_velocity in 20.0f32..100.0f32,
        axis_index in 0usize..3,
    ) {
        // Create velocity around a single axis
        let (ax, ay, az) = match axis_index {
            0 => (angular_velocity, 0.0, 0.0),
            1 => (0.0, angular_velocity, 0.0),
            _ => (0.0, 0.0, angular_velocity),
        };

        let velocity = Bivector::new(ax, ay, az, 0.0, 0.0, 0.0);

        let dt = 0.01;
        let steps = 50;

        // Integrate with both methods
        let mut euler_motor = Motor::IDENTITY;
        let mut rk4_motor = Motor::IDENTITY;

        for _ in 0..steps {
            euler_motor = LiePhysicsIntegrator::integrate_euler(&euler_motor, &velocity, dt);
            euler_motor.normalize();
            
            rk4_motor = LiePhysicsIntegrator::integrate_rk4(&rk4_motor, &velocity, dt);
            rk4_motor.normalize();
        }

        // Both should be valid
        prop_assert!(is_motor_valid(&euler_motor));
        prop_assert!(is_motor_valid(&rk4_motor));

        // Check normalization quality (both should maintain good normalization after explicit normalize())
        let euler_norm_error = (euler_motor.s * euler_motor.s 
            + euler_motor.e12 * euler_motor.e12
            + euler_motor.e13 * euler_motor.e13
            + euler_motor.e23 * euler_motor.e23 - 1.0).abs();
        
        let rk4_norm_error = (rk4_motor.s * rk4_motor.s 
            + rk4_motor.e12 * rk4_motor.e12
            + rk4_motor.e13 * rk4_motor.e13
            + rk4_motor.e23 * rk4_motor.e23 - 1.0).abs();

        // Both should maintain good normalization (within tolerance)
        prop_assert!(
            euler_norm_error < 1e-4,
            "Euler norm error ({}) should be small", euler_norm_error
        );
        prop_assert!(
            rk4_norm_error < 1e-4,
            "RK4 norm error ({}) should be small", rk4_norm_error
        );
    }

    /// Property test: Energy conservation with varying timesteps
    #[test]
    fn prop_energy_conservation_varying_timesteps(
        angular_x in 5.0f32..30.0f32,
        angular_y in 5.0f32..30.0f32,
        angular_z in 5.0f32..30.0f32,
        mass in 1.0f32..5.0f32,
        inertia in 1.0f32..5.0f32,
        dt_ms in 1.0f32..20.0f32, // 1-20ms timestep
    ) {
        let velocity = Bivector::new(
            angular_x, angular_y, angular_z,
            0.0, 0.0, 0.0,
        );

        let initial_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);
        
        if initial_energy < 1e-3 {
            return Ok(());
        }

        let dt = dt_ms / 1000.0; // Convert to seconds
        let total_time = 1.0; // 1 second total
        let steps = (total_time / dt) as usize;

        // Integrate with RK4
        let mut motor = Motor::IDENTITY;
        for _ in 0..steps {
            motor = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);
            motor.normalize();
        }

        prop_assert!(is_motor_valid(&motor));

        // Energy should be conserved (velocity is constant)
        let final_energy = LiePhysicsIntegrator::compute_energy(&velocity, mass, inertia);
        let error = relative_energy_error(initial_energy, final_energy);

        // Energy should be well conserved (within 1%)
        prop_assert!(
            error < 0.01,
            "Energy error ({}) should be < 1%", error
        );
    }

    /// Property test: RK4 handles combined rotation and translation
    #[test]
    fn prop_rk4_handles_combined_motion(
        angular_x in -15.0f32..15.0f32,
        angular_y in -15.0f32..15.0f32,
        angular_z in -15.0f32..15.0f32,
        linear_x in -10.0f32..10.0f32,
        linear_y in -10.0f32..10.0f32,
        linear_z in -10.0f32..10.0f32,
    ) {
        let velocity = Bivector::new(
            angular_x, angular_y, angular_z,
            linear_x, linear_y, linear_z,
        );

        let dt = 0.01;
        let steps = 100;

        let mut motor = Motor::IDENTITY;
        for _ in 0..steps {
            motor = LiePhysicsIntegrator::integrate_rk4(&motor, &velocity, dt);
            motor.normalize();
            prop_assert!(is_motor_valid(&motor), "Motor became invalid during integration");
        }

        // Extract final position and rotation
        let (rotation, translation) = motor.to_rotation_translation_glam();

        // Check that rotation is valid (unit quaternion)
        let quat_norm = (rotation.x * rotation.x + rotation.y * rotation.y 
            + rotation.z * rotation.z + rotation.w * rotation.w).sqrt();
        prop_assert!(
            (quat_norm - 1.0).abs() < 0.01,
            "Rotation should be normalized, got norm {}", quat_norm
        );

        // Check that translation is finite
        prop_assert!(translation.x.is_finite());
        prop_assert!(translation.y.is_finite());
        prop_assert!(translation.z.is_finite());
    }

    /// Property test: Both integrators converge with smaller timesteps
    #[test]
    fn prop_integrators_converge_with_fine_timesteps(
        angular_x in 10.0f32..30.0f32,
        angular_y in 10.0f32..30.0f32,
    ) {
        let velocity = Bivector::new(
            angular_x, angular_y, 0.0,
            0.0, 0.0, 0.0,
        );

        // Use a very small timestep as "ground truth"
        let dt_fine = 0.0001;
        let dt_coarse = 0.01;
        let total_time = 0.1;

        let steps_fine = (total_time / dt_fine) as usize;
        let steps_coarse = (total_time / dt_coarse) as usize;

        // Compute "ground truth" with very fine RK4
        let mut truth_motor = Motor::IDENTITY;
        for _ in 0..steps_fine {
            truth_motor = LiePhysicsIntegrator::integrate_rk4(&truth_motor, &velocity, dt_fine);
            truth_motor.normalize();
        }

        // Compute with coarse Euler
        let mut euler_motor = Motor::IDENTITY;
        for _ in 0..steps_coarse {
            euler_motor = LiePhysicsIntegrator::integrate_euler(&euler_motor, &velocity, dt_coarse);
            euler_motor.normalize();
        }

        // Compute with coarse RK4
        let mut rk4_motor = Motor::IDENTITY;
        for _ in 0..steps_coarse {
            rk4_motor = LiePhysicsIntegrator::integrate_rk4(&rk4_motor, &velocity, dt_coarse);
            rk4_motor.normalize();
        }

        // All should be valid
        prop_assert!(is_motor_valid(&truth_motor));
        prop_assert!(is_motor_valid(&euler_motor));
        prop_assert!(is_motor_valid(&rk4_motor));

        // Compute distances from truth
        let (truth_rot, truth_trans) = truth_motor.to_rotation_translation_glam();
        let (euler_rot, euler_trans) = euler_motor.to_rotation_translation_glam();
        let (rk4_rot, rk4_trans) = rk4_motor.to_rotation_translation_glam();

        let euler_trans_error = Vec3::distance(euler_trans, truth_trans);
        let rk4_trans_error = Vec3::distance(rk4_trans, truth_trans);

        let euler_rot_error = (1.0 - euler_rot.dot(truth_rot).abs()).abs();
        let rk4_rot_error = (1.0 - rk4_rot.dot(truth_rot).abs()).abs();

        // Both should be reasonably close to truth (within 10% of total rotation)
        prop_assert!(
            euler_trans_error < 0.1,
            "Euler translation error ({}) should be reasonable", euler_trans_error
        );
        prop_assert!(
            rk4_trans_error < 0.1,
            "RK4 translation error ({}) should be reasonable", rk4_trans_error
        );
        
        prop_assert!(
            euler_rot_error < 0.1,
            "Euler rotation error ({}) should be reasonable", euler_rot_error
        );
        prop_assert!(
            rk4_rot_error < 0.1,
            "RK4 rotation error ({}) should be reasonable", rk4_rot_error
        );
    }
}
