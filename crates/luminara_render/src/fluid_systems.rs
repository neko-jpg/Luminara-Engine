//! Systems for fluid simulation and rendering.

use luminara_core::shared_types::{Query, Res, ResMut};
use luminara_core::{Entity, Time};
use luminara_asset::AssetServer;

use crate::fluid::{FluidRenderer, FluidSolverResource};
use crate::{Texture, TextureData, TextureFormat};

/// System that initializes fluid solvers for new FluidRenderer components.
///
/// This system creates solver instances for entities that have FluidRenderer
/// components but don't yet have a solver in the resource.
pub fn init_fluid_solvers_system(
    fluid_renderers: Query<(Entity, &FluidRenderer)>,
    mut solver_resource: ResMut<FluidSolverResource>,
) {
    for (entity, renderer) in fluid_renderers.iter() {
        let entity_id = entity.id();
        
        // Check if solver already exists
        if solver_resource.get_solver(entity_id).is_none() {
            // Create new solver
            solver_resource.get_or_create_solver(
                entity_id,
                renderer.width,
                renderer.height,
                renderer.viscosity,
            );
            
            log::info!(
                "Initialized fluid solver for entity {} ({}x{}, viscosity: {})",
                entity_id,
                renderer.width,
                renderer.height,
                renderer.viscosity
            );
        }
    }
}

/// System that steps the fluid simulation forward in time.
///
/// This system updates all fluid solvers based on the frame delta time.
pub fn update_fluid_simulation_system(
    time: Res<Time>,
    fluid_renderers: Query<(Entity, &FluidRenderer)>,
    mut solver_resource: ResMut<FluidSolverResource>,
) {
    let dt = time.delta_seconds();
    
    for (entity, _renderer) in fluid_renderers.iter() {
        let entity_id = entity.id();
        
        if let Some(solver) = solver_resource.get_solver_mut(entity_id) {
            // Step the simulation
            solver.step(dt);
        }
    }
}

/// System that synchronizes fluid solver state to GPU textures.
///
/// This system extracts velocity and pressure fields from the solvers
/// and uploads them as textures for rendering.
pub fn sync_fluid_textures_system(
    mut fluid_renderers: Query<(Entity, &mut FluidRenderer)>,
    solver_resource: Res<FluidSolverResource>,
    asset_server: ResMut<AssetServer>,
) {
    for (entity, mut renderer) in fluid_renderers.iter_mut() {
        let entity_id = entity.id();
        
        if let Some(solver) = solver_resource.get_solver(entity_id) {
            // Extract velocity field from solver
            // Note: In a real implementation, this would read from GPU textures
            // For now, we create placeholder textures
            
            // Create velocity texture if it doesn't exist
            if renderer.velocity_texture.is_none() {
                let velocity_data = create_velocity_texture_data(
                    renderer.width,
                    renderer.height,
                    solver,
                );
                
                let texture = Texture::new(velocity_data);
                let handle = asset_server.add(texture);
                renderer.velocity_texture = Some(handle);
            }
            
            // Create pressure texture if it doesn't exist
            if renderer.pressure_texture.is_none() {
                let pressure_data = create_pressure_texture_data(
                    renderer.width,
                    renderer.height,
                    solver,
                );
                
                let texture = Texture::new(pressure_data);
                let handle = asset_server.add(texture);
                renderer.pressure_texture = Some(handle);
            }
            
            // TODO: Update existing textures with current solver state
            // This would involve reading from GPU textures and updating the handles
        }
    }
}

/// Create velocity texture data from solver state.
///
/// In a real implementation, this would read from the solver's GPU textures.
/// For now, we create a placeholder gradient pattern.
fn create_velocity_texture_data(
    width: usize,
    height: usize,
    _solver: &luminara_math::dynamics::SpectralFluidSolver2D,
) -> TextureData {
    let mut data = Vec::with_capacity(width * height * 2);
    
    // Create a simple gradient pattern as placeholder
    for y in 0..height {
        for x in 0..width {
            let u = (x as f32 / width as f32 * 255.0) as u8;
            let v = (y as f32 / height as f32 * 255.0) as u8;
            data.push(u);
            data.push(v);
        }
    }
    
    TextureData {
        width: width as u32,
        height: height as u32,
        data,
        format: TextureFormat::Rg8,
    }
}

/// Create pressure texture data from solver state.
///
/// In a real implementation, this would read from the solver's GPU textures.
/// For now, we create a placeholder pattern.
fn create_pressure_texture_data(
    width: usize,
    height: usize,
    _solver: &luminara_math::dynamics::SpectralFluidSolver2D,
) -> TextureData {
    let mut data = Vec::with_capacity(width * height);
    
    // Create a simple radial pattern as placeholder
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_dist = (center_x * center_x + center_y * center_y).sqrt();
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let value = ((1.0 - dist / max_dist) * 255.0) as u8;
            data.push(value);
        }
    }
    
    TextureData {
        width: width as u32,
        height: height as u32,
        data,
        format: TextureFormat::R8,
    }
}

/// System that cleans up fluid solvers when FluidRenderer components are removed.
pub fn cleanup_fluid_solvers_system(
    fluid_renderers: Query<(Entity, &FluidRenderer)>,
    mut solver_resource: ResMut<FluidSolverResource>,
) {
    // Collect all active entity IDs
    let active_entities: std::collections::HashSet<u32> = fluid_renderers
        .iter()
        .map(|(entity, _)| entity.id())
        .collect();
    
    // Remove solvers for entities that no longer have FluidRenderer components
    let to_remove: Vec<u32> = solver_resource
        .entity_ids()
        .filter(|&entity_id| !active_entities.contains(&entity_id))
        .collect();
    
    for entity_id in to_remove {
        solver_resource.remove_solver(entity_id);
        log::info!("Cleaned up fluid solver for entity {}", entity_id);
    }
}
