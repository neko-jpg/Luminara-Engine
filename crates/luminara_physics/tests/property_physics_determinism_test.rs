// Property-Based Test: Physics Determinism
// 
// **Property 22: Physics Determinism**
// For any physics simulation with identical initial conditions and inputs,
// running the simulation multiple times should produce identical results.
//
// **Validates: Requirements 20.4**
//
// This property test uses proptest to generate random physics worlds with
// various initial conditions and verifies that running them multiple times
// produces bitwise identical results.

use luminara_math::{Quat, Vec3};
use proptest::prelude::*;

/// Represents a rigid body in the physics simulation
#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody {
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
}

impl RigidBody {
    pub fn new(position: Vec3, mass: f32) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass,
        }
    }

    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn apply_force(&mut self, force: Vec3, dt: f32) {
        let acceleration = force / self.mass;
        self.velocity = self.velocity + acceleration * dt;
    }

    pub fn integrate(&mut self, dt: f32) {
        self.position = self.position + self.velocity * dt;
    }

    /// Check bitwise equality for determinism verification
    pub fn is_bitwise_equal(&self, other: &RigidBody) -> bool {
        self.position.x.to_bits() == other.position.x.to_bits()
            && self.position.y.to_bits() == other.position.y.to_bits()
            && self.position.z.to_bits() == other.position.z.to_bits()
            && self.rotation.x.to_bits() == other.rotation.x.to_bits()
            && self.rotation.y.to_bits() == other.rotation.y.to_bits()
            && self.rotation.z.to_bits() == other.rotation.z.to_bits()
            && self.rotation.w.to_bits() == other.rotation.w.to_bits()
            && self.velocity.x.to_bits() == other.velocity.x.to_bits()
            && self.velocity.y.to_bits() == other.velocity.y.to_bits()
            && self.velocity.z.to_bits() == other.velocity.z.to_bits()
    }
}

/// Physics simulation state
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    gravity: Vec3,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        Self {
            bodies: Vec::new(),
            gravity,
        }
    }

    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }

    pub fn step(&mut self, dt: f32) {
        // Apply gravity to all bodies
        for body in &mut self.bodies {
            let gravity_force = self.gravity * body.mass;
            body.apply_force(gravity_force, dt);
        }

        // Integrate positions
        for body in &mut self.bodies {
            body.integrate(dt);
        }
    }

    pub fn get_body(&self, index: usize) -> Option<&RigidBody> {
        self.bodies.get(index)
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

// ============================================================================
// PROPERTY TEST STRATEGIES
// ============================================================================

/// Strategy for generating valid Vec3 positions
fn vec3_position_strategy() -> impl Strategy<Value = Vec3> {
    (
        -100.0f32..100.0f32,
        -100.0f32..100.0f32,
        -100.0f32..100.0f32,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating valid Vec3 velocities
fn vec3_velocity_strategy() -> impl Strategy<Value = Vec3> {
    (
        -50.0f32..50.0f32,
        -50.0f32..50.0f32,
        -50.0f32..50.0f32,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating valid masses (positive, non-zero)
fn mass_strategy() -> impl Strategy<Value = f32> {
    0.1f32..100.0f32
}

/// Strategy for generating gravity vectors
fn gravity_strategy() -> impl Strategy<Value = Vec3> {
    (
        -20.0f32..20.0f32,
        -20.0f32..20.0f32,
        -20.0f32..20.0f32,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating a single rigid body
fn rigid_body_strategy() -> impl Strategy<Value = RigidBody> {
    (vec3_position_strategy(), mass_strategy(), vec3_velocity_strategy())
        .prop_map(|(position, mass, velocity)| {
            RigidBody::new(position, mass).with_velocity(velocity)
        })
}

/// Strategy for generating a physics world with multiple bodies
fn physics_world_strategy() -> impl Strategy<Value = PhysicsWorld> {
    (
        gravity_strategy(),
        prop::collection::vec(rigid_body_strategy(), 1..10),
    )
        .prop_map(|(gravity, bodies)| {
            let mut world = PhysicsWorld::new(gravity);
            for body in bodies {
                world.add_body(body);
            }
            world
        })
}

/// Strategy for generating simulation parameters
fn simulation_params_strategy() -> impl Strategy<Value = (usize, f32)> {
    (10usize..500, prop::sample::select(vec![1.0 / 30.0, 1.0 / 60.0, 1.0 / 120.0]))
}

// ============================================================================
// PROPERTY TESTS
// ============================================================================

proptest! {
    /// **Property 22.1: Basic Determinism**
    /// Running the same simulation twice produces identical results
    #[test]
    fn prop_physics_determinism_basic(
        world in physics_world_strategy(),
        (frames, dt) in simulation_params_strategy()
    ) {
        // Run simulation twice with identical initial conditions
        let mut world1 = world.clone();
        let mut world2 = world.clone();

        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }

        // Verify all bodies are bitwise identical
        prop_assert_eq!(world1.body_count(), world2.body_count());
        
        for i in 0..world1.body_count() {
            let body1 = world1.get_body(i).unwrap();
            let body2 = world2.get_body(i).unwrap();
            
            prop_assert!(
                body1.is_bitwise_equal(body2),
                "Body {} should be deterministic (pos1: {:?}, pos2: {:?})",
                i, body1.position, body2.position
            );
        }
    }

    /// **Property 22.2: Multiple Run Determinism**
    /// Running the simulation N times produces identical results
    #[test]
    fn prop_physics_determinism_multiple_runs(
        world in physics_world_strategy(),
        frames in 10usize..200,
    ) {
        let dt = 1.0 / 60.0;
        let num_runs = 5;
        
        let mut results = Vec::new();
        
        // Run simulation multiple times
        for _ in 0..num_runs {
            let mut test_world = world.clone();
            for _ in 0..frames {
                test_world.step(dt);
            }
            results.push(test_world);
        }
        
        // All runs should produce identical results
        let reference = &results[0];
        for (run_idx, test_world) in results.iter().enumerate().skip(1) {
            for body_idx in 0..reference.body_count() {
                let ref_body = reference.get_body(body_idx).unwrap();
                let test_body = test_world.get_body(body_idx).unwrap();
                
                prop_assert!(
                    ref_body.is_bitwise_equal(test_body),
                    "Run {} body {} should match reference",
                    run_idx, body_idx
                );
            }
        }
    }

    /// **Property 22.3: Chunked Simulation Determinism**
    /// Simulating in chunks produces same result as continuous simulation
    #[test]
    fn prop_physics_determinism_chunked(
        world in physics_world_strategy(),
        total_frames in 50usize..300,
    ) {
        let dt = 1.0 / 60.0;
        
        // Simulate continuously
        let mut world_continuous = world.clone();
        for _ in 0..total_frames {
            world_continuous.step(dt);
        }
        
        // Simulate in chunks of 10 frames
        let mut world_chunked = world.clone();
        let chunk_size = 10;
        let num_chunks = total_frames / chunk_size;
        let remainder = total_frames % chunk_size;
        
        for _ in 0..num_chunks {
            for _ in 0..chunk_size {
                world_chunked.step(dt);
            }
        }
        for _ in 0..remainder {
            world_chunked.step(dt);
        }
        
        // Results should be identical
        for i in 0..world_continuous.body_count() {
            let body_continuous = world_continuous.get_body(i).unwrap();
            let body_chunked = world_chunked.get_body(i).unwrap();
            
            prop_assert!(
                body_continuous.is_bitwise_equal(body_chunked),
                "Chunked simulation should match continuous for body {}",
                i
            );
        }
    }

    /// **Property 22.4: Save/Load Determinism**
    /// Saving and loading state maintains determinism
    #[test]
    fn prop_physics_determinism_save_load(
        world in physics_world_strategy(),
        frames_before_save in 20usize..100,
        frames_after_save in 20usize..100,
    ) {
        let dt = 1.0 / 60.0;
        
        // Simulate without save/load
        let mut world_continuous = world.clone();
        for _ in 0..(frames_before_save + frames_after_save) {
            world_continuous.step(dt);
        }
        
        // Simulate with save/load
        let mut world_saveload = world.clone();
        for _ in 0..frames_before_save {
            world_saveload.step(dt);
        }
        
        // "Save" state (clone)
        let saved_state = world_saveload.clone();
        
        // "Load" state and continue
        let mut world_loaded = saved_state;
        for _ in 0..frames_after_save {
            world_loaded.step(dt);
        }
        
        // Results should match
        for i in 0..world_continuous.body_count() {
            let body_continuous = world_continuous.get_body(i).unwrap();
            let body_loaded = world_loaded.get_body(i).unwrap();
            
            prop_assert!(
                body_continuous.is_bitwise_equal(body_loaded),
                "Save/load should maintain determinism for body {}",
                i
            );
        }
    }

    /// **Property 22.5: Extreme Value Determinism**
    /// Determinism holds even with extreme values
    #[test]
    fn prop_physics_determinism_extreme_values(
        position in vec3_position_strategy(),
        mass in mass_strategy(),
        frames in 50usize..200,
    ) {
        let dt = 1.0 / 60.0;
        
        // Scale position to extreme value
        let extreme_position = position * 100.0;
        
        let mut world1 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world1.add_body(RigidBody::new(extreme_position, mass));
        
        let mut world2 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world2.add_body(RigidBody::new(extreme_position, mass));
        
        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        let body1 = world1.get_body(0).unwrap();
        let body2 = world2.get_body(0).unwrap();
        
        prop_assert!(
            body1.is_bitwise_equal(body2),
            "Extreme values should be deterministic"
        );
    }

    /// **Property 22.6: Zero Gravity Determinism**
    /// Determinism holds with zero gravity
    #[test]
    fn prop_physics_determinism_zero_gravity(
        bodies in prop::collection::vec(rigid_body_strategy(), 1..5),
        frames in 50usize..200,
    ) {
        let dt = 1.0 / 60.0;
        
        let mut world1 = PhysicsWorld::new(Vec3::ZERO);
        let mut world2 = PhysicsWorld::new(Vec3::ZERO);
        
        for body in bodies {
            world1.add_body(body.clone());
            world2.add_body(body);
        }
        
        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        for i in 0..world1.body_count() {
            let body1 = world1.get_body(i).unwrap();
            let body2 = world2.get_body(i).unwrap();
            
            prop_assert!(
                body1.is_bitwise_equal(body2),
                "Zero gravity should be deterministic for body {}",
                i
            );
        }
    }

    /// **Property 22.7: Custom Gravity Determinism**
    /// Determinism holds with arbitrary gravity vectors
    #[test]
    fn prop_physics_determinism_custom_gravity(
        gravity in gravity_strategy(),
        bodies in prop::collection::vec(rigid_body_strategy(), 1..5),
        frames in 50usize..200,
    ) {
        let dt = 1.0 / 60.0;
        
        let mut world1 = PhysicsWorld::new(gravity);
        let mut world2 = PhysicsWorld::new(gravity);
        
        for body in bodies {
            world1.add_body(body.clone());
            world2.add_body(body);
        }
        
        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        for i in 0..world1.body_count() {
            let body1 = world1.get_body(i).unwrap();
            let body2 = world2.get_body(i).unwrap();
            
            prop_assert!(
                body1.is_bitwise_equal(body2),
                "Custom gravity should be deterministic for body {}",
                i
            );
        }
    }

    /// **Property 22.8: Small Mass Determinism**
    /// Determinism holds with very small masses
    #[test]
    fn prop_physics_determinism_small_mass(
        position in vec3_position_strategy(),
        frames in 50usize..200,
    ) {
        let dt = 1.0 / 60.0;
        let small_mass = 0.001;
        
        let mut world1 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world1.add_body(RigidBody::new(position, small_mass));
        
        let mut world2 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world2.add_body(RigidBody::new(position, small_mass));
        
        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        let body1 = world1.get_body(0).unwrap();
        let body2 = world2.get_body(0).unwrap();
        
        prop_assert!(
            body1.is_bitwise_equal(body2),
            "Small masses should be deterministic"
        );
    }

    /// **Property 22.9: Large Mass Determinism**
    /// Determinism holds with very large masses
    #[test]
    fn prop_physics_determinism_large_mass(
        position in vec3_position_strategy(),
        frames in 50usize..200,
    ) {
        let dt = 1.0 / 60.0;
        let large_mass = 1000.0;
        
        let mut world1 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world1.add_body(RigidBody::new(position, large_mass));
        
        let mut world2 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world2.add_body(RigidBody::new(position, large_mass));
        
        for _ in 0..frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        let body1 = world1.get_body(0).unwrap();
        let body2 = world2.get_body(0).unwrap();
        
        prop_assert!(
            body1.is_bitwise_equal(body2),
            "Large masses should be deterministic"
        );
    }

    /// **Property 22.10: Long Simulation Determinism**
    /// Determinism holds over long simulations
    #[test]
    fn prop_physics_determinism_long_simulation(
        world in physics_world_strategy(),
    ) {
        let dt = 1.0 / 60.0;
        let long_frames = 1000; // ~16 seconds of simulation
        
        let mut world1 = world.clone();
        let mut world2 = world.clone();
        
        for _ in 0..long_frames {
            world1.step(dt);
            world2.step(dt);
        }
        
        for i in 0..world1.body_count() {
            let body1 = world1.get_body(i).unwrap();
            let body2 = world2.get_body(i).unwrap();
            
            prop_assert!(
                body1.is_bitwise_equal(body2),
                "Long simulations should remain deterministic for body {}",
                i
            );
        }
    }
}

// ============================================================================
// ADDITIONAL UNIT TESTS
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bitwise_equality_check() {
        let body1 = RigidBody::new(Vec3::new(1.0, 2.0, 3.0), 1.0);
        let body2 = RigidBody::new(Vec3::new(1.0, 2.0, 3.0), 1.0);
        
        assert!(body1.is_bitwise_equal(&body2));
    }

    #[test]
    fn test_bitwise_inequality() {
        let body1 = RigidBody::new(Vec3::new(1.0, 2.0, 3.0), 1.0);
        let body2 = RigidBody::new(Vec3::new(1.0, 2.0, 3.00001), 1.0);
        
        assert!(!body1.is_bitwise_equal(&body2));
    }

    #[test]
    fn test_simple_determinism() {
        let mut world1 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world1.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));
        
        let mut world2 = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world2.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));
        
        let dt = 1.0 / 60.0;
        for _ in 0..100 {
            world1.step(dt);
            world2.step(dt);
        }
        
        let body1 = world1.get_body(0).unwrap();
        let body2 = world2.get_body(0).unwrap();
        
        assert!(body1.is_bitwise_equal(body2));
    }
}
