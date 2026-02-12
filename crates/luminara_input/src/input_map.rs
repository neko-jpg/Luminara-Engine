use std::collections::HashMap;
use luminara_core::shared_types::Resource;
use serde::{Deserialize, Serialize};
use crate::keyboard::Key;
use crate::mouse::MouseButton;
use crate::axis::{MouseAxisType, GamepadAxisType};
use crate::gamepad::GamepadButton;

/// A resource that defines how raw inputs are mapped to virtual axes and actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMap {
    /// Virtual axes that map to one or more input sources.
    pub axes: HashMap<String, AxisBinding>,
    /// Virtual actions that map to one or more input sources.
    pub actions: HashMap<String, ActionBinding>,
}

/// Defines how a virtual axis is calculated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisBinding {
    /// Inputs that move the axis toward 1.0.
    pub positive: Vec<InputSource>,
    /// Inputs that move the axis toward -1.0.
    pub negative: Vec<InputSource>,
    /// Speed at which the axis moves toward the target value.
    pub sensitivity: f32,
    /// Speed at which the axis moves toward 0 when no input is provided.
    pub gravity: f32,
    /// Values smaller than this will be treated as zero.
    pub dead_zone: f32,
    /// If true, the axis snaps to zero when the input direction changes.
    pub snap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBinding {
    pub inputs: Vec<InputSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    Key(Key),
    MouseButton(MouseButton),
    MouseAxis(MouseAxisType),
    GamepadAxis(GamepadAxisType),
    GamepadButton(GamepadButton),
}

impl Resource for InputMap {}

impl Default for InputMap {
    fn default() -> Self {
        Self::default_mappings()
    }
}

impl InputMap {
    pub fn default_mappings() -> Self {
        let mut axes = HashMap::new();

        // Horizontal axis
        axes.insert("horizontal".to_string(), AxisBinding {
            positive: vec![InputSource::Key(Key::Right), InputSource::Key(Key::D)],
            negative: vec![InputSource::Key(Key::Left), InputSource::Key(Key::A)],
            sensitivity: 3.0,
            gravity: 3.0,
            dead_zone: 0.05,
            snap: true,
        });

        // Vertical axis
        axes.insert("vertical".to_string(), AxisBinding {
            positive: vec![InputSource::Key(Key::Up), InputSource::Key(Key::W)],
            negative: vec![InputSource::Key(Key::Down), InputSource::Key(Key::S)],
            sensitivity: 3.0,
            gravity: 3.0,
            dead_zone: 0.05,
            snap: true,
        });

        let mut actions = HashMap::new();
        actions.insert("jump".to_string(), ActionBinding {
            inputs: vec![InputSource::Key(Key::Space), InputSource::GamepadButton(GamepadButton::South)],
        });
        actions.insert("fire".to_string(), ActionBinding {
            inputs: vec![InputSource::MouseButton(MouseButton::Left), InputSource::GamepadButton(GamepadButton::East)],
        });
        actions.insert("submit".to_string(), ActionBinding {
            inputs: vec![InputSource::Key(Key::Enter), InputSource::GamepadButton(GamepadButton::Start)],
        });
        actions.insert("cancel".to_string(), ActionBinding {
            inputs: vec![InputSource::Key(Key::Escape), InputSource::GamepadButton(GamepadButton::Select)],
        });

        Self {
            axes,
            actions,
        }
    }
}
