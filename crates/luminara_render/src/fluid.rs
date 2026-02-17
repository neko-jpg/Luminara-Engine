//! Fluid rendering integration for spectral fluid solver.
//!
//! This module connects the spectral fluid solver from luminara_math
//! to the rendering pipeline, exposing velocity and pressure fields
//! as textures for visualization.

use luminara_asset::Handle;
use luminara_core::Component;
use luminara_math::dynamics::{SpectralFluidSolver2D, BoundaryMethod};
use serde::{Deserialize, Serialize};

use crate::Texture;

/// Fluid renderer component that visualizes fluid simulation.
///
/// This component manages a spectral fluid solver and exposes its
/// velocity and pressure fields as textures for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidRenderer {
    /// Width of the fluid simulation grid
    pub width: usize,
    /// Height of the fluid simulation grid
    pub height: usize,
    /// Fluid viscosity (higher = more viscous)
    pub viscosity: f32,
    /// Boundary condition method
    pub boundary_method: BoundaryMethod,
    /// Velocity field texture handle (RG format: u, v components)
    pub velocity_texture: Option<Handle<Texture>>,
    /// Pressure field texture handle (R format: scalar pressure)
    pub pressure_texture: Option<Handle<Texture>>,
    /// Visualization mode
    pub visualization_mode: FluidVisualizationMode,
    /// Color scale for visualization
    pub color_scale: f32,
}

/// Visualization modes for fluid rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FluidVisualizationMode {
    /// Visualize velocity magnitude as color
    VelocityMagnitude,
    /// Visualize velocity direction as color (HSV wheel)
    VelocityDirection,
    /// Visualize vorticity (curl of velocity)
    Vorticity,
    /// Visualize pressure field
    Pressure,
    /// Visualize velocity as streamlines
    Streamlines,
}

impl FluidRenderer {
    /// Create a new fluid renderer with default settings
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            viscosity: 0.001,
            boundary_method: BoundaryMethod::Periodic,
            velocity_texture: None,
            pressure_texture: None,
            visualization_mode: FluidVisualizationMode::VelocityMagnitude,
            color_scale: 1.0,
        }
    }

    /// Create a new fluid renderer with custom viscosity
    pub fn with_viscosity(width: usize, height: usize, viscosity: f32) -> Self {
        Self {
            viscosity,
            ..Self::new(width, height)
        }
    }

    /// Set the boundary method
    pub fn with_boundary_method(mut self, method: BoundaryMethod) -> Self {
        self.boundary_method = method;
        self
    }

    /// Set the visualization mode
    pub fn with_visualization_mode(mut self, mode: FluidVisualizationMode) -> Self {
        self.visualization_mode = mode;
        self
    }
}

impl Component for FluidRenderer {
    fn type_name() -> &'static str {
        "FluidRenderer"
    }
}

/// Resource that manages fluid solver instances for all FluidRenderer components.
///
/// This resource maintains the actual solver state and updates it each frame.
pub struct FluidSolverResource {
    /// Map from entity ID to solver instance
    solvers: std::collections::HashMap<u32, SpectralFluidSolver2D>,
}

impl FluidSolverResource {
    /// Create a new fluid solver resource
    pub fn new() -> Self {
        Self {
            solvers: std::collections::HashMap::new(),
        }
    }

    /// Get or create a solver for an entity
    pub fn get_or_create_solver(
        &mut self,
        entity_id: u32,
        width: usize,
        height: usize,
        viscosity: f32,
    ) -> &mut SpectralFluidSolver2D {
        self.solvers
            .entry(entity_id)
            .or_insert_with(|| SpectralFluidSolver2D::new(width, height, viscosity))
    }

    /// Remove a solver for an entity
    pub fn remove_solver(&mut self, entity_id: u32) {
        self.solvers.remove(&entity_id);
    }

    /// Get a solver for an entity
    pub fn get_solver(&self, entity_id: u32) -> Option<&SpectralFluidSolver2D> {
        self.solvers.get(&entity_id)
    }

    /// Get a mutable solver for an entity
    pub fn get_solver_mut(&mut self, entity_id: u32) -> Option<&mut SpectralFluidSolver2D> {
        self.solvers.get_mut(&entity_id)
    }

    /// Get an iterator over all entity IDs with solvers
    pub fn entity_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.solvers.keys().copied()
    }
}

impl Default for FluidSolverResource {
    fn default() -> Self {
        Self::new()
    }
}

impl luminara_core::Resource for FluidSolverResource {}
