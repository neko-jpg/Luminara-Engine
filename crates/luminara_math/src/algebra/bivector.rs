//! Bivector (Lie algebra elements) for PGA.
//!
//! Represents elements of the Lie algebra se(3) in PGA representation.
//! A Bivector represents infinitesimal rotations and translations in 3D space.

/// A bivector in PGA(3,0,1) representing an element of the Lie algebra se(3).
///
/// The six components represent:
/// - `e12`, `e13`, `e23`: Rotational components (angular velocity)
/// - `e01`, `e02`, `e03`: Translational components (linear velocity)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bivector {
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
}

impl Bivector {
    /// Create a new bivector with all components set to zero.
    pub const ZERO: Self = Self {
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
    };

    /// Create a new bivector from individual components.
    #[inline]
    pub const fn new(e12: f32, e13: f32, e23: f32, e01: f32, e02: f32, e03: f32) -> Self {
        Self {
            e12,
            e13,
            e23,
            e01,
            e02,
            e03,
        }
    }

    /// Add two bivectors component-wise.
    #[inline]
    pub fn add(&self, other: &Bivector) -> Bivector {
        Bivector {
            e12: self.e12 + other.e12,
            e13: self.e13 + other.e13,
            e23: self.e23 + other.e23,
            e01: self.e01 + other.e01,
            e02: self.e02 + other.e02,
            e03: self.e03 + other.e03,
        }
    }

    /// Subtract two bivectors component-wise.
    #[inline]
    pub fn sub(&self, other: &Bivector) -> Bivector {
        Bivector {
            e12: self.e12 - other.e12,
            e13: self.e13 - other.e13,
            e23: self.e23 - other.e23,
            e01: self.e01 - other.e01,
            e02: self.e02 - other.e02,
            e03: self.e03 - other.e03,
        }
    }

    /// Scale a bivector by a scalar.
    #[inline]
    pub fn scale(&self, s: f32) -> Bivector {
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
    pub fn norm_squared(&self) -> f32 {
        self.e12 * self.e12
            + self.e13 * self.e13
            + self.e23 * self.e23
            + self.e01 * self.e01
            + self.e02 * self.e02
            + self.e03 * self.e03
    }

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
        assert_eq!(b.e23, 0.0);
        assert_eq!(b.e01, 0.0);
        assert_eq!(b.e02, 0.0);
        assert_eq!(b.e03, 0.0);
    }

    #[test]
    fn bivector_new_creates_correct_components() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(b.e12, 1.0);
        assert_eq!(b.e13, 2.0);
        assert_eq!(b.e23, 3.0);
        assert_eq!(b.e01, 4.0);
        assert_eq!(b.e02, 5.0);
        assert_eq!(b.e03, 6.0);
    }

    #[test]
    fn bivector_add_works_correctly() {
        let b1 = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b2 = Bivector::new(0.5, 1.5, 2.5, 3.5, 4.5, 5.5);
        let result = b1.add(&b2);
        
        assert_eq!(result.e12, 1.5);
        assert_eq!(result.e13, 3.5);
        assert_eq!(result.e23, 5.5);
        assert_eq!(result.e01, 7.5);
        assert_eq!(result.e02, 9.5);
        assert_eq!(result.e03, 11.5);
    }

    #[test]
    fn bivector_sub_works_correctly() {
        let b1 = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b2 = Bivector::new(0.5, 1.5, 2.5, 3.5, 4.5, 5.5);
        let result = b1.sub(&b2);
        
        assert_eq!(result.e12, 0.5);
        assert_eq!(result.e13, 0.5);
        assert_eq!(result.e23, 0.5);
        assert_eq!(result.e01, 0.5);
        assert_eq!(result.e02, 0.5);
        assert_eq!(result.e03, 0.5);
    }

    #[test]
    fn bivector_scale_works_correctly() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let result = b.scale(2.0);
        
        assert_eq!(result.e12, 2.0);
        assert_eq!(result.e13, 4.0);
        assert_eq!(result.e23, 6.0);
        assert_eq!(result.e01, 8.0);
        assert_eq!(result.e02, 10.0);
        assert_eq!(result.e03, 12.0);
    }

    #[test]
    fn bivector_scale_by_zero_gives_zero() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let result = b.scale(0.0);
        assert_eq!(result, Bivector::ZERO);
    }

    #[test]
    fn bivector_norm_squared_is_correct() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let norm_sq = b.norm_squared();
        // 1^2 + 2^2 + 3^2 + 4^2 + 5^2 + 6^2 = 1 + 4 + 9 + 16 + 25 + 36 = 91
        assert_eq!(norm_sq, 91.0);
    }

    #[test]
    fn bivector_norm_is_correct() {
        let b = Bivector::new(3.0, 0.0, 0.0, 4.0, 0.0, 0.0);
        let norm = b.norm();
        // sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5
        assert_eq!(norm, 5.0);
    }

    #[test]
    fn bivector_zero_has_zero_norm() {
        let b = Bivector::ZERO;
        assert_eq!(b.norm_squared(), 0.0);
        assert_eq!(b.norm(), 0.0);
    }

    #[test]
    fn bivector_add_is_commutative() {
        let b1 = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b2 = Bivector::new(0.5, 1.5, 2.5, 3.5, 4.5, 5.5);
        
        let result1 = b1.add(&b2);
        let result2 = b2.add(&b1);
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn bivector_add_zero_is_identity() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let result = b.add(&Bivector::ZERO);
        assert_eq!(result, b);
    }

    #[test]
    fn bivector_sub_self_gives_zero() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let result = b.sub(&b);
        assert_eq!(result, Bivector::ZERO);
    }

    #[test]
    fn bivector_scale_by_one_is_identity() {
        let b = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let result = b.scale(1.0);
        assert_eq!(result, b);
    }

    #[test]
    fn bivector_scale_is_distributive() {
        let b1 = Bivector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b2 = Bivector::new(0.5, 1.5, 2.5, 3.5, 4.5, 5.5);
        let s = 2.0;
        
        // s * (b1 + b2) = s * b1 + s * b2
        let left = b1.add(&b2).scale(s);
        let right = b1.scale(s).add(&b2.scale(s));
        
        assert_eq!(left, right);
    }
}
