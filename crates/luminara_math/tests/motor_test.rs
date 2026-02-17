use luminara_math::algebra::transform_motor::TransformMotor;
use luminara_math::{Quat, Vec3};

// Simplified propagation system test structure
// We just test TransformMotor logic directly since ECS integration depends on core

#[test]
fn test_motor_transform_composition() {
    // T1: Translate +X 10
    let t1 = TransformMotor::from_translation(Vec3::X * 10.0);
    // T2: Rotate 90 deg around Y
    let t2 = TransformMotor::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));

    let combined = t1.compose(&t2);

    let (rot, trans) = combined.to_rotation_translation();

    // Validate transformation consistency
    // Transform origin (0,0,0) to verify translation part
    use luminara_math::algebra::vector::Vector3;
    let p_origin = Vector3::new(0.0, 0.0, 0.0);
    let p_transformed = combined.motor.transform_point(p_origin);
    let p_vec = Vec3::new(p_transformed.x, p_transformed.y, p_transformed.z);

    // Debugging info
    println!("Extracted translation: {:?}", trans);
    println!("Transformed origin: {:?}", p_vec);

    let p_vec_len = p_vec.length();
    assert!(
        (p_vec_len - 10.0).abs() < 0.001,
        "Translation magnitude should be 10.0"
    );

    // Verify rotation is 90 deg Y
    let expected_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let dot = rot.dot(expected_rot).abs();
    assert!((dot - 1.0).abs() < 0.001);
}

#[test]
fn test_motor_transform_associativity() {
    let t1 = TransformMotor::from_translation(Vec3::X * 10.0);
    let t2 = TransformMotor::from_rotation(Quat::from_rotation_z(0.5));
    let t3 = TransformMotor::from_translation(Vec3::Y * 5.0);

    let c1 = t1.compose(&t2).compose(&t3);
    let c2 = t1.compose(&t2.compose(&t3));

    let (r1, tr1) = c1.to_rotation_translation();
    let (r2, tr2) = c2.to_rotation_translation();

    assert!((tr1 - tr2).length() < 0.001);
    assert!((r1.w - r2.w).abs() < 0.001);
}
