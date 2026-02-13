//! Dual quaternion for smooth skinning and linear blending.
//!
//! Provides dual quaternion linear blending (DLB) for character animation.

use glam::{Quat, Vec3};

/// A dual quaternion representing a rigid body transformation.
///
/// Consists of a real part (rotation) and a dual part (related to translation).
/// Q = real + ε * dual
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct DualQuat {
    pub real: Quat,
    pub dual: Quat,
}

impl DualQuat {
    /// The identity dual quaternion (no rotation, no translation).
    pub const IDENTITY: Self = Self {
        real: Quat::IDENTITY,
        dual: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
    };

    /// Create a new dual quaternion from real and dual parts.
    #[inline]
    pub const fn new(real: Quat, dual: Quat) -> Self {
        Self { real, dual }
    }

    /// Create a dual quaternion from rotation and translation.
    ///
    /// # Arguments
    /// * `rot` - Rotation quaternion
    /// * `trans` - Translation vector
    #[inline]
    pub fn from_rotation_translation(rot: Quat, trans: Vec3) -> Self {
        // Q_real = r
        // Q_dual = 0.5 * t * r
        // where t is treated as a pure quaternion (0, x, y, z)

        let t_quat = Quat::from_xyzw(trans.x, trans.y, trans.z, 0.0);
        let dual = t_quat.mul_quat(rot) * 0.5;

        Self {
            real: rot,
            dual,
        }
    }

    /// Normalize the dual quaternion.
    ///
    /// Ensures the real part has unit magnitude and the dual part is orthogonal to the real part
    /// (condition for rigid transformation).
    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.real.length();

        if mag < 1e-8 {
            return Self::IDENTITY;
        }

        let inv_mag = 1.0 / mag;
        let real_norm = self.real * inv_mag;

        // For rigid transform, dot(real, dual) must be 0.
        // We project dual part onto the tangent space.
        // But simply dividing both by magnitude is usually sufficient if we started close to valid.
        // Correct normalization for Dual Quat Q = R + eps D:
        // Q_norm = Q / |R|.
        // And we subtract component of D parallel to R?
        // D' = D - R * dot(R, D) / dot(R, R).

        let dual_div = self.dual * inv_mag;
        let dot = real_norm.dot(dual_div);
        let dual_norm = dual_div - real_norm * dot;

        Self {
            real: real_norm,
            dual: dual_norm,
        }
    }

    /// Transform a point using the dual quaternion.
    ///
    /// # Arguments
    /// * `p` - The point to transform
    #[inline]
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        // p' = R p R* + 2 D R*
        // Translation t = 2 * dual * real_conj

        // Rotate point
        let p_rotated = self.real.mul_vec3(p);

        // Extract translation
        // t = 2 * dual * real^*
        let real_conj = self.real.conjugate();

        // D = 0.5 * t * R
        // t = 2 * D * R^*
        let t_comb = self.dual.mul_quat(real_conj);
        let translation = Vec3::new(t_comb.x * 2.0, t_comb.y * 2.0, t_comb.z * 2.0);

        p_rotated + translation
    }

    /// Blend two dual quaternions with a factor t.
    ///
    /// Performs shortest-path interpolation (handling sign flip).
    /// Result is normalized.
    ///
    /// # Arguments
    /// * `other` - The target dual quaternion
    /// * `t` - Blend factor [0, 1]
    #[inline]
    pub fn blend(&self, other: &Self, t: f32) -> Self {
        let dot = self.real.dot(other.real);
        let sign = if dot < 0.0 { -1.0 } else { 1.0 };

        let scale_a = 1.0 - t;
        let scale_b = t * sign;

        let blended_real = self.real * scale_a + other.real * scale_b;
        let blended_dual = self.dual * scale_a + other.dual * scale_b;

        let result = Self {
            real: blended_real,
            dual: blended_dual,
        };

        result.normalize()
    }
}
