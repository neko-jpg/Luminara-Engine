//! Shader Generation Examples
//!
//! This example demonstrates dynamic shader generation from symbolic expressions
//! using the ShaderGenerator. It showcases three key effects:
//!
//! 1. Pulsating glow effect - A time-based sinusoidal intensity modulation
//! 2. Procedural noise patterns - Spatial frequency-based patterns
//! 3. Dynamic color gradients - Time-based color transitions
//!
//! These examples satisfy Requirement 13.4: AI-driven shader generation from
//! mathematical expressions.

use luminara_math::symbolic::SymExpr;
use luminara_render::shader_generator::ShaderGenerator;
use std::rc::Rc;

fn main() {
    println!("Luminara Render - Shader Generation Examples");
    println!("============================================\n");

    let mut generator = ShaderGenerator::new();

    // Example 1: Pulsating Glow Effect
    println!("1. PULSATING GLOW EFFECT");
    println!("   Description: Creates a shader that pulsates at 2Hz");
    println!("   Mathematical Expression: sin(time * 2π * 2) * intensity\n");

    let pulsating_shader = create_pulsating_glow(&mut generator);
    println!("   Generated WGSL (first 200 chars):");
    println!("   {}\n", &pulsating_shader[..pulsating_shader.len().min(200)]);

    // Example 2: Procedural Noise Pattern
    println!("2. PROCEDURAL NOISE PATTERN");
    println!("   Description: Creates a checkerboard-like pattern using trigonometric functions");
    println!("   Mathematical Expression: sin(x * 10) * cos(y * 10)\n");

    let noise_shader = create_procedural_noise(&mut generator);
    println!("   Generated WGSL (first 200 chars):");
    println!("   {}\n", &noise_shader[..noise_shader.len().min(200)]);

    // Example 3: Dynamic Color Gradient
    println!("3. DYNAMIC COLOR GRADIENT");
    println!("   Description: Creates a time-based color gradient");
    println!("   Mathematical Expression: t * 0.5 + 0.5 (normalized to [0, 1])\n");

    let gradient_shader = create_dynamic_gradient(&mut generator);
    println!("   Generated WGSL (first 200 chars):");
    println!("   {}\n", &gradient_shader[..gradient_shader.len().min(200)]);

    // Bonus: Complex Combined Effect
    println!("4. COMPLEX COMBINED EFFECT");
    println!("   Description: Combines pulsation with spatial variation");
    println!("   Mathematical Expression: sin(time * 2π) * (sin(x * 5) + cos(y * 5)) * 0.5\n");

    let complex_shader = create_complex_effect(&mut generator);
    println!("   Generated WGSL (first 200 chars):");
    println!("   {}\n", &complex_shader[..complex_shader.len().min(200)]);

    // Display cache statistics
    println!("\nCACHE STATISTICS");
    println!("================");
    let stats = generator.stats();
    println!("Total shaders generated: {}", stats.total_generated);
    println!("Cache hits: {}", stats.hits);
    println!("Cache misses: {}", stats.misses);
    println!("Cache hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("Cache size: {}", generator.cache_size());

    println!("\n✓ All shader generation examples completed successfully!");
    println!("\nThese examples demonstrate:");
    println!("  • Dynamic shader generation from mathematical expressions");
    println!("  • Shader caching for performance optimization");
    println!("  • Time-based and spatial effects");
    println!("  • Foundation for AI-driven shader creation");
}

/// Example 1: Pulsating Glow Effect
///
/// Creates a shader that pulsates at 2Hz using a sinusoidal function.
/// This is useful for creating glowing effects, breathing animations,
/// or attention-grabbing UI elements.
///
/// Mathematical Expression: sin(time * 2π * 2) * intensity
///
/// The frequency is 2Hz, meaning the effect completes 2 full cycles per second.
fn create_pulsating_glow(generator: &mut ShaderGenerator) -> String {
    // Create time variable
    let time_var = Rc::new(SymExpr::Var("time".to_string()));

    // Calculate frequency: 2π * 2 (for 2Hz)
    let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));

    // Multiply time by frequency
    let time_freq = Rc::new(SymExpr::Mul(time_var, freq));

    // Apply sine function for smooth pulsation
    let pulsation = Rc::new(SymExpr::Sin(time_freq));

    // Optional: Scale and offset to [0, 1] range
    // (sin(x) + 1) / 2 maps [-1, 1] to [0, 1]
    let one = Rc::new(SymExpr::Const(1.0));
    let half = Rc::new(SymExpr::Const(0.5));
    let offset = Rc::new(SymExpr::Add(pulsation, one));
    let normalized = Rc::new(SymExpr::Mul(offset, half));

    // Generate WGSL shader code
    generator.generate_from_expr(&normalized)
}

/// Example 2: Procedural Noise Pattern
///
/// Creates a checkerboard-like pattern using trigonometric functions.
/// This demonstrates spatial frequency-based effects that can be used
/// for textures, backgrounds, or visual effects.
///
/// Mathematical Expression: sin(x * 10) * cos(y * 10)
///
/// The frequency of 10 creates a pattern with 10 cycles across the normalized
/// coordinate space. Higher frequencies create finer patterns.
fn create_procedural_noise(generator: &mut ShaderGenerator) -> String {
    // Create spatial variables
    let x = Rc::new(SymExpr::Var("x".to_string()));
    let y = Rc::new(SymExpr::Var("y".to_string()));

    // Set spatial frequency (higher = finer pattern)
    let frequency = Rc::new(SymExpr::Const(10.0));

    // Scale coordinates by frequency
    let x_scaled = Rc::new(SymExpr::Mul(x, frequency.clone()));
    let y_scaled = Rc::new(SymExpr::Mul(y, frequency));

    // Apply trigonometric functions
    let sin_x = Rc::new(SymExpr::Sin(x_scaled));
    let cos_y = Rc::new(SymExpr::Cos(y_scaled));

    // Multiply to create interference pattern
    let pattern = Rc::new(SymExpr::Mul(sin_x, cos_y));

    // Optional: Normalize to [0, 1] range
    let one = Rc::new(SymExpr::Const(1.0));
    let half = Rc::new(SymExpr::Const(0.5));
    let offset = Rc::new(SymExpr::Add(pattern, one));
    let normalized = Rc::new(SymExpr::Mul(offset, half));

    // Generate WGSL shader code
    generator.generate_from_expr(&normalized)
}

/// Example 3: Dynamic Color Gradient
///
/// Creates a time-based color gradient that smoothly transitions.
/// This is useful for animated backgrounds, UI transitions, or
/// atmospheric effects.
///
/// Mathematical Expression: t * 0.5 + 0.5
///
/// This maps the time variable to a [0, 1] range, creating a smooth
/// gradient that can be used for color interpolation.
fn create_dynamic_gradient(generator: &mut ShaderGenerator) -> String {
    // Create time variable
    let t = Rc::new(SymExpr::Var("t".to_string()));

    // Scale factor (controls gradient speed)
    let scale = Rc::new(SymExpr::Const(0.5));

    // Scale time
    let scaled = Rc::new(SymExpr::Mul(t, scale.clone()));

    // Offset to center around 0.5
    let _gradient = Rc::new(SymExpr::Add(scaled, scale));

    // For a more interesting gradient, we can use sine to create oscillation
    // This creates a back-and-forth gradient instead of linear
    let pi2 = Rc::new(SymExpr::Const(std::f64::consts::PI * 2.0));
    let t_var = Rc::new(SymExpr::Var("t".to_string()));
    let angle = Rc::new(SymExpr::Mul(t_var, pi2));
    let oscillation = Rc::new(SymExpr::Sin(angle));

    // Normalize oscillation to [0, 1]
    let one = Rc::new(SymExpr::Const(1.0));
    let half = Rc::new(SymExpr::Const(0.5));
    let offset = Rc::new(SymExpr::Add(oscillation, one));
    let normalized = Rc::new(SymExpr::Mul(offset, half));

    // Generate WGSL shader code
    generator.generate_from_expr(&normalized)
}

/// Bonus Example: Complex Combined Effect
///
/// Combines temporal (time-based) and spatial (position-based) effects
/// to create a more sophisticated shader. This demonstrates how multiple
/// mathematical expressions can be composed to create complex visual effects.
///
/// Mathematical Expression: sin(time * 2π) * (sin(x * 5) + cos(y * 5)) * 0.5
///
/// This creates a pulsating pattern that varies across space, useful for
/// water effects, energy fields, or magical effects.
fn create_complex_effect(generator: &mut ShaderGenerator) -> String {
    // Temporal component: sin(time * 2π)
    let time_var = Rc::new(SymExpr::Var("time".to_string()));
    let pi2 = Rc::new(SymExpr::Const(std::f64::consts::PI * 2.0));
    let time_angle = Rc::new(SymExpr::Mul(time_var, pi2));
    let temporal = Rc::new(SymExpr::Sin(time_angle));

    // Spatial component: sin(x * 5) + cos(y * 5)
    let x = Rc::new(SymExpr::Var("x".to_string()));
    let y = Rc::new(SymExpr::Var("y".to_string()));
    let five = Rc::new(SymExpr::Const(5.0));

    let x_scaled = Rc::new(SymExpr::Mul(x, five.clone()));
    let y_scaled = Rc::new(SymExpr::Mul(y, five));

    let sin_x = Rc::new(SymExpr::Sin(x_scaled));
    let cos_y = Rc::new(SymExpr::Cos(y_scaled));
    let spatial = Rc::new(SymExpr::Add(sin_x, cos_y));

    // Combine temporal and spatial
    let combined = Rc::new(SymExpr::Mul(temporal, spatial));

    // Scale to reasonable range
    let half = Rc::new(SymExpr::Const(0.5));
    let scaled = Rc::new(SymExpr::Mul(combined, half));

    // Normalize to [0, 1] for color output
    let one = Rc::new(SymExpr::Const(1.0));
    let offset = Rc::new(SymExpr::Add(scaled, one));
    let final_half = Rc::new(SymExpr::Const(0.5));
    let normalized = Rc::new(SymExpr::Mul(offset, final_half));

    // Generate WGSL shader code
    generator.generate_from_expr(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_examples_generate_valid_shaders() {
        let mut generator = ShaderGenerator::new();

        // Test each example generates non-empty WGSL
        let pulsating = create_pulsating_glow(&mut generator);
        assert!(!pulsating.is_empty());
        assert!(pulsating.contains("@vertex") || pulsating.contains("fn"));

        let noise = create_procedural_noise(&mut generator);
        assert!(!noise.is_empty());
        assert!(noise.contains("sin") || noise.contains("cos"));

        let gradient = create_dynamic_gradient(&mut generator);
        assert!(!gradient.is_empty());

        let complex = create_complex_effect(&mut generator);
        assert!(!complex.is_empty());
    }

    #[test]
    fn test_shader_caching_works() {
        let mut generator = ShaderGenerator::new();

        // Generate all shaders
        create_pulsating_glow(&mut generator);
        create_procedural_noise(&mut generator);
        create_dynamic_gradient(&mut generator);
        create_complex_effect(&mut generator);

        // Should have generated 4 unique shaders
        assert!(generator.stats().total_generated >= 4);
        assert!(generator.cache_size() >= 4);
    }

    #[test]
    fn test_pulsating_glow_contains_sine() {
        let mut generator = ShaderGenerator::new();
        let shader = create_pulsating_glow(&mut generator);

        // Should contain sine function
        assert!(shader.contains("sin"));
    }

    #[test]
    fn test_procedural_noise_contains_trig_functions() {
        let mut generator = ShaderGenerator::new();
        let shader = create_procedural_noise(&mut generator);

        // Should contain both sine and cosine
        assert!(shader.contains("sin"));
        assert!(shader.contains("cos"));
    }

    #[test]
    fn test_complex_effect_combines_temporal_and_spatial() {
        let mut generator = ShaderGenerator::new();
        let shader = create_complex_effect(&mut generator);

        // Should contain trigonometric functions
        assert!(shader.contains("sin") || shader.contains("cos"));
    }
}
