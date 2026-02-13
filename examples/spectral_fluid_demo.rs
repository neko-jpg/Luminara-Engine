use luminara_math::dynamics::SpectralFluidSolver2D;

fn main() {
    println!("Luminara Math - Spectral Fluid Demo");
    println!("===================================\n");

    let width = 64;
    let height = 64;
    let mut solver = SpectralFluidSolver2D::new(width, height, 0.001);

    println!("Initialized solver {}x{}, viscosity {}", width, height, solver.viscosity);

    // Simulation loop (mock)
    let dt = 0.01;
    for i in 0..5 {
        println!("Step {}: simulating...", i);
        solver.step(dt);
    }

    println!("Simulation complete.");
}
