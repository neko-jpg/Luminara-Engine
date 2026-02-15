// ============================================================================
// Deserialization Validation Module
// ============================================================================
//
// This module provides validation utilities for deserialization operations,
// ensuring that all required fields are present and providing clear error
// messages with suggestions for common errors.
//
// **Validates: Requirements 8.2**

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

/// Validation error that occurs during deserialization
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// The type being deserialized
    pub type_name: String,
    /// The specific validation failure
    pub kind: ValidationErrorKind,
    /// Suggested fix for the error
    pub suggestion: String,
}

/// Specific kinds of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorKind {
    /// A required field is missing
    MissingField { field_name: String },
    /// A field has an invalid value
    InvalidValue {
        field_name: String,
        value: String,
        reason: String,
    },
    /// A field has the wrong type
    TypeMismatch {
        field_name: String,
        expected: String,
        found: String,
    },
    /// The data format is invalid
    InvalidFormat { reason: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validation error in '{}': {}\nSuggestion: {}",
            self.type_name, self.kind, self.suggestion
        )
    }
}

impl fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationErrorKind::MissingField { field_name } => {
                write!(f, "Missing required field '{}'", field_name)
            }
            ValidationErrorKind::InvalidValue {
                field_name,
                value,
                reason,
            } => {
                write!(
                    f,
                    "Invalid value '{}' for field '{}': {}",
                    value, field_name, reason
                )
            }
            ValidationErrorKind::TypeMismatch {
                field_name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Type mismatch for field '{}': expected {}, found {}",
                    field_name, expected, found
                )
            }
            ValidationErrorKind::InvalidFormat { reason } => {
                write!(f, "Invalid format: {}", reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

// ============================================================================
// Validation Traits
// ============================================================================

/// Trait for types that can validate themselves after deserialization
pub trait Validate {
    /// Validate the deserialized value
    ///
    /// Returns Ok(()) if valid, or Err(ValidationError) with details about
    /// what's wrong and how to fix it.
    fn validate(&self) -> Result<(), ValidationError>;
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Helper to validate that a float is finite (not NaN or infinite)
pub fn validate_finite_f32(
    type_name: &str,
    field_name: &str,
    value: f32,
) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError {
            type_name: type_name.to_string(),
            kind: ValidationErrorKind::InvalidValue {
                field_name: field_name.to_string(),
                value: value.to_string(),
                reason: "Value must be finite (not NaN or infinite)".to_string(),
            },
            suggestion: format!(
                "Ensure '{}' is a valid number. Common causes: division by zero, \
                 invalid mathematical operations, or corrupted data.",
                field_name
            ),
        });
    }
    Ok(())
}

/// Helper to validate that a value is within a range
pub fn validate_range_f32(
    type_name: &str,
    field_name: &str,
    value: f32,
    min: f32,
    max: f32,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError {
            type_name: type_name.to_string(),
            kind: ValidationErrorKind::InvalidValue {
                field_name: field_name.to_string(),
                value: value.to_string(),
                reason: format!("Value must be between {} and {}", min, max),
            },
            suggestion: format!(
                "Clamp '{}' to the valid range [{}, {}]. Current value: {}",
                field_name, min, max, value
            ),
        });
    }
    Ok(())
}

/// Helper to validate that a quaternion is normalized
pub fn validate_quaternion_normalized(
    type_name: &str,
    quat: &crate::Quat,
) -> Result<(), ValidationError> {
    let length_sq = quat.length_squared();
    let epsilon = 1e-4; // Allow some tolerance for floating point errors

    if (length_sq - 1.0).abs() > epsilon {
        return Err(ValidationError {
            type_name: type_name.to_string(),
            kind: ValidationErrorKind::InvalidValue {
                field_name: "rotation".to_string(),
                value: format!("Quat({}, {}, {}, {})", quat.x, quat.y, quat.z, quat.w),
                reason: format!(
                    "Quaternion must be normalized (length = 1.0), but length² = {}",
                    length_sq
                ),
            },
            suggestion: format!(
                "Normalize the quaternion. You can fix this by dividing all components by \
                 the length: sqrt({}) ≈ {}. Or use Quat::normalize() in code.",
                length_sq,
                length_sq.sqrt()
            ),
        });
    }
    Ok(())
}

/// Helper to validate that a scale vector has positive components
pub fn validate_scale_positive(
    type_name: &str,
    scale: &crate::Vec3,
) -> Result<(), ValidationError> {
    if scale.x <= 0.0 || scale.y <= 0.0 || scale.z <= 0.0 {
        return Err(ValidationError {
            type_name: type_name.to_string(),
            kind: ValidationErrorKind::InvalidValue {
                field_name: "scale".to_string(),
                value: format!("Vec3({}, {}, {})", scale.x, scale.y, scale.z),
                reason: "Scale components must be positive".to_string(),
            },
            suggestion: format!(
                "Ensure all scale components are positive. Current: ({}, {}, {}). \
                 Use Vec3::splat(1.0) for uniform scale, or Vec3::new(x, y, z) with positive values.",
                scale.x, scale.y, scale.z
            ),
        });
    }
    Ok(())
}

/// Helper to validate color components are in [0, 1] range
pub fn validate_color_range(type_name: &str, color: &crate::Color) -> Result<(), ValidationError> {
    let components = [
        ("r", color.r),
        ("g", color.g),
        ("b", color.b),
        ("a", color.a),
    ];

    for (name, value) in &components {
        if *value < 0.0 || *value > 1.0 {
            return Err(ValidationError {
                type_name: type_name.to_string(),
                kind: ValidationErrorKind::InvalidValue {
                    field_name: name.to_string(),
                    value: value.to_string(),
                    reason: "Color components must be in range [0.0, 1.0]".to_string(),
                },
                suggestion: format!(
                    "Clamp color component '{}' to [0.0, 1.0]. Current value: {}. \
                     If you have values in [0, 255], divide by 255.0.",
                    name, value
                ),
            });
        }
    }
    Ok(())
}

// ============================================================================
// Validation Implementations for Core Types
// ============================================================================

impl Validate for crate::Vec3 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_finite_f32("Vec3", "x", self.x)?;
        validate_finite_f32("Vec3", "y", self.y)?;
        validate_finite_f32("Vec3", "z", self.z)?;
        Ok(())
    }
}

impl Validate for crate::Quat {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_finite_f32("Quat", "x", self.x)?;
        validate_finite_f32("Quat", "y", self.y)?;
        validate_finite_f32("Quat", "z", self.z)?;
        validate_finite_f32("Quat", "w", self.w)?;
        validate_quaternion_normalized("Quat", self)?;
        Ok(())
    }
}

impl Validate for crate::Transform {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate translation
        self.translation.validate().map_err(|mut e| {
            e.type_name = "Transform".to_string();
            e.kind = match e.kind {
                ValidationErrorKind::InvalidValue {
                    field_name,
                    value,
                    reason,
                } => ValidationErrorKind::InvalidValue {
                    field_name: format!("translation.{}", field_name),
                    value,
                    reason,
                },
                other => other,
            };
            e
        })?;

        // Validate rotation
        self.rotation.validate().map_err(|mut e| {
            e.type_name = "Transform".to_string();
            e
        })?;

        // Validate scale
        self.scale.validate().map_err(|mut e| {
            e.type_name = "Transform".to_string();
            e.kind = match e.kind {
                ValidationErrorKind::InvalidValue {
                    field_name,
                    value,
                    reason,
                } => ValidationErrorKind::InvalidValue {
                    field_name: format!("scale.{}", field_name),
                    value,
                    reason,
                },
                other => other,
            };
            e
        })?;

        // Additional validation: scale should be positive
        validate_scale_positive("Transform", &self.scale)?;

        Ok(())
    }
}

impl Validate for crate::Color {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_finite_f32("Color", "r", self.r)?;
        validate_finite_f32("Color", "g", self.g)?;
        validate_finite_f32("Color", "b", self.b)?;
        validate_finite_f32("Color", "a", self.a)?;
        validate_color_range("Color", self)?;
        Ok(())
    }
}

// ============================================================================
// Validated Wrapper Type
// ============================================================================

/// A wrapper that ensures a value is validated during deserialization
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Validated<T: Validate> {
    #[serde(flatten)]
    inner: T,
}

impl<T: Validate> Validated<T> {
    /// Create a new validated value, checking validation immediately
    pub fn new(value: T) -> Result<Self, ValidationError> {
        value.validate()?;
        Ok(Self { inner: value })
    }

    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Consume the wrapper and return the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'de, T: Validate + Deserialize<'de>> Deserialize<'de> for Validated<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        value
            .validate()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Self { inner: value })
    }
}

impl<T: Validate> std::ops::Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Deserialize and validate from RON string
pub fn from_ron_validated<T>(s: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Validate,
{
    let value: T = ron::from_str(s).map_err(|e| format!("RON parse error: {}", e))?;
    value
        .validate()
        .map_err(|e| format!("Validation error: {}", e))?;
    Ok(value)
}

/// Deserialize and validate from binary
pub fn from_binary_validated<T>(bytes: &[u8]) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Validate,
{
    let value: T = bincode::deserialize(bytes).map_err(|e| format!("Binary parse error: {}", e))?;
    value
        .validate()
        .map_err(|e| format!("Validation error: {}", e))?;
    Ok(value)
}
