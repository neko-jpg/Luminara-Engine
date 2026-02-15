use luminara_math::algebra::vector::Vector3;
use luminara_math::algebra::{Bivector, Motor};
use luminara_physics::LiePhysicsIntegrator;

#[test]
fn test_lie_integration_constant_velocity() {
    let mut motor = Motor::IDENTITY;

    // Constant velocity: 1 rad/s around Y axis
    let velocity = Bivector::new(
        0.0, 0.0, 0.0, // e12, e13, e23 (rotation xy, xz, yz -> z, y, x)
        // e12=Z, e13=Y, e23=X? No.
        // e12=Z, e13=-Y, e23=X.
        // Rot Y axis: e13
        0.0, 1.0, 0.0, // e13 = 1.0
    );
    // Bivector::new arguments: e12, e13, e23, e01, e02, e03
    // Wait, Bivector fields are e12, e13, e23, e01, e02, e03?
    // Let's check definition.
    // In `luminara_math/src/algebra/bivector.rs`:
    // pub struct Bivector<T> { pub e12: T, pub e13: T, pub e23: T, pub e01: T, pub e02: T, pub e03: T }

    // Rotation Y axis corresponds to e13 (XZ plane) or e31?
    // Usually e31 = -e13.
    // If Y is up, rotation in XZ plane.

    let rot_y_velocity = Bivector::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);

    // Integrate for 1 second with dt = 0.1
    let dt = 0.1;
    for _ in 0..10 {
        LiePhysicsIntegrator::step(&mut motor, &rot_y_velocity, dt);
    }

    // Should rotate approximately 1 radian around Y
    // (s, e12, e13, e23)
    // q = cos(0.5) + sin(0.5) * e13
    let half_angle: f32 = 0.5; // 1.0 radian / 2
    let expected_s = half_angle.cos();
    let expected_e13 = half_angle.sin();

    assert!((motor.s - expected_s).abs() < 0.01);
    assert!((motor.e13 - expected_e13).abs() < 0.01);
}

#[test]
fn test_energy_conservation() {
    // Rotating object should maintain unit norm of motor (which relates to energy stability)
    let mut motor = Motor::IDENTITY;
    let velocity = Bivector::new(1.0, 2.0, 3.0, 0.0, 0.0, 0.0); // Random rotation

    let dt = 0.01;
    for _ in 0..100 {
        LiePhysicsIntegrator::step(&mut motor, &velocity, dt);
    }

    // Check normalization (Motor::normalize() is called in step, verifying it works)
    let norm_sq =
        motor.s * motor.s + motor.e12 * motor.e12 + motor.e13 * motor.e13 + motor.e23 * motor.e23;
    assert!((norm_sq - 1.0).abs() < 0.0001);
}
