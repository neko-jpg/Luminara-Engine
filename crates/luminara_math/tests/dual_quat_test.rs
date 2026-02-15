use glam::{Quat, Vec3};
use luminara_math::algebra::DualQuat;
use proptest::prelude::*;

fn assert_vec3_eq(a: Vec3, b: Vec3, epsilon: f32) {
    let diff = (a - b).abs();
    assert!(
        diff.x < epsilon && diff.y < epsilon && diff.z < epsilon,
        "Vec3 mismatch: {:?} != {:?} (epsilon {})",
        a,
        b,
        epsilon
    );
}

// Helper to generate random dual quat
prop_compose! {
    fn arb_dual_quat()(
        axis_x in -1.0f32..1.0,
        axis_y in -1.0f32..1.0,
        axis_z in -1.0f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
        tx in -10.0f32..10.0,
        ty in -10.0f32..10.0,
        tz in -10.0f32..10.0,
    ) -> DualQuat {
        let axis = Vec3::new(axis_x, axis_y, axis_z);
        let axis = if axis.length_squared() < 1e-6 { Vec3::Z } else { axis.normalize() };
        let rot = Quat::from_axis_angle(axis, angle);
        let trans = Vec3::new(tx, ty, tz);
        DualQuat::from_rotation_translation(rot, trans)
    }
}

proptest! {
    // Property 11: Dual Quaternion Blend Normalization
    // Validates: Requirements 4.2
    // Checks that the result of blending is always normalized (unit magnitude real part).
    #[test]
    fn prop_blend_normalization(q1 in arb_dual_quat(), q2 in arb_dual_quat(), t in 0.0f32..1.0) {
        let blended = q1.blend(&q2, t);

        // Magnitude of real part should be 1
        prop_assert!((blended.real.length() - 1.0).abs() < 1e-4);

        // Dot product of real and dual part should be 0 (rigid transform condition)
        // Wait, DLB doesn't strictly preserve rigid transform (screw motion)
        // DLB is an approximation (ScLERP is exact).
        // But normalize() enforces orthogonality if implemented correctly?
        // My implementation:
        // let dual_norm = dual_div - real_norm * dot;
        // So yes, it enforces orthogonality.
        prop_assert!(blended.real.dot(blended.dual).abs() < 1e-4);
    }

    // Property 12: Dual Quaternion Shortest Path
    // Validates: Requirements 4.3
    // Checks that blending takes the shortest path on the rotation hypersphere.
    // If dot(q1, q2) < 0, it should use -q2.
    // This is tested by checking continuity? Or directly checking sign handling?
    #[test]
    fn prop_shortest_path(q1 in arb_dual_quat(), q2 in arb_dual_quat()) {
        let t = 0.5;
        let blended = q1.blend(&q2, t);

        // If q1 and q2 are opposite, dot < 0.
        // Blended should be "between" q1 and -q2.
        // So dot(blended, q1) should be positive?
        // Yes, if we interpolate between q1 and something close to q1, dot > 0.
        // If we interpolated between q1 and q2 (where dot < 0), we would go the long way, passing through 0?
        // With shortest path, we flip q2 to -q2 so that dot(q1, -q2) > 0.
        // Then interpolation stays in the hemisphere where dot > 0.

        prop_assert!(blended.real.dot(q1.real) >= -1e-4);

        // Also check endpoint q2 (or -q2)
        let _dot2 = blended.real.dot(q2.real);
        // dot2 magnitude should be consistent
        // If we flipped q2, dot2 might be negative.
        // But abs(dot2) should be reasonable.
    }
}

#[test]
fn test_dual_quat_identity() {
    let dq = DualQuat::IDENTITY;
    let p = Vec3::new(1.0, 2.0, 3.0);
    let p_prime = dq.transform_point(p);
    assert_vec3_eq(p, p_prime, 1e-6);
}

#[test]
fn test_dual_quat_transform() {
    let t = Vec3::new(10.0, -5.0, 0.5);
    let r = Quat::from_rotation_z(std::f32::consts::PI / 2.0);
    let dq = DualQuat::from_rotation_translation(r, t);

    let p = Vec3::new(1.0, 0.0, 0.0);
    let p_prime = dq.transform_point(p);

    // Rotate (1,0,0) -> (0,1,0). Then add (10, -5, 0.5) -> (10, -4, 0.5)
    assert_vec3_eq(p_prime, Vec3::new(10.0, -4.0, 0.5), 1e-5);
}

#[test]
fn test_dual_quat_blend_endpoints() {
    let q1 = DualQuat::IDENTITY;
    let t = Vec3::new(1.0, 0.0, 0.0);
    let q2 = DualQuat::from_rotation_translation(Quat::IDENTITY, t);

    // t=0
    let b0 = q1.blend(&q2, 0.0);
    assert_vec3_eq(b0.transform_point(Vec3::ZERO), Vec3::ZERO, 1e-5);

    // t=1
    let b1 = q1.blend(&q2, 1.0);
    assert_vec3_eq(b1.transform_point(Vec3::ZERO), t, 1e-5);

    // t=0.5
    let b05 = q1.blend(&q2, 0.5);
    assert_vec3_eq(b05.transform_point(Vec3::ZERO), t * 0.5, 1e-5);
}
