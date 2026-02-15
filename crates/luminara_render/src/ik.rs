use luminara_core::{Component, Entity};
use luminara_math::{Quat, Vec3};

pub struct TwoBoneIK {
    pub target: Entity,              // Entity to track
    pub pole_target: Option<Entity>, // Entity for pole vector (elbow direction)
    pub bone1: Entity,               // Upper arm / Thigh
    pub bone2: Entity,               // Lower arm / Calf
    pub end_effector: Entity,        // Hand / Foot
    pub chain_length: f32,           // Precomputed sum of bone lengths
    pub iterations: usize,
}

impl Component for TwoBoneIK {
    fn type_name() -> &'static str {
        "TwoBoneIK"
    }
}

pub struct TwoBoneIKSolver;

impl TwoBoneIKSolver {
    /// Solve 2-bone IK in local space.
    ///
    /// - `root`: Transform of the root bone (bone1) parent.
    /// - `bone1_len`: Length of the first bone.
    /// - `bone2_len`: Length of the second bone.
    /// - `target_pos`: Target position in root space.
    /// - `pole_vector`: Pole vector direction in root space.
    ///
    /// Returns: (Rotation for bone1, Rotation for bone2)
    pub fn solve(
        root_pos: Vec3,
        bone1_len: f32,
        bone2_len: f32,
        target_pos: Vec3,
        pole_vector: Vec3,
    ) -> (Quat, Quat) {
        // Geometric 2-Bone IK Solver

        let target_dir = target_pos - root_pos;
        let target_dist_sq = target_dir.length_squared();
        let target_dist = target_dist_sq.sqrt();

        // Clamp target distance to reachable range
        let max_dist = bone1_len + bone2_len;
        let clamped_dist = target_dist.clamp(0.001, max_dist - 0.001);

        // Law of Cosines
        // a = bone2_len, b = bone1_len, c = clamped_dist
        // cos(C) = (a^2 + b^2 - c^2) / 2ab
        // angle_at_elbow (internal) = acos((b1^2 + b2^2 - dist^2) / (2 * b1 * b2))

        let cos_elbow = (bone1_len * bone1_len + bone2_len * bone2_len
            - clamped_dist * clamped_dist)
            / (2.0 * bone1_len * bone2_len);
        let angle_elbow = cos_elbow.clamp(-1.0, 1.0).acos();

        // cos(A) = (b^2 + c^2 - a^2) / (2bc)
        // angle_at_shoulder_offset = acos((b1^2 + dist^2 - b2^2) / (2 * b1 * dist))
        let cos_shoulder = (bone1_len * bone1_len + clamped_dist * clamped_dist
            - bone2_len * bone2_len)
            / (2.0 * bone1_len * clamped_dist);
        let angle_shoulder = cos_shoulder.clamp(-1.0, 1.0).acos();

        // Construct Basis
        // We assume default bone direction is +Y (0, 1, 0)

        // 1. Rotate chain to point at target
        let look_rot = Quat::from_rotation_arc(Vec3::Y, target_dir.normalize());

        // 2. Resolve Pole Vector
        // Current UP vector after look_rot (assuming default UP was Z or X?)
        // Let's calculate the plane normal from (Root, Target, Pole)
        let plane_normal = target_dir.cross(pole_vector).normalize();

        // If target and pole are collinear, pick arbitrary normal
        let plane_normal = if plane_normal.length_squared() < 0.001 {
            // Find arbitrary perpendicular
            let v = if target_dir.normalize().dot(Vec3::Y).abs() > 0.99 {
                Vec3::X
            } else {
                Vec3::Y
            };
            target_dir.cross(v).normalize()
        } else {
            plane_normal
        };

        // Current 'Right' axis of the look_rot basis
        // Default Right for Y-up look_at is X?
        // Quat::from_rotation_arc takes shortest path.
        // It's ambiguous what the roll is.
        // We should explicitly construct basis.

        // Explicit Basis for Bone 1 (Root):
        // Y_axis = (Target - Root).normalize()  <-- Temporarily, before bending
        // Z_axis = plane_normal (The hinge axis)
        // X_axis = Y x Z

        let y_axis = target_dir.normalize();
        let z_axis = plane_normal; // This is the axis perpendicular to the bend plane
        let x_axis = y_axis.cross(z_axis).normalize();

        // Correct Z_axis to be orthogonal
        let z_axis = x_axis.cross(y_axis).normalize();

        // This rotation `basis_rot` aligns:
        // Local Y -> Target Dir
        // Local Z -> Plane Normal (Hinge Axis)
        // Local X -> In Plane Perpendicular

        let basis_rot = Quat::from_mat3(&luminara_math::glam::Mat3::from_cols(
            x_axis, y_axis, z_axis,
        ));

        // Now apply the shoulder offset angle.
        // Bone 1 rotates *away* from the target vector by `angle_shoulder`.
        // Rotation is around the Z_axis (Plane Normal).
        // Direction? "Up" in the plane corresponds to -X (since X = Y x Z).
        // Wait, X is in plane.
        // If we rotate around Z, we move Y towards X or -X.
        // Pole vector is roughly in the +X direction (or -X)?
        // Pole vector defined the plane normal Z. So pole is in XY plane of this basis.
        // Check `plane_normal = target x pole`.
        // If target=Y, pole=X -> Z = Y x X = -Z_world.
        // So Z points "away" from right hand rule.
        // The pole is to the "Right" (X cross Y)? No.
        // Pole is in the plane perpendicular to Z.
        // We want to bend *towards* the pole.
        // The pole vector projection on the plane should be "positive" bend direction?

        // Let's simplify: Rotate around Z axis by `angle_shoulder` or `-angle_shoulder`.
        // Let's assume we bend towards +X.
        // `basis_rot` puts Y at target.
        // Rotating around +Z moves Y towards -X. (Right Hand Rule: Thumb=Z, Fingers Y->X? No, Y->-X)
        // Wait: X (1,0,0), Y (0,1,0), Z (0,0,1).
        // Rot(Z, 90) * Y = (-1, 0, 0) = -X.
        // So positive rotation moves Y towards -X.

        // We want to move Bone1 away from TargetDir (Y) towards the pole.
        // Pole is somewhere in the plane perpendicular to Z.
        // In our basis, TargetDir is Y. Pole is in XY plane.
        // Where is Pole relative to X?
        // Z = Target x Pole.
        // Target = Y.
        // Y x Pole_proj = Z.
        // If Pole_proj is X, Y x X = -Z. != Z.
        // If Pole_proj is -X, Y x -X = Z. Correct.
        // So Pole is in the -X direction in this basis.

        // So we want to rotate Y towards -X.
        // That is a positive rotation around Z.
        let shoulder_bend = Quat::from_rotation_z(angle_shoulder);

        // Note: Quat multiplication is applied right-to-left in code but means `basis_rot` *then* `shoulder_bend`?
        // No, `q2 * q1` means apply q1 then q2.
        // We want to apply shoulder bend in the LOCAL space established by basis_rot.
        // So: Global = Basis * Local.
        // bone1_rot = basis_rot * shoulder_bend.
        let bone1_rot = basis_rot * shoulder_bend;

        // Bone 2 relative rotation
        // Bone 2 bends back towards the target to close the triangle.
        // Elbow angle `angle_elbow` is the internal angle.
        // Deflection is 180 - internal.
        // We need to rotate around Z axis (hinge) in the opposite direction.
        // Bone 1 is at `angle_shoulder` from Target Line.
        // We need to rotate by `PI - angle_elbow`? No.
        // We rotate by `- (PI - angle_elbow)`? Or just `- deflection`.
        // Let's visualize:
        // Root -> (rot +angle) -> Elbow.
        // To get back to target, we need to turn "in".
        // That corresponds to a negative rotation around Z.
        // Angle magnitude = PI - angle_elbow.

        let deflection = std::f32::consts::PI - angle_elbow;
        let elbow_bend = Quat::from_rotation_z(-deflection);

        // Bone2 Global = Bone1 Global * ElbowLocal
        let bone2_rot = bone1_rot * elbow_bend;

        (bone1_rot, bone2_rot)
    }
}
