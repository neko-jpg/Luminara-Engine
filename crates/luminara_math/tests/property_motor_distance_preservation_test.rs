//! Property-based tests for Motor point transformation distance preservation.
//!
//! **Validates: Requirements 2.4**
//! **Property 6: Motor Point Transformation Preserves Distance**

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

/// Generate a random 3D point for property testing.
fn point_strategy() -> impl Strategy<Value = Vec3> {
    (-100.0f32..100.0, -100.0f32..100.0, -100.0f32..100.0)
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 6: Motor Point Transformation Preserves Distance**
    ///
    /// For any Motor M representing a rigid transformation and any two points p1, p2,
    /// the distance between the transformed points should equal the distance between
    /// the original points: |M(p1) - M(p2)| = |p1 - p2|
    ///
    /// This property verifies that motors represent isometric transformations (rigid
    /// body transformations that preserve distances and angles).
    ///
    /// **Validates: Requirements 2.4**
    #[test]
    fn motor_transformation_preserves_distance(
        motor in motor_strategy(),
        p1 in point_strategy(),
        p2 in point_strategy(),
    ) {
        // Compute original distance
        let original_distance = p1.distance(p2);
        
        // Transform both points
        let p1_transformed = motor.transform_point(p1);
        let p2_transformed = motor.transform_point(p2);
        
        // Compute transformed distance
        let transformed_distance = p1_transformed.distance(p2_transformed);
        
        // The distances should be equal (within floating-point tolerance)
        let relative_error = if original_distance > 1e-6 {
            ((transformed_distance - original_distance) / original_distance).abs()
        } else {
            (transformed_distance - original_distance).abs()
        };
        
        assert!(
            relative_error < 1e-4,
            "Distance not preserved:\n  Original distance: {}\n  Transformed distance: {}\n  Relative error: {}\n  Motor: {:?}\n  p1: {:?}\n  p2: {:?}",
            original_distance,
            transformed_distance,
            relative_error,
            motor,
            p1,
            p2
        );
    }

    /// Test that pure translation preserves distance.
    #[test]
    fn translation_preserves_distance(
        tx in -10.0f32..10.0,
        ty in -10.0f32..10.0,
        tz in -10.0f32..10.0,
        p1 in point_strategy(),
        p2 in point_strategy(),
    ) {
        let motor = Motor::from_translation(Vec3::new(tx, ty, tz));
        
        let original_distance = p1.distance(p2);
        let p1_transformed = motor.transform_point(p1);
        let p2_transformed = motor.transform_point(p2);
        let transformed_distance = p1_transformed.distance(p2_transformed);
        
        let relative_error = if original_distance > 1e-6 {
            ((transformed_distance - original_distance) / original_distance).abs()
        } else {
            (transformed_distance - original_distance).abs()
        };
        
        assert!(
            relative_error < 1e-4,
            "Translation did not preserve distance: original={}, transformed={}, relative_error={}",
            original_distance,
            transformed_distance,
            relative_error
        );
    }

    /// Test that pure rotation preserves distance.
    #[test]
    fn rotation_preserves_distance(
        ax in -1.0f32..1.0,
        ay in -1.0f32..1.0,
        az in -1.0f32..1.0,
        angle in -PI..PI,
        p1 in point_strategy(),
        p2 in point_strategy(),
    ) {
        let axis = Vec3::new(ax, ay, az);
        
        // Skip if axis is too small
        prop_assume!(axis.length_squared() > 1e-6);
        
        let motor = Motor::from_axis_angle(axis.normalize(), angle);
        
        let original_distance = p1.distance(p2);
        let p1_transformed = motor.transform_point(p1);
        let p2_transformed = motor.transform_point(p2);
        let transformed_distance = p1_transformed.distance(p2_transformed);
        
        let relative_error = if original_distance > 1e-6 {
            ((transformed_distance - original_distance) / original_distance).abs()
        } else {
            (transformed_distance - original_distance).abs()
        };
        
        assert!(
            relative_error < 1e-4,
            "Rotation did not preserve distance: original={}, transformed={}, relative_error={}",
            original_distance,
            transformed_distance,
            relative_error
        );
    }
}
