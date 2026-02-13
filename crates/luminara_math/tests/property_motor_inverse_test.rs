//! Property-based tests for Motor inverse property.
//!
//! **Validates: Requirements 2.7**
//! **Property 7: Motor Inverse Property**

use luminara_math::algebra::Motor;
use glam::Vec3;
use proptest::prelude::*;
use std::f32::consts::PI;

/// Generate a random motor for property testing.
fn motor_strategy() -> impl Strategy<Value = Motor<f32>> {
    (
        -10.0f32..10.0,  // translation x
        -10.0f32..10.0,  // translation y
        -10.0f32..10.0,  // translation z
        -1.0f32..1.0,    // rotation axis x
        -1.0f32..1.0,    // rotation axis y
        -1.0f32..1.0,    // rotation axis z
        -PI..PI,         // rotation angle
    ).prop_map(|(tx, ty, tz, ax, ay, az, angle)| {
        let trans = Vec3::new(tx, ty, tz);
        let axis = Vec3::new(ax, ay, az);
        
        // Handle zero axis case
        if axis.length_squared() < 1e-6 {
            Motor::from_translation(trans.into())
        } else {
            let axis_normalized = axis.normalize();
            let rot = Motor::from_axis_angle(axis_normalized.into(), angle);
            let trans_motor = Motor::from_translation(trans.into());
            let mut motor = trans_motor.geometric_product(&rot);
            // Normalize to ensure it's a valid motor
            motor.normalize();
            motor
        }
    })
}

/// Helper function to check if two motors are approximately equal.
fn assert_motors_approx_equal(a: &Motor<f32>, b: &Motor<f32>, epsilon: f32) {
    let diff_s = (a.s - b.s).abs();
    let diff_e12 = (a.e12 - b.e12).abs();
    let diff_e13 = (a.e13 - b.e13).abs();
    let diff_e23 = (a.e23 - b.e23).abs();
    let diff_e01 = (a.e01 - b.e01).abs();
    let diff_e02 = (a.e02 - b.e02).abs();
    let diff_e03 = (a.e03 - b.e03).abs();
    let diff_e0123 = (a.e0123 - b.e0123).abs();
    
    assert!(
        diff_s < epsilon &&
        diff_e12 < epsilon &&
        diff_e13 < epsilon &&
        diff_e23 < epsilon &&
        diff_e01 < epsilon &&
        diff_e02 < epsilon &&
        diff_e03 < epsilon &&
        diff_e0123 < epsilon,
        "Motors not approximately equal:\n  a = {:?}\n  b = {:?}\n  diffs = [{}, {}, {}, {}, {}, {}, {}, {}]",
        a, b, diff_s, diff_e12, diff_e13, diff_e23, diff_e01, diff_e02, diff_e03, diff_e0123
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 7: Motor Inverse Property**
    ///
    /// For any normalized Motor M, composing it with its inverse should yield the identity:
    /// M * M^-1 ≈ Identity (within numerical tolerance)
    ///
    /// In PGA, for a normalized motor, the inverse is the reverse (conjugate).
    /// This property verifies that the reverse operation correctly computes the inverse,
    /// and that the geometric product of a motor with its reverse yields the identity.
    ///
    /// **Validates: Requirements 2.7**
    #[test]
    fn motor_times_inverse_equals_identity(motor in motor_strategy()) {
        // Compute the inverse (reverse for normalized motors)
        let motor_inverse = motor.reverse();
        
        // Compute M * M^-1
        let result = motor.geometric_product(&motor_inverse);
        
        // The result should be approximately the identity motor
        assert_motors_approx_equal(&result, &Motor::IDENTITY, 1e-4);
    }

    /// Test that inverse times motor also equals identity (commutativity of inverse).
    #[test]
    fn inverse_times_motor_equals_identity(motor in motor_strategy()) {
        // Compute the inverse (reverse for normalized motors)
        let motor_inverse = motor.reverse();
        
        // Compute M^-1 * M
        let result = motor_inverse.geometric_product(&motor);
        
        // The result should be approximately the identity motor
        assert_motors_approx_equal(&result, &Motor::IDENTITY, 1e-4);
    }

    /// Test that pure translation inverse works correctly.
    #[test]
    fn translation_inverse_cancels(
        tx in -10.0f32..10.0,
        ty in -10.0f32..10.0,
        tz in -10.0f32..10.0,
    ) {
        let motor = Motor::from_translation(Vec3::new(tx, ty, tz).into());
        let motor_inverse = motor.reverse();
        
        let result = motor.geometric_product(&motor_inverse);
        
        assert_motors_approx_equal(&result, &Motor::IDENTITY, 1e-5);
    }

    /// Test that pure rotation inverse works correctly.
    #[test]
    fn rotation_inverse_cancels(
        ax in -1.0f32..1.0,
        ay in -1.0f32..1.0,
        az in -1.0f32..1.0,
        angle in -PI..PI,
    ) {
        let axis = Vec3::new(ax, ay, az);
        
        // Skip if axis is too small
        prop_assume!(axis.length_squared() > 1e-6);
        
        let motor = Motor::from_axis_angle(axis.normalize().into(), angle);
        let motor_inverse = motor.reverse();
        
        let result = motor.geometric_product(&motor_inverse);
        
        assert_motors_approx_equal(&result, &Motor::IDENTITY, 1e-5);
    }

    /// Test that applying a motor and its inverse to a point returns the original point.
    /// 
    /// This test verifies that the composition M^-1 * M applied to a point via the
    /// sandwich product returns the identity transformation.
    #[test]
    fn motor_inverse_restores_point(
        motor in motor_strategy(),
        px in -100.0f32..100.0,
        py in -100.0f32..100.0,
        pz in -100.0f32..100.0,
    ) {
        let point = Vec3::new(px, py, pz);
        let motor_inverse = motor.reverse();
        
        // Compose M^-1 * M to get identity
        let identity_motor = motor_inverse.geometric_product(&motor);
        
        // Apply the composed motor to the point
        let result = identity_motor.transform_point(point.into());
        
        // Should get back the original point (since identity_motor ≈ I)
        let diff = (Vec3::from(result) - point).length();
        assert!(
            diff < 1e-3,
            "Point not restored: original={:?}, result={:?}, diff={}, identity_motor={:?}",
            point,
            result,
            diff,
            identity_motor
        );
    }
}
