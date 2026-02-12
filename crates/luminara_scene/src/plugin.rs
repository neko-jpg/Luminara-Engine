use luminara_core::{Plugin, App, AppInterface, CoreStage};
use crate::hierarchy::transform_propagate_system;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn name(&self) -> &str {
        "ScenePlugin"
    }

    fn build(&self, app: &mut App) {
        app.add_system(CoreStage::PostUpdate, transform_propagate_system);
    }
}
