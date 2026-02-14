use crate::input_map::InputMap;
use crate::Input;
use luminara_core::shared_types::{App, AppInterface, CoreStage, Plugin, ResMut};
use luminara_core::system::FunctionMarker;
use luminara_platform::Time;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn name(&self) -> &str {
        "InputPlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Input::default())
            .insert_resource(InputMap::default_mappings())
            // Update axes at PreUpdate (does NOT clear just-states so that
            // user systems in Update can read just_pressed / just_released).
            .add_system::<(
                FunctionMarker,
                ResMut<'static, Input>,
                ResMut<'static, InputMap>,
                ResMut<'static, Time>,
            )>(CoreStage::PreUpdate, update_input_system)
            // Clear just-states at PostUpdate, AFTER user systems have consumed them.
            .add_system::<(FunctionMarker, ResMut<'static, Input>)>(
                CoreStage::PostUpdate,
                clear_input_just_states_system,
            );
    }
}

/// PreUpdate: update virtual axes and poll gamepads.  
/// just_pressed / just_released are left intact for downstream systems.
pub fn update_input_system(
    mut input: ResMut<Input>,
    input_map: ResMut<InputMap>,
    time: ResMut<Time>,
) {
    #[cfg(feature = "gamepad")]
    if let Some(gamepad) = &mut input.gamepad {
        gamepad.update();
    }

    // Update mouse smoothing
    input.mouse.update_smoothing();

    // Update axes based on the current input state and mapping
    let delta_time = time.delta_seconds();
    let map = (*input_map).clone();
    input.update_axes(&map, delta_time);
}

/// PostUpdate: clear "just" states so they don't persist into the next frame.
/// This runs AFTER Update, FixedUpdate, and PostUpdate user systems, ensuring
/// every system that needs just_pressed/just_released has already seen them.
pub fn clear_input_just_states_system(mut input: ResMut<Input>) {
    input.keyboard.clear_just_states();
    input.mouse.clear_just_states();
    if let Some(gamepad) = &mut input.gamepad {
        gamepad.clear_just_states();
    }
}
