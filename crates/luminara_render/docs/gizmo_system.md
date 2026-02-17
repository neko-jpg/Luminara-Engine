# Unified Gizmo System

## Overview

The `GizmoSystem` provides a centralized, unified interface for all debug visualization in Luminara Engine. It supports multiple visualization modes (physics, rendering, transforms, audio) and integrates seamlessly with both the 3D `CommandBuffer` and 2D `OverlayRenderer`.

## Features

- **Multiple Visualization Modes**: Physics, Rendering, Transforms, Audio, and custom modes
- **Granular Control**: Enable/disable individual modes or the entire system
- **Configurable Settings**: Each mode has its own settings for colors, scales, and visibility options
- **Integration**: Works with existing `CommandBuffer` for 3D gizmos and `OverlayRenderer` for 2D overlays
- **Category Sync**: Automatically syncs with `GizmoCategories` for backward compatibility

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      GizmoSystem                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Physics    │  │  Rendering   │  │  Transforms  │     │
│  │  Settings    │  │   Settings   │  │   Settings   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐                       │
│  │    Audio     │  │    Custom    │                       │
│  │  Settings    │  │    Modes     │                       │
│  └──────────────┘  └──────────────┘                       │
└─────────────────────────────────────────────────────────────┘
           │                              │
           ▼                              ▼
    ┌─────────────┐              ┌─────────────────┐
    │ CommandBuffer│              │ OverlayRenderer │
    │  (3D Gizmos) │              │  (2D Overlays)  │
    └─────────────┘              └─────────────────┘
```

## Usage

### Basic Setup

```rust
use luminara_render::{GizmoSystem, VisualizationMode};

// Create the gizmo system
let mut gizmo_system = GizmoSystem::new();

// Enable specific modes
gizmo_system.enable_mode(VisualizationMode::Physics);
gizmo_system.enable_mode(VisualizationMode::Transforms);
```

### Global Control

```rust
// Disable all gizmo rendering
gizmo_system.set_enabled(false);

// Toggle gizmo system on/off
let is_enabled = gizmo_system.toggle();

// Check if enabled
if gizmo_system.is_enabled() {
    // Draw gizmos
}
```

### Mode Control

```rust
// Enable a mode
gizmo_system.enable_mode(VisualizationMode::Physics);

// Disable a mode
gizmo_system.disable_mode(VisualizationMode::Rendering);

// Toggle a mode
let is_active = gizmo_system.toggle_mode(VisualizationMode::Audio);

// Check if mode is active
if gizmo_system.is_mode_active(VisualizationMode::Physics) {
    // Draw physics gizmos
}

// Get all active modes
let active_modes = gizmo_system.active_modes();
```

### Custom Modes

```rust
// Define a custom mode
let navigation_mode = VisualizationMode::Custom("navigation");

// Use it like any other mode
gizmo_system.enable_mode(navigation_mode);
```

### Drawing Physics Gizmos

```rust
use luminara_math::Vec3;

// Draw collider
gizmo_system.draw_physics(
    &mut command_buffer,
    Vec3::new(0.0, 1.0, 0.0),  // position
    Vec3::new(0.5, 0.5, 0.5),  // half extents
);

// Draw velocity vector
gizmo_system.draw_velocity(
    &mut command_buffer,
    Vec3::new(0.0, 1.0, 0.0),  // position
    Vec3::new(1.0, 0.0, 0.0),  // velocity
);

// Draw contact point
gizmo_system.draw_contact_point(
    &mut command_buffer,
    Vec3::new(0.0, 0.0, 0.0),  // contact position
);
```

### Drawing Transform Gizmos

```rust
// Draw coordinate axes
gizmo_system.draw_transform_axes(
    &mut command_buffer,
    Vec3::new(0.0, 1.0, 0.0),  // position
    1.0,                        // scale
);
```

### Drawing Rendering Gizmos

```rust
// Draw bounding box
gizmo_system.draw_bounding_box(
    &mut command_buffer,
    Vec3::new(0.0, 1.0, 0.0),  // center
    Vec3::new(1.0, 1.0, 1.0),  // half extents
);
```

### Drawing Audio Gizmos

```rust
// Draw audio source with attenuation range
gizmo_system.draw_audio_source(
    &mut command_buffer,
    Vec3::new(0.0, 1.0, 0.0),  // position
    10.0,                       // attenuation radius
);
```

### Drawing 2D Overlays

```rust
// Draw text overlay
gizmo_system.draw_text_overlay(
    &mut overlay_renderer,
    10.0,                       // x
    10.0,                       // y
    "Debug Info",               // text
    [1.0, 1.0, 1.0, 1.0],      // color (RGBA)
);

// Draw status overlay showing active modes
gizmo_system.draw_status_overlay(
    &mut overlay_renderer,
    10.0,                       // x
    10.0,                       // y
);
```

### Configuring Settings

```rust
// Configure physics visualization
let physics_settings = gizmo_system.physics_settings_mut();
physics_settings.show_colliders = true;
physics_settings.show_velocities = true;
physics_settings.velocity_scale = 2.0;
physics_settings.collider_color = Color::rgba(0.0, 1.0, 0.0, 0.5);

// Configure rendering visualization
let rendering_settings = gizmo_system.rendering_settings_mut();
rendering_settings.show_wireframe = true;
rendering_settings.show_bounds = true;

// Configure transform visualization
let transform_settings = gizmo_system.transform_settings_mut();
transform_settings.axes_length = 2.0;
transform_settings.show_hierarchy = true;

// Configure audio visualization
let audio_settings = gizmo_system.audio_settings_mut();
audio_settings.show_attenuation = true;
```

### Integration with GizmoCategories

For backward compatibility with existing code using `GizmoCategories`:

```rust
// Sync GizmoSystem state with GizmoCategories
gizmo_system.sync_with_categories(&mut gizmo_categories);
```

This ensures that the legacy `GizmoCategories` system reflects the current state of the `GizmoSystem`.

## Visualization Modes

### Physics Mode

Visualizes physics simulation state:
- **Colliders**: Wireframe boxes/spheres showing collision shapes
- **Velocities**: Arrows showing velocity vectors
- **Contacts**: Spheres at collision contact points

Settings:
- `show_colliders`: Enable/disable collider visualization
- `show_velocities`: Enable/disable velocity vectors
- `show_contacts`: Enable/disable contact points
- `collider_color`: Color for collider wireframes
- `velocity_color`: Color for velocity arrows
- `contact_color`: Color for contact points
- `velocity_scale`: Scale factor for velocity arrows

### Rendering Mode

Visualizes rendering state:
- **Wireframe**: Wireframe overlay on meshes
- **Normals**: Normal vectors at vertices
- **Bounds**: Bounding boxes around meshes

Settings:
- `show_wireframe`: Enable/disable wireframe mode
- `show_normals`: Enable/disable normal visualization
- `show_bounds`: Enable/disable bounding boxes
- `wireframe_color`: Color for wireframe lines
- `normal_color`: Color for normal vectors
- `bounds_color`: Color for bounding boxes

### Transforms Mode

Visualizes transform hierarchy:
- **Axes**: RGB coordinate axes (X=Red, Y=Green, Z=Blue)
- **Hierarchy**: Lines connecting parent-child transforms

Settings:
- `show_axes`: Enable/disable coordinate axes
- `show_hierarchy`: Enable/disable hierarchy connections
- `axes_length`: Length of coordinate axes
- `hierarchy_color`: Color for hierarchy lines

### Audio Mode

Visualizes audio sources:
- **Sources**: Spheres at audio source positions
- **Attenuation**: Circles showing attenuation ranges

Settings:
- `show_sources`: Enable/disable source visualization
- `show_attenuation`: Enable/disable attenuation ranges
- `source_color`: Color for source markers
- `attenuation_color`: Color for attenuation circles

## Example: Complete System Integration

```rust
use luminara_core::shared_types::{Res, ResMut};
use luminara_render::{CommandBuffer, GizmoSystem, OverlayRenderer, VisualizationMode};
use luminara_math::Vec3;

fn debug_visualization_system(
    mut gizmo_system: ResMut<GizmoSystem>,
    mut command_buffer: ResMut<CommandBuffer>,
    mut overlay: ResMut<OverlayRenderer>,
    // ... other resources and queries
) {
    // Enable physics mode
    gizmo_system.enable_mode(VisualizationMode::Physics);
    
    // Draw physics gizmos for all rigid bodies
    // for (position, velocity, collider) in rigid_bodies.iter() {
    //     gizmo_system.draw_physics(&mut command_buffer, position, collider.half_extents);
    //     gizmo_system.draw_velocity(&mut command_buffer, position, velocity);
    // }
    
    // Draw status overlay
    gizmo_system.draw_status_overlay(&mut overlay, 10.0, 10.0);
}
```

## Performance Considerations

- Gizmo rendering is designed to be lightweight and only active in debug builds
- Each draw call checks if the mode is active before adding to the command buffer
- Disabling the entire system (`set_enabled(false)`) prevents all gizmo rendering with minimal overhead
- Settings are stored per-mode and accessed by reference, avoiding unnecessary copies

## Best Practices

1. **Use Modes Appropriately**: Enable only the modes you need for the current debugging task
2. **Configure Colors**: Use distinct colors for different gizmo types to avoid visual confusion
3. **Scale Appropriately**: Adjust `velocity_scale` and `axes_length` based on your scene scale
4. **Toggle in Development**: Bind gizmo toggles to keyboard shortcuts for quick debugging
5. **Integrate with UI**: Use `draw_status_overlay` to show which modes are active

## Migration from Legacy System

If you're using the old `Gizmos` and `GizmoCategories` directly:

**Before:**
```rust
Gizmos::cube_cat(&mut buffer, position, half_extents, color, "physics");
```

**After:**
```rust
gizmo_system.draw_physics(&mut buffer, position, half_extents);
```

The new system provides:
- Automatic mode checking
- Centralized settings
- Consistent API across all visualization types
- Better integration with overlay rendering

## Future Enhancements

Planned features for future versions:
- GPU-accelerated gizmo rendering
- Instanced gizmo rendering for large numbers of objects
- Custom gizmo shapes and materials
- Recording and playback of gizmo sequences
- Integration with profiling tools for performance visualization
