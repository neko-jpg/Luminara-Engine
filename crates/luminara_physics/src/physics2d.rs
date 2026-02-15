use luminara_core::{Component, Entity, Plugin, Query, Res, ResMut, Resource};
use luminara_math::{Quat, Transform, Vec3};
use rapier2d::prelude::*;
use std::collections::HashMap;

use crate::components::{CollisionEvent, RigidBody};

/// Resource containing the Rapier 2D physics world
pub struct PhysicsWorld2D {
    pub gravity: Vector<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub entity_to_body: HashMap<Entity, RigidBodyHandle>,
    pub entity_to_collider: HashMap<Entity, ColliderHandle>,
    pub body_to_entity: HashMap<RigidBodyHandle, Entity>,
    pub collider_to_entity: HashMap<ColliderHandle, Entity>,
}

impl Resource for PhysicsWorld2D {}

impl Default for PhysicsWorld2D {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, -9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            entity_to_body: HashMap::new(),
            entity_to_collider: HashMap::new(),
            body_to_entity: HashMap::new(),
            collider_to_entity: HashMap::new(),
        }
    }
}

/// Collision events resource for 2D
pub struct CollisionEvents2D(pub Vec<CollisionEvent>);

impl Resource for CollisionEvents2D {}

impl Default for CollisionEvents2D {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Plugin for 2D physics simulation
pub struct Physics2dPlugin;

impl Plugin for Physics2dPlugin {
    fn name(&self) -> &str {
        "Physics2dPlugin"
    }

    fn build(&self, app: &mut luminara_core::App) {
        // Initialize physics world resource
        app.world.insert_resource(PhysicsWorld2D::default());

        // Register collision event
        app.world.insert_resource(CollisionEvents2D::default());

        log::info!("Physics2dPlugin initialized");
    }
}

/// Marker component to indicate physics body has been created
#[derive(Debug)]
pub struct PhysicsBodyCreated2D;

impl Component for PhysicsBodyCreated2D {
    fn type_name() -> &'static str {
        "PhysicsBodyCreated2D"
    }
}

/// Marker component to indicate collider has been created
#[derive(Debug)]
pub struct PhysicsColliderCreated2D;

impl Component for PhysicsColliderCreated2D {
    fn type_name() -> &'static str {
        "PhysicsColliderCreated2D"
    }
}


/// System to step the physics simulation
pub fn physics_step_system_2d(mut physics_world: ResMut<PhysicsWorld2D>) {
    let PhysicsWorld2D {
        ref gravity,
        ref integration_parameters,
        ref mut physics_pipeline,
        ref mut island_manager,
        ref mut broad_phase,
        ref mut narrow_phase,
        ref mut rigid_body_set,
        ref mut collider_set,
        ref mut impulse_joint_set,
        ref mut multibody_joint_set,
        ref mut ccd_solver,
        ref mut query_pipeline,
        ..
    } = *physics_world;

    physics_pipeline.step(
        gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        Some(query_pipeline),
        &(),
        &(),
    );
}

/// System to sync physics state back to ECS transforms
pub fn physics_sync_system_2d(
    physics_world: Res<PhysicsWorld2D>,
    mut query: Query<(Entity, &mut Transform, &RigidBody)>,
) {
    for (entity, transform, _) in query.iter_mut() {
        if let Some(&body_handle) = physics_world.entity_to_body.get(&entity) {
            if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                let position: &Vector<f32> = body.translation();
                let rotation = body.rotation();

                // Check for NaN
                if position.x.is_nan() || position.y.is_nan() {
                    log::error!(
                        "NaN detected in physics body for entity {:?}, resetting",
                        entity
                    );
                    transform.translation = Vec3::ZERO;
                    transform.rotation = Quat::IDENTITY;
                    continue;
                }

                // Update position (keep Z unchanged for 2D)
                transform.translation.x = position.x;
                transform.translation.y = position.y;

                // Update rotation (Z-axis rotation for 2D)
                let angle = rotation.angle();
                transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), angle);
            }
        }
    }
}

/// System to detect collisions and emit events
pub fn collision_detection_system_2d(
    physics_world: Res<PhysicsWorld2D>,
    mut collision_events: ResMut<CollisionEvents2D>,
) {
    collision_events.0.clear();

    // Iterate through contact pairs
    for contact_pair in physics_world.narrow_phase.contact_pairs() {
        let collider1 = contact_pair.collider1;
        let collider2 = contact_pair.collider2;

        if let (Some(&entity_a), Some(&entity_b)) = (
            physics_world.collider_to_entity.get(&collider1),
            physics_world.collider_to_entity.get(&collider2),
        ) {
            // Check if there's actual contact
            let has_contact = contact_pair.has_any_active_contact;

            if has_contact {
                collision_events.0.push(CollisionEvent {
                    entity_a,
                    entity_b,
                    started: true,
                });
            }
        }
    }
}
