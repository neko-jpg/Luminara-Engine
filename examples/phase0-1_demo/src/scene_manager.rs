//! Scene management module
//! Handles multiple demo scenes and switching between them
#![allow(dead_code)]

use luminara::prelude::*;
use std::collections::HashMap;

/// Scene definition
#[derive(Debug, Clone)]
pub struct SceneDefinition {
    pub name: String,
    pub description: String,
    pub file_path: String,
}

/// Scene manager resource
pub struct SceneManager {
    pub scenes: Vec<SceneDefinition>,
    pub current_scene_index: usize,
    pub scene_cache: HashMap<String, Vec<Entity>>,
}

impl Resource for SceneManager {}

impl SceneManager {
    pub fn new() -> Self {
        let scenes = vec![
            SceneDefinition {
                name: "Physics Lab".to_string(),
                description: "Classic physics playground with towers and pyramids".to_string(),
                file_path: "assets/scenes/ultimate.scene.ron".to_string(),
            },
            SceneDefinition {
                name: "Chaos Arena".to_string(),
                description: "High-energy chaos with explosions and particles".to_string(),
                file_path: "assets/scenes/chaos_arena.scene.ron".to_string(),
            },
            SceneDefinition {
                name: "Target Range".to_string(),
                description: "Shooting gallery with moving targets".to_string(),
                file_path: "assets/scenes/target_range.scene.ron".to_string(),
            },
        ];

        Self {
            scenes,
            current_scene_index: 0,
            scene_cache: HashMap::new(),
        }
    }

    pub fn current_scene(&self) -> &SceneDefinition {
        &self.scenes[self.current_scene_index]
    }

    pub fn next_scene(&mut self) {
        self.current_scene_index = (self.current_scene_index + 1) % self.scenes.len();
    }

    pub fn previous_scene(&mut self) {
        if self.current_scene_index == 0 {
            self.current_scene_index = self.scenes.len() - 1;
        } else {
            self.current_scene_index -= 1;
        }
    }

    pub fn load_current_scene(&mut self, world: &mut World) -> Result<(), String> {
        let scene_def = self.current_scene();
        let path = std::path::Path::new(&scene_def.file_path);

        // Try to load the scene
        match luminara_scene::Scene::load_from_file(path) {
            Ok(scene) => {
                // Clear existing spawned entities
                // (This would be handled by the demo state in practice)
                scene.spawn_into(world);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load scene '{}': {}", scene_def.name, e)),
        }
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}
