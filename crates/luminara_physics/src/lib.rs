pub mod components;
pub mod physics2d;
pub mod physics3d;

pub use components::*;
pub use physics2d::{CollisionEvents2D, Physics2dPlugin, PhysicsWorld2D};
pub use physics3d::{CollisionEvents, PhysicsPlugin, PhysicsWorld3D};

// Re-export physics systems for manual scheduling if needed
pub use physics3d::{
    collision_detection_system, physics_body_creation_system, physics_collider_creation_system,
    physics_step_system, physics_sync_system,
};

pub use physics2d::{
    collision_detection_system_2d, physics_body_creation_system_2d,
    physics_collider_creation_system_2d, physics_step_system_2d, physics_sync_system_2d,
};
