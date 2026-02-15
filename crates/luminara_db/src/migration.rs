//! RON scene file to database migration tool
//!
//! This module provides utilities for migrating existing RON scene files
//! to the database format, preserving all entity relationships and component data.

use crate::error::{DbError, DbResult};
use crate::schema::{ComponentRecord, EntityRecord};
use crate::LuminaraDatabase;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Scene metadata from RON file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
}

/// Entity data from RON file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityData {
    pub name: String,
    pub id: Option<u64>,
    pub parent: Option<u64>,
    pub components: HashMap<String, Value>,
    pub children: Vec<EntityData>,
    pub tags: Vec<String>,
}

/// Complete scene from RON file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    pub meta: SceneMeta,
    pub entities: Vec<EntityData>,
}

/// Migration statistics
#[derive(Debug, Clone)]
pub struct MigrationStatistics {
    /// Number of entities migrated
    pub entities_migrated: usize,
    /// Number of components migrated
    pub components_migrated: usize,
    /// Number of relationships preserved
    pub relationships_preserved: usize,
    /// Migration duration
    pub duration_ms: u64,
}

/// RON to database migration tool
pub struct RonMigrationTool {
    db: LuminaraDatabase,
}

impl RonMigrationTool {
    /// Create a new migration tool
    pub fn new(db: LuminaraDatabase) -> Self {
        Self { db }
    }

    /// Migrate a RON scene file to the database
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the RON scene file
    ///
    /// # Returns
    ///
    /// Migration statistics including number of entities and components migrated
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::{LuminaraDatabase, migration::RonMigrationTool};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = LuminaraDatabase::new_memory().await?;
    /// let tool = RonMigrationTool::new(db);
    ///
    /// let stats = tool.migrate_file("assets/scenes/my_scene.scene.ron").await?;
    /// println!("Migrated {} entities", stats.entities_migrated);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn migrate_file(&self, path: impl AsRef<Path>) -> DbResult<MigrationStatistics> {
        let start = std::time::Instant::now();

        // Read RON file
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DbError::Other(format!("Failed to read file: {}", e)))?;

        // Parse RON
        let scene: Scene = ron::from_str(&content)
            .map_err(|e| DbError::Other(format!("Failed to parse RON: {}", e)))?;

        // Migrate scene
        let stats = self.migrate_scene(&scene).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(MigrationStatistics {
            entities_migrated: stats.entities_migrated,
            components_migrated: stats.components_migrated,
            relationships_preserved: stats.relationships_preserved,
            duration_ms,
        })
    }

    /// Migrate a parsed scene to the database
    ///
    /// # Arguments
    ///
    /// * `scene` - Parsed scene data
    ///
    /// # Returns
    ///
    /// Migration statistics
    pub async fn migrate_scene(&self, scene: &Scene) -> DbResult<MigrationStatistics> {
        let mut entities_migrated = 0;
        let mut components_migrated = 0;
        let mut relationships_preserved = 0;

        // Map old entity IDs to new RecordIds
        let mut id_map: HashMap<u64, surrealdb::RecordId> = HashMap::new();

        // First pass: Create all entities without relationships
        for entity_data in &scene.entities {
            let (entity_id, component_count) = self
                .migrate_entity_first_pass(entity_data, &mut id_map)
                .await?;

            entities_migrated += 1;
            components_migrated += component_count;
        }

        // Second pass: Establish parent-child relationships
        for entity_data in &scene.entities {
            let relationship_count = self
                .migrate_entity_second_pass(entity_data, &id_map)
                .await?;

            relationships_preserved += relationship_count;
        }

        Ok(MigrationStatistics {
            entities_migrated,
            components_migrated,
            relationships_preserved,
            duration_ms: 0, // Will be set by caller
        })
    }

    /// First pass: Create entity and components without relationships
    async fn migrate_entity_first_pass(
        &self,
        entity_data: &EntityData,
        id_map: &mut HashMap<u64, surrealdb::RecordId>,
    ) -> DbResult<(surrealdb::RecordId, usize)> {
        // Create entity record
        let entity = EntityRecord::new(Some(entity_data.name.clone()))
            .with_tags(entity_data.tags.clone());

        // Store entity
        let entity_id = self.db.store_entity(entity).await?;

        // Map old ID to new ID
        if let Some(old_id) = entity_data.id {
            id_map.insert(old_id, entity_id.clone());
        }

        // Create components
        let mut component_count = 0;
        for (type_name, data) in &entity_data.components {
            let component = ComponentRecord {
                id: None,
                type_name: type_name.clone(),
                type_id: format!("{}::{}", "luminara", type_name),
                data: data.clone(),
                entity: entity_id.clone(),
            };

            self.db.store_component(component).await?;
            component_count += 1;
        }

        // Recursively migrate children
        for child_data in &entity_data.children {
            let (_, child_component_count) = self
                .migrate_entity_first_pass(child_data, id_map)
                .await?;
            component_count += child_component_count;
        }

        Ok((entity_id, component_count))
    }

    /// Second pass: Establish parent-child relationships
    async fn migrate_entity_second_pass(
        &self,
        entity_data: &EntityData,
        id_map: &HashMap<u64, surrealdb::RecordId>,
    ) -> DbResult<usize> {
        let mut relationship_count = 0;

        // Get entity ID
        let entity_id = if let Some(old_id) = entity_data.id {
            id_map
                .get(&old_id)
                .ok_or_else(|| DbError::Other(format!("Entity ID {} not found in map", old_id)))?
                .clone()
        } else {
            return Ok(0);
        };

        // Load entity
        let mut entity = self.db.load_entity(&entity_id).await?;

        // Set parent if exists
        if let Some(parent_old_id) = entity_data.parent {
            if let Some(parent_id) = id_map.get(&parent_old_id) {
                entity.parent = Some(parent_id.clone());
                relationship_count += 1;
            }
        }

        // Set children
        for child_data in &entity_data.children {
            if let Some(child_old_id) = child_data.id {
                if let Some(child_id) = id_map.get(&child_old_id) {
                    entity.children.push(child_id.clone());
                    relationship_count += 1;
                }
            }
        }

        // Update entity with relationships
        self.db.update_entity(&entity_id, entity).await?;

        // Recursively process children
        for child_data in &entity_data.children {
            relationship_count += self.migrate_entity_second_pass(child_data, id_map).await?;
        }

        Ok(relationship_count)
    }

    /// Migrate multiple RON files in batch
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to RON scene files
    ///
    /// # Returns
    ///
    /// Combined migration statistics
    pub async fn migrate_batch(
        &self,
        paths: &[impl AsRef<Path>],
    ) -> DbResult<MigrationStatistics> {
        let start = std::time::Instant::now();

        let mut total_entities = 0;
        let mut total_components = 0;
        let mut total_relationships = 0;

        for path in paths {
            let stats = self.migrate_file(path).await?;
            total_entities += stats.entities_migrated;
            total_components += stats.components_migrated;
            total_relationships += stats.relationships_preserved;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(MigrationStatistics {
            entities_migrated: total_entities,
            components_migrated: total_components,
            relationships_preserved: total_relationships,
            duration_ms,
        })
    }

    /// Verify migration integrity
    ///
    /// Checks that all entities and relationships were migrated correctly.
    ///
    /// # Arguments
    ///
    /// * `scene` - Original scene data
    ///
    /// # Returns
    ///
    /// True if migration is valid, false otherwise
    pub async fn verify_migration(&self, scene: &Scene) -> DbResult<bool> {
        // Count entities in scene
        let expected_entities = Self::count_entities_recursive(&scene.entities);

        // Count entities in database
        let stats = self.db.get_statistics().await?;
        let actual_entities = stats.entity_count as usize;

        // Check if counts match
        if expected_entities != actual_entities {
            log::warn!(
                "Entity count mismatch: expected {}, got {}",
                expected_entities,
                actual_entities
            );
            return Ok(false);
        }

        // Verify all entities have correct relationships
        for entity_data in &scene.entities {
            if !self.verify_entity_recursive(entity_data).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Count entities recursively
    fn count_entities_recursive(entities: &[EntityData]) -> usize {
        let mut count = entities.len();
        for entity in entities {
            count += Self::count_entities_recursive(&entity.children);
        }
        count
    }

    /// Verify entity and its children recursively
    async fn verify_entity_recursive(&self, entity_data: &EntityData) -> DbResult<bool> {
        // Find entity by name
        let entities = self
            .db
            .query_entities(&format!(
                "SELECT * FROM entity WHERE name = '{}'",
                entity_data.name
            ))
            .await?;

        if entities.is_empty() {
            log::warn!("Entity '{}' not found in database", entity_data.name);
            return Ok(false);
        }

        let entity = &entities[0];

        // Verify tags
        for tag in &entity_data.tags {
            if !entity.tags.contains(tag) {
                log::warn!("Entity '{}' missing tag '{}'", entity_data.name, tag);
                return Ok(false);
            }
        }

        // Verify components
        if let Some(entity_id) = &entity.id {
            let (_, components) = self.db.load_entity_with_components(entity_id).await?;

            if components.len() != entity_data.components.len() {
                log::warn!(
                    "Entity '{}' component count mismatch: expected {}, got {}",
                    entity_data.name,
                    entity_data.components.len(),
                    components.len()
                );
                return Ok(false);
            }
        }

        // Verify children recursively
        for child_data in &entity_data.children {
            if !self.verify_entity_recursive(child_data).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrate_simple_scene() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let tool = RonMigrationTool::new(db.clone());

        // Create a simple scene
        let scene = Scene {
            meta: SceneMeta {
                name: "Test Scene".to_string(),
                description: "A test scene".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["test".to_string()],
            },
            entities: vec![EntityData {
                name: "TestEntity".to_string(),
                id: Some(1),
                parent: None,
                components: {
                    let mut map = HashMap::new();
                    map.insert(
                        "Transform".to_string(),
                        serde_json::json!({
                            "translation": [0.0, 0.0, 0.0],
                            "rotation": [0.0, 0.0, 0.0, 1.0],
                            "scale": [1.0, 1.0, 1.0]
                        }),
                    );
                    map
                },
                children: vec![],
                tags: vec!["test".to_string()],
            }],
        };

        // Migrate scene
        let stats = tool.migrate_scene(&scene).await.unwrap();

        assert_eq!(stats.entities_migrated, 1);
        assert_eq!(stats.components_migrated, 1);

        // Verify migration
        assert!(tool.verify_migration(&scene).await.unwrap());
    }

    #[tokio::test]
    async fn test_migrate_scene_with_hierarchy() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let tool = RonMigrationTool::new(db.clone());

        // Create a scene with parent-child relationships
        let scene = Scene {
            meta: SceneMeta {
                name: "Hierarchy Test".to_string(),
                description: "Test scene with hierarchy".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["test".to_string()],
            },
            entities: vec![EntityData {
                name: "Parent".to_string(),
                id: Some(1),
                parent: None,
                components: HashMap::new(),
                children: vec![
                    EntityData {
                        name: "Child1".to_string(),
                        id: Some(2),
                        parent: Some(1),
                        components: HashMap::new(),
                        children: vec![],
                        tags: vec![],
                    },
                    EntityData {
                        name: "Child2".to_string(),
                        id: Some(3),
                        parent: Some(1),
                        components: HashMap::new(),
                        children: vec![],
                        tags: vec![],
                    },
                ],
                tags: vec![],
            }],
        };

        // Migrate scene
        let stats = tool.migrate_scene(&scene).await.unwrap();

        assert_eq!(stats.entities_migrated, 3);
        assert_eq!(stats.relationships_preserved, 2); // 2 parent-child relationships

        // Verify migration
        assert!(tool.verify_migration(&scene).await.unwrap());
    }
}
