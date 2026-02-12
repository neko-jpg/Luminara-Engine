use luminara_core::shared_types::{App, AppInterface, CoreStage, Plugin, ResMut};
use luminara_platform::Time;
use crate::Input;
use crate::input_map::InputMap;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn name(&self) -> &str {
        "InputPlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Input::default())
           .insert_resource(InputMap::default_mappings())
           .add_system(CoreStage::PreUpdate, update_input_system);
    }
}

// In a real engine, this system would also read from an EventReader
pub fn update_input_system(
    mut input: ResMut<Input>,
    input_map: ResMut<InputMap>,
    time: ResMut<Time>,
) {
    // Clear "just" states at the beginning of the frame
    input.keyboard.clear_just_states();
    input.mouse.clear_just_states();
    if let Some(gamepad) = &mut input.gamepad {
        gamepad.clear_just_states();
    }

    #[cfg(feature = "gamepad")]
    if let Some(gamepad) = &mut input.gamepad {
        gamepad.update();
    }

    // Update axes based on the current input state and mapping
    let delta_time = time.delta_seconds();
    let map = (*input_map).clone();
    input.update_axes(&map, delta_time);
}
