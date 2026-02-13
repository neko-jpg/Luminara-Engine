//! Generic Vector3 implementation.
//!
//! Provides a generic Vector3 struct to support scalar types other than f32 (e.g., f64, Dual numbers).

use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};
use super::traits::Scalar;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + Mul<Output = T> + Sub<Output = T>> Vector3<T> {
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<T: Copy + Mul<Output = T> + Add<Output = T>> Vector3<T> {
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> T {
        self.dot(self)
    }
}

impl<T: Scalar> Vector3<T> {
    pub fn length(self) -> T {
        self.length_squared().sqrt()
    }

    pub fn distance(self, other: Self) -> T {
        (self - other).length()
    }

    pub fn normalize(self) -> Self {
        let l = self.length();
        if l != T::zero() {
            self / l
        } else {
            self
        }
    }
}

impl<T: Copy + Add<Output = T>> Add for Vector3<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Vector3<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Vector3<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Vector3<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Copy + Neg<Output = T>> Neg for Vector3<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Interop with glam::Vec3 for f32

impl From<glam::Vec3> for Vector3<f32> {
    fn from(v: glam::Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}


impl From<Vector3<f32>> for glam::Vec3 {
    fn from(v: Vector3<f32>) -> Self {
        glam::Vec3::new(v.x, v.y, v.z)
    }
}
