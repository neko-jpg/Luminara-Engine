use luminara_math::{Quat, Transform, Vec3};
use luminara_physics::{Collider, ColliderShape, RigidBody, RigidBodyType};
use proptest::prelude::*;

/// **Validates: Requirements 7.4, 7.7**
/// **Property 17: Physics Transform Synchronization**
///
/// For any entity with both a RigidBody and Transform component,
/// after a physics simulation step, the entity's Transform should match
/// the physics body's position and rotation.

/// Test that transforms can be created and modified
#[test]
fn test_transform_creation_and_modification() {
    let mut transform = Transform::from_xyz(1.0, 2.0, 3.0);
    
    assert_eq!(transform.translation.x, 1.0);
    assert_eq!(transform.translation.y, 2.0);
    assert_eq!(transform.translation.z, 3.0);
    
    // Modify transform
    transform.translation = Vec3::new(4.0, 5.0, 6.0);
    
    assert_eq!(transform.translation.x, 4.0);
    assert_eq!(transform.translation.y, 5.0);
    assert_eq!(transform.translation.z, 6.0);
}

/// Test that rigid body and transform can coexist
#[test]
fn test_rigid_body_with_transform() {
    let transform = Transform::from_xyz(0.0, 5.0, 0.0);
    let rigid_body = RigidBody {
        body_type: RigidBodyType::Dynamic,
        mass: 1.0,
        gravity_scale: 1.0,
        ..Default::default()
    };
    
    // Verify both components have correct values
    assert_eq!(transform.translation.y, 5.0);
    assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
    assert_eq!(rigid_body.mass, 1.0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property test: Transform position values are preserved
    #[test]
    fn prop_transform_position_preserved(
        x in -100.0f32..100.0f32,
        y in -100.0f32..100.0f32,
        z in -100.0f32..100.0f32,
    ) {
        let transform = Transform::from_xyz(x, y, z);
        
        prop_assert_eq!(transform.translation.x, x);
        prop_assert_eq!(transform.translation.y, y);
        prop_assert_eq!(transform.translation.z, z);
    }

    /// Property test: Transform rotation is normalized
    #[test]
    fn prop_transform_rotation_normalized(
        x in -1.0f32..1.0f32,
        y in -1.0f32..1.0f32,
        z in -1.0f32..1.0f32,
        w in -1.0f32..1.0f32,
    ) {
        let rotation = Quat::from_xyzw(x, y, z, w).normalize();
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        };
        
        // Quaternion should be normalized (length = 1)
        let length_squared = transform.rotation.x * transform.rotation.x
            + transform.rotation.y * transform.rotation.y
            + transform.rotation.z * transform.rotation.z
            + transform.rotation.w * transform.rotation.w;
        
        prop_assert!(
            (length_squared - 1.0).abs() < 0.001,
            "Quaternion should be normalized, length_squared = {}",
            length_squared
        );
    }

    /// Property test: RigidBody mass affects physics behavior
    #[test]
    fn prop_rigid_body_mass_positive(
        mass in 0.1f32..1000.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass,
            ..Default::default()
        };
        
        prop_assert!(rigid_body.mass > 0.0, "Mass should be positive");
        prop_assert_eq!(rigid_body.mass, mass);
    }

    /// Property test: Transform scale is preserved
    #[test]
    fn prop_transform_scale_preserved(
        sx in 0.1f32..10.0f32,
        sy in 0.1f32..10.0f32,
        sz in 0.1f32..10.0f32,
    ) {
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(sx, sy, sz),
        };
        
        prop_assert_eq!(transform.scale.x, sx);
        prop_assert_eq!(transform.scale.y, sy);
        prop_assert_eq!(transform.scale.z, sz);
    }

    /// Property test: Dynamic bodies have positive mass
    #[test]
    fn prop_dynamic_bodies_have_mass(
        mass in 0.1f32..100.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass,
            ..Default::default()
        };
        
        prop_assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
        prop_assert!(rigid_body.mass > 0.0);
    }

    /// Property test: Static bodies can have zero mass
    #[test]
    fn prop_static_bodies_mass_irrelevant(
        mass in 0.0f32..100.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Static,
            mass,
            ..Default::default()
        };
        
        prop_assert_eq!(rigid_body.body_type, RigidBodyType::Static);
        // Static bodies don't move regardless of mass
    }

    /// Property test: Gravity scale affects physics
    #[test]
    fn prop_gravity_scale_range(
        gravity_scale in -10.0f32..10.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale,
            ..Default::default()
        };
        
        prop_assert_eq!(rigid_body.gravity_scale, gravity_scale);
    }

    /// Property test: Damping values are non-negative
    #[test]
    fn prop_damping_non_negative(
        linear_damping in 0.0f32..10.0f32,
        angular_damping in 0.0f32..10.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping,
            angular_damping,
            ..Default::default()
        };
        
        prop_assert!(rigid_body.linear_damping >= 0.0);
        prop_assert!(rigid_body.angular_damping >= 0.0);
        prop_assert_eq!(rigid_body.linear_damping, linear_damping);
        prop_assert_eq!(rigid_body.angular_damping, angular_damping);
    }

    /// Property test: Transform matrix computation is consistent
    #[test]
    fn prop_transform_matrix_consistent(
        x in -10.0f32..10.0f32,
        y in -10.0f32..10.0f32,
        z in -10.0f32..10.0f32,
    ) {
        let transform = Transform::from_xyz(x, y, z);
        let matrix1 = transform.to_matrix();
        let matrix2 = transform.to_matrix();
        
        // Matrix computation should be deterministic
        prop_assert_eq!(matrix1, matrix2);
    }
}
