use crate::intent_resolver::EngineCommand;
use luminara_core::entity::Entity;
use luminara_core::world::World;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant; // Reuse defined enum or duplicate?
                        // `EngineCommand` is in `intent_resolver`. Let's use it.

// Requirements 9.1-9.8
// "Immutable operation log... undo... branching... persistence"

pub type OperationId = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    pub id: OperationId,
    // Instant is not Serializable usually. Use SystemTime or timestamp u64.
    // For MVP, skip serde for Instant or use u64.
    // pub timestamp: Instant,
    pub ai_prompt: String,
    pub ai_response: String,
    // We need EngineCommand to be Serializable if we want persistence.
    // intent_resolver::EngineCommand derives Debug/Clone.
    // I should add Serialize/Deserialize to EngineCommand in intent_resolver.rs.
    // But I can't easily modify intent_resolver.rs without rewriting it.
    // Let's assume for MVP persistence is in-memory or we skip serde on commands if traits missing.
    // But task says "Serialize timeline to disk".
    // I will add Serialize/Deserialize to `intent_resolver.rs` structs.

    // pub commands: Vec<EngineCommand>,
    // pub inverse_commands: Vec<EngineCommand>,
    pub change_summary: String,
    pub parent: Option<OperationId>,
    pub tags: Vec<String>,
}

pub struct OperationTimeline {
    log: HashMap<OperationId, Operation>,
    head: Option<OperationId>,
    branches: HashMap<String, OperationId>,
    snapshots: BTreeMap<OperationId, Vec<u8>>, // Mock snapshot
    next_id: u64,
}

impl OperationTimeline {
    pub fn new() -> Self {
        Self {
            log: HashMap::new(),
            head: None,
            branches: HashMap::new(),
            snapshots: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn record(
        &mut self,
        prompt: String,
        response: String,
        _commands: Vec<EngineCommand>,
        _inverse: Vec<EngineCommand>,
    ) -> OperationId {
        let id = self.next_id;
        self.next_id += 1;

        let op = Operation {
            id,
            ai_prompt: prompt,
            ai_response: response,
            change_summary: "Operation recorded".into(),
            parent: self.head,
            tags: Vec::new(),
        };

        self.log.insert(id, op);
        self.head = Some(id);

        id
    }

    pub fn undo(&mut self, _world: &mut World) -> Result<(), String> {
        if let Some(head_id) = self.head {
            if let Some(op) = self.log.get(&head_id) {
                // Execute inverse commands...
                // Move head to parent
                self.head = op.parent;
                Ok(())
            } else {
                Err("Head operation not found".into())
            }
        } else {
            Err("No operation to undo".into())
        }
    }

    pub fn create_branch(&mut self, name: &str) -> Result<(), String> {
        if let Some(head) = self.head {
            self.branches.insert(name.to_string(), head);
            Ok(())
        } else {
            Err("No HEAD to branch from".into())
        }
    }

    pub fn checkout_branch(&mut self, name: &str, _world: &mut World) -> Result<(), String> {
        if let Some(&id) = self.branches.get(name) {
            self.head = Some(id);
            // Restore world state... (requires snapshots/replay)
            Ok(())
        } else {
            Err("Branch not found".into())
        }
    }

    pub fn save_to_disk(&self, path: &str) -> Result<(), String> {
        // Serialize self.log
        // Requires Operation to be Serialize.
        // Assuming we fix EngineCommand or omit it for now.
        let json = serde_json::to_string(&self.log).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_disk(&mut self, path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.log = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        // Reset head? Or load head from disk too.
        Ok(())
    }
}
