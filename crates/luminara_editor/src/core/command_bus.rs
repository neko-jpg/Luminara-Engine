//! Command Bus (Vizia version)

use crate::core::state::EditorStateManager;

pub trait Command: Send + Sync {
    fn execute(&self, state: &EditorStateManager);
    fn name(&self) -> &'static str;
}

pub struct CommandExecutor {
    state: EditorStateManager,
}

impl CommandExecutor {
    pub fn new(state: EditorStateManager) -> Self {
        Self { state }
    }

    pub fn execute(&self, command: &dyn Command) {
        println!("Executing command: {}", command.name());
        command.execute(&self.state);
    }

    pub fn state(&self) -> &EditorStateManager {
        &self.state
    }
}

pub struct CommandBus {
    executor: CommandExecutor,
}

impl CommandBus {
    pub fn new(state: EditorStateManager) -> Self {
        Self {
            executor: CommandExecutor::new(state),
        }
    }

    pub fn publish(&self, command: &dyn Command) {
        self.executor.execute(command);
    }

    pub fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}
