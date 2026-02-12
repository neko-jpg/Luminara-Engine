use crate::{PlatformInfo, Time};
use luminara_core::shared_types::{App, AppInterface, CoreStage, Plugin, ResMut};

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn name(&self) -> &str {
        "PlatformPlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Time::new())
           .insert_resource(PlatformInfo::current())
           .add_system(CoreStage::PreUpdate, time_update_system);
    }
}

fn time_update_system(mut time: ResMut<Time>) {
    time.update();
}
