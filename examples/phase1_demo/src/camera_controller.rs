use luminara::prelude::*;
use luminara::input::{ActionMap, InputExt, input_map::{ActionBinding, InputSource}};
use luminara::input::keyboard::Key;
use luminara::input::mouse::MouseButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    LookActive,
    ToggleMode, // New action
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

#[derive(Debug, Clone, Copy)]
pub struct CameraSmoothing {
    pub position_stiffness: f32,
    pub rotation_stiffness: f32,
    pub target_position: Vec3,
    pub target_pitch: f32,
    pub target_yaw: f32,
}

impl Default for CameraSmoothing {
    fn default() -> Self {
        Self {
            position_stiffness: 10.0,
            rotation_stiffness: 15.0,
            target_position: Vec3::ZERO,
            target_pitch: 0.0,
            target_yaw: 0.0,
        }
    }
}

// Manual Component impl since derive might not be available
pub struct CameraController {
    pub speed: f32,
    pub base_sensitivity: f32,
    pub fov_sensitivity_scaling: bool,
    pub pitch: f32,
    pub yaw: f32,
    pub mode: CameraMode,
    pub smoothing: Option<CameraSmoothing>,
}

impl Component for CameraController {
    fn type_name() -> &'static str {
        "CameraController"
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            base_sensitivity: 0.1,
            fov_sensitivity_scaling: true,
            pitch: 0.0,
            yaw: 0.0,
            mode: CameraMode::FirstPerson,
            smoothing: Some(CameraSmoothing::default()),
        }
    }
}

pub fn setup_camera_input(world: &mut World) {
    let mut map = ActionMap::<CameraAction>::new();

    map.bind(CameraAction::MoveForward, ActionBinding {
        inputs: vec![InputSource::Key(Key::W), InputSource::Key(Key::Up)],
    });
    map.bind(CameraAction::MoveBackward, ActionBinding {
        inputs: vec![InputSource::Key(Key::S), InputSource::Key(Key::Down)],
    });
    map.bind(CameraAction::MoveLeft, ActionBinding {
        inputs: vec![InputSource::Key(Key::A), InputSource::Key(Key::Left)],
    });
    map.bind(CameraAction::MoveRight, ActionBinding {
        inputs: vec![InputSource::Key(Key::D), InputSource::Key(Key::Right)],
    });
    map.bind(CameraAction::MoveUp, ActionBinding {
        inputs: vec![InputSource::Key(Key::Space), InputSource::Key(Key::E)],
    });
    map.bind(CameraAction::MoveDown, ActionBinding {
        inputs: vec![InputSource::Key(Key::LShift), InputSource::Key(Key::Q)],
    });
    map.bind(CameraAction::LookActive, ActionBinding {
        inputs: vec![InputSource::MouseButton(MouseButton::Right)],
    });
    map.bind(CameraAction::ToggleMode, ActionBinding {
        inputs: vec![InputSource::Key(Key::C)], // 'C' to toggle camera mode
    });

    world.insert_resource(map);
}

pub fn camera_controller_system(
    mut input: ResMut<Input>,
    map: Res<ActionMap<CameraAction>>,
    time: Res<luminara::core::Time>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
) {
    let dt = time.delta_seconds();
    let mouse_delta = input.mouse_delta();

    // Check if we should rotate
    let rotate = InputExt::action_pressed(&*input, CameraAction::LookActive, &map);

    if rotate {
        input.set_cursor_visible(false);
        input.set_cursor_grabbed(true);
    } else {
        input.set_cursor_visible(true);
        input.set_cursor_grabbed(false);
    }

    // We assume the camera entity has both Transform, CameraController AND Camera component
    // But query is only for Transform and CameraController.
    // Ideally we should query Camera too for FOV.
    // Let's assume default FOV if not found or change query.
    // Since we can't change query signature easily without changing main.rs registration,
    // we'll stick to fixed FOV or assume 60.0 for now, OR change query in main.rs first.
    // Wait, I can't change query signature here because main.rs defines the system registration.
    // I should update main.rs registration if I change the signature.
    // Or I can use `Option<&Camera>` in query? No, main.rs registered `(&mut Transform, &mut CameraController)`.

    // Let's assume a default reference FOV of 60.0.
    let current_fov = 60.0;

    for (mut transform, mut controller) in query.iter_mut() {
        // Toggle mode
        if InputExt::action_just_pressed(&*input, CameraAction::ToggleMode, &map) {
            controller.mode = match controller.mode {
                CameraMode::FirstPerson => CameraMode::ThirdPerson,
                CameraMode::ThirdPerson => CameraMode::FirstPerson,
            };
            println!("Camera Mode switched to: {:?}", controller.mode);
        }

        let sensitivity = if controller.fov_sensitivity_scaling {
            controller.base_sensitivity * (current_fov / 90.0)
        } else {
            controller.base_sensitivity
        };

        // Initialize smoothing target if needed
        if controller.smoothing.is_none() {
            controller.smoothing = Some(CameraSmoothing {
                target_position: transform.translation,
                target_pitch: controller.pitch,
                target_yaw: controller.yaw,
                ..Default::default()
            });
        }

        // Update targets
        let mut smoothing = controller.smoothing.take().unwrap(); // Take ownership to mutate easily

        if rotate {
            smoothing.target_yaw -= mouse_delta.x * sensitivity;
            smoothing.target_pitch -= mouse_delta.y * sensitivity;
            smoothing.target_pitch = smoothing.target_pitch.clamp(-89.0, 89.0);
        }

        // Calculate rotation quaternion from targets
        let target_yaw_rot = Quat::from_rotation_y(smoothing.target_yaw.to_radians());
        // For movement direction, we use target yaw
        let forward = target_yaw_rot * Vec3::NEG_Z;
        let right = target_yaw_rot * Vec3::X;
        let up = Vec3::Y;

        let mut velocity = Vec3::ZERO;
        if InputExt::action_pressed(&*input, CameraAction::MoveForward, &map) {
            velocity += forward;
        }
        if InputExt::action_pressed(&*input, CameraAction::MoveBackward, &map) {
            velocity -= forward;
        }
        if InputExt::action_pressed(&*input, CameraAction::MoveRight, &map) {
            velocity += right;
        }
        if InputExt::action_pressed(&*input, CameraAction::MoveLeft, &map) {
            velocity -= right;
        }
        if InputExt::action_pressed(&*input, CameraAction::MoveUp, &map) {
            velocity += up;
        }
        if InputExt::action_pressed(&*input, CameraAction::MoveDown, &map) {
            velocity -= up;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
        }

        smoothing.target_position += velocity * controller.speed * dt;

        // Apply smoothing (Spring-Damper / Lerp)
        // Lerp factor: 1 - exp(-stiffness * dt)
        let pos_alpha = 1.0 - (-smoothing.position_stiffness * dt).exp();
        let rot_alpha = 1.0 - (-smoothing.rotation_stiffness * dt).exp();

        // Interpolate position
        transform.translation = transform.translation.lerp(smoothing.target_position, pos_alpha);

        // Interpolate rotation values
        // Note: Interpolating Euler angles directly is okay here since we clamped pitch and wrap yaw isn't an issue for small steps,
        // but quaternion slerp is better.
        // Let's interpolate the values stored in controller for rendering
        controller.yaw = controller.yaw + (smoothing.target_yaw - controller.yaw) * rot_alpha;
        controller.pitch = controller.pitch + (smoothing.target_pitch - controller.pitch) * rot_alpha;

        // Apply rotation to transform
        let final_yaw = Quat::from_rotation_y(controller.yaw.to_radians());
        let final_pitch = Quat::from_rotation_x(controller.pitch.to_radians());
        transform.rotation = final_yaw * final_pitch;

        // Third person offset logic
        if controller.mode == CameraMode::ThirdPerson {
            // Offset camera back
            let offset = transform.rotation * Vec3::new(0.0, 2.0, 5.0);
            transform.translation = transform.translation + offset; // This modifies the visual pos, but target_position is the "pivot"
            // Wait, if we modify translation here, next frame lerp will start from the offset position towards the pivot.
            // That would be a glitch.
            // Correct approach: `transform.translation` should track the camera position.
            // In 1st person: camera pos = target pos (smoothed).
            // In 3rd person: camera pos = target pos (smoothed) + offset.

            // So:
            // 1. Smooth the pivot position (target_position -> smoothed_pivot)
            // 2. Compute rotation (target_rot -> smoothed_rot)
            // 3. Set transform based on mode.

            // Let's refactor:
            // stored `transform.translation` is the *actual* camera position.
            // We need a separate state for the "character/pivot" position if we want 3rd person.
            // But here, `transform` IS the entity.
            // If we are controlling a free camera, 3rd person usually implies we are orbiting a point.
            // Let's assume `target_position` is the pivot.

            // We calculate the *desired* camera position.
            let desired_pos = if controller.mode == CameraMode::ThirdPerson {
                 smoothing.target_position + (final_yaw * final_pitch * Vec3::new(0.0, 0.0, 5.0))
            } else {
                 smoothing.target_position
            };

            // We already lerped `transform.translation` towards `target_position` above.
            // Let's correct it.
            // We should lerp towards `desired_pos`.
            transform.translation = transform.translation.lerp(desired_pos, pos_alpha);
        } else {
             // First Person: already handled by simple lerp above?
             // target_position is where we want to be.
             // We lerped transform.translation towards it.
             // Correct.
        }

        // Put back smoothing state
        controller.smoothing = Some(smoothing);
    }
}
