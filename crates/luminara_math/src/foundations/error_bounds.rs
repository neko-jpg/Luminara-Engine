//! Error bound constants for adaptive precision arithmetic.
//!
//! These constants are derived from IEEE 754 floating-point arithmetic
//! properties and are used in the adaptive filter stages of exact geometric predicates.
//!
//! The constants follow Shewchuk's adaptive precision arithmetic approach,
//! where predicates use a multi-stage filter:
//! - Stage A: Fast f64 arithmetic with error bound check (99.5% of cases)
//! - Stage B: Intermediate precision with partial error terms
//! - Stage C: Higher precision with more error terms
//! - Stage D: Full expansion arithmetic (exact result)

/// Machine epsilon for IEEE 754 double precision (f64).
/// This is the smallest value ε such that 1.0 + ε ≠ 1.0.
/// For f64, this is 2^-52 ≈ 2.220446049250313e-16
pub const EPSILON: f64 = f64::EPSILON;

/// Error bound for Stage A (fast filter) of orient2d predicate.
/// This bound determines when the fast floating-point result is reliable.
/// Derived from the error analysis of the orient2d determinant computation.
///
/// The orient2d predicate computes: (pa.x - pc.x) * (pb.y - pc.y) - (pa.y - pc.y) * (pb.x - pc.x)
/// The error bound accounts for rounding errors in subtraction and multiplication.
pub const RESULT_ERRBOUND_A: f64 = (3.0 + 16.0 * EPSILON) * EPSILON;

/// Error bound for Stage B (intermediate precision) of orient2d predicate.
/// Used when Stage A fails to produce a definitive result.
/// This stage includes some error correction terms.
pub const RESULT_ERRBOUND_B: f64 = (2.0 + 12.0 * EPSILON) * EPSILON;

/// Error bound for Stage C (higher precision) of orient2d predicate.
/// Used when Stage B fails to produce a definitive result.
/// This stage includes more error correction terms.
pub const RESULT_ERRBOUND_C: f64 = (9.0 + 64.0 * EPSILON) * EPSILON * EPSILON;

/// CCW (counterclockwise) error bound for Stage A of orient2d.
/// Alias for RESULT_ERRBOUND_A for clarity in orient2d implementation.
pub const CCWERRBOUND_A: f64 = RESULT_ERRBOUND_A;

/// CCW error bound for Stage B of orient2d.
pub const CCWERRBBOUND_B: f64 = RESULT_ERRBOUND_B;

/// CCW error bound for Stage C of orient2d.
pub const CCWERRBBOUND_C: f64 = RESULT_ERRBOUND_C;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epsilon_value() {
        // Verify that EPSILON is the machine epsilon for f64
        assert_eq!(EPSILON, 2.220446049250313e-16);

        // Verify the property: 1.0 + EPSILON != 1.0
        assert_ne!(1.0 + EPSILON, 1.0);

        // Verify that EPSILON/2 is too small to distinguish
        assert_eq!(1.0 + EPSILON / 2.0, 1.0);
    }

    #[test]
    fn test_error_bounds_ordering() {
        // Error bounds should be in increasing order of magnitude
        // (smaller bounds for faster stages)
        assert!(RESULT_ERRBOUND_A > RESULT_ERRBOUND_B);
        assert!(RESULT_ERRBOUND_B > RESULT_ERRBOUND_C);
    }

    #[test]
    fn test_error_bounds_positive() {
        // All error bounds should be positive
        assert!(RESULT_ERRBOUND_A > 0.0);
        assert!(RESULT_ERRBOUND_B > 0.0);
        assert!(RESULT_ERRBOUND_C > 0.0);
    }

    #[test]
    fn test_error_bounds_finite() {
        // All error bounds should be finite
        assert!(RESULT_ERRBOUND_A.is_finite());
        assert!(RESULT_ERRBOUND_B.is_finite());
        assert!(RESULT_ERRBOUND_C.is_finite());
    }
}

/// Incircle error bound for Stage A (fast filter).
pub const INCIRCLE_ERRBOUND_A: f64 = (10.0 + 96.0 * EPSILON) * EPSILON;

/// Incircle error bound for Stage B (intermediate precision).
pub const INCIRCLE_ERRBOUND_B: f64 = (5.0 + 48.0 * EPSILON) * EPSILON;

/// Orient3d error bound for Stage A (fast filter).
pub const ORIENT3D_ERRBOUND_A: f64 = (7.0 + 56.0 * EPSILON) * EPSILON;

/// Insphere error bound for Stage A (fast filter).
pub const INSPHERE_ERRBOUND_A: f64 = (16.0 + 224.0 * EPSILON) * EPSILON;
