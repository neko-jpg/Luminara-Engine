//! PGA Rotor (Rotation).
//!
//! Represents a rotation in 3D space using the even subalgebra of PGA.
//! Isomorphic to a quaternion.


use glam::{Quat, Vec3};
use std::ops::{Add, Sub, Mul};

/// A rotor representing a rotation.
///
/// Elements: scalar (s), bivectors (e12, e13, e23).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Rotor<T> {
    pub s: T,
    pub e12: T,
    pub e13: T,
    pub e23: T,
}

impl Rotor<f32> {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
    };


    pub fn from_quat(q: Quat) -> Self {
        Self {
            s: q.w,
            e12: q.z,
            e13: q.y,
            e23: q.x,
        }
    }


    pub fn to_quat(&self) -> Quat {
        Quat::from_xyzw(self.e23, self.e13, self.e12, self.s)
    }
}

impl<T> Rotor<T> {
    pub fn new(s: T, e12: T, e13: T, e23: T) -> Self {
        Self { s, e12, e13, e23 }
    }
}

impl<T: Copy + Neg<Output=T>> Rotor<T> {
    pub fn reverse(&self) -> Self {
        Self {
            s: self.s,
            e12: -self.e12,
            e13: -self.e13,
            e23: -self.e23,
        }
    }
}

use std::ops::Neg;

impl<T: Copy + Add<Output=T> + Sub<Output=T> + Mul<Output=T>> Rotor<T> {
    pub fn geometric_product(&self, other: &Rotor<T>) -> Rotor<T> {
        // Quaternion multiplication
        Rotor {
            s: self.s * other.s - self.e12 * other.e12 - self.e13 * other.e13 - self.e23 * other.e23,
            e12: self.s * other.e12 + self.e12 * other.s - self.e13 * other.e23 + self.e23 * other.e13,
            e13: self.s * other.e13 + self.e13 * other.s + self.e12 * other.e23 - self.e23 * other.e12,
            e23: self.s * other.e23 + self.e23 * other.s - self.e12 * other.e13 + self.e13 * other.e12,
        }
    }
}

impl<T: Copy + Add<Output=T> + Sub<Output=T> + Mul<Output=T>> std::ops::Mul for Rotor<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.geometric_product(&rhs)
    }
}
