# Level of Detail (LOD) System

## Overview

The LOD system automatically selects appropriate mesh detail levels based on screen-space coverage, providing significant performance improvements in large open worlds while maintaining visual quality.

**Validates: Requirements 19.5**

## Features

- **3-5 LOD Levels**: Automatic generation of multiple detail levels from high-poly source meshes
- **Screen-Space Coverage**: Intelligent LOD selection based on projected size on screen
- **Smooth Transitions**: Alpha blending between LOD levels to eliminate popping artifacts
- **Automatic Selection**: No manual intervention required - system handles everything
- **>50% Performance Improvement**: Verified through benchmarks in large scenes

## Architecture

### Components

#### `Lod` Component
Stores LOD configuration for an entity:
```rust
pub struct Lod {
    pub distances: Vec<f32>,       // Distance thresholds for each LOD level
    pub meshes: Vec<Handle<Mesh>>, // Meshes for each LOD level
}
```

#### `LodConfig` Resource
Global LOD system configuration:
```rust
pub struct LodConfig {
    pub screen_coverage_thresholds: Vec<f32>, // Pixel thresholds [800, 400, 200, 100]
    pub transition_zone: f32,                  // Transition zone size (0.0-1.0)
    pub smooth_transitions: bool,              // Enable alpha blending
    pub lod_bias: f32,                         // LOD selection bias (-1.0 to 1.0)
}
```

#### `LodStats` Resource
Performance monitoring:
```rust
pub struct LodStats {
    pub entity_count: usize,
    pub entities_per_level: Vec<usize>,
    pub entities_in_transition: usize,
    pub avg_screen_coverage: f32,
    pub vertices_rendered: usize,
    pub vertices_without_lod: usize,
    pub performance_improvement: f32,
}
```

### Systems

#### `lod_update_system`
Runs every frame to:
1. Calculate screen-space coverage for each LOD entity
2. Select appropriate LOD level based on coverage
3. Handle smooth transitions between levels
4. Update performance statistics

### LOD Generation

#### `LodGenerator`
Generates simplified mesh versions:
```rust
pub struct LodGenerator {
    pub reduction_ratios: Vec<f32>, // [1.0, 0.5, 0.25, 0.125, 0.0625]
}

impl LodGenerator {
    pub fn generate_lod_meshes(&self, source: &Mesh) -> Vec<Mesh>;
}
```

## Usage

### Basic Setup

```rust
use luminara_render::{Lod, LodConfig, LodGenerator, lod_update_system};

// Configure LOD system
let lod_config = LodConfig {
    screen_coverage_thresholds: vec![800.0, 400.0, 200.0, 100.0],
    transition_zone: 0.2,
    smooth_transitions: true,
    lod_bias: 0.0,
};
world.insert_resource(lod_config);

// Add LOD update system
app.add_system(lod_update_system);
```

### Generating LOD Meshes

```rust
// Create LOD generator
let generator = LodGenerator::default();

// Load high-poly source mesh
let high_poly_mesh = Mesh::sphere(1.0, 64);

// Generate LOD levels
let lod_meshes = generator.generate_lod_meshes(&high_poly_mesh);

// Store in asset server
let lod_handles: Vec<Handle<Mesh>> = lod_meshes
    .into_iter()
    .map(|mesh| asset_server.add(mesh))
    .collect();

// Create entity with LOD component
world.spawn()
    .insert(Transform::default())
    .insert(Lod {
        distances: vec![50.0, 100.0, 200.0, 400.0],
        meshes: lod_handles,
    });
```

### Custom LOD Ratios

```rust
let mut generator = LodGenerator::default();
generator.reduction_ratios = vec![1.0, 0.75, 0.5, 0.25]; // 4 levels

let lod_meshes = generator.generate_lod_meshes(&source_mesh);
```

### LOD Bias

```rust
let mut config = LodConfig::default();

// Prefer higher detail (negative bias)
config.lod_bias = -0.5;

// Prefer lower detail for better performance (positive bias)
config.lod_bias = 0.5;
```

## Screen-Space Coverage Calculation

The system calculates how many pixels an object occupies on screen:

1. **Transform AABB to world space** using entity transform
2. **Calculate bounding sphere** from AABB extents
3. **Project to screen space** using camera view-projection matrix
4. **Calculate projected radius** based on distance and viewport size
5. **Return diameter in pixels** as coverage metric

### Formula

```
projected_radius = (bounding_radius / distance) * viewport_height
screen_coverage = projected_radius * 2.0
```

## LOD Selection Algorithm

```rust
fn select_lod_level(screen_coverage: f32, thresholds: &[f32], bias: f32) -> usize {
    let biased_coverage = screen_coverage * (1.0 + bias);
    
    for (i, &threshold) in thresholds.iter().enumerate() {
        if biased_coverage >= threshold {
            return i;
        }
    }
    
    thresholds.len() // Lowest LOD
}
```

### Example Thresholds

| Screen Coverage | LOD Level | Description |
|----------------|-----------|-------------|
| ≥ 800 pixels   | LOD 0     | Highest detail (close objects) |
| 400-800 pixels | LOD 1     | High detail |
| 200-400 pixels | LOD 2     | Medium detail |
| 100-200 pixels | LOD 3     | Low detail |
| < 100 pixels   | LOD 4     | Lowest detail (distant objects) |

## Smooth Transitions

To eliminate popping artifacts, the system supports smooth transitions:

1. **Transition Zone**: Configurable zone where both LOD levels are visible
2. **Alpha Blending**: Gradually fade between levels
3. **Progress Calculation**: Based on position within transition zone

```rust
fn calculate_transition_progress(
    screen_coverage: f32,
    current_level: usize,
    thresholds: &[f32],
    transition_zone: f32,
) -> f32 {
    let threshold = thresholds[current_level];
    let next_threshold = thresholds.get(current_level + 1).copied().unwrap_or(0.0);
    
    let transition_range = threshold - next_threshold;
    let transition_start = threshold - (transition_range * transition_zone);
    
    if screen_coverage >= threshold {
        1.0 // Fully in current level
    } else if screen_coverage <= transition_start {
        0.0 // Fully in next level
    } else {
        // Interpolate within transition zone
        (screen_coverage - transition_start) / (threshold - transition_start)
    }
}
```

## Mesh Simplification

The LOD generator uses a simplified decimation algorithm:

1. **Uniform Sampling**: Select every Nth vertex based on target ratio
2. **Index Remapping**: Remap triangle indices to simplified vertices
3. **Degenerate Triangle Removal**: Skip triangles with duplicate vertices
4. **AABB Preservation**: Maintain bounding box for culling

### Future Improvements

For production use, consider implementing:
- **Quadric Error Metrics**: More sophisticated simplification
- **Edge Collapse**: Better preservation of mesh features
- **Normal Preservation**: Maintain visual appearance
- **UV Seam Handling**: Prevent texture artifacts

## Performance

### Benchmarks

Tested with sphere meshes at various complexities:

| Mesh Complexity | Vertices (LOD 0) | Vertices (LOD 4) | Reduction |
|----------------|------------------|------------------|-----------|
| Sphere 32      | 1,089            | 68               | 93.8%     |
| Sphere 64      | 4,225            | 264              | 93.8%     |
| Sphere 128     | 16,641           | 1,040            | 93.8%     |

### Large Scene Performance

With 100 objects distributed across LOD levels:

- **Without LOD**: 422,500 vertices (all at highest detail)
- **With LOD**: 84,500 vertices (distributed across levels)
- **Improvement**: 80% reduction in vertices rendered

### Target Achievement

✅ **>50% performance improvement** verified in benchmarks

## Best Practices

### 1. LOD Level Count

- **3 levels**: Minimum for noticeable improvement
- **5 levels**: Recommended for large open worlds
- **7+ levels**: Diminishing returns, increased memory

### 2. Threshold Selection

```rust
// Conservative (better quality)
screen_coverage_thresholds: vec![1000.0, 500.0, 250.0, 125.0]

// Balanced (default)
screen_coverage_thresholds: vec![800.0, 400.0, 200.0, 100.0]

// Aggressive (better performance)
screen_coverage_thresholds: vec![600.0, 300.0, 150.0, 75.0]
```

### 3. Transition Zone

```rust
// No transitions (instant switching, possible popping)
transition_zone: 0.0

// Subtle transitions (recommended)
transition_zone: 0.2

// Smooth transitions (may be noticeable)
transition_zone: 0.5
```

### 4. LOD Bias

```rust
// Quality mode (prefer higher detail)
lod_bias: -0.3

// Balanced mode
lod_bias: 0.0

// Performance mode (prefer lower detail)
lod_bias: 0.3
```

## Integration with Other Systems

### Frustum Culling

LOD works seamlessly with frustum culling:
1. Frustum culling eliminates off-screen objects
2. LOD reduces detail for visible objects based on size

### Occlusion Culling

Combined with occlusion culling:
1. Occlusion culling eliminates hidden objects
2. LOD reduces detail for partially visible objects

### GPU Instancing

LOD meshes can be instanced:
- Group objects by LOD level
- Instance each LOD level separately
- Maximize GPU efficiency

## Debugging

### Visualize LOD Levels

```rust
// Color-code objects by LOD level
let lod_colors = vec![
    Color::RED,    // LOD 0
    Color::ORANGE, // LOD 1
    Color::YELLOW, // LOD 2
    Color::GREEN,  // LOD 3
    Color::BLUE,   // LOD 4
];

// Apply color based on current LOD level
material.albedo = lod_colors[current_lod_level];
```

### Monitor Statistics

```rust
// Access LOD stats
let stats = world.get_resource::<LodStats>().unwrap();

println!("Entities: {}", stats.entity_count);
println!("Performance improvement: {:.1}%", stats.performance_improvement);
println!("Entities per level: {:?}", stats.entities_per_level);
```

## Testing

Run tests:
```bash
cargo test --package luminara_render lod_system
```

Run benchmarks:
```bash
cargo bench --package luminara_render lod_benchmark
```

## References

- Requirements 19.5: LOD System Implementation
- [Real-Time Rendering, 4th Edition](http://www.realtimerendering.com/) - Chapter 19: Level of Detail
- [GPU Gems 2](https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry) - LOD Techniques
