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
        if task_description.contains("level") {
            vec![
                (AgentRole::SceneArchitect, "Layout geometry".into()),
                (AgentRole::ArtDirector, "Apply lighting".into())
            ]
        } else if task_description.contains("mechanic") {
            vec![
                (AgentRole::GameplayProgrammer, "Implement logic".into()),
                (AgentRole::QAEngineer, "Verify logic".into())
            ]
        } else {
            vec![(AgentRole::ProjectDirector, "Analyze request".into())]
        }
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

    pub fn detect_conflicts(&self, operations: &[(String, OperationId)]) -> Vec<String> {
        // operations: AgentID -> OpID
        // Check if operations touch same entities?
        // Need to look up op details.
        // For MVP mock: check if multiple agents submit ops.
        if operations.len() > 1 {
            vec!["Potential conflict: Multiple agents submitting operations.".into()]
        } else {
            vec![]
        }
    }
}
