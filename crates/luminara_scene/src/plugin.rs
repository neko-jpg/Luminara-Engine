use crate::hierarchy::transform_propagate_system;
use crate::motor_transform::{
    motor_transform_propagate_system, sync_global_motor_to_transform_system,
    sync_motor_to_transform_system, sync_transform_to_motor_system,
};
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

        // Register motor transform sync systems (run before transform propagation)
        // These systems sync between TransformMotor and Transform components
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, sync_motor_to_transform_system);
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, sync_transform_to_motor_system);

        // Register transform propagation systems
        // Standard transform propagation for Transform components
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, transform_propagate_system);
        
        // Motor-based transform propagation for TransformMotor components
        // This runs in parallel with standard propagation and uses motor composition
        app.add_system::<ExclusiveMarker>(CoreStage::PostUpdate, motor_transform_propagate_system);
        
        // Sync global motor transforms to global transforms for compatibility
        app.add_system::<ExclusiveMarker>(
            CoreStage::PostUpdate,
            sync_global_motor_to_transform_system,
        );
    }
}
