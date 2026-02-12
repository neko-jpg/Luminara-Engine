use std::collections::HashSet;
use luminara_core::shared_types::Resource;
use luminara_math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MouseInput {
    pub buttons: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
    pub just_released: HashSet<MouseButton>,
    pub position: Vec2,
    pub delta: Vec2,
    pub scroll: f32,
    pub cursor_visible: bool,
    pub cursor_grabbed: bool,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            buttons: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll: 0.0,
            cursor_visible: true,
            cursor_grabbed: false,
        }
    }
}

impl MouseInput {
    pub fn pressed(&self, button: MouseButton) -> bool {
        self.buttons.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    pub fn clear_just_states(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.delta = Vec2::ZERO;
        self.scroll = 0.0;
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let btn = MouseButton::from_winit(*button);
                match state {
                    winit::event::ElementState::Pressed => {
                        if !self.buttons.contains(&btn) {
                            self.buttons.insert(btn);
                            self.just_pressed.insert(btn);
                        }
                    }
                    winit::event::ElementState::Released => {
                        if self.buttons.contains(&btn) {
                            self.buttons.remove(&btn);
                            self.just_released.insert(btn);
                        }
                    }
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.delta += new_pos - self.position;
                self.position = new_pos;
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.scroll += y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.scroll += pos.y as f32 / 10.0; // Arbitrary scaling
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl Resource for MouseInput {}

impl MouseButton {
    pub fn from_winit(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(c) => MouseButton::Other(c),
            _ => MouseButton::Other(0), // Handle other cases if any
        }
    }
}
