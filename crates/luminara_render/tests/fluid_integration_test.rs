//! Integration tests for fluid rendering system.

use luminara_core::{App, CoreStage, World};
use luminara_render::{FluidRenderer, FluidSolverResource, FluidVisualizationMode};
use luminara_math::dynamics::BoundaryMethod;

#[test]
fn test_fluid_renderer_component_creation() {
    // Test that we can create a FluidRenderer component
    let renderer = FluidRenderer::new(128, 128);
    
    assert_eq!(renderer.width, 128);
    assert_eq!(renderer.height, 128);
    assert_eq!(renderer.viscosity, 0.001);
    assert_eq!(renderer.boundary_method, BoundaryMethod::Periodic);
    assert_eq!(renderer.visualization_mode, FluidVisualizationMode::VelocityMagnitude);
    assert!(renderer.velocity_texture.is_none());
    assert!(renderer.pressure_texture.is_none());
}

#[test]
fn test_fluid_renderer_with_custom_settings() {
    // Test creating a FluidRenderer with custom settings
    let renderer = FluidRenderer::with_viscosity(256, 256, 0.01)
        .with_boundary_method(BoundaryMethod::Penalization)
        .with_visualization_mode(FluidVisualizationMode::Vorticity);
    
    assert_eq!(renderer.width, 256);
    assert_eq!(renderer.height, 256);
    assert_eq!(renderer.viscosity, 0.01);
    assert_eq!(renderer.boundary_method, BoundaryMethod::Penalization);
    assert_eq!(renderer.visualization_mode, FluidVisualizationMode::Vorticity);
}

#[test]
fn test_fluid_solver_resource() {
    // Test that we can create and manage fluid solvers
    let mut resource = FluidSolverResource::new();
    
    // Create a solver for entity 1
    let solver = resource.get_or_create_solver(1, 64, 64, 0.001);
    assert_eq!(solver.width, 64);
    assert_eq!(solver.height, 64);
    assert_eq!(solver.viscosity, 0.001);
    
    // Verify we can retrieve it
    assert!(resource.get_solver(1).is_some());
    assert!(resource.get_solver(2).is_none());
    
    // Remove the solver
    resource.remove_solver(1);
    assert!(resource.get_solver(1).is_none());
}

#[test]
fn test_fluid_solver_resource_multiple_entities() {
    // Test managing multiple fluid solvers
    let mut resource = FluidSolverResource::new();
    
    // Create solvers for multiple entities
    resource.get_or_create_solver(1, 64, 64, 0.001);
    resource.get_or_create_solver(2, 128, 128, 0.002);
    resource.get_or_create_solver(3, 256, 256, 0.003);
    
    // Verify all exist
    assert!(resource.get_solver(1).is_some());
    assert!(resource.get_solver(2).is_some());
    assert!(resource.get_solver(3).is_some());
    
    // Verify entity IDs
    let ids: Vec<u32> = resource.entity_ids().collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

#[test]
fn test_visualization_modes() {
    // Test all visualization modes
    let modes = vec![
        FluidVisualizationMode::VelocityMagnitude,
        FluidVisualizationMode::VelocityDirection,
        FluidVisualizationMode::Vorticity,
        FluidVisualizationMode::Pressure,
        FluidVisualizationMode::Streamlines,
    ];
    
    for mode in modes {
        let renderer = FluidRenderer::new(128, 128).with_visualization_mode(mode);
        assert_eq!(renderer.visualization_mode, mode);
    }
}

#[test]
fn test_boundary_methods() {
    // Test all boundary methods
    let methods = vec![
        BoundaryMethod::Periodic,
        BoundaryMethod::Penalization,
        BoundaryMethod::ImmersedBoundary,
    ];
    
    for method in methods {
        let renderer = FluidRenderer::new(128, 128).with_boundary_method(method);
        assert_eq!(renderer.boundary_method, method);
    }
}
