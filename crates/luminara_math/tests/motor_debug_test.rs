// Minimal test to debug the motor geometric product issue
use luminara_math::algebra::Motor;

#[test]
fn debug_associativity_failure() {
    // From the failing counter-example:
    let m1: Motor<f64> = Motor {
        s: 0.745154,
        e12: 0.6668924,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e0123: 0.0,
    };

    let m2 = Motor {
        s: 0.9814418,
        e12: 0.19176012,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e0123: 0.0,
    };

    let m3 = Motor {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 4.5306315,
        e0123: 0.0,
    };

    println!("m1 (pure Z rotation): {:?}", m1);
    println!("m2 (pure Z rotation): {:?}", m2);
    println!("m3 (pure Z translation): {:?}", m3);
    println!();

    // Compute (m1 * m2) * m3
    let m1_m2 = m1.geometric_product(&m2);
    println!("m1 * m2 = {:?}", m1_m2);
    
    let left = m1_m2.geometric_product(&m3);
    println!("(m1 * m2) * m3 = {:?}", left);
    println!("  e03 component: {}", left.e03);
    println!();

    // Compute m1 * (m2 * m3)
    let m2_m3 = m2.geometric_product(&m3);
    println!("m2 * m3 = {:?}", m2_m3);
    
    let right = m1.geometric_product(&m2_m3);
    println!("m1 * (m2 * m3) = {:?}", right);
    println!("  e03 component: {}", right.e03);
    println!();

    let diff = (left.e03 - right.e03).abs();
    println!("Difference in e03: {}", diff);
    
    // This should be small for associativity
    assert!(diff < 1e-4, "Associativity violated: difference = {}", diff);
}
