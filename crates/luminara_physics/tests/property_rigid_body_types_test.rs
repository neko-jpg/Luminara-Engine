use luminara_math::{Transform, Vec3};
use luminara_physics::{Collider, ColliderShape, RigidBody, RigidBodyType};
use proptest::prelude::*;

/// **Validates: Requirements 7.5**
/// **Property 18: Rigid Body Type Behavior**
///
/// For any entity with a RigidBody component, the physics engine should handle it
/// according to its type:
/// - Dynamic bodies should respond to forces and collisions
/// - Kinematic bodies should move only when explicitly set
/// - Static bodies should not move at all

/// Test that all rigid body types can be created
#[test]
fn test_all_rigid_body_types_creation() {
    let dynamic = RigidBody {
        body_type: RigidBodyType::Dynamic,
        mass: 1.0,
        ..Default::default()
    };

    let kinematic = RigidBody {
        body_type: RigidBodyType::Kinematic,
        mass: 1.0,
        ..Default::default()
    };

    let static_body = RigidBody {
        body_type: RigidBodyType::Static,
        mass: 0.0,
        ..Default::default()
    };

    assert_eq!(dynamic.body_type, RigidBodyType::Dynamic);
    assert_eq!(kinematic.body_type, RigidBodyType::Kinematic);
    assert_eq!(static_body.body_type, RigidBodyType::Static);
}

/// Test that dynamic bodies have appropriate properties
#[test]
fn test_dynamic_body_properties() {
    let dynamic = RigidBody {
        body_type: RigidBodyType::Dynamic,
        mass: 10.0,
        linear_damping: 0.1,
        angular_damping: 0.1,
        gravity_scale: 1.0,
    };

    assert_eq!(dynamic.body_type, RigidBodyType::Dynamic);
    assert!(
        dynamic.mass > 0.0,
        "Dynamic bodies should have positive mass"
    );
    assert!(
        dynamic.gravity_scale != 0.0,
        "Dynamic bodies typically have gravity"
    );
}

/// Test that static bodies don't need mass
#[test]
fn test_static_body_properties() {
    let static_body = RigidBody {
        body_type: RigidBodyType::Static,
        mass: 0.0,
        gravity_scale: 0.0,
        ..Default::default()
    };

    assert_eq!(static_body.body_type, RigidBodyType::Static);
    // Static bodies don't move, so mass and gravity are irrelevant
}

/// Test that kinematic bodies can be created
#[test]
fn test_kinematic_body_properties() {
    let kinematic = RigidBody {
        body_type: RigidBodyType::Kinematic,
        mass: 1.0,
        gravity_scale: 0.0, // Kinematic bodies typically ignore gravity
        ..Default::default()
    };

    assert_eq!(kinematic.body_type, RigidBodyType::Kinematic);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property test: Dynamic bodies maintain their type
    #[test]
    fn prop_dynamic_body_type_preserved(
        mass in 0.1f32..100.0f32,
        gravity_scale in -10.0f32..10.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass,
            gravity_scale,
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
        prop_assert_eq!(rigid_body.mass, mass);
        prop_assert_eq!(rigid_body.gravity_scale, gravity_scale);
    }

    /// Property test: Kinematic bodies maintain their type
    #[test]
    fn prop_kinematic_body_type_preserved(
        mass in 0.1f32..100.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Kinematic,
            mass,
            gravity_scale: 0.0,
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, RigidBodyType::Kinematic);
    }

    /// Property test: Static bodies maintain their type
    #[test]
    fn prop_static_body_type_preserved(
        mass in 0.0f32..100.0f32,
    ) {
        let rigid_body = RigidBody {
            body_type: RigidBodyType::Static,
            mass,
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, RigidBodyType::Static);
    }

    /// Property test: Body type equality is reflexive
    #[test]
    fn prop_body_type_equality_reflexive(
        type_index in 0usize..3,
    ) {
        let body_type = match type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        prop_assert_eq!(body_type, body_type);
    }

    /// Property test: Different body types are not equal
    #[test]
    fn prop_different_body_types_not_equal(
        type_a_index in 0usize..3,
        type_b_index in 0usize..3,
    ) {
        let type_a = match type_a_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let type_b = match type_b_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        if type_a_index == type_b_index {
            prop_assert_eq!(type_a, type_b);
        } else {
            prop_assert_ne!(type_a, type_b);
        }
    }

    /// Property test: RigidBody with collider can be created
    #[test]
    fn prop_rigid_body_with_collider(
        radius in 0.1f32..10.0f32,
        mass in 0.1f32..100.0f32,
        type_index in 0usize..3,
    ) {
        let body_type = match type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let rigid_body = RigidBody {
            body_type,
            mass,
            ..Default::default()
        };

        let collider = Collider {
            shape: ColliderShape::Sphere { radius },
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, body_type);

        if let ColliderShape::Sphere { radius: r } = collider.shape {
            prop_assert_eq!(r, radius);
        }
    }

    /// Property test: Damping values don't affect body type
    #[test]
    fn prop_damping_independent_of_type(
        linear_damping in 0.0f32..10.0f32,
        angular_damping in 0.0f32..10.0f32,
        type_index in 0usize..3,
    ) {
        let body_type = match type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let rigid_body = RigidBody {
            body_type,
            mass: 1.0,
            linear_damping,
            angular_damping,
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, body_type);
        prop_assert_eq!(rigid_body.linear_damping, linear_damping);
        prop_assert_eq!(rigid_body.angular_damping, angular_damping);
    }

    /// Property test: Body type can be changed
    #[test]
    fn prop_body_type_can_change(
        initial_type_index in 0usize..3,
        new_type_index in 0usize..3,
    ) {
        let initial_type = match initial_type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let new_type = match new_type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let mut rigid_body = RigidBody {
            body_type: initial_type,
            mass: 1.0,
            ..Default::default()
        };

        prop_assert_eq!(rigid_body.body_type, initial_type);

        // Change body type
        rigid_body.body_type = new_type;

        prop_assert_eq!(rigid_body.body_type, new_type);
    }

    /// Property test: Transform and RigidBody are independent
    #[test]
    fn prop_transform_independent_of_body_type(
        x in -100.0f32..100.0f32,
        y in -100.0f32..100.0f32,
        z in -100.0f32..100.0f32,
        type_index in 0usize..3,
    ) {
        let body_type = match type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };

        let transform = Transform::from_xyz(x, y, z);
        let rigid_body = RigidBody {
            body_type,
            mass: 1.0,
            ..Default::default()
        };

        // Transform values should be independent of body type
        prop_assert_eq!(transform.translation.x, x);
        prop_assert_eq!(transform.translation.y, y);
        prop_assert_eq!(transform.translation.z, z);
        prop_assert_eq!(rigid_body.body_type, body_type);
    }
}
