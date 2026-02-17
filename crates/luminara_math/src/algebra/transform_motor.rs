//! Motor-based transform representation for gimbal-lock-free rotations.
//!
//! This module provides `TransformMotor`, an alternative transform representation
//! using PGA Motors instead of quaternions. Motors provide:
//! - Gimbal-lock-free rotations
//! - Unified rotation and translation representation
//! - Efficient composition through geometric product
//! - Smooth interpolation via SLERP
//!
//! Use `TransformMotor` when you need robust rotation handling, especially for
//! physics simulations with high angular velocities or animation systems requiring
//! smooth interpolation.

use crate::algebra::motor::Motor;
use crate::algebra::vector::Vector3;
use crate::{Mat4, Quat, Transform, Vec3};

use serde::{Deserialize, Serialize};

/// Transform using PGA Motor (gimbal-lock-free).
///
/// This component provides an alternative to the standard `Transform` component,
/// using a Motor to encode rotation and translation as a single algebraic object.
/// Scale is stored separately as it's not part of the motor representation.
///
/// # Benefits
/// - **Gimbal-lock-free**: Motors avoid gimbal lock issues inherent in Euler angles
/// - **Efficient composition**: Combining transforms uses the geometric product
/// - **Smooth interpolation**: SLERP for rotation, LERP for translation
/// - **Unified representation**: Rotation and translation in one structure
///
/// # Usage
/// ```rust
/// use luminara_math::{TransformMotor, Vec3, Quat};
///
/// // Create from position and rotation
/// let transform = TransformMotor::from_position_rotation(
///     Vec3::new(1.0, 2.0, 3.0),
///     Quat::from_rotation_y(std::f32::consts::PI / 4.0)
/// );
///
/// // Interpolate between transforms
/// let other = TransformMotor::from_position_rotation(
///     Vec3::new(5.0, 6.0, 7.0),
///     Quat::from_rotation_y(std::f32::consts::PI / 2.0)
/// );
/// let interpolated = transform.interpolate(&other, 0.5);
///
/// // Convert to standard Transform
/// let standard_transform = transform.to_transform();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C, align(32))]
pub struct TransformMotor {
    /// Motor encoding rotation and translation
    pub motor: Motor<f32>,
    /// Scale (not part of motor)
    pub scale: Vec3,
}

// Implement Component trait for ECS integration
#[cfg(feature = "ecs")]
impl luminara_core::Component for TransformMotor {
    fn type_name() -> &'static str {
        "TransformMotor"
    }
}

impl Default for TransformMotor {
    fn default() -> Self {
        Self {
            motor: Motor::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl TransformMotor {
    /// The identity transform (no rotation, no translation, unit scale).
    pub const IDENTITY: Self = Self {
        motor: Motor::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Create from position and rotation.
    ///
    /// # Arguments
    /// * `position` - The translation vector
    /// * `rotation` - The rotation quaternion
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{TransformMotor, Vec3, Quat};
    ///
    /// let transform = TransformMotor::from_position_rotation(
    ///     Vec3::new(1.0, 2.0, 3.0),
    ///     Quat::IDENTITY
    /// );
    /// ```
    #[inline]
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(rotation, position),
            scale: Vec3::ONE,
        }
    }

    /// Create from translation only.
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            motor: Motor::from_translation(translation.into()),
            scale: Vec3::ONE,
        }
    }

    /// Create from rotation only.
    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(rotation, Vec3::ZERO),
            scale: Vec3::ONE,
        }
    }

    /// Create from scale only.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            motor: Motor::IDENTITY,
            scale,
        }
    }

    /// Create from position, rotation, and scale.
    #[inline]
    pub fn from_position_rotation_scale(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(rotation, position),
            scale,
        }
    }

    /// Convert to standard Transform.
    ///
    /// This extracts the rotation and translation from the motor and combines
    /// them with the scale to create a standard `Transform` component.
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{TransformMotor, Vec3, Quat};
    ///
    /// let motor_transform = TransformMotor::from_position_rotation(
    ///     Vec3::new(1.0, 2.0, 3.0),
    ///     Quat::IDENTITY
    /// );
    /// let transform = motor_transform.to_transform();
    /// ```
    #[inline]
    pub fn to_transform(&self) -> Transform {
        let (rotation, translation) = self.motor.to_rotation_translation_glam();
        Transform {
            translation,
            rotation,
            scale: self.scale,
        }
    }

    /// Convert from standard Transform.
    ///
    /// This creates a `TransformMotor` from a standard `Transform` by encoding
    /// the rotation and translation into a motor.
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{Transform, TransformMotor, Vec3, Quat};
    ///
    /// let transform = Transform {
    ///     translation: Vec3::new(1.0, 2.0, 3.0),
    ///     rotation: Quat::IDENTITY,
    ///     scale: Vec3::ONE,
    /// };
    /// let motor_transform = TransformMotor::from_transform(&transform);
    /// ```
    #[inline]
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(transform.rotation, transform.translation),
            scale: transform.scale,
        }
    }

    /// Interpolate between two transforms (SLERP for rotation, LERP for translation).
    ///
    /// This performs spherical linear interpolation (SLERP) for the rotational
    /// component and linear interpolation (LERP) for the translational component
    /// and scale.
    ///
    /// # Arguments
    /// * `other` - The target transform to interpolate towards
    /// * `t` - Interpolation parameter in range [0, 1]
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{TransformMotor, Vec3, Quat};
    ///
    /// let start = TransformMotor::from_position_rotation(
    ///     Vec3::ZERO,
    ///     Quat::IDENTITY
    /// );
    /// let end = TransformMotor::from_position_rotation(
    ///     Vec3::new(10.0, 0.0, 0.0),
    ///     Quat::from_rotation_y(std::f32::consts::PI)
    /// );
    /// let mid = start.interpolate(&end, 0.5);
    /// ```
    #[inline]
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let interpolated_motor = self.motor.interpolate(&other.motor, t);
        let interpolated_scale = self.scale.lerp(other.scale, t);
        
        Self {
            motor: interpolated_motor,
            scale: interpolated_scale,
        }
    }

    /// Apply motor to point.
    ///
    /// Transforms a point in space using the motor's rotation and translation.
    /// Note: This does not apply scale.
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{TransformMotor, Vec3, Quat};
    ///
    /// let transform = TransformMotor::from_position_rotation(
    ///     Vec3::new(1.0, 0.0, 0.0),
    ///     Quat::IDENTITY
    /// );
    /// let point = Vec3::new(0.0, 1.0, 0.0);
    /// let transformed = transform.transform_point(point);
    /// ```
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let rotated_translated = self.motor.transform_point(point.into());
        Vec3::from(rotated_translated) * self.scale
    }

    /// Apply motor to vector (direction).
    ///
    /// Transforms a vector using only the motor's rotation (no translation).
    /// Note: This does not apply scale.
    #[inline]
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        // For vectors, we only apply rotation (no translation)
        // We can use the motor but subtract the translation effect
        let origin = self.motor.transform_point(Vector3::new(0.0, 0.0, 0.0));
        let transformed = self.motor.transform_point(vector.into());
        Vec3::from(transformed) - Vec3::from(origin)
    }

    /// Compose two transforms.
    ///
    /// Combines two transforms using the geometric product for motors.
    /// The result represents applying `self` followed by `other`.
    ///
    /// # Example
    /// ```rust
    /// use luminara_math::{TransformMotor, Vec3, Quat};
    ///
    /// let t1 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
    /// let t2 = TransformMotor::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// let combined = t1.compose(&t2);
    /// ```
    #[inline]
    pub fn compose(&self, other: &Self) -> Self {
        let combined_motor = self.motor.geometric_product(&other.motor);
        Self {
            motor: combined_motor,
            scale: self.scale * other.scale,
        }
    }

    /// Compute the inverse transform.
    ///
    /// Returns a transform that, when composed with this transform, yields the identity.
    #[inline]
    pub fn inverse(&self) -> Self {
        Self {
            motor: self.motor.reverse(),
            scale: Vec3::ONE / self.scale,
        }
    }

    /// Convert to a 4x4 transformation matrix.
    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        let (rotation, translation) = self.motor.to_rotation_translation_glam();
        Mat4::from_scale_rotation_translation(self.scale, rotation, translation)
    }

    /// Get the rotation and translation components.
    #[inline]
    pub fn to_rotation_translation(&self) -> (Quat, Vec3) {
        self.motor.to_rotation_translation_glam()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let transform = TransformMotor::IDENTITY;
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = transform.transform_point(point);
        assert!((transformed - point).length() < 1e-6);
    }

    #[test]
    fn test_translation() {
        let transform = TransformMotor::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::ZERO;
        let transformed = transform.transform_point(point);
        assert!((transformed - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
    }

    #[test]
    fn test_conversion_roundtrip() {
        let original = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };
        
        let motor_transform = TransformMotor::from_transform(&original);
        let converted_back = motor_transform.to_transform();
        
        assert!((converted_back.translation - original.translation).length() < 1e-5);
        assert!((converted_back.rotation.dot(original.rotation)).abs() > 0.9999);
        assert!((converted_back.scale - original.scale).length() < 1e-5);
    }

    #[test]
    fn test_interpolation() {
        let start = TransformMotor::from_position_rotation(
            Vec3::ZERO,
            Quat::IDENTITY
        );
        let end = TransformMotor::from_position_rotation(
            Vec3::new(10.0, 0.0, 0.0),
            Quat::from_rotation_y(std::f32::consts::PI)
        );
        
        let mid = start.interpolate(&end, 0.5);
        let (_, translation) = mid.to_rotation_translation();
        
        // Translation should be halfway
        assert!((translation - Vec3::new(5.0, 0.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn test_composition() {
        let t1 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t2 = TransformMotor::from_translation(Vec3::new(0.0, 1.0, 0.0));
        let combined = t1.compose(&t2);
        
        let point = Vec3::ZERO;
        let transformed = combined.transform_point(point);
        
        // Should be translated by (1, 1, 0)
        assert!((transformed - Vec3::new(1.0, 1.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn test_inverse() {
        let transform = TransformMotor::from_position_rotation(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(std::f32::consts::PI / 4.0)
        );
        
        let inverse = transform.inverse();
        let identity = transform.compose(&inverse);
        
        let point = Vec3::new(5.0, 6.0, 7.0);
        let transformed = identity.transform_point(point);
        
        // Should be close to original point
        assert!((transformed - point).length() < 1e-4);
    }
}
