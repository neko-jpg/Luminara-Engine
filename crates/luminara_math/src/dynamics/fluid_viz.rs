use crate::dynamics::spectral_fluid::SpectralFluidSolver2D;
use luminara_core::component::Component;

pub struct FluidVisualization {
    pub solver: SpectralFluidSolver2D,
    pub width: usize,
    pub height: usize,
}

impl Component for FluidVisualization {
    fn type_name() -> &'static str {
        "FluidVisualization"
    }
}

impl FluidVisualization {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            solver: SpectralFluidSolver2D::new(width, height, 0.001),
            width,
            height,
        }
    }

    pub fn step(&mut self, _dt: f32) {
        // In a real implementation, this would step the simulation
        // self.solver.step(dt);
    }
}
