// Requirements 15.1-15.8
// "AgentOrchestrator... roles... decomposition... planning... conflict... messaging"

use crate::timeline::{OperationTimeline, OperationId};
use crate::intent_resolver::AiIntent;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentRole {
    SceneArchitect,
    GameplayProgrammer,
    ArtDirector,
    QAEngineer,
    ProjectDirector,
}

pub struct Agent {
    pub id: String,
    pub role: AgentRole,
    pub permissions: Vec<String>,
}

pub struct AgentOrchestrator {
    agents: HashMap<String, Agent>,
    responsibility_map: HashMap<AgentRole, Vec<String>>, // Role -> Responsibilities
    message_bus_tx: Sender<AgentMessage>,
    message_bus_rx: Receiver<AgentMessage>,
    // timeline: OperationTimeline, // Assuming reference or own
}

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub from: String,
    pub to: String, // "broadcast" or agent id
    pub content: String,
    pub intent: Option<AiIntent>,
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            agents: HashMap::new(),
            responsibility_map: HashMap::new(),
            message_bus_tx: tx,
            message_bus_rx: rx,
        }
    }

    pub fn register_agent(&mut self, id: String, role: AgentRole) {
        let permissions = match role {
            AgentRole::SceneArchitect => vec!["spawn".into(), "transform".into()],
            AgentRole::GameplayProgrammer => vec!["script".into(), "logic".into()],
            AgentRole::ArtDirector => vec!["material".into(), "lighting".into()],
            AgentRole::QAEngineer => vec!["test".into()],
            AgentRole::ProjectDirector => vec!["*".into()],
        };

        self.agents.insert(id.clone(), Agent { id, role, permissions });
    }

    pub fn decompose_task(&self, task_description: &str) -> Vec<(AgentRole, String)> {
        // Mock decomposition logic
        // Real implementation calls LLM.
        let desc = task_description.to_lowercase();
        let mut tasks = Vec::new();

        if desc.contains("level") || desc.contains("terrain") || desc.contains("geometry") || desc.contains("spawn") {
            tasks.push((AgentRole::SceneArchitect, "Layout scene geometry and entities".into()));
        }

        if desc.contains("material") || desc.contains("texture") || desc.contains("lighting") || desc.contains("color") || desc.contains("art") {
            tasks.push((AgentRole::ArtDirector, "Apply visual style and lighting".into()));
        }

        if desc.contains("mechanic") || desc.contains("logic") || desc.contains("script") || desc.contains("ai") || desc.contains("behavior") {
            tasks.push((AgentRole::GameplayProgrammer, "Implement gameplay logic".into()));
        }

        if desc.contains("test") || desc.contains("verify") || desc.contains("check") || desc.contains("qa") {
            tasks.push((AgentRole::QAEngineer, "Verify implementation".into()));
        }

        if tasks.is_empty() {
             tasks.push((AgentRole::ProjectDirector, "Analyze request and refine requirements".into()));
        }

        tasks
    }

    pub fn plan_execution(&self, subtasks: Vec<(AgentRole, String)>) -> Vec<Vec<(AgentRole, String)>> {
        // Build dependency graph -> layers
        // Mock: just sequence them in one layer or parallel if different roles?
        // Let's assume independent tasks can run parallel.
        // Return layers of tasks.

        // Simple heuristic: all unique roles can work in parallel?
        // Or just return as is.
        vec![subtasks]
    }

    pub fn send_message(&self, msg: AgentMessage) -> Result<(), String> {
        self.message_bus_tx.send(msg).map_err(|e| e.to_string())
    }

    pub fn process_messages(&self) -> Vec<AgentMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.message_bus_rx.try_recv() {
            messages.push(msg);
        }
        messages
    }

    // Changed signature to accept affected entity IDs directly for conflict detection
    pub fn detect_conflicts(&self, proposed_changes: &[(String, Vec<u64>)]) -> Vec<String> {
        // proposed_changes: List of (AgentID, List of EntityIDs modified)
        let mut conflicts = Vec::new();
        let mut touched_entities = HashMap::new();

        for (agent_id, entities) in proposed_changes {
            for &entity_id in entities {
                if let Some(other_agent) = touched_entities.get(&entity_id) {
                    if *other_agent != agent_id {
                        conflicts.push(format!("Conflict: Agent {} and Agent {} are both modifying entity {}", other_agent, agent_id, entity_id));
                    }
                } else {
                    touched_entities.insert(entity_id, agent_id);
                }
            }
        }
        conflicts
    }
}
