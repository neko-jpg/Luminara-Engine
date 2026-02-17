pub mod camera_shake;
pub mod components;
pub mod debug;
pub mod explosion;
pub mod integration_config;
pub mod interaction;
pub mod lie_integrator;
pub mod physics2d;
pub mod physics3d;
pub mod spatial_acceleration;
pub mod target_game;

pub use components::*;
pub use debug::PhysicsDebugConfig;
pub use integration_config::{IntegrationMethod, IntegrationMethodOverride, PhysicsIntegrationConfig};
pub use lie_integrator::LiePhysicsIntegrator;
pub use physics2d::{CollisionEvents2D, Physics2dPlugin, PhysicsWorld2D};
pub use physics3d::{CollisionEvents, PhysicsPlugin, PhysicsWorld3D};
pub use target_game::{Target, TargetGameState};

// Re-export physics systems for manual scheduling if needed
pub use physics3d::{
    collision_detection_system, physics_step_system, physics_sync_system,
};

pub use physics2d::{
    collision_detection_system_2d, physics_step_system_2d, physics_sync_system_2d,
};
