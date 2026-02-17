//! Integration tests for ShaderGenerator
//!
//! These tests verify that the ShaderGenerator correctly wraps SymExpr WGSL codegen,
//! implements caching, and supports dynamic shader compilation.

use luminara_math::symbolic::SymExpr;
use luminara_render::shader_generator::ShaderGenerator;
use std::rc::Rc;

#[test]
fn test_generate_simple_constant_shader() {
    let mut generator = ShaderGenerator::new();

    // Create a simple constant expression
    let expr = Rc::new(SymExpr::Const(1.0));

    // Generate WGSL
    let wgsl = generator.generate_from_expr(&expr);

    // Verify the WGSL contains the constant
    assert!(wgsl.contains("1.0") || wgsl.contains("1."));
    assert_eq!(generator.stats().total_generated, 1);
}

#[test]
fn test_generate_pulsating_effect() {
    let mut generator = ShaderGenerator::new();

    // Create a pulsating effect: sin(time * 2π * 2)
    // This simulates a 2Hz pulsation
    let time_var = Rc::new(SymExpr::Var("time".to_string()));
    let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));
    let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq))));

    // Generate WGSL
    let wgsl = generator.generate_from_expr(&expr);

    // Verify the WGSL contains expected elements
    assert!(wgsl.contains("sin"));
    assert!(wgsl.contains("@vertex"));
    assert!(wgsl.contains("@fragment"));
    assert!(wgsl.contains("time") || wgsl.contains("t"));
}

#[test]
fn test_caching_behavior() {
    let mut generator = ShaderGenerator::new();

    // Create two different expressions
    let expr1 = Rc::new(SymExpr::Const(1.0));
    let expr2 = Rc::new(SymExpr::Const(2.0));

    // Generate first shader (cache miss)
    let wgsl1_first = generator.generate_from_expr(&expr1);
    assert_eq!(generator.stats().misses, 1);
    assert_eq!(generator.stats().hits, 0);

    // Generate second shader (cache miss)
    let wgsl2 = generator.generate_from_expr(&expr2);
    assert_eq!(generator.stats().misses, 2);
    assert_eq!(generator.stats().hits, 0);

    // Generate first shader again (cache hit)
    let wgsl1_second = generator.generate_from_expr(&expr1);
    assert_eq!(generator.stats().misses, 2);
    assert_eq!(generator.stats().hits, 1);

    // Verify results are identical
    assert_eq!(wgsl1_first, wgsl1_second);
    assert_ne!(wgsl1_first, wgsl2);
}

#[test]
fn test_complex_mathematical_expression() {
    let mut generator = ShaderGenerator::new();

    // Create a complex expression: (sin(x) + cos(y)) * t
    // Note: Using simpler operations that are supported by the shader generator
    let x = Rc::new(SymExpr::Var("x".to_string()));
    let y = Rc::new(SymExpr::Var("y".to_string()));
    let t = Rc::new(SymExpr::Var("t".to_string()));

    let sin_x = Rc::new(SymExpr::Sin(x));
    let cos_y = Rc::new(SymExpr::Cos(y));

    let sum = Rc::new(SymExpr::Add(sin_x, cos_y));
    let expr = Rc::new(SymExpr::Mul(sum, t));

    // Generate WGSL
    let wgsl = generator.generate_from_expr(&expr);

    // Verify the WGSL contains expected functions
    assert!(wgsl.contains("sin"));
    assert!(wgsl.contains("cos"));
    // The multiplication and addition should be present in the shader
    assert!(wgsl.contains("*") || wgsl.contains("+"));
}

#[test]
fn test_cache_management() {
    let mut generator = ShaderGenerator::new();

    let expr1 = Rc::new(SymExpr::Const(1.0));
    let expr2 = Rc::new(SymExpr::Const(2.0));

    // Generate two shaders
    generator.generate_from_expr(&expr1);
    generator.generate_from_expr(&expr2);

    assert_eq!(generator.cache_size(), 2);
    assert!(generator.is_cached(&expr1));
    assert!(generator.is_cached(&expr2));

    // Remove one from cache
    assert!(generator.remove_from_cache(&expr1));
    assert_eq!(generator.cache_size(), 1);
    assert!(!generator.is_cached(&expr1));
    assert!(generator.is_cached(&expr2));

    // Clear entire cache
    generator.clear_cache();
    assert_eq!(generator.cache_size(), 0);
    assert!(!generator.is_cached(&expr2));
}

#[test]
fn test_cache_hit_rate_calculation() {
    let mut generator = ShaderGenerator::new();

    let expr = Rc::new(SymExpr::Const(1.0));

    // First access: miss
    generator.generate_from_expr(&expr);
    assert_eq!(generator.stats().hit_rate(), 0.0);

    // Second access: hit
    generator.generate_from_expr(&expr);
    assert_eq!(generator.stats().hit_rate(), 0.5);

    // Third access: hit
    generator.generate_from_expr(&expr);
    assert_eq!(generator.stats().hit_rate(), 2.0 / 3.0);
}

#[test]
fn test_procedural_noise_pattern() {
    let mut generator = ShaderGenerator::new();

    // Create a procedural noise-like pattern: sin(x * 10) * cos(y * 10)
    let x = Rc::new(SymExpr::Var("x".to_string()));
    let y = Rc::new(SymExpr::Var("y".to_string()));
    let ten = Rc::new(SymExpr::Const(10.0));

    let x_scaled = Rc::new(SymExpr::Mul(x, ten.clone()));
    let y_scaled = Rc::new(SymExpr::Mul(y, ten));

    let sin_x = Rc::new(SymExpr::Sin(x_scaled));
    let cos_y = Rc::new(SymExpr::Cos(y_scaled));

    let expr = Rc::new(SymExpr::Mul(sin_x, cos_y));

    // Generate WGSL
    let wgsl = generator.generate_from_expr(&expr);

    // Verify the pattern is present
    assert!(wgsl.contains("sin"));
    assert!(wgsl.contains("cos"));
    assert!(wgsl.contains("10.0") || wgsl.contains("10."));
}

#[test]
fn test_dynamic_color_gradient() {
    let mut generator = ShaderGenerator::new();

    // Create a time-based color gradient: t * 0.5 + 0.5
    let t = Rc::new(SymExpr::Var("t".to_string()));
    let half = Rc::new(SymExpr::Const(0.5));

    let scaled = Rc::new(SymExpr::Mul(t, half.clone()));
    let expr = Rc::new(SymExpr::Add(scaled, half));

    // Generate WGSL
    let wgsl = generator.generate_from_expr(&expr);

    // Verify the gradient computation is present
    assert!(wgsl.contains("0.5"));
    assert!(wgsl.contains("time") || wgsl.contains("t"));
}

#[test]
fn test_shader_generation_is_deterministic() {
    let mut generator = ShaderGenerator::new();

    let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Var("x".to_string()))));

    // Generate the same shader multiple times
    let wgsl1 = generator.generate_from_expr(&expr);
    generator.clear_cache(); // Clear to force regeneration
    let wgsl2 = generator.generate_from_expr(&expr);

    // Results should be identical (deterministic generation)
    assert_eq!(wgsl1, wgsl2);
}
