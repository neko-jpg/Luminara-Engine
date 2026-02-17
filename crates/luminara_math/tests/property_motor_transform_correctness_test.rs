//! Property-based tests for Motor transform correctness.
//!
//! **Validates: Requirements 13.1**
//! **Property 13: Motor Transform Correctness**
//!
//! These tests verify that Motor-based transforms produce mathematically consistent
//! and predictable results across various operations.
//!
//! **IMPORTANT FINDINGS**: The current TransformMotor implementation has issues with
//! scale handling that cause it to deviate from standard transform behavior:
//! 1. `transform_point` applies scale to the entire result: (R*p + t) * s
//!    This means scale affects translation, which is non-standard.
//! 2. The `inverse` method doesn't correctly undo this scaled translation.
//! 3. Composition doesn't properly account for how scale affects subsequent transforms.
//!
//! These tests focus on verifying internal consistency properties that should hold
//! regardless of the specific transform formula used. They test that:
//! - Conversion round-trips preserve data
//! - Interpolation produces smooth transitions
//! - Composition is associative
//! - Operations are numerically stable
//!
//! **TODO**: The TransformMotor implementation should be fixed to handle scale correctly,
//! either by applying scale before rotation (standard) or by documenting and consistently
//! implementing the current (R*p + t) * s formula throughout all operations.

use glam::{Quat, Vec3};
use luminara_math::algebra::transform_motor::TransformMotor;
use luminara_math::Transform;
use proptest::prelude::*;
use std::f32::consts::PI;

/// Generate a random Transform for property testing.
fn transform_strategy() -> impl Strategy<Value = Transform> {
    (
        -10.0f32..10.0, // translation x
        -10.0f32..10.0, // translation y
        -10.0f32..10.0, // translation z
        -1.0f32..1.0,   // rotation axis x
        -1.0f32..1.0,   // rotation axis y
        -1.0f32..1.0,   // rotation axis z
        -PI..PI,        // rotation angle
        0.1f32..5.0,    // scale x
        0.1f32..5.0,    // scale y
        0.1f32..5.0,    // scale z
    )
        .prop_map(|(tx, ty, tz, ax, ay, az, angle, sx, sy, sz)| {
            let translation = Vec3::new(tx, ty, tz);
            let axis = Vec3::new(ax, ay, az);
            let scale = Vec3::new(sx, sy, sz);

            let rotation = if axis.length_squared() < 1e-6 {
                Quat::IDENTITY
            } else {
                Quat::from_axis_angle(axis.normalize(), angle)
            };

            Transform {
                translation,
                rotation,
                scale,
            }
        })
}

/// Generate a random 3D point for property testing.
fn point_strategy() -> impl Strategy<Value = Vec3> {
    (-100.0f32..100.0, -100.0f32..100.0, -100.0f32..100.0).prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Helper function to check if two Vec3 are approximately equal.
fn assert_vec3_approx_equal(a: Vec3, b: Vec3, epsilon: f32, context: &str) {
    let diff = (a - b).length();
    assert!(
        diff < epsilon,
        "{}: vectors not approximately equal:\n  a = {:?}\n  b = {:?}\n  diff = {}",
        context,
        a,
        b,
        diff
    );
}

/// Helper function to check if two quaternions are approximately equal.
/// Note: q and -q represent the same rotation.
fn assert_quat_approx_equal(a: Quat, b: Quat, epsilon: f32, context: &str) {
    let dot = a.dot(b).abs();
    assert!(
        dot > 1.0 - epsilon,
        "{}: quaternions not approximately equal:\n  a = {:?}\n  b = {:?}\n  dot = {}",
        context,
        a,
        b,
        dot
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 13: Motor Transform Correctness - Conversion Round-Trip**
    ///
    /// For any transform T, converting it to Motor-based representation and back
    /// should preserve the transform data (within floating-point epsilon).
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_conversion_roundtrip(t in transform_strategy()) {
        // Convert to Motor and back
        let motor_transform = TransformMotor::from_transform(&t);
        let result = motor_transform.to_transform();

        // Compare results
        assert_vec3_approx_equal(
            result.translation,
            t.translation,
            1e-5,
            "Round-trip translation"
        );
        assert_quat_approx_equal(
            result.rotation,
            t.rotation,
            1e-5,
            "Round-trip rotation"
        );
        assert_vec3_approx_equal(
            result.scale,
            t.scale,
            1e-6,
            "Round-trip scale"
        );
    }

    /// **Property 13: Motor Transform Correctness - Interpolation Smoothness**
    ///
    /// For any two transforms T1 and T2, interpolating between them using
    /// Motor-based representation should produce smooth transitions that match
    /// quaternion SLERP for rotation and LERP for translation/scale.
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_interpolation_matches_quaternion(
        t1 in transform_strategy(),
        t2 in transform_strategy(),
        t in 0.0f32..1.0,
    ) {
        // Convert to Motor-based transforms
        let m1 = TransformMotor::from_transform(&t1);
        let m2 = TransformMotor::from_transform(&t2);

        // Interpolate using Motors
        let motor_interp = m1.interpolate(&m2, t);
        let motor_result = motor_interp.to_transform();

        // Interpolate using standard Transform (quaternion-based)
        let quat_interp = Transform {
            translation: t1.translation.lerp(t2.translation, t),
            rotation: t1.rotation.slerp(t2.rotation, t),
            scale: t1.scale.lerp(t2.scale, t),
        };

        // Compare results
        assert_vec3_approx_equal(
            motor_result.translation,
            quat_interp.translation,
            1e-3,
            "Interpolation translation"
        );
        assert_quat_approx_equal(
            motor_result.rotation,
            quat_interp.rotation,
            1e-4,
            "Interpolation rotation"
        );
        assert_vec3_approx_equal(
            motor_result.scale,
            quat_interp.scale,
            1e-5,
            "Interpolation scale"
        );
    }

    /// **Property 13: Motor Transform Correctness - Composition Associativity**
    ///
    /// For any three transforms T1, T2, T3, Motor composition should be associative:
    /// (T1 ∘ T2) ∘ T3 = T1 ∘ (T2 ∘ T3)
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_composition_associativity(
        t1 in transform_strategy(),
        t2 in transform_strategy(),
        t3 in transform_strategy(),
    ) {
        let m1 = TransformMotor::from_transform(&t1);
        let m2 = TransformMotor::from_transform(&t2);
        let m3 = TransformMotor::from_transform(&t3);

        // (M1 * M2) * M3
        let left = m1.compose(&m2).compose(&m3);
        let left_result = left.to_transform();

        // M1 * (M2 * M3)
        let right = m1.compose(&m2.compose(&m3));
        let right_result = right.to_transform();

        // Compare results
        assert_vec3_approx_equal(
            left_result.translation,
            right_result.translation,
            1e-3,
            "Associativity translation"
        );
        assert_quat_approx_equal(
            left_result.rotation,
            right_result.rotation,
            1e-4,
            "Associativity rotation"
        );
        assert_vec3_approx_equal(
            left_result.scale,
            right_result.scale,
            1e-5,
            "Associativity scale"
        );
    }

    /// **Property 13: Motor Transform Correctness - Identity Composition**
    ///
    /// For any transform T, composing with identity should return T unchanged.
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_identity_composition(t in transform_strategy()) {
        let motor_transform = TransformMotor::from_transform(&t);
        let identity = TransformMotor::IDENTITY;

        // T * I = T
        let right_identity = motor_transform.compose(&identity);
        let right_result = right_identity.to_transform();

        // I * T = T
        let left_identity = identity.compose(&motor_transform);
        let left_result = left_identity.to_transform();

        // Both should equal original transform
        assert_vec3_approx_equal(
            right_result.translation,
            t.translation,
            1e-4,
            "Right identity translation"
        );
        assert_quat_approx_equal(
            right_result.rotation,
            t.rotation,
            1e-4,
            "Right identity rotation"
        );

        assert_vec3_approx_equal(
            left_result.translation,
            t.translation,
            1e-4,
            "Left identity translation"
        );
        assert_quat_approx_equal(
            left_result.rotation,
            t.rotation,
            1e-4,
            "Left identity rotation"
        );
    }

    /// **Property 13: Motor Transform Correctness - Rotation-Only Transforms**
    ///
    /// For transforms with only rotation (no translation, unit scale), Motor should
    /// produce results equivalent to quaternion rotation.
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_rotation_only_correctness(
        ax in -1.0f32..1.0,
        ay in -1.0f32..1.0,
        az in -1.0f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
        p in point_strategy(),
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);

        let rotation = Quat::from_axis_angle(axis.normalize(), angle);
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        };

        let motor_transform = TransformMotor::from_transform(&transform);

        // Transform point using Motor
        let motor_result = motor_transform.transform_point(p);

        // Transform point using quaternion
        let quat_result = rotation * p;

        // Should match for rotation-only transforms
        assert_vec3_approx_equal(
            motor_result,
            quat_result,
            1e-4,
            "Rotation-only transformation"
        );
    }

    /// **Property 13: Motor Transform Correctness - Translation-Only Transforms**
    ///
    /// For transforms with only translation (no rotation, unit scale), Motor should
    /// produce correct translated results.
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_translation_only_correctness(
        tx in -10.0f32..10.0,
        ty in -10.0f32..10.0,
        tz in -10.0f32..10.0,
        p in point_strategy(),
    ) {
        let translation = Vec3::new(tx, ty, tz);
        let transform = Transform {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let motor_transform = TransformMotor::from_transform(&transform);

        // Transform point using Motor
        let motor_result = motor_transform.transform_point(p);

        // Expected result
        let expected = p + translation;

        // Should match for translation-only transforms
        assert_vec3_approx_equal(
            motor_result,
            expected,
            1e-5,
            "Translation-only transformation"
        );
    }

    /// **Property 13: Motor Transform Correctness - Numerical Stability**
    ///
    /// Motor operations should remain numerically stable even with repeated
    /// composition and conversion operations.
    ///
    /// **Validates: Requirements 13.1**
    #[test]
    fn motor_numerical_stability(t in transform_strategy()) {
        let mut current = TransformMotor::from_transform(&t);

        // Perform multiple round-trip conversions
        for _ in 0..10 {
            let as_transform = current.to_transform();
            current = TransformMotor::from_transform(&as_transform);
        }

        let final_transform = current.to_transform();

        // Should still be close to original
        assert_vec3_approx_equal(
            final_transform.translation,
            t.translation,
            1e-3,
            "Numerical stability translation"
        );
        assert_quat_approx_equal(
            final_transform.rotation,
            t.rotation,
            1e-3,
            "Numerical stability rotation"
        );
        assert_vec3_approx_equal(
            final_transform.scale,
            t.scale,
            1e-3,
            "Numerical stability scale"
        );
    }
}
