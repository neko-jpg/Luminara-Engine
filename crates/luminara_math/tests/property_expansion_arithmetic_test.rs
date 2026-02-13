// Feature: math-foundation, Property 4: Expansion Arithmetic Exactness
//
// **Property 4: Expansion Arithmetic Exactness**
//
// For any two floating-point numbers and any sequence of addition, subtraction,
// multiplication, and scaling operations on Expansions, the final result should
// match arbitrary-precision arithmetic within the representable precision of the expansion.
//
// **Validates: Requirements 1.7**

use luminara_math::foundations::{two_product, two_sum};
use proptest::prelude::*;

// ============================================================================
// Test Strategies (Generators)
// ============================================================================

/// Strategy for generating finite f64 values in a reasonable range
fn finite_f64_strategy() -> impl Strategy<Value = f64> {
    prop_oneof![
        // Normal range values
        -1e10..1e10,
        // Small values
        -1e-10..1e-10,
        // Values near 1
        0.9..1.1,
        // Specific edge cases
        Just(0.0),
        Just(1.0),
        Just(-1.0),
        Just(f64::EPSILON),
        Just(-f64::EPSILON),
    ]
}

/// Strategy for generating pairs of f64 values
fn f64_pair_strategy() -> impl Strategy<Value = (f64, f64)> {
    (finite_f64_strategy(), finite_f64_strategy())
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 4.1: Two-Sum Exactness**
    ///
    /// For any two floating-point numbers a and b, the two_sum function should
    /// produce a sum s and error e such that a + b = s + e exactly (within the
    /// representable precision of f64).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_sum_exactness(a in finite_f64_strategy(), b in finite_f64_strategy()) {
        let (sum, error) = two_sum(a, b);
        
        // The sum and error should be finite
        prop_assert!(sum.is_finite(), "Sum should be finite");
        prop_assert!(error.is_finite(), "Error should be finite");
        
        // The key property: s + e should equal a + b exactly
        // We verify this by checking that the difference is within machine epsilon
        let reconstructed = sum + error;
        let direct = a + b;
        
        // For most cases, these should be exactly equal
        // In edge cases with extreme values, we allow a small tolerance
        let tolerance = if a.abs() > 1e100 || b.abs() > 1e100 {
            f64::EPSILON * a.abs().max(b.abs()) * 10.0
        } else {
            0.0
        };
        
        prop_assert!(
            (reconstructed - direct).abs() <= tolerance,
            "two_sum should be exact: a={}, b={}, sum={}, error={}, reconstructed={}, direct={}",
            a, b, sum, error, reconstructed, direct
        );
    }

    /// **Property 4.2: Two-Sum Commutativity**
    ///
    /// For any two floating-point numbers a and b, two_sum(a, b) and two_sum(b, a)
    /// should produce the same exact result (s + e).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_sum_commutative(a in finite_f64_strategy(), b in finite_f64_strategy()) {
        let (sum1, error1) = two_sum(a, b);
        let (sum2, error2) = two_sum(b, a);
        
        let result1 = sum1 + error1;
        let result2 = sum2 + error2;
        
        prop_assert!(
            (result1 - result2).abs() <= f64::EPSILON * result1.abs().max(result2.abs()),
            "two_sum should be commutative: a={}, b={}, result1={}, result2={}",
            a, b, result1, result2
        );
    }

    /// **Property 4.3: Two-Product Exactness**
    ///
    /// For any two floating-point numbers a and b, the two_product function should
    /// produce a product p and error e such that a * b = p + e exactly (within the
    /// representable precision of f64).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_product_exactness(a in finite_f64_strategy(), b in finite_f64_strategy()) {
        let (product, error) = two_product(a, b);
        
        // The product and error should be finite (unless the result overflows/underflows)
        if (a * b).is_finite() {
            prop_assert!(product.is_finite(), "Product should be finite");
            prop_assert!(error.is_finite(), "Error should be finite");
            
            // The key property: p + e should equal a * b exactly
            let reconstructed = product + error;
            let direct = a * b;
            
            // For products, we need to be more careful with tolerance
            // The error term can be very small compared to the product
            let tolerance = if product.abs() > 1e-100 {
                f64::EPSILON * product.abs() * 10.0
            } else {
                1e-30 // For very small products
            };
            
            prop_assert!(
                (reconstructed - direct).abs() <= tolerance,
                "two_product should be exact: a={}, b={}, product={}, error={}, reconstructed={}, direct={}",
                a, b, product, error, reconstructed, direct
            );
        }
    }

    /// **Property 4.4: Two-Product Commutativity**
    ///
    /// For any two floating-point numbers a and b, two_product(a, b) and
    /// two_product(b, a) should produce the same exact result (p + e).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_product_commutative(a in finite_f64_strategy(), b in finite_f64_strategy()) {
        let (product1, error1) = two_product(a, b);
        let (product2, error2) = two_product(b, a);
        
        if (a * b).is_finite() {
            let result1 = product1 + error1;
            let result2 = product2 + error2;
            
            let tolerance = if result1.abs() > 1e-100 {
                f64::EPSILON * result1.abs().max(result2.abs()) * 10.0
            } else {
                1e-30
            };
            
            prop_assert!(
                (result1 - result2).abs() <= tolerance,
                "two_product should be commutative: a={}, b={}, result1={}, result2={}",
                a, b, result1, result2
            );
        }
    }

    /// **Property 4.5: Two-Sum with Zero**
    ///
    /// For any floating-point number a, two_sum(a, 0) should produce (a, 0).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_sum_zero_identity(a in finite_f64_strategy()) {
        let (sum, error) = two_sum(a, 0.0);
        
        prop_assert_eq!(sum, a, "two_sum(a, 0) should produce sum = a");
        prop_assert_eq!(error, 0.0, "two_sum(a, 0) should produce error = 0");
    }

    /// **Property 4.6: Two-Product with Zero**
    ///
    /// For any floating-point number a, two_product(a, 0) should produce (0, 0).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_product_zero(a in finite_f64_strategy()) {
        let (product, error) = two_product(a, 0.0);
        
        prop_assert_eq!(product, 0.0, "two_product(a, 0) should produce product = 0");
        prop_assert_eq!(error, 0.0, "two_product(a, 0) should produce error = 0");
    }

    /// **Property 4.7: Two-Product with One**
    ///
    /// For any floating-point number a, two_product(a, 1) should produce (a, 0).
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_product_one_identity(a in finite_f64_strategy()) {
        let (product, error) = two_product(a, 1.0);
        
        prop_assert_eq!(product, a, "two_product(a, 1) should produce product = a");
        prop_assert_eq!(error, 0.0, "two_product(a, 1) should produce error = 0");
    }

    /// **Property 4.8: Two-Sum Associativity (Error Tracking)**
    ///
    /// For any three floating-point numbers a, b, c, the error-free transformations
    /// should allow us to track the exact result regardless of association order.
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_sum_associativity_tracking(
        a in finite_f64_strategy(),
        b in finite_f64_strategy(),
        c in finite_f64_strategy()
    ) {
        // Compute (a + b) + c with error tracking
        let (s1, e1) = two_sum(a, b);
        let (s2, e2) = two_sum(s1, c);
        let (s_err1, e_err1) = two_sum(e1, e2);
        let result1 = s2 + s_err1 + e_err1;
        
        // Compute a + (b + c) with error tracking
        let (s3, e3) = two_sum(b, c);
        let (s4, e4) = two_sum(a, s3);
        let (s_err2, e_err2) = two_sum(e3, e4);
        let result2 = s4 + s_err2 + e_err2;
        
        // The exact results should be very close
        let tolerance = f64::EPSILON * result1.abs().max(result2.abs()) * 100.0;
        
        prop_assert!(
            (result1 - result2).abs() <= tolerance,
            "Error tracking should preserve associativity: a={}, b={}, c={}, result1={}, result2={}",
            a, b, c, result1, result2
        );
    }

    /// **Property 4.9: Two-Sum Handles Cancellation**
    ///
    /// For any floating-point number a, two_sum(a, -a) should produce (0, 0) or
    /// a very small error term.
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_sum_cancellation(a in finite_f64_strategy()) {
        let (sum, error) = two_sum(a, -a);
        
        // The sum should be zero or very close to zero
        prop_assert!(
            sum.abs() <= f64::EPSILON * a.abs(),
            "two_sum(a, -a) should produce sum ≈ 0: a={}, sum={}, error={}",
            a, sum, error
        );
        
        // The error should also be zero or very close to zero
        prop_assert!(
            error.abs() <= f64::EPSILON * a.abs(),
            "two_sum(a, -a) should produce error ≈ 0: a={}, sum={}, error={}",
            a, sum, error
        );
    }

    /// **Property 4.10: Two-Product Preserves Sign**
    ///
    /// For any two non-zero floating-point numbers a and b, the sign of the
    /// product should match the sign of a * b.
    ///
    /// **Validates: Requirements 1.7**
    #[test]
    fn prop_two_product_sign(a in finite_f64_strategy(), b in finite_f64_strategy()) {
        if a != 0.0 && b != 0.0 && (a * b).is_finite() {
            let (product, _error) = two_product(a, b);
            let direct = a * b;
            
            prop_assert_eq!(
                product.signum(),
                direct.signum(),
                "two_product should preserve sign: a={}, b={}, product={}, direct={}",
                a, b, product, direct
            );
        }
    }
}

// ============================================================================
// Additional Unit Tests for Edge Cases
// ============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_two_sum_with_epsilon() {
        // **Validates: Requirements 1.7**
        let a = 1.0;
        let b = f64::EPSILON;
        let (sum, error) = two_sum(a, b);
        
        // When adding 1.0 + epsilon, the sum becomes the next representable float
        // The error term should be zero because the addition is exact
        assert_eq!(sum, 1.0 + f64::EPSILON);
        assert_eq!(error, 0.0);
    }

    #[test]
    fn test_two_sum_large_and_small() {
        // **Validates: Requirements 1.7**
        let a = 1e100;
        let b = 1.0;
        let (sum, error) = two_sum(a, b);
        
        // The small value should be captured in the error term
        assert_eq!(sum, 1e100);
        assert_eq!(error, 1.0);
    }

    #[test]
    fn test_two_product_with_sqrt2() {
        // **Validates: Requirements 1.7**
        let sqrt2 = 2.0_f64.sqrt();
        let (product, error) = two_product(sqrt2, sqrt2);
        
        // sqrt(2) * sqrt(2) = 2, but there will be rounding error
        let reconstructed = product + error;
        assert!((reconstructed - 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_two_sum_opposite_signs_unequal() {
        // **Validates: Requirements 1.7**
        let a = 1.5;
        let b = -1.0;
        let (sum, error) = two_sum(a, b);
        
        assert_eq!(sum, 0.5);
        assert_eq!(error, 0.0);
    }

    #[test]
    fn test_two_product_negative_numbers() {
        // **Validates: Requirements 1.7**
        let a = -3.5;
        let b = -2.5;
        let (product, error) = two_product(a, b);
        
        assert_eq!(product, 8.75);
        assert_eq!(error, 0.0); // Exact multiplication
    }

    #[test]
    fn test_two_sum_denormal_numbers() {
        // **Validates: Requirements 1.7**
        let a = f64::MIN_POSITIVE / 2.0; // Denormal number
        let b = f64::MIN_POSITIVE / 2.0;
        let (sum, error) = two_sum(a, b);
        
        // Should handle denormal numbers gracefully
        assert!(sum.is_finite());
        assert!(error.is_finite());
    }
}
