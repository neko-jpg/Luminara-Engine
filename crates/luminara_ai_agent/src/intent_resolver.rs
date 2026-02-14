// ... imports ...
use crate::semantic_index::SemanticIndex;
use luminara_core::world::World;
use luminara_core::shared_types::Entity;
use std::sync::Arc;

pub struct IntentResolver {
    semantic_index: Arc<SemanticIndex>,
}

impl IntentResolver {
    pub fn new(semantic_index: Arc<SemanticIndex>) -> Self {
        Self {
            semantic_index,
        }
    }

    pub fn resolve(&self, intent: &AiIntent, world: &World) -> Result<Vec<EngineCommand>, String> {
        match intent {
            AiIntent::SpawnRelative { anchor, offset, template } => {
                let _anchor_entity = self.resolve_reference(anchor, world)?;

                let anchor_pos = luminara_math::Vec3::ZERO;
                let anchor_rot = luminara_math::Quat::IDENTITY;

                let spawn_pos = self.resolve_position(offset, anchor_pos, anchor_rot)?;

                Ok(vec![EngineCommand::SpawnEntity {
                    template: template.clone(),
                    position: spawn_pos,
                }])
            },
            _ => Ok(vec![])
        }
    }

    pub fn resolve_reference(&self, reference: &EntityReference, _world: &World) -> Result<Entity, String> {
        match reference {
            EntityReference::ByName(name) => {
                let matches = self.semantic_index.search(name, 1);
                if let Some((id, _)) = matches.first() {
                    Err(format!("Found entity ID {} but construction not impl", id))
                } else {
                    Err(format!("Entity '{}' not found", name))
                }
            },
            EntityReference::ById(_id) => {
               Err("ById not fully implemented".into())
            },
            EntityReference::Semantic(desc) => {
                let matches = self.semantic_index.search(desc, 1);
                if let Some((id, _)) = matches.first() {
                    Err(format!("Found entity ID {} via semantic search", id))
                } else {
                    Err(format!("No entity matching '{}'", desc))
                }
            }
            _ => Err("Unsupported reference type".into()),
        }
    }

    pub(crate) fn resolve_position(&self, position: &RelativePosition, anchor_pos: luminara_math::Vec3, anchor_rot: luminara_math::Quat) -> Result<luminara_math::Vec3, String> {
        use luminara_math::Vec3;
        match position {
            RelativePosition::Forward(dist) => {
                let forward = anchor_rot * Vec3::NEG_Z;
                Ok(anchor_pos + forward * *dist)
            },
            RelativePosition::Above(dist) => {
                Ok(anchor_pos + Vec3::Y * *dist)
            },
            RelativePosition::AtOffset(offset) => {
                Ok(anchor_pos + *offset)
            },
            RelativePosition::RandomInRadius(radius) => {
                Ok(anchor_pos + Vec3::new(1.0, 0.0, 0.0) * *radius)
            },
            _ => Ok(anchor_pos)
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
    Nearest { to: Box<EntityReference>, with_tag: Option<String> },
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
    SpawnEntity { template: String, position: luminara_math::Vec3 },
}
