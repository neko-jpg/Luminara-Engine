//! PGA Point (3D Position).
//!
//! Represents a point in 3D space using the odd subalgebra (trivectors) of PGA.
//! Points are dual to planes.
//! P = w*e123 + x*e032 + y*e013 + z*e021

use super::traits::Scalar;
use super::vector::Vector3;

use glam::Vec3;

/// A point in 3D space.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: Scalar> Point<T> {
    /// Create a new point from coordinates.
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
            w: T::one(),
        }
    }

    /// Create a new point with homogeneous coordinate.
    pub fn new_hom(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    /// Convert to Vector3 (divide by w).
    pub fn to_vector3(&self) -> Vector3<T> {
        if self.w != T::zero() {
            let inv_w = T::one() / self.w;
            Vector3::new(self.x * inv_w, self.y * inv_w, self.z * inv_w)
        } else {
            // Point at infinity
            Vector3::new(self.x, self.y, self.z)
        }
    }

    /// Normalize the point so w=1.
    pub fn normalize(&mut self) {
        if self.w != T::zero() {
            let inv_w = T::one() / self.w;
            self.x *= inv_w;
            self.y *= inv_w;
            self.z *= inv_w;
            self.w = T::one();
        }
    }
}

// Interop
impl Point<f32> {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::from(self.to_vector3())
    }
}
