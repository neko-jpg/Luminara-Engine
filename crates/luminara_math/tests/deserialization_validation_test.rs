// ============================================================================
// Deserialization Validation Tests
// Feature: pre-editor-engine-audit
// Validates: Requirements 8.2
// ============================================================================
//
// These tests verify that deserialization validation:
// 1. Validates all required fields are present
// 2. Provides clear error messages
// 3. Suggests fixes for common errors

use luminara_math::validation::{
    from_binary_validated, from_ron_validated, Validate, ValidationError, ValidationErrorKind,
};
use luminara_math::{Color, Quat, Transform, Vec3};

// ============================================================================
// Vec3 Validation Tests
// ============================================================================

#[test]
fn test_vec3_valid() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    assert!(vec.validate().is_ok());
}

#[test]
fn test_vec3_nan_x() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let result = vec.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Vec3");
    assert!(matches!(err.kind, ValidationErrorKind::InvalidValue { .. }));
    assert!(err.to_string().contains("finite"), "Error message: {}", err);
    assert!(
        err.suggestion.contains("division by zero") || err.suggestion.contains("corrupted"),
        "Suggestion: {}",
        err.suggestion
    );
}

#[test]
fn test_vec3_infinity_y() {
    let vec = Vec3::new(1.0, f32::INFINITY, 3.0);
    let result = vec.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("finite"));
    assert!(err.suggestion.contains("division by zero"));
}

#[test]
fn test_vec3_neg_infinity_z() {
    let vec = Vec3::new(1.0, 2.0, f32::NEG_INFINITY);
    let result = vec.validate();
    assert!(result.is_err());
}

// ============================================================================
// Quat Validation Tests
// ============================================================================

#[test]
fn test_quat_valid_normalized() {
    let quat = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
    assert!(quat.validate().is_ok());
}

#[test]
fn test_quat_identity() {
    let quat = Quat::IDENTITY;
    assert!(quat.validate().is_ok());
}

#[test]
fn test_quat_not_normalized() {
    // Create an unnormalized quaternion
    let quat = Quat::from_xyzw(1.0, 1.0, 1.0, 1.0); // length² = 4.0
    let result = quat.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Quat");
    assert!(matches!(err.kind, ValidationErrorKind::InvalidValue { .. }));
    assert!(err.to_string().contains("normalized"));
    assert!(err.suggestion.contains("Normalize"));
}

#[test]
fn test_quat_nan_component() {
    let quat = Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0);
    let result = quat.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("finite"));
}

#[test]
fn test_quat_zero_length() {
    let quat = Quat::from_xyzw(0.0, 0.0, 0.0, 0.0);
    let result = quat.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("normalized"));
}

// ============================================================================
// Transform Validation Tests
// ============================================================================

#[test]
fn test_transform_valid() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        scale: Vec3::splat(1.0),
    };
    assert!(transform.validate().is_ok());
}

#[test]
fn test_transform_identity() {
    let transform = Transform::IDENTITY;
    assert!(transform.validate().is_ok());
}

#[test]
fn test_transform_invalid_translation() {
    let transform = Transform {
        translation: Vec3::new(f32::NAN, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(1.0),
    };
    let result = transform.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Transform");
    assert!(err.to_string().contains("translation"));
}

#[test]
fn test_transform_invalid_rotation() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
        scale: Vec3::splat(1.0),
    };
    let result = transform.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Transform");
    assert!(err.to_string().contains("normalized"));
}

#[test]
fn test_transform_negative_scale() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, -1.0, 1.0), // Negative scale
    };
    let result = transform.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Transform");
    assert!(err.to_string().contains("scale"));
    assert!(err.to_string().contains("positive"));
    assert!(err.suggestion.contains("positive"));
}

#[test]
fn test_transform_zero_scale() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, 0.0, 1.0), // Zero scale
    };
    let result = transform.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("scale"));
    assert!(err.to_string().contains("positive"));
}

// ============================================================================
// Color Validation Tests
// ============================================================================

#[test]
fn test_color_valid() {
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    assert!(color.validate().is_ok());
}

#[test]
fn test_color_white() {
    assert!(Color::WHITE.validate().is_ok());
}

#[test]
fn test_color_black() {
    assert!(Color::BLACK.validate().is_ok());
}

#[test]
fn test_color_transparent() {
    assert!(Color::TRANSPARENT.validate().is_ok());
}

#[test]
fn test_color_component_too_large() {
    let color = Color::rgba(1.5, 0.5, 0.5, 1.0); // r > 1.0
    let result = color.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.type_name, "Color");
    assert!(err.to_string().contains("[0.0, 1.0]"));
    assert!(err.suggestion.contains("Clamp"));
    assert!(err.suggestion.contains("255")); // Suggests common mistake
}

#[test]
fn test_color_component_negative() {
    let color = Color::rgba(0.5, -0.1, 0.5, 1.0); // g < 0.0
    let result = color.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("[0.0, 1.0]"));
}

#[test]
fn test_color_nan_component() {
    let color = Color::rgba(0.5, 0.5, f32::NAN, 1.0);
    let result = color.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("finite"));
}

// ============================================================================
// RON Deserialization with Validation Tests
// ============================================================================

#[test]
fn test_ron_vec3_valid() {
    // Create a Vec3 and serialize it to see the actual format
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let ron_str = ron::to_string(&vec).unwrap();
    // Now deserialize and validate
    let result: Result<Vec3, String> = from_ron_validated(&ron_str);
    assert!(
        result.is_ok(),
        "Failed with RON: {}, error: {:?}",
        ron_str,
        result
    );
    let deserialized = result.unwrap();
    assert_eq!(deserialized, vec);
}

#[test]
fn test_ron_vec3_nan() {
    // Create a Vec3 with NaN, serialize it, then try to validate
    let vec_with_nan = Vec3::new(f32::NAN, 2.0, 3.0);
    let ron_str = ron::to_string(&vec_with_nan).unwrap();
    let result: Result<Vec3, String> = from_ron_validated(&ron_str);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"));
    assert!(err.contains("finite"));
}

#[test]
fn test_ron_transform_valid() {
    // Transform serializes with glam types as arrays
    let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    let ron_str = ron::to_string(&transform).unwrap();
    let result: Result<Transform, String> = from_ron_validated(&ron_str);
    assert!(result.is_ok(), "Failed to deserialize: {:?}", result);
}

#[test]
fn test_ron_transform_invalid_rotation() {
    // Create a transform with unnormalized rotation
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
        scale: Vec3::splat(1.0),
    };
    let ron_str = ron::to_string(&transform).unwrap();
    let result: Result<Transform, String> = from_ron_validated(&ron_str);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"), "Error was: {}", err);
    assert!(err.contains("normalized"), "Error was: {}", err);
    assert!(err.contains("Normalize"), "Error was: {}", err);
}

#[test]
fn test_ron_transform_negative_scale() {
    // Create a transform with negative scale
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, -1.0, 1.0),
    };
    let ron_str = ron::to_string(&transform).unwrap();
    let result: Result<Transform, String> = from_ron_validated(&ron_str);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"), "Error was: {}", err);
    assert!(err.contains("scale"), "Error was: {}", err);
    assert!(err.contains("positive"), "Error was: {}", err);
}

#[test]
fn test_ron_color_valid() {
    // Color serializes as a struct
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    let ron_str = ron::to_string(&color).unwrap();
    let result: Result<Color, String> = from_ron_validated(&ron_str);
    assert!(result.is_ok());
}

#[test]
fn test_ron_color_out_of_range() {
    // Create a color with out-of-range value
    let color = Color::rgba(1.5, 0.5, 0.5, 1.0);
    let ron_str = ron::to_string(&color).unwrap();
    let result: Result<Color, String> = from_ron_validated(&ron_str);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"), "Error was: {}", err);
    assert!(err.contains("[0.0, 1.0]"), "Error was: {}", err);
    assert!(err.contains("255"), "Error was: {}", err); // Suggests common mistake
}

// ============================================================================
// Binary Deserialization with Validation Tests
// ============================================================================

#[test]
fn test_binary_vec3_valid() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let bytes = bincode::serialize(&vec).unwrap();
    let result: Result<Vec3, String> = from_binary_validated(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec);
}

#[test]
fn test_binary_vec3_nan() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let bytes = bincode::serialize(&vec).unwrap();
    let result: Result<Vec3, String> = from_binary_validated(&bytes);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"));
    assert!(err.contains("finite"));
}

#[test]
fn test_binary_transform_valid() {
    let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    let bytes = bincode::serialize(&transform).unwrap();
    let result: Result<Transform, String> = from_binary_validated(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_binary_transform_invalid() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
        scale: Vec3::splat(1.0),
    };
    let bytes = bincode::serialize(&transform).unwrap();
    let result: Result<Transform, String> = from_binary_validated(&bytes);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Validation error"));
    assert!(err.contains("normalized"));
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_contains_type_name() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let err = vec.validate().unwrap_err();
    assert!(err.to_string().contains("Vec3"));
}

#[test]
fn test_error_message_contains_field_name() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let err = vec.validate().unwrap_err();
    assert!(err.to_string().contains("'x'"));
}

#[test]
fn test_error_message_contains_reason() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let err = vec.validate().unwrap_err();
    assert!(err.to_string().contains("finite"));
}

#[test]
fn test_error_message_contains_suggestion() {
    let vec = Vec3::new(f32::NAN, 2.0, 3.0);
    let err = vec.validate().unwrap_err();
    assert!(err.to_string().contains("Suggestion:"));
    assert!(err.suggestion.contains("division by zero"));
}

#[test]
fn test_quat_error_suggests_normalization() {
    let quat = Quat::from_xyzw(1.0, 1.0, 1.0, 1.0);
    let err = quat.validate().unwrap_err();
    assert!(err.suggestion.contains("Normalize"));
    assert!(err.suggestion.contains("sqrt"));
}

#[test]
fn test_scale_error_suggests_positive_values() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, -1.0, 1.0),
    };
    let err = transform.validate().unwrap_err();
    assert!(err.suggestion.contains("positive"));
    assert!(err.suggestion.contains("Vec3::splat"));
}

#[test]
fn test_color_error_suggests_255_conversion() {
    let color = Color::rgba(1.5, 0.5, 0.5, 1.0);
    let err = color.validate().unwrap_err();
    assert!(err.suggestion.contains("255"));
    assert!(err.suggestion.contains("Clamp"));
}

// ============================================================================
// Complex Structure Validation Tests
// ============================================================================

#[test]
fn test_complex_structure_with_validation() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Entity {
        name: String,
        transform: Transform,
        color: Color,
    }

    // Valid entity
    let entity = Entity {
        name: "Player".to_string(),
        transform: Transform::from_xyz(1.0, 2.0, 3.0),
        color: Color::RED,
    };
    let ron_str = ron::to_string(&entity).unwrap();
    let deserialized: Entity = ron::from_str(&ron_str).unwrap();
    assert!(deserialized.transform.validate().is_ok());
    assert!(deserialized.color.validate().is_ok());

    // Invalid entity (bad rotation)
    let entity_invalid = Entity {
        name: "Player".to_string(),
        transform: Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
            scale: Vec3::splat(1.0),
        },
        color: Color::RED,
    };
    let ron_str_invalid = ron::to_string(&entity_invalid).unwrap();
    let deserialized_invalid: Entity = ron::from_str(&ron_str_invalid).unwrap();
    assert!(deserialized_invalid.transform.validate().is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_very_small_values() {
    let vec = Vec3::new(1e-30, 1e-30, 1e-30);
    assert!(vec.validate().is_ok());
}

#[test]
fn test_very_large_values() {
    let vec = Vec3::new(1e30, 1e30, 1e30);
    assert!(vec.validate().is_ok());
}

#[test]
fn test_negative_values() {
    let vec = Vec3::new(-1000.0, -2000.0, -3000.0);
    assert!(vec.validate().is_ok());
}

#[test]
fn test_mixed_valid_invalid() {
    // Only one component is invalid
    let vec = Vec3::new(1.0, f32::NAN, 3.0);
    let err = vec.validate().unwrap_err();
    assert!(err.to_string().contains("'y'"));
}
