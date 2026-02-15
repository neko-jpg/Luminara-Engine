// ============================================================================
// Property Test: Deserialization Validation
// Feature: pre-editor-engine-audit
// Task: 11.4
// **Validates: Requirements 8.2**
// ============================================================================
//
// **Property 10: Deserialization Validation**
// *For any* invalid serialized data, deserialization should fail with a clear
// error message describing the validation failure.

use luminara_math::validation::{from_ron_validated, Validate};
use luminara_math::{Color, Quat, Transform, Vec3};
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Strategy for generating invalid Vec3 values (with NaN or Infinity)
fn invalid_vec3() -> impl Strategy<Value = Vec3> {
    prop_oneof![
        // NaN in x component
        Just(Vec3::new(f32::NAN, 0.0, 0.0)),
        // NaN in y component
        Just(Vec3::new(0.0, f32::NAN, 0.0)),
        // NaN in z component
        Just(Vec3::new(0.0, 0.0, f32::NAN)),
        // Infinity in x component
        Just(Vec3::new(f32::INFINITY, 0.0, 0.0)),
        // Infinity in y component
        Just(Vec3::new(0.0, f32::INFINITY, 0.0)),
        // Infinity in z component
        Just(Vec3::new(0.0, 0.0, f32::INFINITY)),
        // Negative infinity in x component
        Just(Vec3::new(f32::NEG_INFINITY, 0.0, 0.0)),
        // Negative infinity in y component
        Just(Vec3::new(0.0, f32::NEG_INFINITY, 0.0)),
        // Negative infinity in z component
        Just(Vec3::new(0.0, 0.0, f32::NEG_INFINITY)),
        // Multiple invalid components
        Just(Vec3::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY)),
    ]
}

/// Strategy for generating valid Vec3 values
fn valid_vec3() -> impl Strategy<Value = Vec3> {
    (
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating invalid Quat values (unnormalized or with NaN/Infinity)
fn invalid_quat() -> impl Strategy<Value = Quat> {
    prop_oneof![
        // Unnormalized quaternions
        (
            prop::num::f32::NORMAL,
            prop::num::f32::NORMAL,
            prop::num::f32::NORMAL,
            prop::num::f32::NORMAL
        )
            .prop_filter("Must be unnormalized", |(x, y, z, w)| {
                let length_sq = x * x + y * y + z * z + w * w;
                (length_sq - 1.0).abs() > 1e-4
            })
            .prop_map(|(x, y, z, w)| Quat::from_xyzw(x, y, z, w)),
        // Zero quaternion (unnormalized)
        Just(Quat::from_xyzw(0.0, 0.0, 0.0, 0.0)),
        // NaN in components
        Just(Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0)),
        Just(Quat::from_xyzw(0.0, f32::NAN, 0.0, 1.0)),
        Just(Quat::from_xyzw(0.0, 0.0, f32::NAN, 1.0)),
        Just(Quat::from_xyzw(0.0, 0.0, 0.0, f32::NAN)),
        // Infinity in components
        Just(Quat::from_xyzw(f32::INFINITY, 0.0, 0.0, 0.0)),
        Just(Quat::from_xyzw(0.0, f32::INFINITY, 0.0, 0.0)),
    ]
}

/// Strategy for generating valid Quat values (normalized)
fn valid_quat() -> impl Strategy<Value = Quat> {
    // Generate random rotation angles
    (0.0f32..std::f32::consts::TAU).prop_map(|angle| Quat::from_rotation_y(angle))
}

/// Strategy for generating invalid Transform values
fn invalid_transform() -> impl Strategy<Value = Transform> {
    prop_oneof![
        // Invalid translation (NaN or Infinity)
        invalid_vec3().prop_map(|translation| Transform {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
        }),
        // Invalid rotation (unnormalized)
        invalid_quat().prop_map(|rotation| Transform {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::splat(1.0),
        }),
        // Invalid scale (negative)
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(-1.0, 1.0, 1.0),
        }),
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, -1.0, 1.0),
        }),
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 1.0, -1.0),
        }),
        // Invalid scale (zero)
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(0.0, 1.0, 1.0),
        }),
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 0.0, 1.0),
        }),
        Just(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 1.0, 0.0),
        }),
        // Multiple invalid fields
        (invalid_vec3(), invalid_quat()).prop_map(|(translation, rotation)| Transform {
            translation,
            rotation,
            scale: Vec3::new(-1.0, 0.0, 1.0),
        }),
    ]
}

/// Strategy for generating valid Transform values
fn valid_transform() -> impl Strategy<Value = Transform> {
    (
        valid_vec3(),
        valid_quat(),
        (0.01f32..100.0, 0.01f32..100.0, 0.01f32..100.0),
    )
        .prop_map(|(translation, rotation, (sx, sy, sz))| Transform {
            translation,
            rotation,
            scale: Vec3::new(sx, sy, sz),
        })
}

/// Strategy for generating invalid Color values (out of range or NaN/Infinity)
fn invalid_color() -> impl Strategy<Value = Color> {
    prop_oneof![
        // Component > 1.0
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(g, b, a)| Color::rgba(1.5, g, b, a)),
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(r, b, a)| Color::rgba(r, 2.0, b, a)),
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(r, g, a)| Color::rgba(r, g, 10.0, a)),
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(r, g, b)| Color::rgba(r, g, b, 5.0)),
        // Component < 0.0
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(g, b, a)| Color::rgba(-0.5, g, b, a)),
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(r, b, a)| Color::rgba(r, -1.0, b, a)),
        // NaN in components
        Just(Color::rgba(f32::NAN, 0.5, 0.5, 1.0)),
        Just(Color::rgba(0.5, f32::NAN, 0.5, 1.0)),
        Just(Color::rgba(0.5, 0.5, f32::NAN, 1.0)),
        Just(Color::rgba(0.5, 0.5, 0.5, f32::NAN)),
        // Infinity in components
        Just(Color::rgba(f32::INFINITY, 0.5, 0.5, 1.0)),
        Just(Color::rgba(0.5, f32::INFINITY, 0.5, 1.0)),
    ]
}

/// Strategy for generating valid Color values
fn valid_color() -> impl Strategy<Value = Color> {
    (0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0)
        .prop_map(|(r, g, b, a)| Color::rgba(r, g, b, a))
}

// ============================================================================
// Property Tests: Invalid Data Must Fail Validation
// ============================================================================

proptest! {
    /// **Property 10.1: Invalid Vec3 values must fail validation**
    ///
    /// For any Vec3 with NaN or Infinity components, validation must fail.
    #[test]
    fn prop_invalid_vec3_fails_validation(vec in invalid_vec3()) {
        let result = vec.validate();
        prop_assert!(result.is_err(), "Invalid Vec3 {:?} should fail validation", vec);

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        prop_assert_eq!(&err.type_name, "Vec3");
        prop_assert!(
            err_msg.contains("finite") || err_msg.contains("NaN") || err_msg.contains("infinite"),
            "Error message should mention finite/NaN/infinite: {}",
            err_msg
        );
    }

    /// **Property 10.2: Invalid Quat values must fail validation**
    ///
    /// For any Quat that is unnormalized or contains NaN/Infinity, validation must fail.
    #[test]
    fn prop_invalid_quat_fails_validation(quat in invalid_quat()) {
        let result = quat.validate();
        prop_assert!(result.is_err(), "Invalid Quat {:?} should fail validation", quat);

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        prop_assert_eq!(&err.type_name, "Quat");
        prop_assert!(
            err_msg.contains("normalized") || err_msg.contains("finite"),
            "Error message should mention normalized or finite: {}",
            err_msg
        );
    }

    /// **Property 10.3: Invalid Transform values must fail validation**
    ///
    /// For any Transform with invalid translation, rotation, or scale, validation must fail.
    #[test]
    fn prop_invalid_transform_fails_validation(transform in invalid_transform()) {
        let result = transform.validate();
        prop_assert!(result.is_err(), "Invalid Transform {:?} should fail validation", transform);

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        prop_assert_eq!(&err.type_name, "Transform");
        // Error should mention one of the invalid fields
        prop_assert!(
            err_msg.contains("translation") ||
            err_msg.contains("rotation") ||
            err_msg.contains("scale") ||
            err_msg.contains("normalized") ||
            err_msg.contains("finite") ||
            err_msg.contains("positive"),
            "Error message should mention the invalid field: {}",
            err_msg
        );
    }

    /// **Property 10.4: Invalid Color values must fail validation**
    ///
    /// For any Color with out-of-range or NaN/Infinity components, validation must fail.
    #[test]
    fn prop_invalid_color_fails_validation(color in invalid_color()) {
        let result = color.validate();
        prop_assert!(result.is_err(), "Invalid Color {:?} should fail validation", color);

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        prop_assert_eq!(&err.type_name, "Color");
        prop_assert!(
            err_msg.contains("[0.0, 1.0]") || err_msg.contains("finite"),
            "Error message should mention range or finite: {}",
            err_msg
        );
    }
}

// ============================================================================
// Property Tests: Valid Data Must Pass Validation
// ============================================================================

proptest! {
    /// **Property 10.5: Valid Vec3 values must pass validation**
    ///
    /// For any Vec3 with finite components, validation must succeed.
    #[test]
    fn prop_valid_vec3_passes_validation(vec in valid_vec3()) {
        let result = vec.validate();
        prop_assert!(result.is_ok(), "Valid Vec3 {:?} should pass validation, but got error: {:?}", vec, result);
    }

    /// **Property 10.6: Valid Quat values must pass validation**
    ///
    /// For any normalized Quat, validation must succeed.
    #[test]
    fn prop_valid_quat_passes_validation(quat in valid_quat()) {
        let result = quat.validate();
        prop_assert!(result.is_ok(), "Valid Quat {:?} should pass validation, but got error: {:?}", quat, result);
    }

    /// **Property 10.7: Valid Transform values must pass validation**
    ///
    /// For any Transform with valid translation, rotation, and scale, validation must succeed.
    #[test]
    fn prop_valid_transform_passes_validation(transform in valid_transform()) {
        let result = transform.validate();
        prop_assert!(result.is_ok(), "Valid Transform {:?} should pass validation, but got error: {:?}", transform, result);
    }

    /// **Property 10.8: Valid Color values must pass validation**
    ///
    /// For any Color with components in [0, 1], validation must succeed.
    #[test]
    fn prop_valid_color_passes_validation(color in valid_color()) {
        let result = color.validate();
        prop_assert!(result.is_ok(), "Valid Color {:?} should pass validation, but got error: {:?}", color, result);
    }
}

// ============================================================================
// Property Tests: Error Messages Must Be Clear and Helpful
// ============================================================================

proptest! {
    /// **Property 10.9: Error messages must contain type name**
    ///
    /// For any validation error, the error message must include the type name.
    #[test]
    fn prop_error_contains_type_name(vec in invalid_vec3()) {
        if let Err(err) = vec.validate() {
            let msg = err.to_string();
            prop_assert!(msg.contains("Vec3"), "Error message should contain type name: {}", msg);
        }
    }

    /// **Property 10.10: Error messages must contain suggestions**
    ///
    /// For any validation error, the error message must include a suggestion for fixing it.
    #[test]
    fn prop_error_contains_suggestion(transform in invalid_transform()) {
        if let Err(err) = transform.validate() {
            let msg = err.to_string();
            prop_assert!(msg.contains("Suggestion:"), "Error message should contain suggestion: {}", msg);
            prop_assert!(!err.suggestion.is_empty(), "Suggestion should not be empty");
        }
    }

    /// **Property 10.11: Quat errors must suggest normalization**
    ///
    /// For any unnormalized Quat, the error message must suggest normalization.
    #[test]
    fn prop_quat_error_suggests_normalization(quat in invalid_quat()) {
        if let Err(err) = quat.validate() {
            let msg = err.to_string();
            if msg.contains("normalized") {
                prop_assert!(
                    err.suggestion.contains("Normalize") || err.suggestion.contains("normalize"),
                    "Quat error should suggest normalization: {}",
                    err.suggestion
                );
            }
        }
    }

    /// **Property 10.12: Color errors must suggest common fixes**
    ///
    /// For any out-of-range Color, the error message must suggest clamping or 255 conversion.
    #[test]
    fn prop_color_error_suggests_fixes(color in invalid_color()) {
        if let Err(err) = color.validate() {
            let msg = err.to_string();
            if msg.contains("[0.0, 1.0]") {
                prop_assert!(
                    err.suggestion.contains("Clamp") || err.suggestion.contains("255"),
                    "Color error should suggest clamping or 255 conversion: {}",
                    err.suggestion
                );
            }
        }
    }
}

// ============================================================================
// Property Tests: Deserialization with Validation
// ============================================================================

proptest! {
    /// **Property 10.13: Invalid data fails RON deserialization with validation**
    ///
    /// For any invalid Transform, deserializing from RON with validation must fail.
    #[test]
    fn prop_invalid_transform_fails_ron_deserialization(transform in invalid_transform()) {
        // Serialize the invalid transform
        let ron_str = ron::to_string(&transform).unwrap();

        // Attempt to deserialize with validation
        let result: Result<Transform, String> = from_ron_validated(&ron_str);

        prop_assert!(
            result.is_err(),
            "Invalid Transform should fail RON deserialization with validation: {:?}",
            transform
        );

        let err = result.unwrap_err();
        prop_assert!(err.contains("Validation error"), "Error should mention validation: {}", err);
    }

    /// **Property 10.14: Valid data passes RON deserialization with validation**
    ///
    /// For any valid Transform, deserializing from RON with validation must succeed.
    #[test]
    fn prop_valid_transform_passes_ron_deserialization(transform in valid_transform()) {
        // Serialize the valid transform
        let ron_str = ron::to_string(&transform).unwrap();

        // Attempt to deserialize with validation
        let result: Result<Transform, String> = from_ron_validated(&ron_str);

        prop_assert!(
            result.is_ok(),
            "Valid Transform should pass RON deserialization with validation: {:?}, error: {:?}",
            transform,
            result
        );
    }

    /// **Property 10.15: Invalid Color fails RON deserialization with validation**
    ///
    /// For any invalid Color, deserializing from RON with validation must fail.
    #[test]
    fn prop_invalid_color_fails_ron_deserialization(color in invalid_color()) {
        let ron_str = ron::to_string(&color).unwrap();
        let result: Result<Color, String> = from_ron_validated(&ron_str);

        prop_assert!(
            result.is_err(),
            "Invalid Color should fail RON deserialization with validation: {:?}",
            color
        );
    }

    /// **Property 10.16: Valid Color passes RON deserialization with validation**
    ///
    /// For any valid Color, deserializing from RON with validation must succeed.
    #[test]
    fn prop_valid_color_passes_ron_deserialization(color in valid_color()) {
        let ron_str = ron::to_string(&color).unwrap();
        let result: Result<Color, String> = from_ron_validated(&ron_str);

        prop_assert!(
            result.is_ok(),
            "Valid Color should pass RON deserialization with validation: {:?}, error: {:?}",
            color,
            result
        );
    }
}
