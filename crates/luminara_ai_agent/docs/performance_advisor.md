# Performance Advisor

## Overview

The Performance Advisor is a key component of Luminara's AI integration system that predicts the performance impact of AI-driven operations. It helps AI agents make performance-aware decisions by estimating FPS changes, providing warnings when performance drops below acceptable thresholds, and suggesting optimizations.

## Features

### 1. FPS Impact Prediction

The Performance Advisor predicts how operations will affect frame rate based on a learned cost model:

```rust
let mut advisor = PerformanceAdvisor::new();
advisor.update_metrics(&world, 60.0);

let intent = AiIntent::SpawnRelative {
    anchor: EntityReference::ByName("player".to_string()),
    offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
    template: "enemy".to_string(),
};

let impact = advisor.estimate_impact(&intent, &world);
println!("Predicted FPS: {:.1}", impact.predicted_fps);
println!("Cost: {:.4}ms", impact.predicted_cost_ms);
```

### 2. Cost Model with CPU/GPU/Memory Tracking

The cost model tracks three dimensions of performance cost per component:

- **CPU Cost**: Time spent on CPU processing (ms per frame)
- **GPU Cost**: Time spent on GPU rendering (ms per frame)  
- **Memory**: Memory footprint (bytes)

Default costs are provided for common components:

| Component | CPU Cost | GPU Cost | Memory |
|-----------|----------|----------|--------|
| Transform | 0.0005ms | 0ms | 48 bytes |
| Mesh | 0.001ms | 0.05ms | 1KB |
| Material | 0.0005ms | 0.01ms | 256 bytes |
| Light | 0.002ms | 0.1ms | 128 bytes |
| RigidBody | 0.01ms | 0ms | 512 bytes |
| Collider | 0.005ms | 0ms | 256 bytes |

### 3. Warning System

When predicted FPS drops below 30, the advisor generates warnings and suggestions:

```rust
let impact = advisor.estimate_impact(&intent, &world);

if !impact.warnings.is_empty() {
    for warning in &impact.warnings {
        println!("⚠️  {}", warning);
    }
}
```

Example warnings:
- "WARNING: Predicted FPS (28.5) drops below minimum threshold (30.0)"

### 4. Optimization Suggestions

The advisor provides context-aware optimization suggestions based on:

- **Entity Count**: Suggests GPU instancing when entity count > 5000, LOD when > 10000
- **Draw Calls**: Suggests batching when draw calls > 500
- **Memory Usage**: Suggests asset unloading when memory > 1GB
- **Intent Type**: Provides intent-specific suggestions (e.g., object pooling for spawns)

Example suggestions:
- "Consider using GPU instancing for repeated meshes to reduce draw calls."
- "Implement LOD (Level of Detail) system to reduce complexity of distant objects."
- "High draw call count detected. Consider batching materials or using instancing."
- "Many entities already exist. Consider reusing existing entities or using object pooling."

### 5. Learning Mechanism

The advisor improves accuracy over time by learning from actual measurements:

```rust
// After executing an operation, measure actual cost
let actual_cost_ms = measure_operation_cost();

// Update the cost model
advisor.learn_from_measurement(&intent, actual_cost_ms);

// Future predictions will be more accurate
```

The learning algorithm uses exponential moving average with a 10% learning rate:

```
new_cost = old_cost + learning_rate * (actual_cost - predicted_cost)
```

### 6. Accuracy Tracking

Monitor prediction accuracy over time:

```rust
let stats = advisor.get_accuracy_stats();
println!("Mean Absolute Error: {:.4}ms", stats.mean_absolute_error);
println!("Mean Percentage Error: {:.1}%", stats.mean_percentage_error);
println!("Sample Count: {}", stats.sample_count);
```

## Integration with AI Context

The Performance Advisor integrates with the AI context generation system:

```rust
let context = advisor.generate_context();
// Returns: "Performance: FPS 60.0, Entities 1234, FrameTime 16.67ms, DrawCalls 150, Memory 256.5MB"
```

This context is included in AI prompts to help the AI make performance-aware decisions.

## Severity Levels

The advisor classifies performance impact into four severity levels:

- **Low**: Minimal impact, predicted FPS > 45 and cost < 1ms
- **Medium**: Noticeable impact, cost > 1ms
- **High**: Significant impact, predicted FPS < 45
- **Critical**: Severe impact, predicted FPS < 30 (minimum threshold)

## Usage Example

Complete workflow:

```rust
use luminara_ai_agent::performance::PerformanceAdvisor;

// Initialize advisor
let mut advisor = PerformanceAdvisor::new();

// Update with current metrics (called each frame)
advisor.update_metrics(&world, current_fps);
advisor.update_render_metrics(draw_calls, memory_mb);

// Before executing an AI operation
let intent = /* ... */;
let impact = advisor.estimate_impact(&intent, &world);

match impact.severity {
    ImpactSeverity::Critical => {
        println!("❌ Operation too expensive!");
        for suggestion in &impact.suggestions {
            println!("  💡 {}", suggestion);
        }
        // Consider rejecting the operation
    }
    ImpactSeverity::High => {
        println!("⚠️  High performance impact");
        // Proceed with caution
    }
    _ => {
        // Safe to proceed
    }
}

// After executing the operation
let actual_cost = measure_actual_cost();
advisor.learn_from_measurement(&intent, actual_cost);
```

## Requirements Validation

This implementation validates **Requirements 28.1**:

1. ✅ **Predicts FPS change based on component costs** - Uses cost model with CPU/GPU/memory tracking
2. ✅ **Warns when FPS drops below 30** - Generates warnings and suggestions when predicted FPS < 30
3. ✅ **Maintains cost model** - Tracks CPU cost, GPU cost, and memory per component
4. ✅ **Includes performance metrics in AI context** - `generate_context()` provides current metrics
5. ✅ **Learns from actual measurements** - `learn_from_measurement()` updates cost model using EMA
6. ✅ **Suggests optimizations** - Provides GPU instancing, LOD, and entity count reduction suggestions
7. ✅ **Measures prediction accuracy** - `get_accuracy_stats()` tracks mean absolute and percentage error

## Testing

Comprehensive test suite with 11 tests covering:

- FPS impact prediction
- Warning threshold behavior
- Severity level classification
- Optimization suggestion generation
- Cost model learning
- Accuracy statistics
- Context generation
- Different intent types
- Learning convergence
- Component cost retrieval
- Memory-based suggestions

Run tests:
```bash
cargo test -p luminara_ai_agent --test performance_advisor_test
```

## Future Enhancements

Potential improvements:

1. **GPU Profiling Integration**: Use actual GPU timing queries for more accurate GPU cost measurements
2. **Per-Component Learning**: Learn costs for individual component types rather than just spawn overhead
3. **Workload Prediction**: Predict performance based on scene complexity (entity count, light count, etc.)
4. **Historical Trends**: Track performance over time to detect degradation
5. **Adaptive Thresholds**: Adjust warning thresholds based on target platform (mobile vs desktop)
6. **Cost Visualization**: Generate graphs showing cost breakdown by component type
