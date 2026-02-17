# Cascaded Shadow Maps Implementation

## Overview

This document describes the cascaded shadow map (CSM) implementation in Luminara Engine, designed to achieve high-quality shadows with smooth transitions and optimal GPU performance (<2ms for 4 cascades at 2048x2048 resolution).

## Architecture

### Key Components

1. **ShadowCascades** - Configuration resource for cascade parameters
2. **ShadowMapResources** - GPU resources (textures, buffers, bind groups)
3. **CascadeUniform** - Per-cascade data (view-projection matrix, split depths, blend regions)
4. **Shadow Shaders** - Optimized WGSL shaders for shadow map generation and sampling

### Cascade Configuration

```rust
pub struct ShadowCascades {
    pub cascade_count: u32,           // Number of cascades (default: 4)
    pub shadow_map_size: u32,         // Resolution per cascade (default: 2048)
    pub split_lambda: f32,            // Split distribution (0.0=uniform, 1.0=logarithmic)
    pub blend_region: f32,            // Blend region size (0.0-1.0, default: 0.1)
    pub depth_bias: f32,              // Depth bias to prevent shadow acne
    pub slope_bias: f32,              // Slope-based bias
}
```

## Cascade Split Calculation

The system uses a hybrid logarithmic-uniform split distribution controlled by the `split_lambda` parameter:

```
split_depth = lambda * log_split + (1 - lambda) * uniform_split
```

- **lambda = 0.0**: Uniform distribution (equal-sized cascades in view space)
- **lambda = 0.5**: Balanced distribution (default, good for most scenes)
- **lambda = 1.0**: Logarithmic distribution (more detail near camera)

### Split Distribution Formula

For cascade `i` out of `N` cascades:

```
p = (i + 1) / N
log_split = near * (far / near)^p
uniform_split = near + (far - near) * p
final_split = lambda * log_split + (1 - lambda) * uniform_split
```

## Smooth Cascade Transitions

To eliminate visible seams between cascades, the system implements smooth blending:

### Blend Region Calculation

Each cascade has a blend region at its far end:

```
blend_size = cascade_range * blend_region_factor
blend_start = split_depth - blend_size
blend_end = split_depth
```

### Shader-Side Blending

In the fragment shader, when a fragment falls within a blend region:

1. Sample shadow from current cascade
2. Sample shadow from next cascade
3. Blend linearly based on depth within blend region:

```wgsl
blend_factor = (view_depth - blend_start) / (blend_end - blend_start)
final_shadow = mix(shadow1, shadow2, blend_factor)
```

## Shadow Map Rendering

### Optimizations

1. **Minimal Vertex Shader**: Only transforms vertices to light space
2. **Empty Fragment Shader**: Depth is written automatically
3. **Tight Frustum Fitting**: Each cascade's projection tightly fits the camera frustum slice
4. **Texel Snapping**: Shadow map coordinates snap to texel grid to reduce shimmering

### Texel Snapping

To prevent shadow shimmering when the camera moves:

```rust
let world_units_per_texel = (max.x - min.x) / shadow_map_size;
min.x = (min.x / world_units_per_texel).floor() * world_units_per_texel;
min.y = (min.y / world_units_per_texel).floor() * world_units_per_texel;
```

## Shadow Sampling

### PCF (Percentage Closer Filtering)

The system uses a 3x3 PCF kernel for smooth shadow edges:

```wgsl
for (var x = -1; x <= 1; x++) {
    for (var y = -1; y <= 1; y++) {
        shadow += textureSampleCompareLevel(
            shadow_map, shadow_sampler,
            coord + offset, cascade_idx, depth
        );
    }
}
shadow /= 9.0; // Average of 9 samples
```

### Slope-Based Bias

To prevent shadow acne while minimizing peter-panning:

```wgsl
let n_dot_l = max(dot(normal, light_dir), 0.0);
let bias = max(depth_bias * (1.0 - n_dot_l), min_bias);
```

## Performance Characteristics

### Target Performance

- **GPU Time**: <2ms for 4 cascades at 2048x2048 resolution
- **Memory**: ~64MB for 4 cascades (4 × 2048² × 4 bytes)
- **Bandwidth**: Optimized through PCF and comparison sampling

### Performance Optimizations

1. **Depth-Only Rendering**: No color attachments, minimal fragment processing
2. **Instanced Rendering**: Batch identical meshes in shadow pass
3. **Frustum Culling**: Per-cascade culling reduces overdraw
4. **Early-Z Optimization**: Depth testing before fragment shader

### Profiling Results

Expected performance on mid-range GPU (GTX 1660):

| Cascade Count | Resolution | GPU Time | Memory |
|---------------|------------|----------|--------|
| 2             | 2048²      | ~1.0ms   | 32MB   |
| 4             | 2048²      | ~1.8ms   | 64MB   |
| 4             | 4096²      | ~4.5ms   | 256MB  |

## Usage Example

### Basic Setup

```rust
use luminara_render::shadow::{ShadowCascades, ShadowMapResources};

// Configure cascades
let mut cascades = ShadowCascades::default();
cascades.cascade_count = 4;
cascades.shadow_map_size = 2048;
cascades.split_lambda = 0.5;
cascades.blend_region = 0.1;

// Add as resource
world.insert_resource(cascades);
world.insert_resource(ShadowMapResources::default());
```

### Directional Light Configuration

```rust
use luminara_render::DirectionalLight;

let light = DirectionalLight {
    color: Color::WHITE,
    intensity: 1.0,
    cast_shadows: true,
    shadow_cascade_count: 4,
};
```

### Custom Split Distribution

```rust
// More detail near camera (good for first-person games)
cascades.split_lambda = 0.8;

// More uniform distribution (good for strategy games)
cascades.split_lambda = 0.2;
```

## Quality vs Performance Trade-offs

### High Quality (Cinematic)

```rust
cascades.cascade_count = 4;
cascades.shadow_map_size = 4096;
cascades.blend_region = 0.15;
// GPU Time: ~4-5ms
```

### Balanced (Default)

```rust
cascades.cascade_count = 4;
cascades.shadow_map_size = 2048;
cascades.blend_region = 0.1;
// GPU Time: ~1.8-2ms
```

### Performance (Mobile/Low-End)

```rust
cascades.cascade_count = 2;
cascades.shadow_map_size = 1024;
cascades.blend_region = 0.05;
// GPU Time: ~0.5ms
```

## Troubleshooting

### Shadow Acne

**Symptom**: Moiré patterns on surfaces
**Solution**: Increase `depth_bias` or `slope_bias`

```rust
cascades.depth_bias = 0.01;  // Increase from default 0.005
cascades.slope_bias = 3.0;   // Increase from default 2.0
```

### Peter-Panning

**Symptom**: Shadows detached from objects
**Solution**: Decrease bias values

```rust
cascades.depth_bias = 0.002;
cascades.slope_bias = 1.0;
```

### Visible Cascade Seams

**Symptom**: Lines visible between cascades
**Solution**: Increase blend region

```rust
cascades.blend_region = 0.15;  // Increase from default 0.1
```

### Shadow Shimmering

**Symptom**: Shadows flicker when camera moves
**Solution**: Texel snapping is automatic, but ensure:
- Shadow map resolution is adequate
- Cascade splits are appropriate for scene scale

## Future Enhancements

Potential improvements for future versions:

1. **Parallel Split Shadow Maps (PSSM)**: Alternative split calculation
2. **Sample Distribution Shadow Maps (SDSM)**: Adaptive split placement
3. **Variance Shadow Maps (VSM)**: Pre-filtered shadows for better performance
4. **Contact-Hardening Shadows**: Variable penumbra based on distance
5. **Point Light Shadows**: Cube map shadows for point lights
6. **Spot Light Shadows**: Single-cascade shadows for spot lights

## References

- [Cascaded Shadow Maps (NVIDIA)](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus)
- [Sample Distribution Shadow Maps](https://developer.nvidia.com/sample-distribution-shadow-maps)
- [Shadow Mapping Best Practices](https://learn.microsoft.com/en-us/windows/win32/dxtecharts/common-techniques-to-improve-shadow-depth-maps)

## Implementation Status

- [x] 4-cascade shadow maps
- [x] Smooth cascade transitions with blend regions
- [x] PCF shadow sampling (3x3 kernel)
- [x] Texel snapping for stability
- [x] Slope-based bias
- [x] Tight frustum fitting
- [x] GPU buffer management
- [x] Shader integration
- [ ] GPU profiling integration
- [ ] Performance benchmarking
- [ ] Point light shadows
- [ ] Spot light shadows

## Performance Target

**Target**: <2ms GPU time for 4 cascades at 2048x2048 resolution

**Status**: Implementation complete, pending GPU profiling verification

**Requirement**: 19.2 - Cascaded shadow maps with smooth transitions and <2ms GPU time
