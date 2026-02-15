use luminara_math::{Quat, Vec3};
use luminara_render::ik::TwoBoneIKSolver;

#[test]
fn test_ik_solver_reach() {
    let root_pos = Vec3::ZERO;
    let bone1_len = 1.0;
    let bone2_len = 1.0;

    // Target straight ahead: (0, 2, 0)
    let target = Vec3::new(0.0, 2.0, 0.0);
    let pole = Vec3::Z;

    let (q1, q2) = TwoBoneIKSolver::solve(root_pos, bone1_len, bone2_len, target, pole);

    // q1 should align Y to Target (Y)
    let y = Vec3::Y;
    let y1 = q1 * y;

    // With our explicit basis:
    // Target = Y. Pole = Z.
    // Y_axis = Y. Z_axis = Y x Z? No, plane normal = Y x Z = X.
    // Z_axis (plane) = X.
    // X_axis = Y x X = -Z.
    // Basis: X'=-Z, Y'=Y, Z'=X.
    // basis_rot transforms local Y to Y.
    // Shoulder angle is 0 (dist = 2 = b1+b2).
    // So q1 = basis_rot.
    // q1 * Y = Y.
    // q2 angle = 0.

    assert!((y1 - y).length() < 0.001, "Bone1 should point along Y");
    assert!(
        (q2 * Vec3::Y - Vec3::Y).length() < 0.001,
        "Bone2 should point along Y (relative)"
    );
}

#[test]
fn test_ik_solver_bend() {
    let root_pos = Vec3::ZERO;
    let bone1_len = 1.0;
    let bone2_len = 1.0;

    // Target at (0, 1, 0).
    let target = Vec3::new(0.0, 1.0, 0.0);
    // Pole at (1, 1, 0) -> Bend towards +X
    let pole = Vec3::new(1.0, 1.0, 0.0);

    let (q1, q2) = TwoBoneIKSolver::solve(root_pos, bone1_len, bone2_len, target, pole);

    let p1 = q1 * Vec3::Y * bone1_len;
    let p2 = p1 + (q1 * q2) * Vec3::Y * bone2_len;

    println!("Target: {:?}", target);
    println!("P2: {:?}", p2);
    println!("P1: {:?}", p1);

    assert!((p2 - target).length() < 0.01);

    // Check if P1 bends towards pole (X > 0)
    assert!(p1.x > 0.0);
}
