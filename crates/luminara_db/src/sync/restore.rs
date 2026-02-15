use luminara_core::*;
use crate::models::scene::*;
use crate::sync::{ComponentRegistry, Persistent};
use std::collections::HashMap;

pub struct WorldSnapshotImporter;

impl WorldSnapshotImporter {
    pub fn import(
        world: &mut World,
        snapshot: &SceneSnapshot,
        registry: &ComponentRegistry,
    ) -> Result<(), crate::error::DbError> {
        let mut db_id_to_entity: HashMap<String, Entity> = HashMap::new();

        // Phase 1: Entity
        for entity_record in &snapshot.entities {
            let thing_raw = entity_record.id.as_ref()
                .map(|t| t.to_raw())
                .unwrap_or_else(|| format!("entity:{}", uuid::Uuid::new_v4()));

            let entity = world.spawn();

            let db_id_pure = thing_raw.strip_prefix("entity:").unwrap_or(&thing_raw).to_string();

            world.add_component(entity, Persistent {
                auto_save: true,
                db_id: Some(db_id_pure.clone()),
                last_saved_hash: None,
            }).map_err(|e| crate::error::DbError::InvalidData(format!("Failed to add Persistent: {}", e)))?;

            db_id_to_entity.insert(thing_raw, entity);
        }

        // Phase 2: Components
        for component_record in &snapshot.components {
            let entity_ref = component_record.entity.to_raw();

            if let Some(&entity) = db_id_to_entity.get(&entity_ref) {
                if let Some(serializer) = registry.get(&component_record.component_type) {
                    serializer.deserialize(world, entity, &component_record.data)?;
                } else {
                    tracing::warn!(
                        "Unknown component type: {}. Skipping.",
                        component_record.component_type
                    );
                }
            } else {
                tracing::warn!("Component references unknown entity: {}", entity_ref);
            }
        }

        Ok(())
    }
}
