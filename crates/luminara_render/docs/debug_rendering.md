# Debug Rendering Visualization

## Overview

The debug rendering system provides specialized visualization modes for debugging rendering issues. It integrates with the unified GizmoSystem and provides three main modes:

1. **Wireframe Mode**: Renders mesh edges to visualize geometry
2. **Normal Visualization**: Shows surface normals as RGB colors
3. **Overdraw Heatmap**: Visualizes pixel overdraw to identify performance issues

## Architecture

### Components

- **DebugRenderingResource**: Main resource managing debug rendering state
- **DebugRenderMode**: Enum defining available debug modes
- **Shader Variants**: Specialized shaders for each visualization mode

### Integration

The debug rendering system integrates with:
- **GizmoSystem**: Unified debug visualization system
- **Rendering Pipeline**: Hooks into the main render pass
- **GPU Context**: Manages GPU resources for debug rendering

## Usage

### Basic Usage

```rust
use luminara_render::{DebugRenderingResource, DebugRenderMode};

// Get the debug rendering resource
let mut debug_rendering = world.get_resource_mut::<DebugRenderingResource>().unwrap();

// Enable wireframe mode
debug_rendering.set_mode(DebugRenderMode::Wireframe);

// Toggle modes
debug_rendering.toggle_wireframe();
debug_rendering.toggle_normals();
debug_rendering.toggle_overdraw();
```

### Integration with GizmoSystem

```rust
use luminara_render::GizmoSystem;

let mut gizmo_system = world.get_resource_mut::<GizmoSystem>().unwrap();

// Enable rendering visualization mode
gizmo_system.enable_mode(VisualizationMode::Rendering);

// Configure rendering settings
let settings = gizmo_system.rendering_settings_mut();
settings.show_wireframe = true;
settings.show_normals = true;
settings.show_overdraw = true;
```

## Debug Modes

### Wireframe Mode

Renders mesh edges in white with slight transparency. Useful for:
- Visualizing mesh topology
- Identifying polygon count
- Debugging mesh deformation
- Checking mesh connectivity

**Shader**: `debug_wireframe.wgsl`

### Normal Visualization

Converts surface normals to RGB colors:
- X component → Red channel
- Y component → Green channel
- Z component → Blue channel

Useful for:
- Verifying normal directions
- Debugging normal mapping
- Identifying inverted normals
- Checking smooth vs flat shading

**Shader**: `debug_normals.wgsl`

### Overdraw Heatmap

Visualizes how many times each pixel is drawn:
- **Black**: No draws (background)
- **Blue**: 1 draw (optimal)
- **Green**: 2-3 draws (acceptable)
- **Yellow**: 4-5 draws (concerning)
- **Red**: 6+ draws (problematic)

Useful for:
- Identifying overdraw hotspots
- Optimizing draw order
- Improving culling
- Reducing fill rate pressure

**Shader**: `debug_overdraw.wgsl`

## Implementation Details

### Wireframe Pipeline

- **Topology**: LineList
- **Polygon Mode**: Line
- **Culling**: None (to see all edges)
- **Blending**: Alpha blending for transparency

### Normal Pipeline

- **Topology**: TriangleList
- **Polygon Mode**: Fill
- **Culling**: Back face culling
- **Color Mapping**: Normal [-1,1] → RGB [0,1]

### Overdraw Pipeline

- **Counter Texture**: R32Uint format for counting draws
- **Visualization**: Full-screen pass converting counts to heatmap
- **Color Gradient**: Black → Blue → Green → Yellow → Red

## Performance Considerations

### Wireframe Mode
- Minimal overhead (~5-10% frame time increase)
- Requires line rasterization support
- May be slower on some mobile GPUs

### Normal Visualization
- Very low overhead (~1-2% frame time increase)
- Simple fragment shader
- No additional GPU resources needed

### Overdraw Heatmap
- Moderate overhead (~10-15% frame time increase)
- Requires additional render target (R32Uint texture)
- Two-pass rendering (count + visualize)
- Memory usage: width × height × 4 bytes

## Best Practices

1. **Use in Development Only**: Debug modes should be disabled in production builds
2. **Toggle with Hotkeys**: Bind debug modes to keyboard shortcuts for quick access
3. **Combine with Profiling**: Use debug modes alongside GPU profiling for comprehensive analysis
4. **Document Findings**: Take screenshots of debug visualizations for issue tracking

## Example: Debugging Overdraw

```rust
// Enable overdraw visualization
debug_rendering.set_mode(DebugRenderMode::Overdraw);

// Render frame and observe heatmap
// - Blue areas: Good (1 draw per pixel)
// - Yellow/Red areas: Bad (multiple draws per pixel)

// Optimize by:
// 1. Improving draw order (front-to-back for opaque)
// 2. Implementing better culling
// 3. Reducing transparent object count
// 4. Using GPU instancing for repeated geometry
```

## Future Enhancements

Potential additions to the debug rendering system:
- **Depth Visualization**: Show depth buffer as grayscale
- **UV Visualization**: Display UV coordinates as colors
- **Tangent/Bitangent Visualization**: Show tangent space vectors
- **LOD Visualization**: Color-code objects by LOD level
- **Culling Visualization**: Highlight culled vs rendered objects
- **Light Coverage**: Show which lights affect each pixel

## Requirements Validation

This implementation satisfies **Requirement 15.2**:
> WHEN debugging rendering, THE System SHALL provide wireframe mode, normal visualization, and overdraw heatmaps

All three required visualization modes are implemented and integrated with the unified GizmoSystem.
