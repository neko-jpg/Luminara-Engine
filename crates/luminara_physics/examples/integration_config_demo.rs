/// Example demonstrating physics integration method configuration
///
/// This example shows how to:
/// 1. Configure global integration method
/// 2. Override integration method per-body
/// 3. Mix Euler and RK4 for optimal performance
///
/// Run with: cargo run --example integration_config_demo

use luminara_core::World;
use luminara_math::Vec3;
use luminara_physics::{
    IntegrationMethod, IntegrationMethodOverride, PhysicsIntegrationConfig, RigidBody,
    RigidBodyType, Velocity,
};

fn main() {
    println!("=== Physics Integration Configuration Demo ===\n");

    // Example 1: Default configuration (Euler)
    println!("Example 1: Default Configuration");
    println!("  Default method: Euler (backward compatible)");
    let config = PhysicsIntegrationConfig::default();
    println!("  Config: {:?}\n", config);

    // Example 2: Global RK4 configuration
    println!("Example 2: Global RK4 Configuration");
    let mut config = PhysicsIntegrationConfig::default();
    config.default_method = IntegrationMethod::Rk4;
    println!("  All bodies will use RK4 by default");
    println!("  Config: {:?}\n", config);

    // Example 3: Per-body override
    println!("Example 3: Per-Body Override");
    let mut world = World::new();

    // Spawn body with RK4 override
    let precise_body = world.spawn();
    world.add_component(
        precise_body,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            ..Default::default()
        },
    );
    world.add_component(precise_body, IntegrationMethodOverride::rk4());
    println!("  Entity {:?}: Uses RK4 (override)", precise_body);

    // Spawn body with Euler override
    let fast_body = world.spawn();
    world.add_component(
        fast_body,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 0.5,
            ..Default::default()
        },
    );
    world.add_component(fast_body, IntegrationMethodOverride::euler());
    println!("  Entity {:?}: Uses Euler (override)", fast_body);

    // Spawn body without override (uses global default)
    let default_body = world.spawn();
    world.add_component(
        default_body,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 1.5,
            ..Default::default()
        },
    );
    println!(
        "  Entity {:?}: Uses global default (Euler)\n",
        default_body
    );

    // Example 4: Mixed scenario (RK4 default, Euler for specific bodies)
    println!("Example 4: Mixed Scenario");
    println!("  Strategy: RK4 by default, Euler for background objects");

    let mut world = World::new();

    // Important gameplay object (uses RK4 default)
    let player = world.spawn();
    world.add_component(
        player,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 70.0, // Player mass
            ..Default::default()
        },
    );
    world.add_component(
        player,
        Velocity {
            linear: Vec3::new(5.0, 0.0, 0.0),
            angular: Vec3::new(0.0, 2.0, 0.0),
        },
    );
    println!("  Player entity {:?}: RK4 (accurate)", player);

    // High-speed projectile (uses RK4 default)
    let projectile = world.spawn();
    world.add_component(
        projectile,
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 0.1,
            ..Default::default()
        },
    );
    world.add_component(
        projectile,
        Velocity {
            linear: Vec3::new(100.0, 50.0, 0.0),
            angular: Vec3::new(0.0, 30.0, 0.0), // High angular velocity
        },
    );
    println!("  Projectile entity {:?}: RK4 (high velocity)", projectile);

    // Background debris (override to Euler for performance)
    println!("  Background debris:");
    for i in 0..5 {
        let debris = world.spawn();
        world.add_component(
            debris,
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 0.2,
                ..Default::default()
            },
        );
        world.add_component(debris, IntegrationMethodOverride::euler());
        println!("    Debris {} entity {:?}: Euler (performance)", i, debris);
    }

    println!("\n=== Configuration Summary ===");
    println!("✓ Default: Euler (backward compatible)");
    println!("✓ Global config: Set default method for all bodies");
    println!("✓ Per-body override: Fine-grained control");
    println!("✓ Mixed scenarios: Optimize performance where needed");
    println!("\nSee docs/integration_methods.md for detailed guide!");
}
