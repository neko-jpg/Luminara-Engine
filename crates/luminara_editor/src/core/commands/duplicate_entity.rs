use crate::services::engine_bridge::EditorCommand;
use luminara_core::{World, Entity};
use luminara_math::Transform;

/// Command to duplicate an entity
#[derive(Debug, Clone)]
pub struct DuplicateEntityCommand {
    entity: Entity,
}

impl DuplicateEntityCommand {
    /// Create a new duplicate entity command
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl EditorCommand for DuplicateEntityCommand {
    fn execute(&mut self, world: &mut World) {
        // Create new entity
        let new_entity = world.spawn();

        // Try to copy Transform component if it exists
        // Note: In a real implementation, we would use reflection to clone all components.
        // For now, we manually clone known components like Transform.
        if let Some(transform) = world.get_component::<Transform>(self.entity) {
            let _ = world.add_component(new_entity, *transform);
        }

        // Also copy Name if we had a Name component (not standard in luminara_core yet?)
        // Assuming just Transform for now.

        println!("Duplicated entity {:?} to {:?}", self.entity, new_entity);
    }

    fn name(&self) -> &str {
        "DuplicateEntity"
    }
}
