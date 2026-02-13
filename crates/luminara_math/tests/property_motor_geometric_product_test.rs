//! Property-based tests for Motor geometric product associativity.
//!
//! **Validates: Requirements 2.2**
//! **Property 5: Motor Geometric Product Associativity**

use luminara_math::algebra::Motor;
use glam::Vec3;
use proptest::prelude::*;
use std::f32::consts::PI;

/// Generate a random motor for property testing.
fn motor_strategy() -> impl Strategy<Value = Motor> {
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
            Motor::from_translation(trans)
        } else {
            let axis_normalized = axis.normalize();
            let rot = Motor::from_axis_angle(axis_normalized, angle);
            let trans_motor = Motor::from_translation(trans);
            trans_motor.geometric_product(&rot)
        }
    })
}

/// Helper function to check if two motors are approximately equal.
fn assert_motors_approx_equal(a: &Motor, b: &Motor, epsilon: f32) {
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

    /// **Property 5: Motor Geometric Product Associativity**
    ///
    /// For any three Motors M1, M2, M3, the geometric product should be associative:
    /// (M1 * M2) * M3 = M1 * (M2 * M3)
    ///
    /// **Validates: Requirements 2.2**
    #[test]
    fn motor_geometric_product_is_associative(
        m1 in motor_strategy(),
        m2 in motor_strategy(),
        m3 in motor_strategy(),
    ) {
        // Compute (M1 * M2) * M3
        let left = m1.geometric_product(&m2).geometric_product(&m3);
        
        // Compute M1 * (M2 * M3)
        let right = m1.geometric_product(&m2.geometric_product(&m3));
        
        // They should be approximately equal
        assert_motors_approx_equal(&left, &right, 1e-4);
    }

    /// Test that identity is a neutral element for geometric product.
    #[test]
    fn motor_identity_is_neutral(m in motor_strategy()) {
        let left = m.geometric_product(&Motor::IDENTITY);
        let right = Motor::IDENTITY.geometric_product(&m);
        
        assert_motors_approx_equal(&left, &m, 1e-5);
        assert_motors_approx_equal(&right, &m, 1e-5);
    }
}
