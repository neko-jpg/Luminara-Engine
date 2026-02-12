use crate::scene::EntityData;
use luminara_core::{Entity, World};
use std::collections::HashMap;

/// 再利用可能なEntityテンプレート
pub struct Prefab {
    pub template: EntityData,
}

impl Prefab {
    pub fn instantiate(&self, world: &mut World) -> Entity {
        let mut id_map = HashMap::new();
        let mut spawned_entities = Vec::new();

        // We need a scene instance to call the recursive spawn method
        // or we can move the logic to a place where both can use it.
        // For now, we'll just use a placeholder scene.
        let scene = crate::scene::Scene {
            meta: crate::scene::SceneMeta {
                name: "Prefab".to_string(),
                description: "".to_string(),
                version: "".to_string(),
                tags: vec![],
            },
            entities: vec![],
        };

        scene.spawn_entity_recursive(
            world,
            &self.template,
            None,
            &mut id_map,
            &mut spawned_entities,
        )
    }
}
