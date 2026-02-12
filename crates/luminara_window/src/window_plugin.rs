use luminara_core::shared_types::{App, Plugin, AppInterface, Events};
use crate::window::WindowDescriptor;
use crate::events::WindowEvent;

pub struct WindowPlugin {
    pub descriptor: WindowDescriptor,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            descriptor: WindowDescriptor::default(),
        }
    }
}

impl Plugin for WindowPlugin {
    fn name(&self) -> &str {
        "WindowPlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(self.descriptor.clone());
        app.insert_resource(Events::<WindowEvent>::default());

        // The runner will be set during the App initialization or by the user.
        // We could set it here if we want to enforce winit_runner.
        app.set_runner(crate::winit_runner);
    }
}
