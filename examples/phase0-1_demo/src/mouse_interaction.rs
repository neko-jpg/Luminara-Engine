//! Mouse interaction system for grabbing and throwing physics objects

use luminara::prelude::*;
use luminara_physics::physics3d::PhysicsWorld3D;
use rapier3d::prelude::{nalgebra, QueryFilter, Ray};

/// Component marking an object as grabbed by the mouse
#[derive(Debug, Clone)]
pub struct MouseGrabbed {
    pub local_anchor: Vec3,
    pub spring_stiffness: f32,
    pub damping: f32,
    pub grab_distance: f32,
}

impl Component for MouseGrabbed {
    fn type_name() -> &'static str {
        "MouseGrabbed"
    }
}

/// Resource tracking mouse interaction state
pub struct MouseInteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_start_time: f32,
    pub last_mouse_positions: Vec<Vec3>,
    pub max_history: usize,
}

impl Resource for MouseInteractionState {}

impl Default for MouseInteractionState {
    fn default() -> Self {
        Self {
            grabbed_entity: None,
            grab_start_time: 0.0,
            last_mouse_positions: Vec::new(),
            max_history: 5,
        }
    }
}

impl MouseInteractionState {
    pub fn record_position(&mut self, pos: Vec3) {
        self.last_mouse_positions.push(pos);
        if self.last_mouse_positions.len() > self.max_history {
            self.last_mouse_positions.remove(0);
        }
    }

    pub fn calculate_throw_velocity(&self, dt: f32) -> Vec3 {
        if self.last_mouse_positions.len() < 2 {
            return Vec3::ZERO;
        }

        let recent = &self.last_mouse_positions[self.last_mouse_positions.len() - 1];
        let previous = &self.last_mouse_positions[0];
        let time_span = dt * self.last_mouse_positions.len() as f32;

        if time_span > 0.0 {
            (*recent - *previous) / time_span
        } else {
            Vec3::ZERO
        }
    }

    pub fn clear(&mut self) {
        self.grabbed_entity = None;
        self.last_mouse_positions.clear();
    }
}

/// System to handle mouse grabbing and throwing
pub fn mouse_interaction_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    // Check for grab input (Left Mouse Button)
    let wants_grab = world
        .get_resource::<Input>()
        .map(|i| i.mouse_just_pressed(luminara_input::mouse::MouseButton::Left))
        .unwrap_or(false);

    let wants_release = world
        .get_resource::<Input>()
        .map(|i| i.mouse_just_released(luminara_input::mouse::MouseButton::Left))
        .unwrap_or(false);

    let is_holding = world
        .get_resource::<Input>()
        .map(|i| i.mouse_pressed(luminara_input::mouse::MouseButton::Left))
        .unwrap_or(false);

    // Get camera info for raycasting
    let (camera_pos, camera_dir) = {
        let mut cam_pos = Vec3::ZERO;
        let mut cam_dir = Vec3::NEG_Z;
        let query = Query::<(&Transform, &crate::camera_controller::CameraController)>::new(world);
        for (transform, controller) in query.iter() {
            let yaw = controller.yaw.to_radians();
            let pitch = controller.pitch.to_radians();
            cam_dir = Vec3::new(
                -yaw.sin() * pitch.cos(),
                pitch.sin(),
                -yaw.cos() * pitch.cos(),
            )
            .normalize();
            cam_pos = Vec3::new(
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            );
            break;
        }
        (cam_pos, cam_dir)
    };

    // Handle grab attempt
    if wants_grab {
        // First pass: raycast to find entity (immutable borrow of world)
        let grab_info: Option<(Entity, f32, Vec3)> = {
            if let Some(physics_world) = world.get_resource::<PhysicsWorld3D>() {
                let ray = Ray::new(
                    nalgebra::Point3::new(camera_pos.x, camera_pos.y, camera_pos.z),
                    nalgebra::Vector3::new(camera_dir.x, camera_dir.y, camera_dir.z),
                );

                let max_toi = 50.0;
                let solid = true;
                let query_filter = QueryFilter::default();

                if let Some((handle, toi)) = physics_world.query_pipeline.cast_ray(
                    &physics_world.rigid_body_set,
                    &physics_world.collider_set,
                    &ray,
                    max_toi,
                    solid,
                    query_filter,
                ) {
                    let mut result = None;
                    if let Some(collider) = physics_world.collider_set.get(handle) {
                        if let Some(body_handle) = collider.parent() {
                            if let Some(entity) = physics_world.body_to_entity.get(&body_handle) {
                                if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                                    if body.is_dynamic() {
                                        let hit_point = ray.point_at(toi);
                                        let hit_vec3 =
                                            Vec3::new(hit_point.x, hit_point.y, hit_point.z);
                                        result = Some((*entity, toi, hit_vec3));
                                    }
                                }
                            }
                        }
                    }
                    result
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Second pass: apply grab (mutable borrow of world)
        if let Some((entity, toi, hit_vec3)) = grab_info {
            let local_anchor = world
                .get_component::<Transform>(entity)
                .map(|t| hit_vec3 - t.translation)
                .unwrap_or(Vec3::ZERO);

            world.add_component(
                entity,
                MouseGrabbed {
                    local_anchor,
                    spring_stiffness: 50.0,
                    damping: 10.0,
                    grab_distance: toi,
                },
            );

            let elapsed = world
                .get_resource::<Time>()
                .map(|t| t.elapsed_seconds())
                .unwrap_or(0.0);

            if let Some(state) = world.get_resource_mut::<MouseInteractionState>() {
                state.grabbed_entity = Some(entity);
                state.grab_start_time = elapsed;
                state.last_mouse_positions.clear();
            }

            println!("Grabbed entity {:?} at distance {:.2}", entity, toi);
        }
    }

    // Handle release (throw)
    if wants_release {
        if let Some(state) = world.get_resource::<MouseInteractionState>() {
            if let Some(entity) = state.grabbed_entity {
                let throw_velocity = state.calculate_throw_velocity(dt);

                // Apply throw velocity to the physics body
                if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld3D>() {
                    // Find the body handle for this entity
                    for (body_handle, ent) in &physics_world.body_to_entity {
                        if *ent == entity {
                            if let Some(body) = physics_world.rigid_body_set.get_mut(*body_handle) {
                                let impulse = throw_velocity * body.mass() * 0.5; // Scale factor
                                let impulse_vec =
                                    nalgebra::Vector3::new(impulse.x, impulse.y, impulse.z);
                                body.apply_impulse(impulse_vec, true);
                                println!("Threw entity with velocity: {:?}", throw_velocity);
                            }
                            break;
                        }
                    }
                }

                // Remove grabbed component
                world.remove_component::<MouseGrabbed>(entity);
            }
        }

        if let Some(state) = world.get_resource_mut::<MouseInteractionState>() {
            state.clear();
        }
    }

    // Update grabbed object position (spring constraint)
    if is_holding {
        if let Some(state) = world.get_resource::<MouseInteractionState>() {
            if let Some(entity) = state.grabbed_entity {
                // Calculate target position based on mouse ray
                let target_pos = camera_pos
                    + camera_dir * {
                        world
                            .get_component::<MouseGrabbed>(entity)
                            .map(|g| g.grab_distance)
                            .unwrap_or(10.0)
                    };

                // Record position for throw velocity calculation
                if let Some(state) = world.get_resource_mut::<MouseInteractionState>() {
                    state.record_position(target_pos);
                }

                // Apply spring force
                if let (Some(grabbed), Some(transform)) = (
                    world.get_component::<MouseGrabbed>(entity),
                    world.get_component::<Transform>(entity),
                ) {
                    let world_anchor = transform.translation + grabbed.local_anchor;
                    let displacement = target_pos - world_anchor;

                    if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld3D>() {
                        for (body_handle, ent) in &physics_world.body_to_entity {
                            if *ent == entity {
                                if let Some(body) =
                                    physics_world.rigid_body_set.get_mut(*body_handle)
                                {
                                    let spring_force = displacement * grabbed.spring_stiffness;
                                    let velocity = body.linvel();
                                    let damping_force =
                                        -Vec3::new(velocity.x, velocity.y, velocity.z)
                                            * grabbed.damping;

                                    let total_force = spring_force + damping_force;
                                    let force_vec = nalgebra::Vector3::new(
                                        total_force.x,
                                        total_force.y,
                                        total_force.z,
                                    );
                                    body.add_force(force_vec, true);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
