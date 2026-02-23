//! Global Search Commands (Vizia version)

use crate::core::command_bus::{Command, CommandBus};
use crate::core::state::EditorStateManager;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct ToggleGlobalSearchCommand;

impl ToggleGlobalSearchCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToggleGlobalSearchCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ToggleGlobalSearchCommand {
    fn execute(&self, _state: &EditorStateManager) {
        println!("Toggle Global Search");
    }

    fn name(&self) -> &'static str {
        "ToggleGlobalSearch"
    }
}

pub struct DuplicateEntityCommand;

impl DuplicateEntityCommand {
    pub fn new(_entity_id: u64) -> Self {
        Self
    }
}

impl Command for DuplicateEntityCommand {
    fn execute(&self, _state: &EditorStateManager) {
        println!("Duplicate Entity");
    }

    fn name(&self) -> &'static str {
        "DuplicateEntity"
    }
}
