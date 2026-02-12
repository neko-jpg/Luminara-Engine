use crate::hierarchy::transform_propagate_system;
use luminara_core::system::ExclusiveMarker;
use luminara_core::{App, AppInterface, CoreStage, Plugin};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn name(&self) -> &str {
        "ScenePlugin"
    }

    fn build(&self, app: &mut App) {
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, transform_propagate_system);
    }
}
