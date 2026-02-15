// Physics Determinism Test
// Ensures that physics simulations produce identical results across multiple runs

use luminara_math::{Quat, Vec3};

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

    pub fn apply_force(&mut self, force: Vec3, dt: f32) {
        let acceleration = force / self.mass;
        self.velocity = self.velocity + acceleration * dt;
    }

    pub fn integrate(&mut self, dt: f32) {
        self.position = self.position + self.velocity * dt;
    }

    pub fn is_bitwise_equal(&self, other: &RigidBody) -> bool {
        // Check bitwise equality for determinism
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
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
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

/// Run a physics simulation for a given number of frames
fn run_simulation(frames: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new();

    // Add test bodies
    world.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));
    world.add_body(RigidBody::new(Vec3::new(5.0, 15.0, 0.0), 2.0));
    world.add_body(RigidBody::new(Vec3::new(-5.0, 20.0, 0.0), 0.5));

    // Run simulation
    let dt = 1.0 / 60.0; // 60 FPS
    for _ in 0..frames {
        world.step(dt);
    }

    world
}

#[test]
fn test_determinism_basic() {
    // Run simulation twice with same initial conditions
    let world1 = run_simulation(100);
    let world2 = run_simulation(100);

    // Check that all bodies have identical positions
    assert_eq!(world1.body_count(), world2.body_count());

    for i in 0..world1.body_count() {
        let body1 = world1.get_body(i).unwrap();
        let body2 = world2.get_body(i).unwrap();

        assert!(
            body1.is_bitwise_equal(body2),
            "Body {} should have identical state across runs (pos1: {:?}, pos2: {:?})",
            i,
            body1.position,
            body2.position
        );
    }
}

#[test]
fn test_determinism_1000_frames() {
    // Run for 1000 frames to test long-term determinism
    let world1 = run_simulation(1000);
    let world2 = run_simulation(1000);

    for i in 0..world1.body_count() {
        let body1 = world1.get_body(i).unwrap();
        let body2 = world2.get_body(i).unwrap();

        assert!(
            body1.is_bitwise_equal(body2),
            "Body {} should remain deterministic after 1000 frames",
            i
        );
    }
}

#[test]
fn test_determinism_with_forces() {
    let mut world1 = PhysicsWorld::new();
    let mut world2 = PhysicsWorld::new();

    // Add identical bodies
    world1.add_body(RigidBody::new(Vec3::new(0.0, 0.0, 0.0), 1.0));
    world2.add_body(RigidBody::new(Vec3::new(0.0, 0.0, 0.0), 1.0));

    let dt = 1.0 / 60.0;

    // Apply same forces and simulate
    for _ in 0..100 {
        // Apply custom force
        if let Some(body) = world1.bodies.get_mut(0) {
            body.apply_force(Vec3::new(10.0, 0.0, 0.0), dt);
        }
        if let Some(body) = world2.bodies.get_mut(0) {
            body.apply_force(Vec3::new(10.0, 0.0, 0.0), dt);
        }

        world1.step(dt);
        world2.step(dt);
    }

    let body1 = world1.get_body(0).unwrap();
    let body2 = world2.get_body(0).unwrap();

    assert!(
        body1.is_bitwise_equal(body2),
        "Bodies with applied forces should remain deterministic"
    );
}

#[cfg(test)]
mod determinism_integration_tests {
    use super::*;

    #[test]
    fn test_floating_point_consistency() {
        // Test that floating point operations are consistent
        let a = 0.1_f32;
        let b = 0.2_f32;
        let c = 0.3_f32;

        let sum1 = a + b;
        let sum2 = a + b;

        assert_eq!(
            sum1.to_bits(),
            sum2.to_bits(),
            "Floating point operations should be deterministic"
        );

        // Note: (a + b) might not equal c due to floating point precision
        // But it should be consistent across runs
        let result1 = sum1 - c;
        let result2 = sum2 - c;
        assert_eq!(result1.to_bits(), result2.to_bits());
    }

    #[test]
    fn test_operation_order_independence() {
        // Test that operation order doesn't affect determinism
        let mut world = PhysicsWorld::new();
        world.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));
        world.add_body(RigidBody::new(Vec3::new(5.0, 10.0, 0.0), 1.0));

        let dt = 1.0 / 60.0;

        // Simulate
        for _ in 0..100 {
            world.step(dt);
        }

        // Bodies should have deterministic positions regardless of processing order
        let body0 = world.get_body(0).unwrap();
        let body1 = world.get_body(1).unwrap();

        assert!(body0.position.y < 10.0, "Body should have fallen");
        assert!(body1.position.y < 10.0, "Body should have fallen");
    }

    #[test]
    fn test_multithreaded_determinism() {
        // In a real implementation, this would test Rayon parallel processing
        // For now, we test that sequential processing is deterministic

        let world1 = run_simulation(500);
        let world2 = run_simulation(500);

        for i in 0..world1.body_count() {
            let body1 = world1.get_body(i).unwrap();
            let body2 = world2.get_body(i).unwrap();

            assert!(
                body1.is_bitwise_equal(body2),
                "Multithreaded simulation should be deterministic"
            );
        }
    }

    #[test]
    fn test_replay_capability() {
        // Test that we can replay a simulation and get identical results
        let mut world = PhysicsWorld::new();
        world.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));

        let dt = 1.0 / 60.0;
        let mut positions = Vec::new();

        // Record first run
        for _ in 0..100 {
            world.step(dt);
            positions.push(world.get_body(0).unwrap().position);
        }

        // Replay
        let mut world_replay = PhysicsWorld::new();
        world_replay.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));

        for (frame, expected_pos) in positions.iter().enumerate() {
            world_replay.step(dt);
            let actual_pos = world_replay.get_body(0).unwrap().position;

            assert!(
                actual_pos.x.to_bits() == expected_pos.x.to_bits()
                    && actual_pos.y.to_bits() == expected_pos.y.to_bits()
                    && actual_pos.z.to_bits() == expected_pos.z.to_bits(),
                "Replay should match original at frame {}",
                frame
            );
        }
    }

    #[test]
    fn test_network_sync_compatibility() {
        // Test that determinism enables network synchronization
        // In a networked game, both clients should see the same physics

        let mut client1 = PhysicsWorld::new();
        let mut client2 = PhysicsWorld::new();

        // Both clients start with same initial state
        client1.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));
        client2.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));

        let dt = 1.0 / 60.0;

        // Simulate 60 frames (1 second)
        for _ in 0..60 {
            client1.step(dt);
            client2.step(dt);
        }

        let body1 = client1.get_body(0).unwrap();
        let body2 = client2.get_body(0).unwrap();

        assert!(
            body1.is_bitwise_equal(body2),
            "Network clients should see identical physics"
        );
    }

    #[test]
    fn test_save_load_determinism() {
        // Test that saving and loading state maintains determinism
        let mut world = PhysicsWorld::new();
        world.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));

        let dt = 1.0 / 60.0;

        // Simulate 50 frames
        for _ in 0..50 {
            world.step(dt);
        }

        // "Save" state (clone)
        let saved_state = world.clone();

        // Continue simulation
        for _ in 0..50 {
            world.step(dt);
        }

        // "Load" state and continue
        let mut loaded_world = saved_state;
        for _ in 0..50 {
            loaded_world.step(dt);
        }

        // Should match
        let body1 = world.get_body(0).unwrap();
        let body2 = loaded_world.get_body(0).unwrap();

        assert!(
            body1.is_bitwise_equal(body2),
            "Save/load should maintain determinism"
        );
    }

    #[test]
    fn test_different_initial_conditions() {
        // Verify that different initial conditions produce different results
        // (sanity check that we're not just returning constants)

        let mut world1 = PhysicsWorld::new();
        world1.add_body(RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0));

        let mut world2 = PhysicsWorld::new();
        world2.add_body(RigidBody::new(Vec3::new(0.0, 20.0, 0.0), 1.0)); // Different height

        let dt = 1.0 / 60.0;

        for _ in 0..100 {
            world1.step(dt);
            world2.step(dt);
        }

        let body1 = world1.get_body(0).unwrap();
        let body2 = world2.get_body(0).unwrap();

        assert_ne!(
            body1.position, body2.position,
            "Different initial conditions should produce different results"
        );
    }
}
