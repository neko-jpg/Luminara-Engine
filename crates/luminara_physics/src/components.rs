use luminara_core::{Component, Entity};
use luminara_math::{Vec3, Transform};
use serde::{Deserialize, Serialize};

/// Component to store the previous frame's transform for interpolation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PreviousTransform(pub Transform);

impl Default for PreviousTransform {
    fn default() -> Self {
        Self(Transform::IDENTITY)
    }
}

impl Component for PreviousTransform {
    fn type_name() -> &'static str {
        "PreviousTransform"
    }
}

/// Rigid body component for physics simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
}

impl Component for RigidBody {
    fn type_name() -> &'static str {
        "RigidBody"
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
        }
    }
}

/// Type of rigid body
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// Dynamic bodies are affected by forces and collisions
    Dynamic,
    /// Kinematic bodies move only when explicitly set
    Kinematic,
    /// Static bodies never move
    Static,
}

/// Collider component for collision detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub is_sensor: bool,
}

impl Component for Collider {
    fn type_name() -> &'static str {
        "Collider"
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        }
    }
}

/// Shape of a collider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape {
    Box {
        half_extents: Vec3,
    },
    Sphere {
        radius: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
    Mesh {
        vertices: Vec<Vec3>,
        indices: Vec<[u32; 3]>,
    },
}

/// Collision event emitted when two entities collide
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub started: bool, // true = collision started, false = collision ended
}

/// Velocity component for physics bodies
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Component for Velocity {
    fn type_name() -> &'static str {
        "Velocity"
    }
}

/// Force accumulator for physics bodies
#[derive(Debug, Clone, Copy, Default)]
pub struct Force {
    pub force: Vec3,
    pub torque: Vec3,
}

impl Component for Force {
    fn type_name() -> &'static str {
        "Force"
    }
}
