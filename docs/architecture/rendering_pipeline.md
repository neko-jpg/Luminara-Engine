# Rendering Pipeline Architecture

## Overview

Luminara Engine uses a modern, high-performance rendering pipeline built on wgpu (WebGPU). The architecture prioritizes performance through aggressive batching, instancing, and culling while maintaining visual quality with PBR materials and advanced lighting.

## Pipeline Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Frame Preparation                        │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Culling   │→ │  Sorting   │→ │  Batch Generation    │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│                      Shadow Pass                              │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Cascade   │→ │  Shadow    │→ │  Shadow Map          │  │
│  │  Setup     │  │  Culling   │  │  Rendering           │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│                      Main Pass                                │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Opaque    │→ │  Sky/      │→ │  Transparent         │  │
│  │  Geometry  │  │  Background│  │  Geometry            │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│                      Post-Processing                          │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Bloom     │→ │  Tone      │→ │  Final Output        │  │
│  │            │  │  Mapping   │  │                      │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Core Components

### Camera System

Cameras define the viewpoint and projection for rendering.

```rust
#[derive(Component)]
pub struct Camera {
    pub projection: Projection,
    pub viewport: Option<Viewport>,
    pub priority: i32,
    pub is_active: bool,
}

pub enum Projection {
    Perspective {
        fov: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        scale: f32,
        near: f32,
        far: f32,
    },
}

// Multiple cameras supported
// Render order determined by priority
```

### Mesh Rendering

Meshes define the geometry to be rendered.

```rust
#[derive(Component)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub primitive_topology: PrimitiveTopology,
}

pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}

// GPU buffers managed automatically
// Supports instancing for repeated meshes
```

### Material System

Materials define surface properties using Physically Based Rendering (PBR).

```rust
#[derive(Component)]
pub struct PbrMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Texture>>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    pub normal_map: Option<Handle<Texture>>,
    pub emissive: Color,
    pub emissive_texture: Option<Handle<Texture>>,
    pub alpha_mode: AlphaMode,
}

pub enum AlphaMode {
    Opaque,
    Mask(f32),  // Alpha cutoff
    Blend,
}
```

### Lighting

Multiple light types with shadow support.

```rust
#[derive(Component)]
pub enum Light {
    Directional {
        direction: Vec3,
        color: Color,
        intensity: f32,
        shadows: bool,
    },
    Point {
        position: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        shadows: bool,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
        shadows: bool,
    },
}

// Forward+ rendering supports 100+ lights efficiently
```

## Optimization Techniques

### 1. Frustum Culling

Eliminates objects outside the camera view.

```rust
pub struct FrustumCuller {
    frustum: Frustum,
}

impl FrustumCuller {
    pub fn cull(&self, bounds: &Aabb) -> bool {
        self.frustum.intersects_aabb(bounds)
    }
}

// Performance: >95% culling efficiency
// Cost: <0.5ms for 10,000 objects
```

**Implementation Details:**
- Extracts frustum planes from view-projection matrix
- Tests AABB against 6 frustum planes
- Early-out on first plane rejection
- SIMD-optimized plane tests

### 2. Occlusion Culling

Eliminates objects hidden behind other objects.

```rust
pub struct OcclusionCuller {
    depth_pyramid: Handle<Texture>,
    query_buffer: wgpu::Buffer,
}

// GPU-driven occlusion queries
// Performance: >80% efficiency in dense scenes
// Minimal CPU overhead
```

**Implementation Details:**
- Generates hierarchical depth buffer (mip chain)
- Tests object bounds against depth pyramid
- GPU-driven: no CPU readback required
- Two-pass rendering: occluders first, then occludees

### 3. Draw Call Batching

Combines multiple objects into single draw calls.

```rust
pub struct DrawCallBatcher {
    batches: Vec<Batch>,
}

pub struct Batch {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub instances: Vec<InstanceData>,
}

// Batching strategy:
// 1. Sort by shader
// 2. Sort by texture
// 3. Sort by material
// 4. Batch identical materials

// Performance: <100 draw calls for 1000+ objects
```

**Batching Rules:**
- Objects with same mesh + material → single draw call
- Automatic instance buffer generation
- State change minimization
- Dynamic batching for moving objects

### 4. GPU Instancing

Renders multiple copies of the same mesh efficiently.

```rust
pub struct InstanceData {
    pub transform: Mat4,
    pub color: Vec4,
    pub custom_data: Vec4,
}

// Instancing automatically applied when:
// - Same mesh used by multiple entities
// - Same material properties
// - Different transforms

// Performance: >10x improvement for repeated meshes
```

**Instance Buffer Layout:**
```wgsl
struct InstanceData {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
    color: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) instance_transform: mat4x4<f32>,
) -> VertexOutput {
    // ...
}
```

### 5. Level of Detail (LOD)

Reduces geometry complexity based on distance.

```rust
#[derive(Component)]
pub struct LodMesh {
    pub lods: Vec<LodLevel>,
    pub current_lod: usize,
}

pub struct LodLevel {
    pub mesh: Handle<Mesh>,
    pub screen_size: f32,  // Switch threshold
}

// LOD selection based on screen space coverage
// Smooth transitions (no popping)
// Performance: >50% improvement in large scenes
```

**LOD Strategy:**
- 3-5 LOD levels per mesh
- Screen space error metric
- Hysteresis to prevent flickering
- Optional cross-fade transitions

## Shadow Rendering

### Cascaded Shadow Maps (CSM)

High-quality shadows for directional lights.

```rust
pub struct CascadedShadowMaps {
    pub cascade_count: usize,
    pub cascade_splits: Vec<f32>,
    pub shadow_maps: Vec<Handle<Texture>>,
    pub resolution: u32,
}

// 4 cascades for optimal quality/performance
// Resolution: 2048x2048 per cascade
// GPU time: <2ms for all cascades
```

**Cascade Splitting:**
- Logarithmic split scheme
- Near cascade: high detail
- Far cascades: lower detail
- Smooth transitions between cascades

**Shadow Map Configuration:**
```rust
const CASCADE_COUNT: usize = 4;
const SHADOW_MAP_SIZE: u32 = 2048;
const CASCADE_SPLITS: [f32; 4] = [0.05, 0.15, 0.40, 1.0];

// Percentage Closer Filtering (PCF) for soft shadows
// Bias to prevent shadow acne
// Slope-scale bias for angled surfaces
```

### Point Light Shadows

Cube map shadows for omnidirectional lights.

```rust
pub struct PointLightShadow {
    pub cube_map: Handle<Texture>,
    pub resolution: u32,
    pub near: f32,
    pub far: f32,
}

// 6 faces rendered per light
// Optimized with geometry shader or instancing
```

## PBR Shading

### Material Model

Cook-Torrance BRDF with GGX distribution.

```wgsl
// Physically Based Rendering shader
fn pbr_lighting(
    N: vec3<f32>,  // Normal
    V: vec3<f32>,  // View direction
    L: vec3<f32>,  // Light direction
    base_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> vec3<f32> {
    let H = normalize(V + L);
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);
    let VdotH = max(dot(V, H), 0.0);
    
    // Fresnel (Schlick approximation)
    let F0 = mix(vec3(0.04), base_color, metallic);
    let F = F0 + (1.0 - F0) * pow(1.0 - VdotH, 5.0);
    
    // Distribution (GGX/Trowbridge-Reitz)
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    let D = a2 / (PI * denom * denom);
    
    // Geometry (Smith's method)
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let G1_V = NdotV / (NdotV * (1.0 - k) + k);
    let G1_L = NdotL / (NdotL * (1.0 - k) + k);
    let G = G1_V * G1_L;
    
    // Cook-Torrance BRDF
    let specular = (D * F * G) / max(4.0 * NdotV * NdotL, 0.001);
    
    // Diffuse (Lambertian)
    let kD = (1.0 - F) * (1.0 - metallic);
    let diffuse = kD * base_color / PI;
    
    return (diffuse + specular) * NdotL;
}
```

### Image-Based Lighting (IBL)

Environment maps for ambient lighting.

```rust
pub struct EnvironmentMap {
    pub irradiance_map: Handle<Texture>,
    pub prefiltered_map: Handle<Texture>,
    pub brdf_lut: Handle<Texture>,
}

// Split-sum approximation for real-time IBL
// Precomputed diffuse irradiance
// Prefiltered specular reflections
```

## Forward+ Rendering

Efficient rendering of many lights.

```
1. Depth Pre-pass
   └─> Generate depth buffer

2. Light Culling (Compute Shader)
   ├─> Divide screen into tiles (16x16 pixels)
   ├─> Cull lights per tile
   └─> Generate light lists

3. Shading Pass
   ├─> Read light list for current tile
   ├─> Evaluate lighting for visible lights only
   └─> Output final color
```

**Performance:**
- Supports 100+ dynamic lights
- Constant shading cost per pixel
- GPU-driven light culling
- Minimal CPU overhead

## Render Graph

Flexible render pass composition.

```rust
pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNode>>,
    edges: Vec<(NodeId, NodeId)>,
}

pub trait RenderNode {
    fn prepare(&mut self, world: &World);
    fn render(&self, encoder: &mut CommandEncoder, resources: &RenderResources);
}

// Example graph:
// ShadowPass → MainPass → PostProcess → UI → Present
```

## Performance Benchmarks

### Draw Call Performance

```
Scene: 1000 unique objects
- Without batching: 1000 draw calls, 45 FPS
- With batching: 87 draw calls, 120 FPS
- With instancing: 12 draw calls, 240 FPS
```

### Culling Performance

```
Scene: 10,000 objects
- No culling: 16.7ms frame time (60 FPS)
- Frustum culling: 8.3ms frame time (120 FPS)
- Frustum + Occlusion: 5.5ms frame time (180 FPS)
```

### Shadow Performance

```
4 Cascades, 2048x2048 resolution:
- Shadow map generation: 1.8ms
- Shadow sampling: 0.4ms
- Total shadow cost: 2.2ms
```

### LOD Performance

```
Scene: 5000 high-poly meshes
- No LOD: 12.5ms frame time (80 FPS)
- With LOD: 6.2ms frame time (160 FPS)
- Performance gain: 2x
```

## Debug Visualization

### Wireframe Mode

```rust
pub struct DebugRenderSettings {
    pub wireframe: bool,
    pub show_normals: bool,
    pub show_bounds: bool,
    pub show_lights: bool,
}

// Toggle with F1-F4 keys in debug builds
```

### Overdraw Heatmap

Visualizes pixel overdraw for optimization.

```rust
// Red = high overdraw (bad)
// Green = low overdraw (good)
// Blue = no overdraw (optimal)
```

### GPU Profiling

```rust
pub struct GpuProfiler {
    pub pass_times: HashMap<String, Duration>,
}

// Tracks GPU time per render pass
// Identifies bottlenecks
// Exports Chrome tracing format
```

## Best Practices

### Material Optimization

- **Minimize unique materials**: Batching requires identical materials
- **Use texture atlases**: Reduce texture bindings
- **Share textures**: Reuse textures across materials
- **Avoid alpha blending**: Use alpha masking when possible

### Mesh Optimization

- **Reduce vertex count**: Use LOD for distant objects
- **Optimize vertex layout**: Minimize vertex size
- **Use index buffers**: Reduce vertex duplication
- **Merge static meshes**: Combine non-moving objects

### Lighting Optimization

- **Limit light count**: Use Forward+ for many lights
- **Disable shadows**: Only important lights need shadows
- **Use baked lighting**: Precompute static lighting
- **Optimize shadow resolution**: Balance quality vs performance

### Culling Optimization

- **Compute tight bounds**: Accurate AABBs improve culling
- **Use spatial partitioning**: Octree/BVH for large scenes
- **Implement LOD**: Reduce geometry at distance
- **Enable occlusion culling**: For dense urban scenes

## Advanced Features

### Custom Shaders

```rust
// Load custom WGSL shader
let shader = asset_server.load("shaders/custom.wgsl");

let material = CustomMaterial {
    shader,
    uniforms: CustomUniforms { /* ... */ },
};
```

### Compute Shaders

```rust
// GPU-driven particle system
let compute_pipeline = device.create_compute_pipeline(/* ... */);

// Dispatch compute work
encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
```

### Render Targets

```rust
// Render to texture
let render_target = RenderTarget::Texture(texture_handle);

camera.target = render_target;
```

## Further Reading

- [PBR Theory](https://learnopengl.com/PBR/Theory) - PBR fundamentals
- [Forward+ Rendering](https://takahiroharada.files.wordpress.com/2015/04/forward_plus.pdf) - Forward+ paper
- [Cascaded Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus) - CSM technique
- [GPU Gems](https://developer.nvidia.com/gpugems) - Advanced rendering techniques
- [Shader Optimization](shader_optimization.md) - Shader performance tips

## References

- [wgpu Documentation](https://wgpu.rs/) - WebGPU API
- [WebGPU Specification](https://www.w3.org/TR/webgpu/) - Standard specification
- [Real-Time Rendering](https://www.realtimerendering.com/) - Rendering textbook
