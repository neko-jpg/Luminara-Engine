//! PGA Plane (3D Plane).
//!
//! Represents a plane in 3D space using the odd subalgebra (1-vectors) of PGA.
//! P = nx*e1 + ny*e2 + nz*e3 + d*e0

use super::vector::Vector3;
use super::traits::Scalar;


use glam::Vec3;

/// A plane in 3D space.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Plane<T> {
    pub nx: T,
    pub ny: T,
    pub nz: T,
    pub d: T,
}

impl<T: Scalar> Plane<T> {
    /// Create a new plane from normal and distance.
    pub fn new(nx: T, ny: T, nz: T, d: T) -> Self {
        Self { nx, ny, nz, d }
    }

    /// Create from normal vector and distance.
    pub fn from_normal_dist(normal: Vector3<T>, dist: T) -> Self {
        Self {
            nx: normal.x,
            ny: normal.y,
            nz: normal.z,
            d: dist,
        }
    }

    /// Normalize the plane equation.
    pub fn normalize(&mut self) {
        let mag = (self.nx * self.nx + self.ny * self.ny + self.nz * self.nz).sqrt();
        if mag > T::zero() { // Check if not too small
            let inv_mag = T::one() / mag;
            self.nx *= inv_mag;
            self.ny *= inv_mag;
            self.nz *= inv_mag;
            self.d *= inv_mag;
        }
    }
}

// Interop
impl Plane<f32> {

    pub fn from_normal_dist_glam(normal: Vec3, dist: f32) -> Self {
        Self {
            nx: normal.x,
            ny: normal.y,
            nz: normal.z,
            d: dist,
        }
    }
}
