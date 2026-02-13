use luminara_core::shared_types::{AppInterface, CoreStage};
use luminara_core::system::ExclusiveMarker;
use luminara_core::{Commands, Component, Entity, Plugin, Query, Res, ResMut, Resource, Without};
use luminara_math::{Quat, Transform, Vec3};
use rapier3d::prelude::*;
use std::collections::HashMap;

use crate::components::{Collider, ColliderShape, CollisionEvent, RigidBody, RigidBodyType};

/// Resource containing the Rapier 3D physics world
pub struct PhysicsWorld3D {
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

impl Resource for PhysicsWorld3D {}

impl Default for PhysicsWorld3D {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, -9.81, 0.0],
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

/// Collision events resource
pub struct CollisionEvents(pub Vec<CollisionEvent>);

impl Resource for CollisionEvents {}

impl Default for CollisionEvents {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Plugin for 3D physics simulation
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn name(&self) -> &str {
        "PhysicsPlugin"
    }

    fn build(&self, app: &mut luminara_core::App) {
        // Initialize physics world resource
        app.world.insert_resource(PhysicsWorld3D::default());

        // Register collision event
        app.world.insert_resource(CollisionEvents::default());

        // Register physics body/collider creation as exclusive systems (need world mutation)
        app.add_system::<ExclusiveMarker>(
            CoreStage::PreUpdate,
            physics_body_creation_system_exclusive,
        );
        app.add_system::<ExclusiveMarker>(
            CoreStage::PreUpdate,
            physics_collider_creation_system_exclusive,
        );

        // Register physics step system
        app.add_system::<(
            luminara_core::system::FunctionMarker,
            ResMut<'static, PhysicsWorld3D>,
            Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, physics_step_system);

        // Register physics sync system (sync rapier state back to ECS transforms)
        app.add_system::<(
            luminara_core::system::FunctionMarker,
            Res<'static, PhysicsWorld3D>,
            Query<'static, (Entity, &mut Transform, &RigidBody)>,
        )>(CoreStage::PostUpdate, physics_sync_system);

        // Register collision detection system
        app.add_system::<(
            luminara_core::system::FunctionMarker,
            Res<'static, PhysicsWorld3D>,
            ResMut<'static, CollisionEvents>,
        )>(CoreStage::PostUpdate, collision_detection_system);

        log::info!("PhysicsPlugin (3D) initialized with systems");
    }
}

/// Marker component to indicate physics body has been created
#[derive(Debug)]
pub struct PhysicsBodyCreated;

impl Component for PhysicsBodyCreated {
    fn type_name() -> &'static str {
        "PhysicsBodyCreated"
    }
}

/// Marker component to indicate collider has been created
#[derive(Debug)]
pub struct PhysicsColliderCreated;

impl Component for PhysicsColliderCreated {
    fn type_name() -> &'static str {
        "PhysicsColliderCreated"
    }
}

/// System to create physics bodies for new entities with RigidBody components
/// (Exclusive system — needs mutable World access to add marker components)
pub fn physics_body_creation_system_exclusive(world: &mut luminara_core::world::World) {
    // Phase 1: collect entities + component data while world is borrowed immutably
    let entities_to_create: Vec<(Entity, RigidBody, Transform)> = {
        let query =
            Query::<(Entity, &RigidBody, &Transform), Without<PhysicsBodyCreated>>::new(world);
        query
            .iter()
            .map(|(entity, rb, t)| (entity, rb.clone(), *t))
            .collect()
    };

    if entities_to_create.is_empty() {
        return;
    }

    // Phase 2: create Rapier bodies and update mappings
    for (entity, rigid_body, transform) in &entities_to_create {
        let rapier_body_type = match rigid_body.body_type {
            RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
            RigidBodyType::Kinematic => rapier3d::prelude::RigidBodyType::KinematicPositionBased,
            RigidBodyType::Static => rapier3d::prelude::RigidBodyType::Fixed,
        };

        let rapier_body = RigidBodyBuilder::new(rapier_body_type)
            .translation(vector![
                transform.translation.x,
                transform.translation.y,
                transform.translation.z
            ])
            .rotation(AngVector::new(
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
            ))
            .linear_damping(rigid_body.linear_damping)
            .angular_damping(rigid_body.angular_damping)
            .gravity_scale(rigid_body.gravity_scale)
            .build();

        {
            let physics_world = world.get_resource_mut::<PhysicsWorld3D>().unwrap();
            let body_handle = physics_world.rigid_body_set.insert(rapier_body);
            physics_world.entity_to_body.insert(*entity, body_handle);
            physics_world.body_to_entity.insert(body_handle, *entity);
        }

        // Mark entity as having a physics body
        world.add_component(*entity, PhysicsBodyCreated);

        log::info!(
            "Created 3D physics body for entity {:?} at {:?}",
            entity,
            transform.translation
        );
    }
}

/// System to create colliders for entities with Collider components
/// (Exclusive system — needs mutable World access to add marker components)
pub fn physics_collider_creation_system_exclusive(world: &mut luminara_core::world::World) {
    // Phase 1: collect entities + component data
    let entities_to_create: Vec<(Entity, Collider)> = {
        let query = Query::<(Entity, &Collider), Without<PhysicsColliderCreated>>::new(world);
        query
            .iter()
            .map(|(entity, c)| (entity, c.clone()))
            .collect()
    };

    if entities_to_create.is_empty() {
        return;
    }

    // Phase 2: create Rapier colliders
    for (entity, collider) in &entities_to_create {
        let shape: SharedShape = match &collider.shape {
            ColliderShape::Box { half_extents } => {
                SharedShape::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            ColliderShape::Sphere { radius } => SharedShape::ball(*radius),
            ColliderShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            ColliderShape::Mesh { vertices, indices } => {
                let vertices: Vec<Point<f32>> =
                    vertices.iter().map(|v| point![v.x, v.y, v.z]).collect();
                SharedShape::trimesh(vertices, indices.clone())
            }
        };

        let rapier_collider = ColliderBuilder::new(shape)
            .friction(collider.friction)
            .restitution(collider.restitution)
            .sensor(collider.is_sensor)
            .build();

        {
            let physics_world = world.get_resource_mut::<PhysicsWorld3D>().unwrap();
            // Attach to rigid body if it exists
            let collider_handle =
                if let Some(&body_handle) = physics_world.entity_to_body.get(entity) {
                    let mut rigid_body_set = std::mem::take(&mut physics_world.rigid_body_set);
                    let handle = physics_world.collider_set.insert_with_parent(
                        rapier_collider,
                        body_handle,
                        &mut rigid_body_set,
                    );
                    physics_world.rigid_body_set = rigid_body_set;
                    handle
                } else {
                    physics_world.collider_set.insert(rapier_collider)
                };

            physics_world
                .entity_to_collider
                .insert(*entity, collider_handle);
            physics_world
                .collider_to_entity
                .insert(collider_handle, *entity);
        }

        world.add_component(*entity, PhysicsColliderCreated);

        log::info!("Created 3D collider for entity {:?}", entity);
    }
}

/// System to create physics bodies for new entities with RigidBody components
pub fn physics_body_creation_system(
    mut physics_world: ResMut<PhysicsWorld3D>,
    query: Query<(Entity, &RigidBody, &Transform), Without<PhysicsBodyCreated>>,
    mut commands: Commands,
) {
    for (entity, rigid_body, transform) in query.iter() {
        // Create Rapier rigid body
        let rapier_body_type = match rigid_body.body_type {
            RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
            RigidBodyType::Kinematic => rapier3d::prelude::RigidBodyType::KinematicPositionBased,
            RigidBodyType::Static => rapier3d::prelude::RigidBodyType::Fixed,
        };

        let rapier_body = RigidBodyBuilder::new(rapier_body_type)
            .translation(vector![
                transform.translation.x,
                transform.translation.y,
                transform.translation.z
            ])
            .rotation(AngVector::new(
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
            ))
            .linear_damping(rigid_body.linear_damping)
            .angular_damping(rigid_body.angular_damping)
            .gravity_scale(rigid_body.gravity_scale)
            .build();

        let body_handle = physics_world.rigid_body_set.insert(rapier_body);

        // Store mappings
        physics_world.entity_to_body.insert(entity, body_handle);
        physics_world.body_to_entity.insert(body_handle, entity);

        // Mark as created
        commands.entity(entity).insert(PhysicsBodyCreated);

        log::debug!("Created 3D physics body for entity {:?}", entity);
    }
}

/// System to create colliders for entities with Collider components
pub fn physics_collider_creation_system(
    mut physics_world: ResMut<PhysicsWorld3D>,
    query: Query<(Entity, &Collider), Without<PhysicsColliderCreated>>,
    mut commands: Commands,
) {
    for (entity, collider) in query.iter() {
        // Create Rapier collider shape
        let shape: SharedShape = match &collider.shape {
            ColliderShape::Box { half_extents } => {
                SharedShape::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            ColliderShape::Sphere { radius } => SharedShape::ball(*radius),
            ColliderShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            ColliderShape::Mesh { vertices, indices } => {
                let vertices: Vec<Point<f32>> =
                    vertices.iter().map(|v| point![v.x, v.y, v.z]).collect();
                SharedShape::trimesh(vertices, indices.clone())
            }
        };

        let rapier_collider = ColliderBuilder::new(shape)
            .friction(collider.friction)
            .restitution(collider.restitution)
            .sensor(collider.is_sensor)
            .build();

        // Attach to rigid body if it exists
        let collider_handle = if let Some(&body_handle) = physics_world.entity_to_body.get(&entity)
        {
            let mut rigid_body_set = std::mem::take(&mut physics_world.rigid_body_set);
            let handle = physics_world.collider_set.insert_with_parent(
                rapier_collider,
                body_handle,
                &mut rigid_body_set,
            );
            physics_world.rigid_body_set = rigid_body_set;
            handle
        } else {
            physics_world.collider_set.insert(rapier_collider)
        };

        // Store mappings
        physics_world
            .entity_to_collider
            .insert(entity, collider_handle);
        physics_world
            .collider_to_entity
            .insert(collider_handle, entity);

        // Mark as created
        commands.entity(entity).insert(PhysicsColliderCreated);

        log::debug!("Created 3D collider for entity {:?}", entity);
    }
}

/// System to step the physics simulation
pub fn physics_step_system(
    mut physics_world: ResMut<PhysicsWorld3D>,
    time: Res<luminara_core::Time>,
) {
    if time.time_scale == 0.0 {
        return;
    }

    let PhysicsWorld3D {
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
pub fn physics_sync_system(
    physics_world: Res<PhysicsWorld3D>,
    mut query: Query<(Entity, &mut Transform, &RigidBody)>,
) {
    for (entity, transform, _) in query.iter_mut() {
        if let Some(&body_handle) = physics_world.entity_to_body.get(&entity) {
            if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                let position: &Vector<f32> = body.translation();
                let rotation = body.rotation();

                // Check for NaN
                if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
                    log::error!(
                        "NaN detected in physics body for entity {:?}, resetting",
                        entity
                    );
                    transform.translation = Vec3::ZERO;
                    transform.rotation = Quat::IDENTITY;
                    continue;
                }

                transform.translation = Vec3::new(position.x, position.y, position.z);
                transform.rotation =
                    Quat::from_xyzw(rotation.i, rotation.j, rotation.k, rotation.w);
            }
        }
    }
}

/// System to detect collisions and emit events
pub fn collision_detection_system(
    physics_world: Res<PhysicsWorld3D>,
    mut collision_events: ResMut<CollisionEvents>,
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
