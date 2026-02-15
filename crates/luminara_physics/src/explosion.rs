use luminara_core::system::ExclusiveMarker;
use luminara_core::{App, AppInterface, Component, CoreStage, Entity, Plugin, Query, World};
use luminara_math::Vec3;
use rapier3d::prelude::*;

use crate::physics3d::PhysicsWorld3D;

#[derive(Clone)]
pub struct Explosion {
    pub radius: f32,
    pub force: f32,
    pub center: Vec3,
    pub processed: bool,
}

impl Component for Explosion {
    fn type_name() -> &'static str {
        "Explosion"
    }
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            radius: 10.0,
            force: 1000.0,
            center: Vec3::ZERO,
            processed: false,
        }
    }
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn name(&self) -> &str {
        "ExplosionPlugin"
    }

    fn build(&self, app: &mut App) {
        app.add_system::<ExclusiveMarker>(CoreStage::Update, explosion_system);
    }
}

pub fn explosion_system(world: &mut World) {
    // 1. Collect explosion data
    let mut explosions_to_process = Vec::new();
    {
        let query = Query::<(Entity, &Explosion)>::new(world);
        for (entity, explosion) in query.iter() {
            // We can't check processed in immutable query if we want to modify it,
            // but we are despawning anyway.
            explosions_to_process.push((entity, explosion.clone()));
        }
    }

    if explosions_to_process.is_empty() {
        return;
    }

    // 2. Process physics
    {
        let mut physics_world = world.get_resource_mut::<PhysicsWorld3D>().unwrap();

        for (_, explosion) in &explosions_to_process {
            let center = point![explosion.center.x, explosion.center.y, explosion.center.z];
            let radius = explosion.radius;
            let force_mag = explosion.force;

            // Spatial query for bodies in radius
            let interaction_groups = InteractionGroups::all();
            let filter = QueryFilter::default().groups(interaction_groups);
            let shape_pos = Isometry::translation(center.x, center.y, center.z);
            let shape = SharedShape::ball(radius);

            let mut affected_bodies = Vec::new();

            // We use a callback to collect bodies
            physics_world.query_pipeline.intersections_with_shape(
                &physics_world.rigid_body_set,
                &physics_world.collider_set,
                &shape_pos,
                &*shape,
                filter,
                |handle| {
                    if let Some(collider) = physics_world.collider_set.get(handle) {
                        if let Some(body_handle) = collider.parent() {
                            affected_bodies.push(body_handle);
                        }
                    }
                    true // Continue
                },
            );

            // Apply forces
            for body_handle in affected_bodies {
                if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                    if body.is_dynamic() {
                        let body_pos = body.translation();
                        let diff = body_pos - center.coords; // Vector<f32>
                        let dist_sq = diff.magnitude_squared();

                        if dist_sq < radius * radius && dist_sq > 0.0001 {
                            let dist = dist_sq.sqrt();
                            // Inverse square falloff
                            let strength = force_mag / dist_sq.max(1.0);
                            let dir = diff / dist;
                            let impulse = dir * strength;

                            body.apply_impulse(impulse, true);
                        }
                    }
                }
            }
        }
    }

    // 3. Despawn
    for (entity, _) in explosions_to_process {
        world.despawn(entity);
    }
}
