use crate::window::WindowDescriptor;
use luminara_core::shared_types::{App, AppInterface, Plugin};

#[derive(Default)]
pub struct WindowPlugin {
    pub descriptor: WindowDescriptor,
}

impl Plugin for WindowPlugin {
    fn name(&self) -> &str {
        "WindowPlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(self.descriptor.clone());

        // The runner will be set during the App initialization or by the user.
        // We could set it here if we want to enforce winit_runner.
        app.set_runner(crate::winit_runner);
    }
}
