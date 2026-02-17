use crate::semantic_index::SemanticIndex;
use luminara_core::shared_types::Entity;
use luminara_core::world::World;
use std::sync::Arc;

/// Intent resolver that handles entity reference resolution and relative position computation.
/// Supports fuzzy name matching, semantic queries, and various reference types.
pub struct IntentResolver {
    semantic_index: Arc<SemanticIndex>,
}

impl IntentResolver {
    pub fn new(semantic_index: Arc<SemanticIndex>) -> Self {
        Self { semantic_index }
    }

    /// Resolve an AI intent into concrete engine commands
    pub fn resolve(&self, intent: &AiIntent, world: &World) -> Result<Vec<EngineCommand>, String> {
        match intent {
            AiIntent::SpawnRelative {
                anchor,
                offset,
                template,
            } => {
                let anchor_entity = self.resolve_reference(anchor, world)?;
                
                // Get anchor transform from world
                let (anchor_pos, anchor_rot) = self.get_entity_transform(anchor_entity, world)?;
                
                let spawn_pos = self.resolve_position(offset, anchor_pos, anchor_rot)?;

                Ok(vec![EngineCommand::SpawnEntity {
                    template: template.clone(),
                    position: spawn_pos,
                }])
            }
            AiIntent::ModifyMatching => Ok(vec![]),
            AiIntent::AttachBehavior => Ok(vec![]),
        }
    }

    /// Resolve an entity reference to a concrete entity
    /// Supports: ByName, ById, ByTag, ByComponent, Nearest, Semantic
    pub fn resolve_reference(
        &self,
        reference: &EntityReference,
        world: &World,
    ) -> Result<Entity, String> {
        match reference {
            EntityReference::ByName(name) => self.resolve_by_name(name, world),
            EntityReference::ById(id) => self.resolve_by_id(*id, world),
            EntityReference::ByTag(tag) => self.resolve_by_tag(tag, world),
            EntityReference::ByComponent(component) => self.resolve_by_component(component, world),
            EntityReference::Nearest { to, with_tag } => {
                self.resolve_nearest(to, with_tag.as_deref(), world)
            }
            EntityReference::Semantic(desc) => self.resolve_semantic(desc, world),
        }
    }

    /// Resolve entity by name with fuzzy matching and suggestions
    fn resolve_by_name(&self, name: &str, world: &World) -> Result<Entity, String> {
        // Try exact match first via semantic index
        let matches = self.semantic_index.search(name, 5);
        
        if matches.is_empty() {
            return Err(format!(
                "Entity '{}' not found. No similar entities available.",
                name
            ));
        }
        
        // Check if first match is exact or very close
        let (best_id, best_score) = &matches[0];
        
        if *best_score > 0.9 {
            // High confidence match
            self.construct_entity(*best_id as u64, world)
        } else {
            // Provide suggestions for similar names
            let suggestions: Vec<String> = matches
                .iter()
                .take(3)
                .map(|(id, score)| format!("ID {} (score: {:.2})", id, score))
                .collect();
            
            Err(format!(
                "Entity '{}' not found. Did you mean one of these? {}",
                name,
                suggestions.join(", ")
            ))
        }
    }

    /// Resolve entity by ID
    fn resolve_by_id(&self, id: u64, world: &World) -> Result<Entity, String> {
        self.construct_entity(id, world)
    }

    /// Resolve entity by tag (returns first match)
    fn resolve_by_tag(&self, tag: &str, world: &World) -> Result<Entity, String> {
        // Search semantic index for entities with this tag
        let query = format!("tag:{}", tag);
        let matches = self.semantic_index.search(&query, 1);
        
        if let Some((id, _)) = matches.first() {
            self.construct_entity(*id as u64, world)
        } else {
            Err(format!("No entity found with tag '{}'", tag))
        }
    }

    /// Resolve entity by component type (returns first match)
    fn resolve_by_component(&self, component: &str, world: &World) -> Result<Entity, String> {
        // Search semantic index for entities with this component
        let query = format!("component:{}", component);
        let matches = self.semantic_index.search(&query, 1);
        
        if let Some((id, _)) = matches.first() {
            self.construct_entity(*id as u64, world)
        } else {
            Err(format!("No entity found with component '{}'", component))
        }
    }

    /// Resolve nearest entity to a reference point
    fn resolve_nearest(
        &self,
        to: &EntityReference,
        with_tag: Option<&str>,
        world: &World,
    ) -> Result<Entity, String> {
        // First resolve the reference entity
        let reference_entity = self.resolve_reference(to, world)?;
        let (ref_pos, _) = self.get_entity_transform(reference_entity, world)?;
        
        // Search for candidates
        let candidates = if let Some(tag) = with_tag {
            let query = format!("tag:{}", tag);
            self.semantic_index.search(&query, 100)
        } else {
            // Get all entities (limited to 100 for performance)
            self.semantic_index.search("", 100)
        };
        
        if candidates.is_empty() {
            return Err("No candidate entities found for nearest search".to_string());
        }
        
        // Find nearest by distance
        let mut nearest_id = candidates[0].0;
        let mut nearest_dist = f32::MAX;
        
        for (id, _) in candidates {
            if let Ok(entity) = self.construct_entity(id as u64, world) {
                if let Ok((pos, _)) = self.get_entity_transform(entity, world) {
                    let dist = (pos - ref_pos).length();
                    if dist < nearest_dist && dist > 0.0 {
                        // Exclude the reference entity itself
                        nearest_dist = dist;
                        nearest_id = id;
                    }
                }
            }
        }
        
        if nearest_dist == f32::MAX {
            Err("No valid nearest entity found".to_string())
        } else {
            self.construct_entity(nearest_id as u64, world)
        }
    }

    /// Resolve entity using semantic/natural language query
    fn resolve_semantic(&self, desc: &str, world: &World) -> Result<Entity, String> {
        let matches = self.semantic_index.search(desc, 5);
        
        if matches.is_empty() {
            return Err(format!("No entity matching '{}'", desc));
        }
        
        // Return best match with suggestions if confidence is low
        let (best_id, best_score) = &matches[0];
        
        if *best_score > 0.7 {
            self.construct_entity(*best_id as u64, world)
        } else {
            let suggestions: Vec<String> = matches
                .iter()
                .take(3)
                .map(|(id, score)| format!("ID {} (score: {:.2})", id, score))
                .collect();
            
            Err(format!(
                "Low confidence match for '{}'. Suggestions: {}",
                desc,
                suggestions.join(", ")
            ))
        }
    }

    /// Construct an Entity from an ID (placeholder implementation)
    fn construct_entity(&self, id: u64, _world: &World) -> Result<Entity, String> {
        // For now, return a placeholder entity with generation 0
        // In a full implementation, this would query the world for the entity
        Ok(Entity::from_raw(id as u32, 0))
    }

    /// Get entity transform from world (placeholder implementation)
    fn get_entity_transform(
        &self,
        _entity: Entity,
        _world: &World,
    ) -> Result<(luminara_math::Vec3, luminara_math::Quat), String> {
        // Placeholder: return identity transform
        // In full implementation, query Transform component from world
        Ok((luminara_math::Vec3::ZERO, luminara_math::Quat::IDENTITY))
    }

    /// Resolve relative position to absolute position at execution time
    /// This is public for testing purposes
    pub fn resolve_position(
        &self,
        position: &RelativePosition,
        anchor_pos: luminara_math::Vec3,
        anchor_rot: luminara_math::Quat,
    ) -> Result<luminara_math::Vec3, String> {
        use luminara_math::Vec3;
        match position {
            RelativePosition::Forward(dist) => {
                let forward = anchor_rot * Vec3::NEG_Z;
                Ok(anchor_pos + forward * *dist)
            }
            RelativePosition::Above(dist) => {
                let up = anchor_rot * Vec3::Y;
                Ok(anchor_pos + up * *dist)
            }
            RelativePosition::AtOffset(offset) => {
                // Apply rotation to offset for local space
                let rotated_offset = anchor_rot * *offset;
                Ok(anchor_pos + rotated_offset)
            }
            RelativePosition::RandomInRadius(radius) => {
                // Simple random offset (in real implementation, use proper RNG)
                let angle: f32 = 0.5; // Placeholder
                let offset = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
                Ok(anchor_pos + offset)
            }
            RelativePosition::RandomReachable { radius } => {
                // Placeholder: similar to RandomInRadius
                let angle: f32 = 0.7;
                let offset = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
                Ok(anchor_pos + offset)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AiIntent {
    SpawnRelative {
        anchor: EntityReference,
        offset: RelativePosition,
        template: String,
    },
    ModifyMatching,
    AttachBehavior,
}

#[derive(Debug, Clone)]
pub enum EntityReference {
    ByName(String),
    ById(u64),
    ByTag(String),
    ByComponent(String),
    Nearest {
        to: Box<EntityReference>,
        with_tag: Option<String>,
    },
    Semantic(String),
}

#[derive(Debug, Clone)]
pub enum RelativePosition {
    Forward(f32),
    Above(f32),
    AtOffset(luminara_math::Vec3),
    RandomInRadius(f32),
    RandomReachable { radius: f32 },
}

#[derive(Debug, Clone)]
pub enum EngineCommand {
    SpawnEntity {
        template: String,
        position: luminara_math::Vec3,
    },
}
