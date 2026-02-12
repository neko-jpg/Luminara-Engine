#[cfg(feature = "gamepad")]
use gilrs::Gilrs;
use luminara_core::shared_types::Resource;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct GamepadInput {
    #[cfg(feature = "gamepad")]
    pub(crate) gilrs: Gilrs,
    pub pressed: HashMap<u32, HashSet<GamepadButton>>,
    pub just_pressed: HashMap<u32, HashSet<GamepadButton>>,
    pub just_released: HashMap<u32, HashSet<GamepadButton>>,
    pub axes: HashMap<u32, HashMap<GamepadAxis, f32>>,
}

impl Resource for GamepadInput {}

impl GamepadInput {
    #[cfg(feature = "gamepad")]
    pub fn new() -> Option<Self> {
        Gilrs::new().ok().map(|gilrs| Self {
            gilrs,
            pressed: HashMap::new(),
            just_pressed: HashMap::new(),
            just_released: HashMap::new(),
            axes: HashMap::new(),
        })
    }

    #[cfg(not(feature = "gamepad"))]
    pub fn new() -> Option<Self> {
        Some(Self {
            pressed: HashMap::new(),
            just_pressed: HashMap::new(),
            just_released: HashMap::new(),
            axes: HashMap::new(),
        })
    }

    pub fn pressed(&self, gamepad_id: u32, button: GamepadButton) -> bool {
        self.pressed
            .get(&gamepad_id)
            .is_some_and(|b| b.contains(&button))
    }

    pub fn just_pressed(&self, gamepad_id: u32, button: GamepadButton) -> bool {
        self.just_pressed
            .get(&gamepad_id)
            .is_some_and(|b| b.contains(&button))
    }

    pub fn just_released(&self, gamepad_id: u32, button: GamepadButton) -> bool {
        self.just_released
            .get(&gamepad_id)
            .is_some_and(|b| b.contains(&button))
    }

    pub fn axis(&self, gamepad_id: u32, axis: GamepadAxis) -> f32 {
        self.axes
            .get(&gamepad_id)
            .and_then(|a| a.get(&axis))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn clear_just_states(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    #[cfg(feature = "gamepad")]
    pub fn update(&mut self) {
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            let gid = Into::<usize>::into(id) as u32;
            match event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(b) = GamepadButton::from_gilrs(button) {
                        let pressed = self.pressed.entry(gid).or_default();
                        if !pressed.contains(&b) {
                            pressed.insert(b);
                            self.just_pressed.entry(gid).or_default().insert(b);
                        }
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(b) = GamepadButton::from_gilrs(button) {
                        if self.pressed.entry(gid).or_default().remove(&b) {
                            self.just_released.entry(gid).or_default().insert(b);
                        }
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    if let Some(a) = GamepadAxis::from_gilrs(axis) {
                        self.axes.entry(gid).or_default().insert(a, value);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    L1,
    R1,
    L2,
    R2,
    Select,
    Start,
    Mode,
    L3,
    R3,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl GamepadButton {
    #[cfg(feature = "gamepad")]
    pub fn from_gilrs(button: gilrs::Button) -> Option<Self> {
        match button {
            gilrs::Button::South => Some(GamepadButton::South),
            gilrs::Button::East => Some(GamepadButton::East),
            gilrs::Button::North => Some(GamepadButton::North),
            gilrs::Button::West => Some(GamepadButton::West),
            gilrs::Button::LeftTrigger => Some(GamepadButton::L1),
            gilrs::Button::RightTrigger => Some(GamepadButton::R1),
            gilrs::Button::LeftTrigger2 => Some(GamepadButton::L2),
            gilrs::Button::RightTrigger2 => Some(GamepadButton::R2),
            gilrs::Button::Select => Some(GamepadButton::Select),
            gilrs::Button::Start => Some(GamepadButton::Start),
            gilrs::Button::Mode => Some(GamepadButton::Mode),
            gilrs::Button::LeftThumb => Some(GamepadButton::L3),
            gilrs::Button::RightThumb => Some(GamepadButton::R3),
            gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
            gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
            gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
            gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftZ,
    RightZ,
}

impl GamepadAxis {
    #[cfg(feature = "gamepad")]
    pub fn from_gilrs(axis: gilrs::Axis) -> Option<Self> {
        match axis {
            gilrs::Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
            gilrs::Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
            gilrs::Axis::RightStickX => Some(GamepadAxis::RightStickX),
            gilrs::Axis::RightStickY => Some(GamepadAxis::RightStickY),
            gilrs::Axis::LeftZ => Some(GamepadAxis::LeftZ),
            gilrs::Axis::RightZ => Some(GamepadAxis::RightZ),
            _ => None,
        }
    }
}
