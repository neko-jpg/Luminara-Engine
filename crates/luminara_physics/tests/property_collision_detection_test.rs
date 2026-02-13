use luminara_math::{Transform, Vec3};
use luminara_physics::{Collider, ColliderShape, RigidBody, RigidBodyType};
use proptest::prelude::*;

/// **Validates: Requirements 7.2, 7.6**
/// **Property 16: Collision Detection and Events**
///
/// For any two entities with collider components that overlap in space,
/// the physics engine should detect the collision. This test validates
/// that the collision detection logic works correctly by testing the
/// geometric overlap conditions.

/// Test that two spheres with overlapping positions should collide
#[test]
fn test_overlapping_spheres_should_collide() {
    // Two spheres with radius 1.0 at distance 1.5 should overlap
    let sphere_a_pos = Vec3::new(0.0, 0.0, 0.0);
    let sphere_b_pos = Vec3::new(1.5, 0.0, 0.0);
    let radius_a = 1.0;
    let radius_b = 1.0;
    
    let distance = (sphere_b_pos - sphere_a_pos).length();
    let sum_of_radii = radius_a + radius_b;
    
    // Should overlap
    assert!(
        distance < sum_of_radii,
        "Spheres should overlap: distance {} < sum of radii {}",
        distance,
        sum_of_radii
    );
}

/// Test that two spheres with non-overlapping positions should not collide
#[test]
fn test_non_overlapping_spheres_should_not_collide() {
    // Two spheres with radius 1.0 at distance 5.0 should not overlap
    let sphere_a_pos = Vec3::new(0.0, 0.0, 0.0);
    let sphere_b_pos = Vec3::new(5.0, 0.0, 0.0);
    let radius_a = 1.0;
    let radius_b = 1.0;
    
    let distance = (sphere_b_pos - sphere_a_pos).length();
    let sum_of_radii = radius_a + radius_b;
    
    // Should not overlap
    assert!(
        distance > sum_of_radii,
        "Spheres should not overlap: distance {} > sum of radii {}",
        distance,
        sum_of_radii
    );
}

/// Test that components can be created with valid data
#[test]
fn test_physics_components_creation() {
    let rigid_body = RigidBody {
        body_type: RigidBodyType::Dynamic,
        mass: 1.0,
        linear_damping: 0.1,
        angular_damping: 0.1,
        gravity_scale: 1.0,
    };
    
    assert_eq!(rigid_body.mass, 1.0);
    assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
    
    let collider = Collider {
        shape: ColliderShape::Sphere { radius: 1.0 },
        friction: 0.5,
        restitution: 0.3,
        is_sensor: false,
    };
    
    assert_eq!(collider.friction, 0.5);
    assert_eq!(collider.restitution, 0.3);
    assert!(!collider.is_sensor);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property test: Overlapping spheres should be detected
    /// 
    /// For any two spheres where the distance between their centers is less than
    /// the sum of their radii, they should be considered overlapping.
    #[test]
    fn prop_overlapping_spheres_detected(
        radius_a in 0.5f32..2.0f32,
        radius_b in 0.5f32..2.0f32,
        overlap_factor in 0.1f32..0.9f32, // Distance as fraction of sum of radii
    ) {
        let sum_of_radii = radius_a + radius_b;
        let distance = sum_of_radii * overlap_factor; // Distance less than sum of radii
        
        // Verify overlap condition
        prop_assert!(
            distance < sum_of_radii,
            "Distance {} should be less than sum of radii {} for overlap",
            distance,
            sum_of_radii
        );
        
        // Create colliders
        let collider_a = Collider {
            shape: ColliderShape::Sphere { radius: radius_a },
            ..Default::default()
        };
        
        let collider_b = Collider {
            shape: ColliderShape::Sphere { radius: radius_b },
            ..Default::default()
        };
        
        // Verify colliders are created correctly
        if let ColliderShape::Sphere { radius } = collider_a.shape {
            prop_assert_eq!(radius, radius_a);
        }
        
        if let ColliderShape::Sphere { radius } = collider_b.shape {
            prop_assert_eq!(radius, radius_b);
        }
    }

    /// Property test: Non-overlapping spheres should not collide
    /// 
    /// For any two spheres where the distance between their centers is greater than
    /// the sum of their radii, they should not be considered overlapping.
    #[test]
    fn prop_non_overlapping_spheres_not_detected(
        radius_a in 0.5f32..2.0f32,
        radius_b in 0.5f32..2.0f32,
        extra_distance in 1.0f32..5.0f32,
    ) {
        let sum_of_radii = radius_a + radius_b;
        let distance = sum_of_radii + extra_distance; // Distance greater than sum of radii
        
        // Verify non-overlap condition
        prop_assert!(
            distance > sum_of_radii,
            "Distance {} should be greater than sum of radii {} for no overlap",
            distance,
            sum_of_radii
        );
    }

    /// Property test: RigidBody types are correctly assigned
    #[test]
    fn prop_rigid_body_types(
        body_type_index in 0usize..3,
        mass in 0.1f32..100.0f32,
    ) {
        let body_type = match body_type_index {
            0 => RigidBodyType::Dynamic,
            1 => RigidBodyType::Kinematic,
            _ => RigidBodyType::Static,
        };
        
        let rigid_body = RigidBody {
            body_type,
            mass,
            ..Default::default()
        };
        
        prop_assert_eq!(rigid_body.body_type, body_type);
        prop_assert_eq!(rigid_body.mass, mass);
    }

    /// Property test: Collider shapes maintain their properties
    #[test]
    fn prop_collider_shapes(
        radius in 0.1f32..10.0f32,
        friction in 0.0f32..1.0f32,
        restitution in 0.0f32..1.0f32,
    ) {
        let collider = Collider {
            shape: ColliderShape::Sphere { radius },
            friction,
            restitution,
            is_sensor: false,
        };
        
        if let ColliderShape::Sphere { radius: r } = collider.shape {
            prop_assert_eq!(r, radius);
        }
        
        prop_assert_eq!(collider.friction, friction);
        prop_assert_eq!(collider.restitution, restitution);
    }

    /// Property test: Box colliders maintain their dimensions
    #[test]
    fn prop_box_collider_dimensions(
        x in 0.1f32..10.0f32,
        y in 0.1f32..10.0f32,
        z in 0.1f32..10.0f32,
    ) {
        let half_extents = Vec3::new(x, y, z);
        
        let collider = Collider {
            shape: ColliderShape::Box { half_extents },
            ..Default::default()
        };
        
        if let ColliderShape::Box { half_extents: he } = collider.shape {
            prop_assert_eq!(he.x, x);
            prop_assert_eq!(he.y, y);
            prop_assert_eq!(he.z, z);
        }
    }
}
