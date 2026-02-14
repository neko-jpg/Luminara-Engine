use luminara_core::{CoreStage, Plugin, Query, Res, ResMut, Resource, AppInterface, Entity};
use luminara_core::system::FunctionMarker;
use luminara_input::{Input, MouseButton};
use luminara_math::{Vec2, Vec3};
use luminara_render::camera::Camera;
use luminara_render::{Gizmos, CommandBuffer};
use luminara_scene::GlobalTransform;
use luminara_window::Window;
use rapier3d::prelude::*;
use std::collections::VecDeque;

use crate::physics3d::PhysicsWorld3D;

pub struct MouseInteractionConfig {
    pub grab_force: f32,
    pub drag_damping: f32,
    pub throw_multiplier: f32,
    pub max_grab_distance: f32,
}

impl Resource for MouseInteractionConfig {}

impl Default for MouseInteractionConfig {
    fn default() -> Self {
        Self {
            grab_force: 150.0,
            drag_damping: 10.0,
            throw_multiplier: 2.0,
            max_grab_distance: 100.0,
        }
    }
}

#[derive(Default)]
pub struct MouseGrabState {
    pub grabbed_entity: Option<Entity>,
    pub grab_offset: Vec3,
    pub grab_distance: f32,
    pub velocity_history: VecDeque<Vec3>,
    pub local_anchor: Point<f32>,
}

impl Resource for MouseGrabState {}

pub struct MouseInteractionPlugin;

impl Plugin for MouseInteractionPlugin {
    fn name(&self) -> &str {
        "MouseInteractionPlugin"
    }

    fn build(&self, app: &mut luminara_core::App) {
        app.insert_resource(MouseInteractionConfig::default());
        app.insert_resource(MouseGrabState::default());

        app.add_system::<(
            FunctionMarker,
            Res<'static, Input>,
            Res<'static, Window>,
            ResMut<'static, PhysicsWorld3D>,
            ResMut<'static, MouseGrabState>,
            Res<'static, MouseInteractionConfig>,
            ResMut<'static, CommandBuffer>,
            Query<'static, (&Camera, &GlobalTransform)>,
        )>(CoreStage::Update, mouse_interaction_system);
    }
}

pub fn mouse_interaction_system(
    input: Res<Input>,
    window: Res<Window>,
    mut physics_world: ResMut<PhysicsWorld3D>,
    mut grab_state: ResMut<MouseGrabState>,
    config: Res<MouseInteractionConfig>,
    mut cmd_buffer: ResMut<CommandBuffer>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    // 1. Get active camera
    let (camera, camera_transform) = if let Some(cam) = cameras.iter().find(|(c, _)| c.is_active) {
        cam
    } else {
        return;
    };

    // 2. Get mouse position
    let mouse_pos = input.mouse_position();
    let window_size = Vec2::new(window.width as f32, window.height as f32);

    // 3. Compute Ray
    let (ray_origin, ray_dir) = screen_to_world_ray(mouse_pos, window_size, camera, camera_transform);

    // Draw ray for debug
    Gizmos::line(
        &mut cmd_buffer,
        ray_origin,
        ray_origin + ray_dir * config.max_grab_distance,
        luminara_math::Color::rgba(1.0, 1.0, 0.0, 0.5),
    );

    // 4. Handle Input
    if input.mouse_just_pressed(MouseButton::Left) {
        // Raycast
        let ray = Ray::new(
            point![ray_origin.x, ray_origin.y, ray_origin.z],
            vector![ray_dir.x, ray_dir.y, ray_dir.z],
        );

        let max_toi = config.max_grab_distance;
        let solid = true;
        let query_filter = QueryFilter::default(); // Exclude sensors?

        if let Some((handle, toi)) = physics_world.query_pipeline.cast_ray(
            &physics_world.rigid_body_set,
            &physics_world.collider_set,
            &ray,
            max_toi,
            solid,
            query_filter,
        ) {
            // Hit something
            if let Some(collider) = physics_world.collider_set.get(handle) {
                if let Some(body_handle) = collider.parent() {
                    // Check if body is dynamic
                    if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                        if body.is_dynamic() {
                            // Grab it
                            if let Some(entity) = physics_world.body_to_entity.get(&body_handle) {
                                grab_state.grabbed_entity = Some(*entity);
                                grab_state.grab_distance = toi;

                                // Calculate local anchor point
                                let hit_point = ray.point_at(toi);
                                let body_transform = body.position();
                                grab_state.local_anchor = body_transform.inverse_transform_point(&hit_point);

                                grab_state.velocity_history.clear();

                                log::info!("Grabbed entity {:?}", entity);
                            }
                        }
                    }
                }
            }
        }
    } else if input.mouse_pressed(MouseButton::Left) {
        // Dragging
        if let Some(entity) = grab_state.grabbed_entity {
            if let Some(&body_handle) = physics_world.entity_to_body.get(&entity) {
                let dt = physics_world.integration_parameters.dt;
                if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                    // Calculate target point
                    let target_point = ray_origin + ray_dir * grab_state.grab_distance;

                    // Calculate current point
                    let current_point = body.position().transform_point(&grab_state.local_anchor);

                    // Spring force: F = k * (Target - Current) - c * Velocity
                    let diff = vector![target_point.x - current_point.x, target_point.y - current_point.y, target_point.z - current_point.z];
                    let velocity = *body.linvel();

                    let force = diff * config.grab_force - velocity * config.drag_damping;

                    body.apply_impulse(force * dt, true);

                    // Also apply damping to angular velocity to prevent spinning
                    let ang_vel = *body.angvel();
                    body.apply_torque_impulse(-ang_vel * config.drag_damping * 0.1 * dt, true);

                    // Draw grab line
                    Gizmos::line(
                        &mut cmd_buffer,
                        Vec3::new(current_point.x, current_point.y, current_point.z),
                        target_point,
                        luminara_math::Color::RED,
                    );
                }
            }
        }
    } else if input.mouse_just_released(MouseButton::Left) {
        // Release
        if let Some(entity) = grab_state.grabbed_entity {
            if let Some(&body_handle) = physics_world.entity_to_body.get(&entity) {
                if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                     // Apply throw impulse if needed, but the spring force already accelerated it.
                     // Often we want an extra boost.
                     let vel = *body.linvel();
                     body.apply_impulse(vel * config.throw_multiplier, true);
                     log::info!("Released entity {:?} with velocity {:?}", entity, vel);
                }
            }
        }
        grab_state.grabbed_entity = None;
    }
}

fn screen_to_world_ray(
    mouse_pos: Vec2,
    window_size: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    // 1. Normalized Device Coordinates (NDC)
    // Mouse (0,0) is usually top-left.
    // NDC: (-1, 1) top-left, (1, -1) bottom-right (Y points up)
    // wgpu uses Y up in clip space? Wait.
    // Winit coords: (0,0) top-left.

    let x = (2.0 * mouse_pos.x) / window_size.x - 1.0;
    let y = 1.0 - (2.0 * mouse_pos.y) / window_size.y; // Flip Y

    let ndc_near = Vec3::new(x, y, 0.0); // Z=0 is near in wgpu (0 to 1)
    let ndc_far = Vec3::new(x, y, 1.0);

    // 2. Inverse View-Projection
    let proj = camera.projection_matrix(window_size.x / window_size.y);
    let cam_mat = camera_transform.0.to_matrix();
    let view = camera.view_matrix(&cam_mat);
    let view_proj = proj * view;
    let inv_view_proj = view_proj.inverse();

    // 3. Unproject
    let unproject = |ndc: Vec3| -> Vec3 {
        let v = inv_view_proj.transform_point3(ndc); // glam handles perspective divide? transform_point3 does.
        v
    };

    let world_near = unproject(ndc_near);
    let world_far = unproject(ndc_far);

    let dir = (world_far - world_near).normalize();

    (world_near, dir)
}
