use crate::services::engine_bridge::EditorCommand;
use luminara_core::{World, Entity, Transform};

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
        let new_entity = world.spawn(()).id();

        // Try to copy Transform component if it exists
        if let Some(transform) = world.get::<Transform>(self.entity) {
            let t = *transform;
            world.entity_mut(new_entity).insert(t);
        }

        println!("Duplicated entity {:?} to {:?}", self.entity, new_entity);
    }
}
