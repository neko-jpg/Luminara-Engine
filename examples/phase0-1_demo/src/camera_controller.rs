use luminara::prelude::*;
use luminara::input::{ActionMap, InputExt, input_map::{ActionBinding, InputSource}};
use luminara::input::keyboard::Key;
use luminara::input::mouse::MouseButton;

// ============================================================================
// Actions
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    LookActive,   // Right mouse button — hold to enable mouse look
    ToggleMode,   // C — switch 1st/3rd person
    LookUp,       // Arrow Up
    LookDown,     // Arrow Down
    LookLeft,     // Arrow Left
    LookRight,    // Arrow Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

// ============================================================================
// Controller
// ============================================================================

pub struct CameraController {
    pub speed: f32,
    /// Mouse sensitivity — degrees per hardware delta unit.
    /// Good range: 0.02 – 0.15. Default: 0.05
    pub mouse_sensitivity: f32,
    /// Keyboard look speed in degrees per second.
    pub keyboard_look_speed: f32,
    /// Current pitch in degrees (clamped −89 .. +89).
    pub pitch: f32,
    /// Current yaw in degrees.
    pub yaw: f32,
    pub mode: CameraMode,
    /// Internal: was the cursor grab active last frame?
    /// Used to skip the first delta after a grab transition (prevents jump).
    grab_was_active: bool,
}

impl Component for CameraController {
    fn type_name() -> &'static str {
        "CameraController"
    }
}

impl Resource for CameraController {}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            mouse_sensitivity: 0.05,
            keyboard_look_speed: 90.0,
            pitch: 0.0,
            yaw: 0.0,
            mode: CameraMode::FirstPerson,
            grab_was_active: false,
        }
    }
}

// ============================================================================
// Input bindings
// ============================================================================

pub fn setup_camera_input(world: &mut World) {
    let mut map = ActionMap::<CameraAction>::new();

    // Movement — WASD only (arrow keys reserved for looking)
    map.bind(CameraAction::MoveForward, ActionBinding {
        inputs: vec![InputSource::Key(Key::W)],
    });
    map.bind(CameraAction::MoveBackward, ActionBinding {
        inputs: vec![InputSource::Key(Key::S)],
    });
    map.bind(CameraAction::MoveLeft, ActionBinding {
        inputs: vec![InputSource::Key(Key::A)],
    });
    map.bind(CameraAction::MoveRight, ActionBinding {
        inputs: vec![InputSource::Key(Key::D)],
    });
    map.bind(CameraAction::MoveUp, ActionBinding {
        inputs: vec![InputSource::Key(Key::Space), InputSource::Key(Key::E)],
    });
    map.bind(CameraAction::MoveDown, ActionBinding {
        inputs: vec![InputSource::Key(Key::LShift), InputSource::Key(Key::Q)],
    });

    // Mouse look — hold right-click
    map.bind(CameraAction::LookActive, ActionBinding {
        inputs: vec![InputSource::MouseButton(MouseButton::Right)],
    });

    // Keyboard look — arrow keys (always work, no button needed)
    map.bind(CameraAction::LookUp, ActionBinding {
        inputs: vec![InputSource::Key(Key::Up)],
    });
    map.bind(CameraAction::LookDown, ActionBinding {
        inputs: vec![InputSource::Key(Key::Down)],
    });
    map.bind(CameraAction::LookLeft, ActionBinding {
        inputs: vec![InputSource::Key(Key::Left)],
    });
    map.bind(CameraAction::LookRight, ActionBinding {
        inputs: vec![InputSource::Key(Key::Right)],
    });

    // Camera mode toggle
    map.bind(CameraAction::ToggleMode, ActionBinding {
        inputs: vec![InputSource::Key(Key::C)],
    });

    world.insert_resource(map);
}

// ============================================================================
// System
// ============================================================================

pub fn camera_controller_system(
    mut input: ResMut<Input>,
    map: Res<ActionMap<CameraAction>>,
    time: Res<luminara::core::Time>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
) {
    let dt = time.delta_seconds();
    if dt <= 0.0 { return; }

    // ── Read raw mouse delta BEFORE any state changes ──────────────
    let mouse_dx = input.mouse.delta.x;
    let mouse_dy = input.mouse.delta.y;

    // ── Grab state management ──────────────────────────────────────
    let grab_active = InputExt::action_pressed(&*input, CameraAction::LookActive, &map);

    if grab_active {
        input.set_cursor_visible(false);
        input.set_cursor_grabbed(true);
        // Request cursor center-warp each frame to prevent hitting screen edges
        input.mouse.center_warp_request = true;
    } else {
        input.set_cursor_visible(true);
        input.set_cursor_grabbed(false);
    }

    // ── Pre-read all input states (avoids borrow issues) ───────────
    let look_up    = InputExt::action_pressed(&*input, CameraAction::LookUp, &map);
    let look_down  = InputExt::action_pressed(&*input, CameraAction::LookDown, &map);
    let look_left  = InputExt::action_pressed(&*input, CameraAction::LookLeft, &map);
    let look_right = InputExt::action_pressed(&*input, CameraAction::LookRight, &map);
    let toggle     = InputExt::action_just_pressed(&*input, CameraAction::ToggleMode, &map);
    let move_fwd   = InputExt::action_pressed(&*input, CameraAction::MoveForward, &map);
    let move_back  = InputExt::action_pressed(&*input, CameraAction::MoveBackward, &map);
    let move_left  = InputExt::action_pressed(&*input, CameraAction::MoveLeft, &map);
    let move_right_key = InputExt::action_pressed(&*input, CameraAction::MoveRight, &map);
    let move_up    = InputExt::action_pressed(&*input, CameraAction::MoveUp, &map);
    let move_down  = InputExt::action_pressed(&*input, CameraAction::MoveDown, &map);

    for (transform, ctrl) in query.iter_mut() {
        // ── Mode toggle ────────────────────────────────────────────
        if toggle {
            ctrl.mode = match ctrl.mode {
                CameraMode::FirstPerson => CameraMode::ThirdPerson,
                CameraMode::ThirdPerson => CameraMode::FirstPerson,
            };
        }

        // ── Mouse look ─────────────────────────────────────────────
        // Only apply mouse delta when grab was ALREADY active last frame.
        // This skips the first frame after right-click (which has a huge
        // accumulated delta from the cursor position jump).
        if grab_active && ctrl.grab_was_active {
            ctrl.yaw   -= mouse_dx * ctrl.mouse_sensitivity;
            ctrl.pitch -= mouse_dy * ctrl.mouse_sensitivity;
        }
        ctrl.grab_was_active = grab_active;

        // ── Keyboard look (always works, no right-click needed) ────
        let kb_speed = ctrl.keyboard_look_speed * dt;
        if look_up    { ctrl.pitch += kb_speed; }
        if look_down  { ctrl.pitch -= kb_speed; }
        if look_left  { ctrl.yaw   += kb_speed; }
        if look_right { ctrl.yaw   -= kb_speed; }

        // Clamp pitch
        ctrl.pitch = ctrl.pitch.clamp(-89.0, 89.0);

        // ── Build rotation quaternion ──────────────────────────────
        // Yaw: rotate around world-Y axis (left / right)
        // Pitch: rotate around local-X axis (up / down)
        // Order: yaw * pitch — standard FPS camera
        let yaw_q   = Quat::from_rotation_y(ctrl.yaw.to_radians());
        let pitch_q = Quat::from_rotation_x(ctrl.pitch.to_radians());
        transform.rotation = yaw_q * pitch_q;

        // ── Movement ───────────────────────────────────────────────
        // Movement direction uses yaw only (ignores pitch so you don't
        // accelerate into the ground when looking down).
        let forward = yaw_q * Vec3::NEG_Z;
        let right   = yaw_q * Vec3::X;

        let mut vel = Vec3::ZERO;
        if move_fwd       { vel += forward; }
        if move_back      { vel -= forward; }
        if move_right_key { vel += right; }
        if move_left      { vel -= right; }
        if move_up        { vel += Vec3::Y; }
        if move_down      { vel -= Vec3::Y; }

        if vel.length_squared() > 0.0 {
            vel = vel.normalize();
        }
        transform.translation += vel * ctrl.speed * dt;

        // ── Third-person offset ────────────────────────────────────
        if ctrl.mode == CameraMode::ThirdPerson {
            let offset = transform.rotation * Vec3::new(0.0, 2.0, 8.0);
            transform.translation += offset;
        }
    }
}
