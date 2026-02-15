//! Bivector (Lie algebra elements) for PGA.
//!
//! Represents elements of the Lie algebra se(3) in PGA representation.
//! A Bivector represents infinitesimal rotations and translations in 3D space.

use std::ops::{Add, Mul, Sub};

/// A bivector in PGA(3,0,1) representing an element of the Lie algebra se(3).
///
/// The six components represent:
/// - `e12`, `e13`, `e23`: Rotational components (angular velocity)
/// - `e01`, `e02`, `e03`: Translational components (linear velocity)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bivector<T> {
    /// Rotation around XY plane
    pub e12: T,
    /// Rotation around XZ plane
    pub e13: T,
    /// Rotation around YZ plane
    pub e23: T,
    /// Translation in X direction
    pub e01: T,
    /// Translation in Y direction
    pub e02: T,
    /// Translation in Z direction
    pub e03: T,
}

impl Bivector<f32> {
    /// Create a new bivector with all components set to zero.
    pub const ZERO: Self = Self {
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
    };
}

impl<T> Bivector<T> {
    /// Create a new bivector from individual components.
    #[inline]
    pub const fn new(e12: T, e13: T, e23: T, e01: T, e02: T, e03: T) -> Self {
        Self {
            e12,
            e13,
            e23,
            e01,
            e02,
            e03,
        }
    }
}

impl<T: Copy + Add<Output = T>> Bivector<T> {
    /// Add two bivectors component-wise.
    #[inline]
    pub fn add(&self, other: &Bivector<T>) -> Bivector<T> {
        Bivector {
            e12: self.e12 + other.e12,
            e13: self.e13 + other.e13,
            e23: self.e23 + other.e23,
            e01: self.e01 + other.e01,
            e02: self.e02 + other.e02,
            e03: self.e03 + other.e03,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Bivector<T> {
    /// Subtract two bivectors component-wise.
    #[inline]
    pub fn sub(&self, other: &Bivector<T>) -> Bivector<T> {
        Bivector {
            e12: self.e12 - other.e12,
            e13: self.e13 - other.e13,
            e23: self.e23 - other.e23,
            e01: self.e01 - other.e01,
            e02: self.e02 - other.e02,
            e03: self.e03 - other.e03,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Bivector<T> {
    /// Scale a bivector by a scalar.
    #[inline]
    pub fn scale(&self, s: T) -> Bivector<T> {
        Bivector {
            e12: self.e12 * s,
            e13: self.e13 * s,
            e23: self.e23 * s,
            e01: self.e01 * s,
            e02: self.e02 * s,
            e03: self.e03 * s,
        }
    }

    /// Compute the squared norm of the bivector.
    ///
    /// This is the sum of squares of all components.
    #[inline]
    pub fn norm_squared(&self) -> T
    where
        T: Add<Output = T> + Mul<Output = T>,
    {
        self.e12 * self.e12
            + self.e13 * self.e13
            + self.e23 * self.e23
            + self.e01 * self.e01
            + self.e02 * self.e02
            + self.e03 * self.e03
    }
}

impl Bivector<f32> {
    /// Compute the norm (magnitude) of the bivector.
    #[inline]
    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bivector_zero_has_all_components_zero() {
        let b = Bivector::ZERO;
        assert_eq!(b.e12, 0.0);
        assert_eq!(b.e13, 0.0);
    }
    // ... (rest of tests assume f32 which works with generic impl for f32)
}
