//! PGA Motor for unified rotation and translation representation.
//!
//! Provides a geometric algebra approach to rigid body transformations using
//! Projective Geometric Algebra (PGA). A Motor represents both rotation and
//! translation as a single algebraic object, avoiding gimbal lock and providing
//! efficient composition through the geometric product.

use super::bivector::Bivector;
use glam::{Quat, Vec3};

/// A motor in PGA(3,0,1) representing a rigid transformation (rotation + translation).
///
/// The eight components represent:
/// - `s`: Scalar part (related to rotation angle)
/// - `e12`, `e13`, `e23`: Rotational bivector components
/// - `e01`, `e02`, `e03`: Translational bivector components
/// - `e0123`: Pseudoscalar part (related to translation)
///
/// Motors are composed using the geometric product and can transform points
/// using the sandwich product: p' = M p M̃ (where M̃ is the reverse).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(32))]
pub struct Motor {
    /// Scalar component
    pub s: f32,
    /// Rotation around XY plane
    pub e12: f32,
    /// Rotation around XZ plane
    pub e13: f32,
    /// Rotation around YZ plane
    pub e23: f32,
    /// Translation in X direction
    pub e01: f32,
    /// Translation in Y direction
    pub e02: f32,
    /// Translation in Z direction
    pub e03: f32,
    /// Pseudoscalar component
    pub e0123: f32,
}

impl Motor {
    /// The identity motor (no rotation, no translation).
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e0123: 0.0,
    };

    /// Create a new motor from individual components.
    #[inline]
    pub const fn new(
        s: f32,
        e12: f32,
        e13: f32,
        e23: f32,
        e01: f32,
        e02: f32,
        e03: f32,
        e0123: f32,
    ) -> Self {
        Self {
            s,
            e12,
            e13,
            e23,
            e01,
            e02,
            e03,
            e0123,
        }
    }

    /// Create a motor representing a pure translation.
    ///
    /// # Arguments
    /// * `t` - Translation vector
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// let motor = Motor::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// ```
    #[inline]
    pub fn from_translation(t: Vec3) -> Self {
        // Pure translation motor: 1 - (1/2) * t * e0
        // In PGA: Motor = 1 + (t.x/2)*e01 + (t.y/2)*e02 + (t.z/2)*e03
        Self {
            s: 1.0,
            e12: 0.0,
            e13: 0.0,
            e23: 0.0,
            e01: t.x * 0.5,
            e02: t.y * 0.5,
            e03: t.z * 0.5,
            e0123: 0.0,
        }
    }

    /// Create a motor representing a rotation around an axis.
    ///
    /// # Arguments
    /// * `axis` - Rotation axis (will be normalized)
    /// * `angle` - Rotation angle in radians
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// # use std::f32::consts::PI;
    /// let motor = Motor::from_axis_angle(Vec3::Z, PI / 4.0);
    /// ```
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let axis = axis.normalize();
        let half_angle = angle * 0.5;
        let s = half_angle.cos();
        let c = half_angle.sin();

        // Rotation motor: cos(θ/2) + sin(θ/2) * (axis.x*e23 + axis.y*e13 + axis.z*e12)
        // Note: PGA uses a different convention for bivector components
        Self {
            s,
            e12: c * axis.z,
            e13: c * axis.y,
            e23: c * axis.x,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }

    /// Create a motor from a rotation quaternion and translation vector.
    ///
    /// # Arguments
    /// * `rot` - Rotation as a quaternion
    /// * `trans` - Translation vector
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::{Quat, Vec3};
    /// let rot = Quat::from_rotation_z(1.0);
    /// let trans = Vec3::new(1.0, 2.0, 3.0);
    /// let motor = Motor::from_rotation_translation(rot, trans);
    /// ```
    #[inline]
    pub fn from_rotation_translation(rot: Quat, trans: Vec3) -> Self {
        // Convert quaternion to motor rotation part
        let rot_motor = Self {
            s: rot.w,
            e12: rot.z,
            e13: rot.y,
            e23: rot.x,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        };

        // Create translation motor
        let trans_motor = Self::from_translation(trans);

        // Compose: translation * rotation (apply rotation first in the sandwich product)
        rot_motor.geometric_product(&trans_motor)
    }

    /// Extract rotation and translation from the motor.
    pub fn to_rotation_translation(&self) -> (Quat, Vec3) {
        // Extract rotation quaternion (normalized)
        let rot = Quat::from_xyzw(self.e23, self.e13, self.e12, self.s).normalize();

        // Extract translation
        // M = R * (1 + t/2)  =>  1 + t/2 = R_rev * M
        // We compute the translational part of R_rev * M
        // R_rev has s=s, e12=-e12, etc.

        let s = self.s;
        let e12 = self.e12;
        let e13 = self.e13;
        let e23 = self.e23;

        // Components of R_rev * M (translational part only)
        // t.x/2 = e01_res
        let half_tx = s * self.e01 - e12 * self.e02 - e13 * self.e03 + e23 * self.e0123;
        let half_ty = s * self.e02 + e12 * self.e01 - e23 * self.e03 - e13 * self.e0123;
        let half_tz = s * self.e03 + e13 * self.e01 + e23 * self.e02 + e12 * self.e0123;

        let trans = Vec3::new(half_tx * 2.0, half_ty * 2.0, half_tz * 2.0);

        (rot, trans)
    }

    /// Compute the geometric product of two motors.
    ///
    /// This composes two rigid transformations. The result represents applying
    /// `self` first, then `other`.
    ///
    /// # Arguments
    /// * `other` - The motor to compose with
    ///
    /// # Returns
    /// The composed motor
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// let m1 = Motor::from_translation(Vec3::X);
    /// let m2 = Motor::from_translation(Vec3::Y);
    /// let composed = m1.geometric_product(&m2);
    /// ```
    #[inline]
    pub fn geometric_product(&self, other: &Motor) -> Motor {
        // PGA Motor composition using the geometric antiproduct
        //
        // In PGA R(3,0,1), motors are elements of the even subalgebra and represent
        // rigid transformations. The composition uses the geometric antiproduct (⟇).
        //
        // This implementation is based on the multiplication table for the geometric
        // antiproduct in R(3,0,1), which differs from the geometric product in how
        // translation and rotation bivectors interact.
        //
        // Reference: Eric Lengyel, "Projective Geometric Algebra Done Right"
        // https://terathon.com/blog/pga-done-right.html
        // Reference: https://rigidgeometricalgebra.org/
        
        let a = self;
        let b = other;

        Motor {
            // Scalar part
            s: a.s * b.s - a.e12 * b.e12 - a.e13 * b.e13 - a.e23 * b.e23,
            
            // Rotation bivectors (e12, e13, e23) - quaternion-like multiplication
            // PGA products: e23*e13=+e12, e13*e23=-e12
            //               e12*e23=+e13, e23*e12=-e13
            //               e13*e12=+e23, e12*e13=-e23
            e12: a.s * b.e12 + a.e12 * b.s - a.e13 * b.e23 + a.e23 * b.e13,
            e13: a.s * b.e13 + a.e13 * b.s + a.e12 * b.e23 - a.e23 * b.e12,
            e23: a.s * b.e23 + a.e23 * b.s - a.e12 * b.e13 + a.e13 * b.e12,
            
            // Translation bivectors (e01, e02, e03)
            // These must transform under rotation correctly for associativity
            // 
            // The key insight: in the geometric antiproduct, the translation bivectors
            // interact with rotation bivectors through a rotation transformation
            e01: a.s * b.e01 + a.e01 * b.s 
                + a.e12 * b.e02 - a.e02 * b.e12 
                + a.e13 * b.e03 - a.e03 * b.e13
                - a.e23 * b.e0123 - a.e0123 * b.e23,
            e02: a.s * b.e02 + a.e02 * b.s 
                - a.e12 * b.e01 + a.e01 * b.e12 
                + a.e23 * b.e03 - a.e03 * b.e23
                + a.e13 * b.e0123 + a.e0123 * b.e13,
            e03: a.s * b.e03 + a.e03 * b.s 
                - a.e13 * b.e01 + a.e01 * b.e13 
                - a.e23 * b.e02 + a.e02 * b.e23
                - a.e12 * b.e0123 - a.e0123 * b.e12,
            
            // Pseudoscalar (e0123)
            e0123: a.s * b.e0123 + a.e0123 * b.s 
                + a.e01 * b.e23 + a.e23 * b.e01
                - a.e02 * b.e13 - a.e13 * b.e02 
                + a.e03 * b.e12 + a.e12 * b.e03,
        }
    }

    /// SIMD-optimized geometric product using AVX2 (x86_64 only).
    ///
    /// This is automatically used when AVX2 is available on the target platform.
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[inline]
    pub fn geometric_product_simd(&self, other: &Motor) -> Motor {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::*;

            // Load motors into 256-bit registers
            let a_vec = _mm256_load_ps(&self.s as *const f32);
            let b_vec = _mm256_load_ps(&other.s as *const f32);

            // For a full SIMD implementation, we would need to carefully arrange
            // the operations to maximize parallelism. For now, we fall back to
            // the scalar version as a proper SIMD implementation requires
            // careful analysis of the geometric product structure.
            
            // TODO: Implement full SIMD version with proper shuffles and FMA
            self.geometric_product(other)
        }
    }

    /// SIMD-optimized geometric product using NEON (ARM only).
    ///
    /// This is automatically used when NEON is available on the target platform.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[inline]
    pub fn geometric_product_simd(&self, other: &Motor) -> Motor {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::*;

            // Load motors into 128-bit registers (NEON uses 128-bit vectors)
            // We would need two loads per motor since we have 8 f32 values
            
            // For now, fall back to scalar version
            // TODO: Implement full NEON version
            self.geometric_product(other)
        }
    }

    /// Compute the reverse (conjugate) of the motor.
    ///
    /// For a normalized motor, the reverse is the inverse.
    ///
    /// # Returns
    /// The reversed motor
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// let m = Motor::IDENTITY;
    /// let m_rev = m.reverse();
    /// ```
    #[inline]
    pub fn reverse(&self) -> Motor {
        // The reverse negates grade-2 elements (bivectors) but preserves
        // grade-0 (scalar) and grade-4 (pseudoscalar e0123).
        // Grade k reversal sign: (-1)^(k(k-1)/2)
        //   k=0: +1, k=2: -1, k=4: +1
        Motor {
            s: self.s,
            e12: -self.e12,
            e13: -self.e13,
            e23: -self.e23,
            e01: -self.e01,
            e02: -self.e02,
            e03: -self.e03,
            e0123: self.e0123,
        }
    }

    /// Transform a point using the motor (sandwich product: M p M̃).
    ///
    /// # Arguments
    /// * `p` - The point to transform
    ///
    /// # Returns
    /// The transformed point
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// let motor = Motor::from_translation(Vec3::new(1.0, 0.0, 0.0));
    /// let point = Vec3::ZERO;
    /// let transformed = motor.transform_point(point);
    /// ```
    #[inline]
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        // In PGA, a point is represented as: x*e032 + y*e013 + z*e021 + e123
        // For efficiency, we compute the sandwich product directly.
        //
        // The quaternion-like mapping is: (w,x,y,z) = (s, e23, e13, e12)
        // where e23 = rotation around X, e13 = rotation around Y, e12 = rotation around Z.
        
        let x = p.x;
        let y = p.y;
        let z = p.z;

        let two_s = 2.0 * self.s;
        let two_e13 = 2.0 * self.e13;
        let two_e23 = 2.0 * self.e23;

        // Rotation matrix using quaternion mapping (w=s, i=e23, j=e13, k=e12)
        let rx = x * (self.s * self.s + self.e23 * self.e23 - self.e13 * self.e13 - self.e12 * self.e12)
            + y * (two_e23 * self.e13 - two_s * self.e12)
            + z * (two_e23 * self.e12 + two_s * self.e13);

        let ry = x * (two_e23 * self.e13 + two_s * self.e12)
            + y * (self.s * self.s - self.e23 * self.e23 + self.e13 * self.e13 - self.e12 * self.e12)
            + z * (two_e13 * self.e12 - two_s * self.e23);

        let rz = x * (two_e23 * self.e12 - two_s * self.e13)
            + y * (two_e13 * self.e12 + two_s * self.e23)
            + z * (self.s * self.s - self.e23 * self.e23 - self.e13 * self.e13 + self.e12 * self.e12);

        // Apply translation (the e01, e02, e03 components encode translation)
        Vec3::new(
            rx + 2.0 * self.e01,
            ry + 2.0 * self.e02,
            rz + 2.0 * self.e03,
        )
    }

    /// Transform a vector using the motor (rotation only, no translation).
    ///
    /// # Arguments
    /// * `v` - The vector to transform
    ///
    /// # Returns
    /// The transformed vector
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// # use std::f32::consts::PI;
    /// let motor = Motor::from_axis_angle(Vec3::Z, PI / 2.0);
    /// let vector = Vec3::X;
    /// let transformed = motor.transform_vector(vector);
    /// ```
    #[inline]
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        // Vectors are transformed by rotation only (no translation)
        // Uses same quaternion mapping as transform_point: (w=s, i=e23, j=e13, k=e12)
        let x = v.x;
        let y = v.y;
        let z = v.z;

        let two_s = 2.0 * self.s;
        let two_e13 = 2.0 * self.e13;
        let two_e23 = 2.0 * self.e23;

        Vec3::new(
            x * (self.s * self.s + self.e23 * self.e23 - self.e13 * self.e13 - self.e12 * self.e12)
                + y * (two_e23 * self.e13 - two_s * self.e12)
                + z * (two_e23 * self.e12 + two_s * self.e13),
            x * (two_e23 * self.e13 + two_s * self.e12)
                + y * (self.s * self.s - self.e23 * self.e23 + self.e13 * self.e13 - self.e12 * self.e12)
                + z * (two_e13 * self.e12 - two_s * self.e23),
            x * (two_e23 * self.e12 - two_s * self.e13)
                + y * (two_e13 * self.e12 + two_s * self.e23)
                + z * (self.s * self.s - self.e23 * self.e23 - self.e13 * self.e13 + self.e12 * self.e12),
        )
    }

    /// Compute the logarithm of the motor, mapping it to a bivector (Lie algebra).
    ///
    /// This extracts the "velocity" representation from the motor.
    ///
    /// # Returns
    /// The bivector representing the logarithm
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// let motor = Motor::from_translation(Vec3::X);
    /// let bivector = motor.log();
    /// ```
    #[inline]
    pub fn log(&self) -> Bivector {
        // For small angles, use first-order approximation
        let rotation_magnitude_sq = self.e12 * self.e12 + self.e13 * self.e13 + self.e23 * self.e23;
        
        if rotation_magnitude_sq < 1e-8 {
            // Small angle approximation: log(Motor) ≈ bivector part
            return Bivector::new(
                self.e12 * 2.0,
                self.e13 * 2.0,
                self.e23 * 2.0,
                self.e01 * 2.0,
                self.e02 * 2.0,
                self.e03 * 2.0,
            );
        }

        // General case: extract rotation angle and axis
        let rotation_magnitude = rotation_magnitude_sq.sqrt();
        let angle = 2.0 * rotation_magnitude.atan2(self.s);
        let scale = angle / rotation_magnitude;

        Bivector::new(
            self.e12 * scale,
            self.e13 * scale,
            self.e23 * scale,
            self.e01 * 2.0,
            self.e02 * 2.0,
            self.e03 * 2.0,
        )
    }

    /// Compute the exponential of a bivector, mapping it to a motor (Lie group).
    ///
    /// This converts a "velocity" representation to a motor.
    ///
    /// # Arguments
    /// * `b` - The bivector to exponentiate
    ///
    /// # Returns
    /// The motor representing the exponential
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::{Motor, Bivector};
    /// let bivector = Bivector::new(0.0, 0.0, 1.0, 0.5, 0.0, 0.0);
    /// let motor = Motor::exp(&bivector);
    /// ```
    #[inline]
    pub fn exp(b: &Bivector) -> Motor {
        // Rotation part
        let rotation_magnitude_sq = b.e12 * b.e12 + b.e13 * b.e13 + b.e23 * b.e23;
        
        if rotation_magnitude_sq < 1e-8 {
            // Small angle approximation
            return Motor {
                s: 1.0,
                e12: b.e12 * 0.5,
                e13: b.e13 * 0.5,
                e23: b.e23 * 0.5,
                e01: b.e01 * 0.5,
                e02: b.e02 * 0.5,
                e03: b.e03 * 0.5,
                e0123: 0.0,
            };
        }

        // General case
        let rotation_magnitude = rotation_magnitude_sq.sqrt();
        let half_angle = rotation_magnitude * 0.5;
        let s = half_angle.cos();
        let c = half_angle.sin() / rotation_magnitude;

        Motor {
            s,
            e12: b.e12 * c,
            e13: b.e13 * c,
            e23: b.e23 * c,
            e01: b.e01 * 0.5,
            e02: b.e02 * 0.5,
            e03: b.e03 * 0.5,
            e0123: 0.0,
        }
    }

    /// Interpolate between two motors.
    ///
    /// This performs decoupled interpolation of rotation (Slerp) and translation (Lerp).
    /// This ensures smoothness and correct endpoint behavior even for large screw motions
    /// where the approximate log/exp map might fail.
    ///
    /// # Arguments
    /// * `other` - The target motor
    /// * `t` - Interpolation parameter (0.0 = self, 1.0 = other)
    ///
    /// # Returns
    /// The interpolated motor
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// # use glam::Vec3;
    /// let m1 = Motor::from_translation(Vec3::ZERO);
    /// let m2 = Motor::from_translation(Vec3::X);
    /// let interpolated = m1.interpolate(&m2, 0.5);
    /// ```
    #[inline]
    pub fn interpolate(&self, other: &Motor, t: f32) -> Motor {
        let (r1, t1) = self.to_rotation_translation();
        let (r2, t2) = other.to_rotation_translation();
        
        let r_interp = r1.slerp(r2, t);
        let t_interp = t1.lerp(t2, t);
        
        Motor::from_rotation_translation(r_interp, t_interp)
    }

    /// Normalize the motor to counteract numerical drift.
    ///
    /// This ensures the motor remains a valid rigid transformation.
    ///
    /// # Example
    /// ```
    /// # use luminara_math::algebra::Motor;
    /// let mut motor = Motor::IDENTITY;
    /// motor.normalize();
    /// ```
    #[inline]
    pub fn normalize(&mut self) {
        // Compute the norm of the motor
        // For a motor, the norm is computed from the scalar and bivector parts
        let norm_sq = self.s * self.s 
            + self.e12 * self.e12 
            + self.e13 * self.e13 
            + self.e23 * self.e23;
        
        if norm_sq > 1e-8 {
            let inv_norm = 1.0 / norm_sq.sqrt();
            self.s *= inv_norm;
            self.e12 *= inv_norm;
            self.e13 *= inv_norm;
            self.e23 *= inv_norm;
            self.e01 *= inv_norm;
            self.e02 *= inv_norm;
            self.e03 *= inv_norm;
            self.e0123 *= inv_norm;
        }
    }

    /// Compute the squared norm of the motor.
    ///
    /// # Returns
    /// The squared norm
    #[inline]
    pub fn norm_squared(&self) -> f32 {
        self.s * self.s 
            + self.e12 * self.e12 
            + self.e13 * self.e13 
            + self.e23 * self.e23
            + self.e01 * self.e01
            + self.e02 * self.e02
            + self.e03 * self.e03
            + self.e0123 * self.e0123
    }

    /// Compute the norm of the motor.
    ///
    /// # Returns
    /// The norm
    #[inline]
    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }
}
