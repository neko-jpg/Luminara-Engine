# Fluid Rendering Integration

This document describes the integration of the spectral fluid solver from `luminara_math` with the rendering pipeline.

## Overview

The fluid rendering system connects the GPU-based spectral fluid solver to the rendering pipeline, allowing real-time visualization of fluid simulations. The system exposes velocity and pressure fields as textures that can be rendered using various visualization modes.

## Architecture

### Components

#### FluidRenderer

The `FluidRenderer` component is attached to entities that should display fluid simulations. It contains:

- **Grid dimensions**: Width and height of the simulation grid
- **Physical parameters**: Viscosity and boundary conditions
- **Texture handles**: References to velocity and pressure field textures
- **Visualization settings**: Mode and color scale for rendering

```rust
use luminara_render::{FluidRenderer, FluidVisualizationMode};
use luminara_math::dynamics::BoundaryMethod;

// Create a basic fluid renderer
let renderer = FluidRenderer::new(256, 256);

// Or with custom settings
let renderer = FluidRenderer::with_viscosity(256, 256, 0.01)
    .with_boundary_method(BoundaryMethod::Penalization)
    .with_visualization_mode(FluidVisualizationMode::Vorticity);
```

### Resources

#### FluidSolverResource

The `FluidSolverResource` manages the actual fluid solver instances. It maintains a mapping from entity IDs to solver instances, allowing multiple fluid simulations to run simultaneously.

The resource is automatically managed by the fluid systems and doesn't require manual interaction in most cases.

### Systems

The fluid rendering integration includes four systems that run at different stages:

1. **init_fluid_solvers_system** (PreUpdate)
   - Creates solver instances for new FluidRenderer components
   - Runs once per entity when the component is first added

2. **update_fluid_simulation_system** (Update)
   - Steps the fluid simulation forward in time
   - Updates all active solvers based on frame delta time

3. **sync_fluid_textures_system** (PreRender)
   - Extracts velocity and pressure fields from solvers
   - Updates GPU textures for rendering
   - Creates texture handles if they don't exist

4. **cleanup_fluid_solvers_system** (PostUpdate)
   - Removes solver instances when FluidRenderer components are removed
   - Prevents memory leaks from orphaned solvers

## Visualization Modes

The system supports multiple visualization modes:

### VelocityMagnitude
Visualizes the magnitude of the velocity field as a color gradient. Useful for seeing flow speed.

### VelocityDirection
Visualizes the direction of velocity vectors using a HSV color wheel. Useful for seeing flow patterns.

### Vorticity
Visualizes the curl of the velocity field, highlighting rotational motion and turbulence.

### Pressure
Visualizes the pressure field, showing areas of high and low pressure.

### Streamlines
Visualizes velocity as streamlines, showing the path particles would follow in the flow.

## Boundary Conditions

The solver supports three boundary condition methods:

### Periodic
Fluid wraps around at the boundaries. Suitable for simulating infinite domains or repeating patterns.

### Penalization
Obstacles are represented by a mask and enforced through penalization. Suitable for simulating flow around objects.

### ImmersedBoundary
Advanced boundary treatment using immersed boundary method. Suitable for complex geometries.

## Usage Example

```rust
use luminara_core::{App, World};
use luminara_render::{FluidRenderer, FluidVisualizationMode};
use luminara_math::dynamics::BoundaryMethod;

fn setup_fluid_simulation(world: &mut World) {
    // Spawn an entity with a fluid renderer
    let entity = world.spawn();
    
    // Create a fluid renderer with custom settings
    let renderer = FluidRenderer::with_viscosity(512, 512, 0.001)
        .with_boundary_method(BoundaryMethod::Periodic)
        .with_visualization_mode(FluidVisualizationMode::Vorticity);
    
    // Attach the component
    world.insert(entity, renderer).unwrap();
}
```

## Performance Considerations

### Grid Resolution

The computational cost scales with grid size:
- 128x128: Suitable for real-time on most hardware
- 256x256: Good balance of quality and performance
- 512x512: High quality, requires good GPU
- 1024x1024: Very high quality, requires high-end GPU

### Viscosity

Lower viscosity values (< 0.001) create more turbulent flows but may require smaller time steps for stability.

### Boundary Methods

- **Periodic**: Fastest, no additional computation
- **Penalization**: Moderate cost, adds penalty term
- **ImmersedBoundary**: Highest cost, most accurate for complex boundaries

## Integration with Render Graph

The fluid textures are exposed as standard texture handles and can be used in custom render passes:

```rust
// Access the velocity texture for custom rendering
if let Some(velocity_handle) = &renderer.velocity_texture {
    // Use velocity_handle in your render pass
}
```

## Future Enhancements

Planned improvements for the fluid rendering system:

1. **GPU Texture Readback**: Currently uses placeholder textures; will be updated to read actual solver state from GPU
2. **Interactive Forces**: Add ability to apply forces to the fluid (e.g., mouse interaction)
3. **Particle Advection**: Render particles advected by the fluid flow
4. **Dye Injection**: Add colored dye that flows with the fluid
5. **3D Fluid Solver**: Extend to 3D simulations
6. **Adaptive Resolution**: Dynamically adjust grid resolution based on flow complexity

## References

- Spectral Fluid Solver: `crates/luminara_math/src/dynamics/spectral_fluid.rs`
- FFT Implementation: `crates/luminara_math/src/dynamics/fft.rs`
- Fluid Systems: `crates/luminara_render/src/fluid_systems.rs`
