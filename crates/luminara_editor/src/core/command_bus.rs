use std::sync::Arc;
use parking_lot::RwLock;
use crate::core::state::EditorState;

/// Trait for all editor commands
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self, state: &Arc<RwLock<EditorState>>);
    
    /// Get the command name for debugging/logging
    fn name(&self) -> &'static str;
}

/// Command executor that manages command execution
pub struct CommandExecutor {
    state: Arc<RwLock<EditorState>>,
}

impl CommandExecutor {
    /// Create a new command executor with the given state
    pub fn new(state: Arc<RwLock<EditorState>>) -> Self {
        Self { state }
    }
    
    /// Execute a command
    pub fn execute(&self, command: &dyn Command) {
        println!("Executing command: {}", command.name());
        command.execute(&self.state);
    }
    
    /// Get the shared state
    pub fn state(&self) -> Arc<RwLock<EditorState>> {
        self.state.clone()
    }
}

/// Command bus for publishing/subscribing to commands
pub struct CommandBus {
    executor: CommandExecutor,
}

impl CommandBus {
    /// Create a new command bus
    pub fn new(state: Arc<RwLock<EditorState>>) -> Self {
        Self {
            executor: CommandExecutor::new(state),
        }
    }
    
    /// Publish (execute) a command
    pub fn publish(&self, command: &dyn Command) {
        self.executor.execute(command);
    }
    
    /// Get the executor
    pub fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}
