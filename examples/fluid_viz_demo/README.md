# Fluid Visualization Demo

This demo showcases the spectral fluid solver integration with the Luminara rendering pipeline.

## Features

- **Real-time fluid simulation** using spectral methods (FFT-based)
- **Multiple visualization modes**:
  - Velocity Magnitude: Shows the speed of fluid flow
  - Velocity Direction: Shows flow direction using color
  - Vorticity: Shows rotational motion in the fluid
  - Pressure: Shows pressure distribution
  - Streamlines: Shows flow paths
- **Interactive controls** for adjusting simulation parameters
- **GPU-accelerated** fluid dynamics

## Requirements Validated

- **Requirement 13.3**: "WHEN rendering fluids, THE System SHALL integrate the spectral fluid solver with the rendering pipeline for visualization"

## Controls

| Key | Action |
|-----|--------|
| **1** | Switch to Velocity Magnitude visualization |
| **2** | Switch to Velocity Direction visualization |
| **3** | Switch to Vorticity visualization |
| **4** | Switch to Pressure visualization |
| **5** | Switch to Streamlines visualization |
| **+** | Increase viscosity |
| **-** | Decrease viscosity |
| **Space** | Pause/Resume simulation |
| **R** | Reset simulation |

## Running the Demo

From the workspace root:

```bash
cargo run --package fluid_viz_demo
```

Or from WSL:

```bash
wsl cargo run --package fluid_viz_demo
```

## Technical Details

### Spectral Fluid Solver

The demo uses a spectral method solver for the incompressible Navier-Stokes equations:

- **FFT-based**: Uses Fast Fourier Transform for efficient computation
- **Periodic boundaries**: Fluid wraps around at domain edges
- **IMEX time integration**: Implicit-Explicit method for stability
- **GPU-accelerated**: All computations run on the GPU

### Visualization Pipeline

1. **Solver Update**: Fluid simulation steps forward in time
2. **Field Extraction**: Velocity and pressure fields extracted from GPU
3. **Texture Upload**: Fields uploaded as textures
4. **Shader Rendering**: Custom shaders visualize the fields
5. **Display**: Rendered to screen as a textured quad

## Architecture

```
FluidRenderer Component
    ↓
FluidSolverResource
    ↓
SpectralFluidSolver2D
    ↓
GPU Compute Shaders
    ↓
Texture Output
    ↓
Rendering Pipeline
```

## Future Enhancements

- [ ] Interactive fluid manipulation (mouse/touch input)
- [ ] Obstacle placement and editing
- [ ] Multiple fluid layers
- [ ] Particle advection for enhanced visualization
- [ ] Export simulation data
- [ ] Performance profiling overlay
