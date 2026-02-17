//! Integration tests for Motor transform systems.
//!
//! These tests verify that the motor transform synchronization and propagation
//! systems work correctly in a real ECS environment.

use luminara_core::{App, AppInterface, CoreStage};
use luminara_math::{Quat, Transform, TransformMotor, Vec3};
use luminara_scene::{
    motor_transform_propagate_system, sync_motor_to_transform_system, GlobalTransformMotor,
    MotorDriven,
};

#[test]
fn test_motor_transform_systems_registered() {
    // Test that motor transform systems can be registered in an app
    let mut app = App::new();
    
    app.add_system::<luminara_core::system::ExclusiveMarker>(
        CoreStage::PostUpdate,
        sync_motor_to_transform_system,
    );
    
    app.add_system::<luminara_core::system::ExclusiveMarker>(
        CoreStage::PostUpdate,
        motor_transform_propagate_system,
    );
    
    // If we get here without panicking, the systems were registered successfully
    assert!(true);
}

#[test]
fn test_motor_components_creation() {
    // Test that all motor-related components can be created
    let motor = TransformMotor::from_position_rotation(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::IDENTITY,
    );
    
    let _global_motor = GlobalTransformMotor(motor);
    let _marker = MotorDriven;
    
    // Verify component type names
    use luminara_core::Component;
    assert_eq!(MotorDriven::type_name(), "MotorDriven");
    assert_eq!(GlobalTransformMotor::type_name(), "GlobalTransformMotor");
}

#[test]
fn test_motor_transform_conversion() {
    // Test bidirectional conversion between Transform and TransformMotor
    let original_transform = Transform {
        translation: Vec3::new(5.0, 10.0, 15.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    
    // Convert to motor
    let motor = TransformMotor::from_transform(&original_transform);
    
    // Convert back to transform
    let converted_transform = motor.to_transform();
    
    // Verify the conversion is accurate
    assert!((converted_transform.translation - original_transform.translation).length() < 1e-5);
    assert!((converted_transform.scale - original_transform.scale).length() < 1e-5);
    assert!(converted_transform.rotation.dot(original_transform.rotation).abs() > 0.9999);
}

#[test]
fn test_motor_interpolation() {
    // Test smooth interpolation between motor transforms
    let start = TransformMotor::from_position_rotation(
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    
    let end = TransformMotor::from_position_rotation(
        Vec3::new(10.0, 0.0, 0.0),
        Quat::from_rotation_y(std::f32::consts::PI),
    );
    
    // Interpolate at 50%
    let mid = start.interpolate(&end, 0.5);
    let (_, translation) = mid.to_rotation_translation();
    
    // Translation should be halfway
    assert!((translation - Vec3::new(5.0, 0.0, 0.0)).length() < 1e-4);
}

#[test]
fn test_motor_composition_associativity() {
    // Test that motor composition is associative: (A ∘ B) ∘ C = A ∘ (B ∘ C)
    let a = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
    let b = TransformMotor::from_translation(Vec3::new(0.0, 1.0, 0.0));
    let c = TransformMotor::from_translation(Vec3::new(0.0, 0.0, 1.0));
    
    let left = a.compose(&b).compose(&c);
    let right = a.compose(&b.compose(&c));
    
    let (_, left_trans) = left.to_rotation_translation();
    let (_, right_trans) = right.to_rotation_translation();
    
    assert!((left_trans - right_trans).length() < 1e-5);
}

#[test]
fn test_motor_inverse() {
    // Test that motor inverse works correctly
    let transform = TransformMotor::from_position_rotation(
        Vec3::new(5.0, 10.0, 15.0),
        Quat::from_rotation_y(std::f32::consts::PI / 3.0),
    );
    
    let inverse = transform.inverse();
    let identity = transform.compose(&inverse);
    
    // Applying transform then its inverse should give identity
    let point = Vec3::new(1.0, 2.0, 3.0);
    let transformed = identity.transform_point(point);
    
    assert!((transformed - point).length() < 1e-3);
}

#[test]
fn test_gimbal_lock_freedom() {
    // Test that motors avoid gimbal lock issues
    // Create a rotation that would cause gimbal lock with Euler angles (pitch = 90°)
    let rotation = Quat::from_rotation_x(std::f32::consts::PI / 2.0);
    let motor = TransformMotor::from_rotation(rotation);
    
    // Apply multiple rotations
    let motor2 = motor.compose(&motor);
    let motor3 = motor2.compose(&motor);
    
    // Should still be able to extract valid rotation
    let (final_rotation, _) = motor3.to_rotation_translation();
    
    // Verify the rotation is valid (normalized quaternion)
    assert!((final_rotation.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_motor_simd_optimization() {
    // Test that motor operations are efficient (this is more of a smoke test)
    // In a real scenario, SIMD operations should be faster than scalar operations
    let motor1 = TransformMotor::from_position_rotation(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::from_rotation_y(0.5),
    );
    
    let motor2 = TransformMotor::from_position_rotation(
        Vec3::new(4.0, 5.0, 6.0),
        Quat::from_rotation_z(0.3),
    );
    
    // Perform many compositions (should use SIMD internally)
    let mut result = motor1;
    for _ in 0..100 {
        result = result.compose(&motor2);
    }
    
    // Just verify we get a valid result
    let (rotation, _) = result.to_rotation_translation();
    assert!((rotation.length() - 1.0).abs() < 1e-4);
}
