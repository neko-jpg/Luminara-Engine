# AI Shader Generation Pipeline

## Overview

The AI Shader Generation Pipeline implements **Requirement 13.4**: "WHEN generating shaders, THE System SHALL use SymExpr to enable AI-driven shader generation from mathematical expressions."

This pipeline provides a complete workflow for generating shaders from natural language descriptions using AI and symbolic mathematics.

## Architecture

```
Natural Language Description
         ↓
   AI Expression Generator
         ↓
     SymExpr (Mathematical Expression)
         ↓
   ShaderGenerator (WGSL Codegen)
         ↓
     WGSL Shader Code
         ↓
   Shader Compilation
         ↓
     Compiled Shader
         ↓
   Material Application
```

## Components

### 1. AiShaderPipeline

The main entry point for AI-driven shader generation. Orchestrates the complete workflow from description to compiled shader.

```rust
use luminara_render::ai_shader_pipeline::AiShaderPipeline;

let mut pipeline = AiShaderPipeline::new();

// Generate WGSL source code
let wgsl = pipeline.generate_shader("Create a pulsating glow effect")?;

// Or generate and compile in one step
let shader = pipeline.generate_and_compile_shader(
    "Create a pulsating glow effect",
    &device
)?;

// Or generate a complete material
let material = pipeline.generate_material(
    "Create a pulsating glow effect that pulses at 2Hz",
    "pulsating_glow",
    &device
)?;
```

### 2. ExpressionGenerator Trait

Abstracts the AI/LLM integration, allowing different implementations:

```rust
pub trait ExpressionGenerator: Send + Sync {
    fn generate_expression(&self, description: &str) -> AiShaderResult<Rc<SymExpr>>;
    fn name(&self) -> &str;
}
```

This design allows for:
- **Mock generators** for testing and development
- **Local LLM integration** (e.g., llama.cpp, GGML models)
- **Remote API integration** (OpenAI, Anthropic, etc.)
- **Custom fine-tuned models** specialized for shader generation

### 3. MockExpressionGenerator

A pattern-matching generator that recognizes common shader effects:

| Pattern | Generated Expression | Example |
|---------|---------------------|---------|
| "pulsating", "pulse" | `sin(time * 2π * freq)` | Pulsating glow effect |
| "gradient", "fade" | `uv.x` | Horizontal gradient |
| "noise", "random" | `sin(uv.x * 10) * sin(uv.y * 10)` | Pseudo-noise pattern |
| "constant", "solid" | `1.0` | Solid color |

## Usage Examples

### Basic Shader Generation

```rust
use luminara_render::ai_shader_pipeline::AiShaderPipeline;

let mut pipeline = AiShaderPipeline::new();

// Generate a pulsating effect
let wgsl = pipeline.generate_shader("Create a pulsating glow effect that pulses at 2Hz")?;
println!("Generated WGSL:\n{}", wgsl);
```

### Custom Expression Generator

Integrate with your own LLM service:

```rust
use luminara_render::ai_shader_pipeline::{AiShaderPipeline, ExpressionGenerator, AiShaderResult};
use luminara_math::symbolic::SymExpr;
use std::rc::Rc;

struct OpenAIGenerator {
    api_key: String,
    client: reqwest::Client,
}

impl ExpressionGenerator for OpenAIGenerator {
    fn generate_expression(&self, description: &str) -> AiShaderResult<Rc<SymExpr>> {
        // Call OpenAI API to generate expression
        // Parse response into SymExpr
        // Return expression
        todo!("Implement OpenAI integration")
    }

    fn name(&self) -> &str {
        "OpenAIGenerator"
    }
}

let generator = OpenAIGenerator {
    api_key: "your-api-key".to_string(),
    client: reqwest::Client::new(),
};

let mut pipeline = AiShaderPipeline::with_generator(Box::new(generator));
```

### Material Generation

Generate a complete material with AI-generated shader:

```rust
use luminara_render::ai_shader_pipeline::AiShaderPipeline;

let mut pipeline = AiShaderPipeline::new();

let material = pipeline.generate_material(
    "Create a pulsating glow effect that pulses at 2Hz",
    "pulsating_glow",
    &device
)?;

// Apply material to entity
world.insert(entity, material)?;
```

### Caching and Performance

The pipeline automatically caches generated shaders:

```rust
let mut pipeline = AiShaderPipeline::new();

// First generation - cache miss
let wgsl1 = pipeline.generate_shader("Create a pulsating effect")?;

// Second generation - cache hit (instant)
let wgsl2 = pipeline.generate_shader("Create a pulsating effect")?;

// Check cache statistics
let stats = pipeline.cache_stats();
println!("Cache hits: {}", stats.hits);
println!("Cache misses: {}", stats.misses);
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

## Integration with LLM Services

### Recommended Approach

1. **Prompt Engineering**: Design prompts that guide the LLM to generate valid SymExpr representations
2. **Validation**: Parse and validate LLM output before converting to SymExpr
3. **Fallback**: Use MockExpressionGenerator as fallback if LLM fails
4. **Caching**: Cache LLM responses to reduce API calls

### Example Prompt Template

```
You are a shader generation assistant. Given a natural language description,
generate a mathematical expression using the following syntax:

- Constants: 1.0, 2.5, 3.14159
- Variables: time, uv_x, uv_y
- Operations: Add(a, b), Sub(a, b), Mul(a, b), Div(a, b)
- Functions: Sin(x), Cos(x), Sqrt(x), Pow(x, y)

Description: {user_description}

Generate a mathematical expression that implements this effect:
```

### Example LLM Response

```json
{
  "expression": "Sin(Mul(Var(time), Const(12.566370614359172)))",
  "explanation": "This creates a pulsating effect at 2Hz by taking the sine of time multiplied by 2π*2"
}
```

## Error Handling

The pipeline provides comprehensive error handling:

```rust
use luminara_render::ai_shader_pipeline::{AiShaderError, AiShaderPipeline};

let mut pipeline = AiShaderPipeline::new();

match pipeline.generate_shader("unsupported description") {
    Ok(wgsl) => println!("Generated: {}", wgsl),
    Err(AiShaderError::ExpressionGenerationFailed(msg)) => {
        eprintln!("AI failed to generate expression: {}", msg);
    }
    Err(AiShaderError::ShaderCompilationFailed(msg)) => {
        eprintln!("Shader compilation failed: {}", msg);
    }
    Err(AiShaderError::InvalidDescription(msg)) => {
        eprintln!("Invalid description: {}", msg);
    }
    Err(AiShaderError::RenderError(msg)) => {
        eprintln!("Render error: {}", msg);
    }
}
```

## Performance Considerations

### Caching Strategy

The pipeline uses a two-level cache:

1. **Expression Cache**: Caches SymExpr → WGSL mappings
2. **Shader Cache**: Caches compiled shaders

This ensures:
- Identical descriptions generate identical shaders (deterministic)
- No redundant WGSL generation
- No redundant shader compilation

### Optimization Tips

1. **Batch Generation**: Generate multiple shaders in parallel if using async LLM APIs
2. **Precompute Common Effects**: Pre-generate common shader effects at startup
3. **Cache Warming**: Warm the cache with frequently used descriptions
4. **LLM Caching**: Cache LLM responses separately to avoid API costs

## Testing

The pipeline includes comprehensive tests:

```bash
# Run all AI shader pipeline tests
cargo test --package luminara_render --test ai_shader_pipeline_test

# Run specific test
cargo test --package luminara_render --test ai_shader_pipeline_test test_pipeline_caching
```

### Test Coverage

- ✅ Basic shader generation
- ✅ Custom expression generators
- ✅ Mock generator patterns (pulsating, gradient, noise, constant)
- ✅ Caching behavior
- ✅ Error handling
- ✅ Edge cases (empty descriptions, long descriptions, special characters)
- ✅ Cache statistics and hit rate calculation

## Future Enhancements

### Planned Features

1. **Fine-tuned Models**: Train specialized models for shader generation
2. **Visual Feedback Loop**: Use rendered output to refine generated shaders
3. **Style Transfer**: Generate shaders that match reference images
4. **Optimization Hints**: AI suggests performance optimizations for generated shaders
5. **Multi-modal Input**: Accept images or videos as shader inspiration

### Integration Opportunities

- **Editor Integration**: Real-time shader preview in editor
- **Asset Pipeline**: Batch generate shader variations
- **Procedural Content**: Generate unique shaders for procedural assets
- **Game Logic**: Dynamic shader generation based on game state

## Related Documentation

- [Shader Generator](./shader_generator.md) - Low-level WGSL generation from SymExpr
- [SymExpr Documentation](../../luminara_math/docs/symbolic.md) - Symbolic expression system
- [Material System](./materials.md) - Material and shader management

## Requirements Validation

This implementation satisfies **Requirement 13.4**:

> "WHEN generating shaders, THE System SHALL use SymExpr to enable AI-driven shader generation from mathematical expressions"

✅ **Validated**:
- AI generates mathematical expressions (via ExpressionGenerator trait)
- Expressions are SymExpr instances
- SymExpr is converted to WGSL via ShaderGenerator
- Shaders are compiled and applied to materials
- Complete pipeline from description to material

## Examples in Codebase

See the following files for complete examples:

- `crates/luminara_render/src/ai_shader_pipeline.rs` - Implementation
- `crates/luminara_render/tests/ai_shader_pipeline_test.rs` - Integration tests
- `examples/ai_shader_demo.rs` - (TODO) Interactive demo

## Contributing

When extending the AI shader pipeline:

1. Implement the `ExpressionGenerator` trait for new LLM integrations
2. Add tests for new generator implementations
3. Document supported patterns and limitations
4. Consider caching and performance implications
5. Provide fallback behavior for failures
