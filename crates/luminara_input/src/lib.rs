pub mod action;
pub mod axis;
pub mod gamepad;
pub mod input_map;
pub mod keyboard;
pub mod mouse;
pub mod plugin;
pub mod smoothing;
pub mod touch;

pub use action::{ActionMap, InputAction, InputExt};
pub use smoothing::MouseSmoothing;
use axis::{GamepadAxisType, MouseAxisType};
use gamepad::GamepadInput;
use input_map::{InputMap, InputSource};
use keyboard::{Key, KeyboardInput};
use luminara_core::shared_types::Resource;
use luminara_math::Vec2;
pub use mouse::{MouseButton, MouseInput};
use std::collections::HashMap;
use touch::TouchInput;

/// The main input resource that provides a unified interface for all input devices.
///
/// This resource combines keyboard, mouse, gamepad, and touch input and supports
/// axis and action mapping similar to Unity.
pub struct Input {
    /// The keyboard input state.
    pub keyboard: KeyboardInput,
    /// The mouse input state.
    pub mouse: MouseInput,
    /// The gamepad input state (if available).
    pub gamepad: Option<GamepadInput>,
    /// The touch input state.
    pub touch: TouchInput,
    /// The ID of the gamepad used for primary single-player input.
    pub primary_gamepad_id: u32,
    pub(crate) axis_values: HashMap<String, f32>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keyboard: KeyboardInput::default(),
            mouse: MouseInput::default(),
            gamepad: GamepadInput::new(),
            touch: TouchInput::default(),
            primary_gamepad_id: 0,
            axis_values: HashMap::new(),
        }
    }
}

impl Resource for Input {}

impl Input {
    /// Returns true if the specified key is currently held down.
    pub fn pressed(&self, key: Key) -> bool {
        self.keyboard.pressed(key)
    }

    /// Returns true if the specified key was pressed this frame.
    pub fn just_pressed(&self, key: Key) -> bool {
        self.keyboard.just_pressed(key)
    }

    /// Returns true if the specified key was released this frame.
    pub fn just_released(&self, key: Key) -> bool {
        self.keyboard.just_released(key)
    }

    /// Returns true if the specified mouse button is currently held down.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse.pressed(button)
    }

    /// Returns true if the specified mouse button was pressed this frame.
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse.just_pressed(button)
    }

    /// Returns true if the specified mouse button was released this frame.
    pub fn mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse.just_released(button)
    }

    /// Returns the current mouse position in pixels.
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse.position()
    }

    /// Returns the mouse movement delta since the last frame.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse.delta()
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.mouse.cursor_visible = visible;
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.mouse.cursor_visible
    }

    pub fn set_cursor_grabbed(&mut self, grabbed: bool) {
        self.mouse.cursor_grabbed = grabbed;
    }

    pub fn is_cursor_grabbed(&self) -> bool {
        self.mouse.cursor_grabbed
    }

    /// Returns the value of the virtual axis identified by `name`.
    ///
    /// The value is between -1.0 and 1.0.
    pub fn axis(&self, name: &str) -> f32 {
        self.axis_values.get(name).cloned().unwrap_or(0.0)
    }

    /// Returns true if the action identified by `name` is currently active.
    pub fn action_pressed(&self, name: &str, input_map: &InputMap) -> bool {
        self.action_pressed_for_player(name, input_map, self.primary_gamepad_id)
    }

    pub fn action_pressed_for_player(
        &self,
        name: &str,
        input_map: &InputMap,
        gamepad_id: u32,
    ) -> bool {
        if let Some(binding) = input_map.actions.get(name) {
            binding
                .inputs
                .iter()
                .any(|&source| self.source_pressed_internal(source, gamepad_id))
        } else {
            false
        }
    }

    pub fn action_just_pressed(&self, name: &str, input_map: &InputMap) -> bool {
        self.action_just_pressed_for_player(name, input_map, self.primary_gamepad_id)
    }

    pub fn action_just_pressed_for_player(
        &self,
        name: &str,
        input_map: &InputMap,
        gamepad_id: u32,
    ) -> bool {
        if let Some(binding) = input_map.actions.get(name) {
            binding
                .inputs
                .iter()
                .any(|&source| self.source_just_pressed_internal(source, gamepad_id))
        } else {
            false
        }
    }

    pub fn action_just_released(&self, name: &str, input_map: &InputMap) -> bool {
        self.action_just_released_for_player(name, input_map, self.primary_gamepad_id)
    }

    pub fn action_just_released_for_player(
        &self,
        name: &str,
        input_map: &InputMap,
        gamepad_id: u32,
    ) -> bool {
        if let Some(binding) = input_map.actions.get(name) {
            binding
                .inputs
                .iter()
                .any(|&source| self.source_just_released_internal(source, gamepad_id))
        } else {
            false
        }
    }

    pub(crate) fn source_pressed_internal(&self, source: InputSource, gamepad_id: u32) -> bool {
        match source {
            InputSource::Key(k) => self.keyboard.pressed(k),
            InputSource::MouseButton(b) => self.mouse.pressed(b),
            InputSource::GamepadButton(b) => self
                .gamepad
                .as_ref()
                .is_some_and(|g| g.pressed(gamepad_id, b)),
            _ => false,
        }
    }

    pub(crate) fn source_just_pressed_internal(&self, source: InputSource, gamepad_id: u32) -> bool {
        match source {
            InputSource::Key(k) => self.keyboard.just_pressed(k),
            InputSource::MouseButton(b) => self.mouse.just_pressed(b),
            InputSource::GamepadButton(b) => self
                .gamepad
                .as_ref()
                .is_some_and(|g| g.just_pressed(gamepad_id, b)),
            _ => false,
        }
    }

    pub(crate) fn source_just_released_internal(&self, source: InputSource, gamepad_id: u32) -> bool {
        match source {
            InputSource::Key(k) => self.keyboard.just_released(k),
            InputSource::MouseButton(b) => self.mouse.just_released(b),
            InputSource::GamepadButton(b) => self
                .gamepad
                .as_ref()
                .is_some_and(|g| g.just_released(gamepad_id, b)),
            _ => false,
        }
    }

    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.keyboard.handle_event(key_event);
            }
            winit::event::WindowEvent::MouseInput { .. }
            | winit::event::WindowEvent::CursorMoved { .. }
            | winit::event::WindowEvent::MouseWheel { .. } => {
                self.mouse.handle_event(event);
            }
            winit::event::WindowEvent::Touch(touch) => {
                self.touch.handle_event(touch);
            }
            _ => {}
        }
    }

    pub fn update_axes(&mut self, input_map: &InputMap, delta_time: f32) {
        self.update_axes_for_player(input_map, delta_time, self.primary_gamepad_id);
    }

    pub fn update_axes_for_player(
        &mut self,
        input_map: &InputMap,
        delta_time: f32,
        gamepad_id: u32,
    ) {
        for (name, binding) in &input_map.axes {
            let mut target = 0.0;

            for source in &binding.positive {
                target += self.get_source_value_internal(*source, gamepad_id);
            }
            for source in &binding.negative {
                target -= self.get_source_value_internal(*source, gamepad_id);
            }

            target = target.clamp(-1.0, 1.0);

            let current = self.axis_values.get(name).cloned().unwrap_or(0.0);
            let mut new_value = current;

            if target.abs() > 0.001 {
                // If snap is on and we are moving in the opposite direction, snap to zero
                if binding.snap
                    && ((target > 0.0 && current < 0.0) || (target < 0.0 && current > 0.0))
                {
                    new_value = 0.0;
                }

                // Move towards target
                let start_value = new_value;
                let step = binding.sensitivity * delta_time;
                if target > start_value {
                    new_value = (start_value + step).min(target);
                } else {
                    new_value = (start_value - step).max(target);
                }
            } else {
                // Move towards zero (gravity)
                let step = binding.gravity * delta_time;
                if current > 0.0 {
                    new_value = (current - step).max(0.0);
                } else if current < 0.0 {
                    new_value = (current + step).min(0.0);
                }
            }

            if new_value.abs() < binding.dead_zone {
                new_value = 0.0;
            }

            self.axis_values
                .insert(name.clone(), new_value.clamp(-1.0, 1.0));
        }
    }

    fn get_source_value_internal(&self, source: InputSource, gamepad_id: u32) -> f32 {
        match source {
            InputSource::Key(k) => {
                if self.keyboard.pressed(k) {
                    1.0
                } else {
                    0.0
                }
            }
            InputSource::MouseButton(b) => {
                if self.mouse.pressed(b) {
                    1.0
                } else {
                    0.0
                }
            }
            InputSource::MouseAxis(a) => match a {
                MouseAxisType::X => self.mouse.delta().x,
                MouseAxisType::Y => self.mouse.delta().y,
                MouseAxisType::Scroll => self.mouse.scroll(),
            },
            InputSource::GamepadAxis(a) => {
                if let Some(gamepad) = &self.gamepad {
                    let axis = match a {
                        GamepadAxisType::LeftStickX => crate::gamepad::GamepadAxis::LeftStickX,
                        GamepadAxisType::LeftStickY => crate::gamepad::GamepadAxis::LeftStickY,
                        GamepadAxisType::RightStickX => crate::gamepad::GamepadAxis::RightStickX,
                        GamepadAxisType::RightStickY => crate::gamepad::GamepadAxis::RightStickY,
                        GamepadAxisType::LeftZ => crate::gamepad::GamepadAxis::LeftZ,
                        GamepadAxisType::RightZ => crate::gamepad::GamepadAxis::RightZ,
                    };
                    gamepad.axis(gamepad_id, axis)
                } else {
                    0.0
                }
            }
            InputSource::GamepadButton(b) => {
                if let Some(gamepad) = &self.gamepad {
                    if gamepad.pressed(gamepad_id, b) {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
        }
    }
}
