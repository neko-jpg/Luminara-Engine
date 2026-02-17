# ShaderGenerator

The `ShaderGenerator` provides dynamic shader generation from symbolic mathematical expressions, enabling AI-driven shader creation and procedural effects.

## Overview

The ShaderGenerator wraps the SymExpr WGSL codegen from `luminara_math` and adds:

- **Shader Caching**: Avoids recompilation of identical expressions
- **Dynamic Compilation**: Generate shaders at runtime from mathematical expressions
- **Performance Tracking**: Monitor cache hit rates and generation statistics

## Basic Usage

```rust
use luminara_render::shader_generator::ShaderGenerator;
use luminara_math::symbolic::SymExpr;
use std::rc::Rc;

// Create a shader generator
let mut generator = ShaderGenerator::new();

// Create a simple pulsating effect: sin(time * 2π * 2)
let time_var = Rc::new(SymExpr::Var("time".to_string()));
let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));
let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq))));

// Generate WGSL shader code
let wgsl_source = generator.generate_from_expr(&expr);

// Or generate and compile in one step
let shader = generator.generate_and_compile(&expr, &device)?;
```

## Caching

The ShaderGenerator automatically caches generated shaders based on the expression structure:

```rust
let mut generator = ShaderGenerator::new();

let expr = Rc::new(SymExpr::Const(1.0));

// First generation: cache miss
let wgsl1 = generator.generate_from_expr(&expr);
assert_eq!(generator.stats().misses, 1);

// Second generation: cache hit (much faster)
let wgsl2 = generator.generate_from_expr(&expr);
assert_eq!(generator.stats().hits, 1);

// Check cache statistics
println!("Hit rate: {:.2}%", generator.stats().hit_rate() * 100.0);
println!("Cache size: {}", generator.cache_size());
```

## Cache Management

```rust
let mut generator = ShaderGenerator::new();

// Check if expression is cached
if generator.is_cached(&expr) {
    println!("Expression already cached");
}

// Remove specific shader from cache
generator.remove_from_cache(&expr);

// Clear entire cache
generator.clear_cache();
```

## Example Shaders

### Pulsating Glow Effect

Creates a shader that pulsates at 2Hz:

```rust
let time_var = Rc::new(SymExpr::Var("time".to_string()));
let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));
let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq))));

let wgsl = generator.generate_from_expr(&expr);
```

### Procedural Noise Pattern

Creates a checkerboard-like pattern:

```rust
let x = Rc::new(SymExpr::Var("x".to_string()));
let y = Rc::new(SymExpr::Var("y".to_string()));
let ten = Rc::new(SymExpr::Const(10.0));

let x_scaled = Rc::new(SymExpr::Mul(x, ten.clone()));
let y_scaled = Rc::new(SymExpr::Mul(y, ten));

let sin_x = Rc::new(SymExpr::Sin(x_scaled));
let cos_y = Rc::new(SymExpr::Cos(y_scaled));

let expr = Rc::new(SymExpr::Mul(sin_x, cos_y));

let wgsl = generator.generate_from_expr(&expr);
```

### Dynamic Color Gradient

Creates a time-based color gradient:

```rust
let t = Rc::new(SymExpr::Var("t".to_string()));
let half = Rc::new(SymExpr::Const(0.5));

let scaled = Rc::new(SymExpr::Mul(t, half.clone()));
let expr = Rc::new(SymExpr::Add(scaled, half));

let wgsl = generator.generate_from_expr(&expr);
```

## AI-Driven Shader Generation

The ShaderGenerator is designed to work with AI systems that can generate mathematical expressions from natural language descriptions:

```rust
// Pseudocode for AI integration
async fn generate_shader_from_description(
    description: &str,
    llm_client: &LlmClient,
    generator: &mut ShaderGenerator,
    device: &wgpu::Device,
) -> Result<Shader, RenderError> {
    // AI generates mathematical expression from description
    let expr = llm_client.generate_expression(description).await?;
    
    // Convert to shader
    let shader = generator.generate_and_compile(&expr, device)?;
    
    Ok(shader)
}

// Example usage:
// "Create a pulsating glow effect that pulses at 2Hz"
// AI generates: sin(time * 2π * 2)
// System generates WGSL shader automatically
```

## Supported SymExpr Operations

The ShaderGenerator supports the following SymExpr operations:

- **Constants**: `SymExpr::Const(value)`
- **Variables**: `SymExpr::Var(name)` - supports `time`, `x`, `y`
- **Arithmetic**: `Add`, `Sub`, `Mul`, `Div`
- **Trigonometric**: `Sin`, `Cos`
- **Other**: Additional operations may be supported by the underlying `luminara_math::symbolic` module

## Performance Considerations

### Cache Hit Rate

Monitor cache performance to ensure efficient shader reuse:

```rust
let stats = generator.stats();
println!("Total generated: {}", stats.total_generated);
println!("Cache hits: {}", stats.hits);
println!("Cache misses: {}", stats.misses);
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

### Memory Usage

The cache stores both WGSL source code and compiled shaders. For applications generating many unique shaders, consider:

- Periodically clearing the cache: `generator.clear_cache()`
- Removing unused shaders: `generator.remove_from_cache(&expr)`
- Monitoring cache size: `generator.cache_size()`

### Compilation Cost

Shader compilation is expensive. The cache significantly reduces this cost for repeated expressions:

- First generation: ~1-10ms (includes WGSL generation + GPU compilation)
- Cached generation: ~0.01ms (simple hash lookup)

## Integration with Rendering Pipeline

The ShaderGenerator integrates seamlessly with the Luminara rendering pipeline:

```rust
use luminara_render::{ShaderGenerator, Material};

// Generate shader
let mut generator = ShaderGenerator::new();
let shader = generator.generate_and_compile(&expr, &gpu.device)?;

// Create material from shader
let material = Material::from_shader(shader);

// Use in rendering
// ... attach material to entity ...
```

## Requirements

This implementation satisfies **Requirement 13.4** from the Pre-Editor Engine Audit:

> WHEN generating shaders, THE System SHALL use SymExpr to enable AI-driven shader generation from mathematical expressions

The ShaderGenerator provides:

1. ✅ Wraps SymExpr WGSL codegen from `luminara_math`
2. ✅ Implements shader caching for performance
3. ✅ Supports dynamic shader compilation at runtime
4. ✅ Enables AI-driven shader generation workflow

## See Also

- `luminara_math::symbolic::SymExpr` - Symbolic expression types
- `luminara_math::symbolic::ShaderGenerator` - Low-level WGSL codegen
- `luminara_render::Shader` - Shader compilation and management
- `luminara_render::Material` - Material system integration


## Complete Examples

For comprehensive, runnable examples demonstrating all shader generation capabilities, see:

```bash
cargo run --package luminara_render --example shader_generation_demo
```

This example demonstrates:
- **Pulsating glow effects** - Time-based sinusoidal intensity modulation
- **Procedural noise patterns** - Spatial frequency-based patterns using trigonometric functions
- **Dynamic color gradients** - Time-based color transitions with oscillation
- **Complex combined effects** - Temporal and spatial effects combined

The example includes:
- ✅ Complete implementations of each effect
- ✅ Detailed mathematical explanations
- ✅ Performance benchmarking with cache statistics
- ✅ Comprehensive test coverage
- ✅ Usage patterns and best practices

See `crates/luminara_render/examples/README.md` for detailed documentation of each example.
