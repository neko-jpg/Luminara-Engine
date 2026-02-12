use std::collections::HashSet;
use luminara_core::shared_types::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct KeyboardInput {
    pub pressed: HashSet<Key>,
    pub just_pressed: HashSet<Key>,
    pub just_released: HashSet<Key>,
}

impl KeyboardInput {
    pub fn pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: Key) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: Key) -> bool {
        self.just_released.contains(&key)
    }

    pub fn clear_just_states(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn handle_event(&mut self, event: &winit::event::KeyEvent) {
        if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
            if let Some(key) = Key::from_winit(code) {
                match event.state {
                    winit::event::ElementState::Pressed => {
                        if !self.pressed.contains(&key) {
                            self.pressed.insert(key);
                            self.just_pressed.insert(key);
                        }
                    }
                    winit::event::ElementState::Released => {
                        if self.pressed.contains(&key) {
                            self.pressed.remove(&key);
                            self.just_released.insert(key);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    Space, Enter, Escape, Tab, Backspace,
    Up, Down, Left, Right,
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

impl Resource for KeyboardInput {}

impl Key {
    pub fn from_winit(code: winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode::*;
        match code {
            KeyA => Some(Key::A),
            KeyB => Some(Key::B),
            KeyC => Some(Key::C),
            KeyD => Some(Key::D),
            KeyE => Some(Key::E),
            KeyF => Some(Key::F),
            KeyG => Some(Key::G),
            KeyH => Some(Key::H),
            KeyI => Some(Key::I),
            KeyJ => Some(Key::J),
            KeyK => Some(Key::K),
            KeyL => Some(Key::L),
            KeyM => Some(Key::M),
            KeyN => Some(Key::N),
            KeyO => Some(Key::O),
            KeyP => Some(Key::P),
            KeyQ => Some(Key::Q),
            KeyR => Some(Key::R),
            KeyS => Some(Key::S),
            KeyT => Some(Key::T),
            KeyU => Some(Key::U),
            KeyV => Some(Key::V),
            KeyW => Some(Key::W),
            KeyX => Some(Key::X),
            KeyY => Some(Key::Y),
            KeyZ => Some(Key::Z),
            Digit0 => Some(Key::Num0),
            Digit1 => Some(Key::Num1),
            Digit2 => Some(Key::Num2),
            Digit3 => Some(Key::Num3),
            Digit4 => Some(Key::Num4),
            Digit5 => Some(Key::Num5),
            Digit6 => Some(Key::Num6),
            Digit7 => Some(Key::Num7),
            Digit8 => Some(Key::Num8),
            Digit9 => Some(Key::Num9),
            Space => Some(Key::Space),
            Enter => Some(Key::Enter),
            Escape => Some(Key::Escape),
            Tab => Some(Key::Tab),
            Backspace => Some(Key::Backspace),
            ArrowUp => Some(Key::Up),
            ArrowDown => Some(Key::Down),
            ArrowLeft => Some(Key::Left),
            ArrowRight => Some(Key::Right),
            ShiftLeft => Some(Key::LShift),
            ShiftRight => Some(Key::RShift),
            ControlLeft => Some(Key::LCtrl),
            ControlRight => Some(Key::RCtrl),
            AltLeft => Some(Key::LAlt),
            AltRight => Some(Key::RAlt),
            F1 => Some(Key::F1),
            F2 => Some(Key::F2),
            F3 => Some(Key::F3),
            F4 => Some(Key::F4),
            F5 => Some(Key::F5),
            F6 => Some(Key::F6),
            F7 => Some(Key::F7),
            F8 => Some(Key::F8),
            F9 => Some(Key::F9),
            F10 => Some(Key::F10),
            F11 => Some(Key::F11),
            F12 => Some(Key::F12),
            _ => None,
        }
    }
}
