use luminara_core::*;
use crate::models::scene::*;
use crate::sync::{ComponentRegistry, Persistent, SaveExclude};
use surrealdb::sql::{Thing, Datetime};
use std::collections::HashMap;

pub struct WorldSnapshotExporter;

impl WorldSnapshotExporter {
    pub fn export(
        world: &World,
        scene_name: &str,
        registry: &ComponentRegistry,
    ) -> SceneSnapshot {
        let mut entities = Vec::new();
        let mut components = Vec::new();
        let hierarchy = Vec::new();
        let mut entity_id_map: HashMap<Entity, String> = HashMap::new();

        for entity in world.entities() {
            if let Some(persistent) = world.get_component::<Persistent>(entity) {
                if world.get_component::<SaveExclude>(entity).is_some() {
                    continue;
                }

                let db_id = persistent.db_id.clone()
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                entity_id_map.insert(entity, db_id.clone());

                let name = format!("Entity_{}", entity.id());

                entities.push(EntityRecord {
                    id: None,
                    name,
                    scene: Thing::from(("scene", scene_name)),
                    enabled: true,
                    tags: vec![],
                    layer: 0,
                    order: 0,
                });

                // Components
                for serializer in registry.get_all() {
                    if let Some(data) = serializer.serialize(world, entity) {
                        let comp_type = serializer.type_name();

                        components.push(ComponentRecord {
                            id: None,
                            entity: Thing::from(("entity", db_id.as_str())),
                            component_type: comp_type.to_string(),
                            data,
                            schema_version: 1,
                        });
                    }
                }
            }
        }

        SceneSnapshot {
            scene_id: scene_name.to_string(),
            scene: SceneRecord {
                id: None,
                name: scene_name.to_string(),
                description: None,
                version: "1.0.0".to_string(),
                tags: vec![],
                settings: SceneSettings::default(),
                created_at: Datetime::from(chrono::Utc::now()),
                updated_at: Datetime::from(chrono::Utc::now()),
            },
            entities,
            components,
            hierarchy,
        }
    }
}
