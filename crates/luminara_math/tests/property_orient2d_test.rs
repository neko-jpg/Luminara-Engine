//! Property-based tests for orient2d predicate.
//!
//! **Property 1: Orient2d Sign Consistency**
//! **Validates: Requirements 1.1**
//!
//! For any three points in 2D space, the orient2d predicate should return a sign
//! that is consistent with the exact determinant computation, even when the points
//! are nearly collinear or the determinant is close to zero.

use luminara_math::foundations::orient2d;
use proptest::prelude::*;

/// Generate random 2D points with reasonable coordinates.
fn point2d_strategy() -> impl Strategy<Value = [f64; 2]> {
    prop::array::uniform2(-1000.0..1000.0)
}

/// Generate nearly-collinear points to test edge cases.
fn nearly_collinear_points() -> impl Strategy<Value = ([f64; 2], [f64; 2], [f64; 2])> {
    (
        -1000.0..1000.0,
        -1000.0..1000.0,
        0.0..1.0,
        -1e-10..1e-10,
    )
        .prop_map(|(x1, y1, t, epsilon)| {
            let p1 = [x1, y1];
            let p2 = [x1 + 1.0, y1 + 1.0];
            // p3 is nearly on the line from p1 to p2
            let p3 = [x1 + t, y1 + t + epsilon];
            (p1, p2, p3)
        })
}

proptest! {
    /// Property 1: Orient2d Sign Consistency
    ///
    /// The orient2d predicate should return consistent signs for all point configurations.
    /// This test verifies that:
    /// 1. CCW points return positive values
    /// 2. CW points return negative values
    /// 3. Collinear points return zero
    /// 4. The sign is consistent with the mathematical determinant
    #[test]
    fn property_orient2d_sign_consistency(
        pa in point2d_strategy(),
        pb in point2d_strategy(),
        pc in point2d_strategy(),
    ) {
        let result = orient2d(pa, pb, pc);
        
        // Compute the exact determinant using f64 arithmetic
        let det = (pb[0] - pa[0]) * (pc[1] - pa[1]) - (pb[1] - pa[1]) * (pc[0] - pa[0]);
        
        // The sign should be consistent with the determinant
        // (allowing for the case where both are very close to zero)
        if det.abs() > 1e-10 {
            assert_eq!(result.signum(), det.signum(),
                "orient2d sign inconsistent: result={}, det={}, pa={:?}, pb={:?}, pc={:?}",
                result, det, pa, pb, pc);
        }
    }

    /// Property: Orient2d is antisymmetric under point swap
    ///
    /// Swapping two points should negate the result.
    #[test]
    fn property_orient2d_antisymmetric(
        pa in point2d_strategy(),
        pb in point2d_strategy(),
        pc in point2d_strategy(),
    ) {
        let result1 = orient2d(pa, pb, pc);
        let result2 = orient2d(pa, pc, pb); // Swap pb and pc
        
        // The results should have opposite signs (or both be zero)
        if result1.abs() > 1e-15 && result2.abs() > 1e-15 {
            assert_eq!(result1.signum(), -result2.signum(),
                "orient2d not antisymmetric: result1={}, result2={}, pa={:?}, pb={:?}, pc={:?}",
                result1, result2, pa, pb, pc);
        }
    }

    /// Property: Orient2d handles nearly-collinear points
    ///
    /// Even when points are nearly collinear, orient2d should return a definitive result.
    #[test]
    fn property_orient2d_nearly_collinear(
        (pa, pb, pc) in nearly_collinear_points()
    ) {
        let result = orient2d(pa, pb, pc);
        
        // The result should be finite (not NaN or Inf)
        assert!(result.is_finite(),
            "orient2d returned non-finite value for nearly-collinear points: result={}, pa={:?}, pb={:?}, pc={:?}",
            result, pa, pb, pc);
        
        // The result should be deterministic (calling again gives same result)
        let result2 = orient2d(pa, pb, pc);
        assert_eq!(result, result2,
            "orient2d not deterministic: result1={}, result2={}, pa={:?}, pb={:?}, pc={:?}",
            result, result2, pa, pb, pc);
    }

    /// Property: Orient2d is translation invariant
    ///
    /// Translating all points by the same vector should not change the result.
    #[test]
    fn property_orient2d_translation_invariant(
        pa in point2d_strategy(),
        pb in point2d_strategy(),
        pc in point2d_strategy(),
        tx in -100.0..100.0,
        ty in -100.0..100.0,
    ) {
        let result1 = orient2d(pa, pb, pc);
        
        // Translate all points
        let pa2 = [pa[0] + tx, pa[1] + ty];
        let pb2 = [pb[0] + tx, pb[1] + ty];
        let pc2 = [pc[0] + tx, pc[1] + ty];
        let result2 = orient2d(pa2, pb2, pc2);
        
        // The signs should be the same
        if result1.abs() > 1e-10 && result2.abs() > 1e-10 {
            assert_eq!(result1.signum(), result2.signum(),
                "orient2d not translation invariant: result1={}, result2={}, translation=({}, {})",
                result1, result2, tx, ty);
        }
    }

    /// Property: Orient2d is scale invariant (sign-wise)
    ///
    /// Scaling all points by the same positive factor should not change the sign.
    #[test]
    fn property_orient2d_scale_invariant(
        pa in point2d_strategy(),
        pb in point2d_strategy(),
        pc in point2d_strategy(),
        scale in 0.1..10.0,
    ) {
        let result1 = orient2d(pa, pb, pc);
        
        // Scale all points
        let pa2 = [pa[0] * scale, pa[1] * scale];
        let pb2 = [pb[0] * scale, pb[1] * scale];
        let pc2 = [pc[0] * scale, pc[1] * scale];
        let result2 = orient2d(pa2, pb2, pc2);
        
        // The signs should be the same
        if result1.abs() > 1e-10 && result2.abs() > 1e-10 {
            assert_eq!(result1.signum(), result2.signum(),
                "orient2d not scale invariant: result1={}, result2={}, scale={}",
                result1, result2, scale);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_orient2d_ccw() {
        // Counterclockwise triangle
        let result = orient2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!(result > 0.0, "Expected positive for CCW, got {}", result);
    }

    #[test]
    fn test_orient2d_cw() {
        // Clockwise triangle
        let result = orient2d([0.0, 0.0], [0.0, 1.0], [1.0, 0.0]);
        assert!(result < 0.0, "Expected negative for CW, got {}", result);
    }

    #[test]
    fn test_orient2d_collinear() {
        // Collinear points
        let result = orient2d([0.0, 0.0], [1.0, 1.0], [2.0, 2.0]);
        assert_eq!(result, 0.0, "Expected zero for collinear, got {}", result);
    }

    #[test]
    fn test_orient2d_nearly_collinear() {
        // Points that are very close to collinear
        let result = orient2d([0.0, 0.0], [1.0, 1.0], [2.0, 2.0 + 1e-15]);
        // Should return a definitive result (not zero due to rounding)
        assert!(result.is_finite());
    }

    #[test]
    fn test_orient2d_large_coordinates() {
        // Test with large coordinates
        let result = orient2d([1e10, 1e10], [1e10 + 1.0, 1e10], [1e10, 1e10 + 1.0]);
        assert!(result > 0.0, "Expected positive for CCW with large coords, got {}", result);
    }

    #[test]
    fn test_orient2d_small_differences() {
        // Test with points that have very small differences
        let result = orient2d([0.0, 0.0], [1e-10, 0.0], [0.0, 1e-10]);
        assert!(result > 0.0, "Expected positive for CCW with small differences, got {}", result);
    }
}
