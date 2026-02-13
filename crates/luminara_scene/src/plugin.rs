use crate::hierarchy::transform_propagate_system;
use crate::scene::init_default_component_schemas;
use luminara_core::system::ExclusiveMarker;
use luminara_core::{App, AppInterface, CoreStage, Plugin};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn name(&self) -> &str {
        "ScenePlugin"
    }

    fn build(&self, app: &mut App) {
        // Initialize component schemas for AI introspection
        init_default_component_schemas();
        
        // Register transform propagation system
        // Note: According to the design document, this should run in a dedicated
        // TransformPropagate stage between PostUpdate and PreRender. For now, we
        // register it in PostUpdate which ensures it runs after game logic updates
        // but before rendering.
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, transform_propagate_system);
    }
}
