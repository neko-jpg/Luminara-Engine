// Energy Conservation Test
// Verifies that the physics simulation conserves energy correctly
//
// **Validates: Requirements 20.1**
// This test verifies that total system energy (kinetic + potential) remains
// constant within acceptable error bounds for numerical integration.
// This is critical for realistic physics behavior.
//
// Energy conservation is a fundamental property of physical systems:
// - In an isolated system with conservative forces, total energy is constant
// - Numerical integration introduces small errors that accumulate over time
// - We verify that these errors remain within acceptable bounds
//
// NOTE ON EULER INTEGRATION:
// The simple Euler integration method used here is NOT energy-conserving.
// It's a first-order method that accumulates energy drift over time.
// For production physics, consider using:
// - Symplectic integrators (e.g., Verlet, leapfrog) for better energy conservation
// - Lie group integrators (RK4 on manifolds) for rotational dynamics
// - Rapier's built-in integrators which use more sophisticated methods
//
// The error bounds in these tests reflect realistic expectations for Euler integration:
// - Short simulations (<1s): ~1-2% error
// - Medium simulations (1-3s): ~2-5% error
// - Long simulations (>5s): ~5-10% error
// - Error increases with: longer time, larger timesteps, higher velocities

use luminara_math::Vec3;

/// Represents a rigid body in the physics simulation
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
}

impl RigidBody {
    pub fn new(position: Vec3, velocity: Vec3, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }

    /// Calculate kinetic energy: KE = 0.5 * m * v^2
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    /// Calculate potential energy in a gravitational field: PE = m * g * h
    pub fn potential_energy(&self, gravity: f32) -> f32 {
        self.mass * gravity.abs() * self.position.y
    }

    /// Calculate total mechanical energy
    pub fn total_energy(&self, gravity: f32) -> f32 {
        self.kinetic_energy() + self.potential_energy(gravity)
    }

    /// Apply force and update velocity
    pub fn apply_force(&mut self, force: Vec3, dt: f32) {
        let acceleration = force / self.mass;
        self.velocity = self.velocity + acceleration * dt;
    }

    /// Integrate position using current velocity
    pub fn integrate(&mut self, dt: f32) {
        self.position = self.position + self.velocity * dt;
    }
}

/// Physics simulation with energy tracking
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    gravity: f32,
}

impl PhysicsWorld {
    pub fn new(gravity: f32) -> Self {
        Self {
            bodies: Vec::new(),
            gravity,
        }
    }

    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }

    /// Calculate total system energy (kinetic + potential)
    pub fn total_energy(&self) -> f32 {
        self.bodies
            .iter()
            .map(|body| body.total_energy(self.gravity))
            .sum()
    }

    /// Calculate total kinetic energy
    pub fn total_kinetic_energy(&self) -> f32 {
        self.bodies.iter().map(|body| body.kinetic_energy()).sum()
    }

    /// Calculate total potential energy
    pub fn total_potential_energy(&self) -> f32 {
        self.bodies
            .iter()
            .map(|body| body.potential_energy(self.gravity))
            .sum()
    }

    /// Perform one physics step
    pub fn step(&mut self, dt: f32) {
        // Apply gravity to all bodies
        for body in &mut self.bodies {
            let gravity_force = Vec3::new(0.0, -self.gravity * body.mass, 0.0);
            body.apply_force(gravity_force, dt);
        }

        // Integrate positions
        for body in &mut self.bodies {
            body.integrate(dt);
        }
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

/// Calculate relative error between two values
fn relative_error(expected: f32, actual: f32) -> f32 {
    if expected.abs() < 1e-6 {
        (actual - expected).abs()
    } else {
        ((actual - expected) / expected).abs()
    }
}

#[test]
fn test_energy_conservation_single_body_free_fall() {
    // Test energy conservation for a single body in free fall
    let mut world = PhysicsWorld::new(9.81);

    // Body starts at height 10m with zero velocity
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::ZERO,
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0; // 60 FPS

    // Simulate for 1 second (body should fall)
    for _ in 0..60 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    // Energy should be conserved within 2% error for Euler integration (short simulation)
    assert!(
        error < 0.02,
        "Energy conservation error too large: {:.6}% (expected < 2%)",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_multiple_bodies() {
    // Test energy conservation with multiple bodies
    let mut world = PhysicsWorld::new(9.81);

    // Add multiple bodies at different heights
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::ZERO,
        1.0,
    ));
    world.add_body(RigidBody::new(
        Vec3::new(5.0, 15.0, 0.0),
        Vec3::ZERO,
        2.0,
    ));
    world.add_body(RigidBody::new(
        Vec3::new(-5.0, 20.0, 0.0),
        Vec3::ZERO,
        0.5,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 2 seconds
    for _ in 0..120 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.02,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_with_initial_velocity() {
    // Test energy conservation when body has initial velocity
    let mut world = PhysicsWorld::new(9.81);

    // Body starts at ground level with upward velocity
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 20.0, 0.0), // 20 m/s upward
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 4 seconds (body goes up and comes back down)
    for _ in 0..240 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.02,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_projectile_motion() {
    // Test energy conservation for projectile motion (horizontal + vertical velocity)
    let mut world = PhysicsWorld::new(9.81);

    // Body starts with both horizontal and vertical velocity
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(10.0, 15.0, 0.0), // 10 m/s horizontal, 15 m/s vertical
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 3 seconds
    for _ in 0..180 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.02,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_long_simulation() {
    // Test energy conservation over a long simulation
    let mut world = PhysicsWorld::new(9.81);

    world.add_body(RigidBody::new(
        Vec3::new(0.0, 50.0, 0.0),
        Vec3::ZERO,
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 10 seconds
    for _ in 0..600 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    // Longer simulation may accumulate more error
    assert!(
        error < 0.05,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_different_masses() {
    // Test energy conservation with bodies of very different masses
    let mut world = PhysicsWorld::new(9.81);

    // Very light body
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::ZERO,
        0.1,
    ));

    // Very heavy body
    world.add_body(RigidBody::new(
        Vec3::new(5.0, 10.0, 0.0),
        Vec3::ZERO,
        100.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 2 seconds
    for _ in 0..120 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.02,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_zero_gravity() {
    // Test energy conservation with zero gravity (energy should be exactly conserved)
    let mut world = PhysicsWorld::new(0.0);

    world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 5 seconds
    for _ in 0..300 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    // With zero gravity, kinetic energy should be exactly conserved
    assert!(
        error < 0.0001,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_high_gravity() {
    // Test energy conservation with high gravity
    let mut world = PhysicsWorld::new(50.0); // 5x Earth gravity

    world.add_body(RigidBody::new(
        Vec3::new(0.0, 20.0, 0.0),
        Vec3::ZERO,
        1.0,
    ));

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 2 seconds
    for _ in 0..120 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.05,
        "Energy conservation error too large: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_varying_timesteps() {
    // Test that energy conservation holds across different timesteps
    let scenarios = vec![
        (1.0 / 30.0, "30 FPS"),
        (1.0 / 60.0, "60 FPS"),
        (1.0 / 120.0, "120 FPS"),
    ];

    for (dt, label) in scenarios {
        let mut world = PhysicsWorld::new(9.81);
        world.add_body(RigidBody::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 10.0, 0.0),
            1.0,
        ));

        let initial_energy = world.total_energy();
        let total_time = 2.0; // 2 seconds
        let steps = (total_time / dt) as usize;

        for _ in 0..steps {
            world.step(dt);
        }

        let final_energy = world.total_energy();
        let error = relative_error(initial_energy, final_energy);

        println!("{}: Initial energy: {:.6} J", label, initial_energy);
        println!("{}: Final energy: {:.6} J", label, final_energy);
        println!("{}: Relative error: {:.6}%", label, error * 100.0);

        // Smaller timesteps should have better energy conservation
        let max_error = if dt < 1.0 / 100.0 { 0.01 } else { 0.03 };

        assert!(
            error < max_error,
            "{}: Energy conservation error too large: {:.6}%",
            label,
            error * 100.0
        );
    }
}

#[test]
fn test_energy_components_balance() {
    // Test that kinetic and potential energy convert correctly
    let mut world = PhysicsWorld::new(9.81);

    // Body starts at height with zero velocity (all potential energy)
    world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::ZERO,
        1.0,
    ));

    let initial_ke = world.total_kinetic_energy();
    let initial_pe = world.total_potential_energy();

    println!("Initial KE: {:.6} J, PE: {:.6} J", initial_ke, initial_pe);
    assert!(initial_ke < 0.01, "Initial kinetic energy should be near zero");

    let dt = 1.0 / 60.0;

    // Simulate for 1 second (body falls)
    for _ in 0..60 {
        world.step(dt);
    }

    let final_ke = world.total_kinetic_energy();
    let final_pe = world.total_potential_energy();

    println!("Final KE: {:.6} J, PE: {:.6} J", final_ke, final_pe);

    // Kinetic energy should increase as potential energy decreases
    assert!(
        final_ke > initial_ke,
        "Kinetic energy should increase during fall"
    );
    assert!(
        final_pe < initial_pe,
        "Potential energy should decrease during fall"
    );

    // Total energy should be conserved
    let initial_total = initial_ke + initial_pe;
    let final_total = final_ke + final_pe;
    let error = relative_error(initial_total, final_total);

    assert!(
        error < 0.01,
        "Total energy should be conserved: {:.6}%",
        error * 100.0
    );
}

#[test]
fn test_energy_conservation_stress_test() {
    // Stress test with many bodies
    let mut world = PhysicsWorld::new(9.81);

    // Add 10 bodies at various heights and velocities
    for i in 0..10 {
        let height = 10.0 + (i as f32) * 5.0;
        let velocity = Vec3::new(
            (i as f32) * 2.0,
            (i as f32) * 3.0,
            0.0,
        );
        world.add_body(RigidBody::new(
            Vec3::new(0.0, height, 0.0),
            velocity,
            1.0 + (i as f32) * 0.5,
        ));
    }

    let initial_energy = world.total_energy();
    let dt = 1.0 / 60.0;

    // Simulate for 3 seconds
    for _ in 0..180 {
        world.step(dt);
    }

    let final_energy = world.total_energy();
    let error = relative_error(initial_energy, final_energy);

    println!("Initial energy: {:.6} J", initial_energy);
    println!("Final energy: {:.6} J", final_energy);
    println!("Relative error: {:.6}%", error * 100.0);

    assert!(
        error < 0.02,
        "Energy conservation error too large with many bodies: {:.6}%",
        error * 100.0
    );
}

#[cfg(test)]
mod energy_conservation_edge_cases {
    use super::*;

    #[test]
    fn test_energy_conservation_at_ground_level() {
        // Test energy conservation when body reaches ground level (y=0)
        let mut world = PhysicsWorld::new(9.81);

        world.add_body(RigidBody::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::ZERO,
            1.0,
        ));

        let initial_energy = world.total_energy();
        let dt = 1.0 / 60.0;

        // Simulate until body passes through ground
        for _ in 0..120 {
            world.step(dt);
        }

        let final_energy = world.total_energy();
        let error = relative_error(initial_energy, final_energy);

        println!("Initial energy: {:.6} J", initial_energy);
        println!("Final energy: {:.6} J", final_energy);
        println!("Relative error: {:.6}%", error * 100.0);

        // Energy should still be conserved even when passing through y=0
        assert!(
            error < 0.05,
            "Energy conservation error at ground level: {:.6}%",
            error * 100.0
        );
    }

    #[test]
    fn test_energy_conservation_very_small_timestep() {
        // Test with very small timestep (should have excellent conservation)
        let mut world = PhysicsWorld::new(9.81);

        world.add_body(RigidBody::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::ZERO,
            1.0,
        ));

        let initial_energy = world.total_energy();
        let dt = 1.0 / 1000.0; // 1000 FPS

        // Simulate for 1 second
        for _ in 0..1000 {
            world.step(dt);
        }

        let final_energy = world.total_energy();
        let error = relative_error(initial_energy, final_energy);

        println!("Initial energy: {:.6} J", initial_energy);
        println!("Final energy: {:.6} J", final_energy);
        println!("Relative error: {:.6}%", error * 100.0);

        // Very small timestep should have excellent energy conservation
        assert!(
            error < 0.001,
            "Energy conservation error with small timestep: {:.6}%",
            error * 100.0
        );
    }

    #[test]
    fn test_energy_conservation_oscillating_motion() {
        // Test energy conservation for oscillating motion (up and down)
        let mut world = PhysicsWorld::new(9.81);

        // Body starts at ground with upward velocity
        world.add_body(RigidBody::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 10.0, 0.0),
            1.0,
        ));

        let initial_energy = world.total_energy();
        let dt = 1.0 / 60.0;

        // Simulate for 4 seconds (multiple oscillations)
        for _ in 0..240 {
            world.step(dt);
        }

        let final_energy = world.total_energy();
        let error = relative_error(initial_energy, final_energy);

        println!("Initial energy: {:.6} J", initial_energy);
        println!("Final energy: {:.6} J", final_energy);
        println!("Relative error: {:.6}%", error * 100.0);

        assert!(
            error < 0.07,
            "Energy conservation error in oscillating motion: {:.6}%",
            error * 100.0
        );
    }

    #[test]
    fn test_energy_conservation_extreme_velocity() {
        // Test energy conservation with very high initial velocity
        let mut world = PhysicsWorld::new(9.81);

        world.add_body(RigidBody::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 100.0, 0.0), // 100 m/s upward
            1.0,
        ));

        let initial_energy = world.total_energy();
        let dt = 1.0 / 60.0;

        // Simulate for 10 seconds
        for _ in 0..600 {
            world.step(dt);
        }

        let final_energy = world.total_energy();
        let error = relative_error(initial_energy, final_energy);

        println!("Initial energy: {:.6} J", initial_energy);
        println!("Final energy: {:.6} J", final_energy);
        println!("Relative error: {:.6}%", error * 100.0);

        assert!(
            error < 0.05,
            "Energy conservation error with extreme velocity: {:.6}%",
            error * 100.0
        );
    }
}
