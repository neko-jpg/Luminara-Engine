//! Undo/Redo command pattern for editor operations.
//!
//! This module provides a command pattern implementation that enables undo/redo
//! functionality for all engine operations. Each command captures sufficient state
//! to enable both execution and reversal.
//!
//! # Requirements
//! - Requirement 9.1: Command trait with execute and undo methods
//! - Requirement 9.2: Record sufficient state to enable undo

use crate::world::World;
use std::fmt;

/// Result type for command operations
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Error type for command operations
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("World error: {0}")]
    WorldError(#[from] crate::error::WorldError),
    #[error("Command error: {0}")]
    CommandError(String),
    #[error("No commands to undo")]
    NoUndo,
    #[error("No commands to redo")]
    NoRedo,
}

/// Trait for undoable commands.
///
/// All editor operations should implement this trait to enable undo/redo functionality.
/// Commands must capture sufficient state during execution to enable complete reversal.
///
/// # Requirements
/// - Requirement 9.1: Provides execute and undo methods
/// - Requirement 9.2: Records sufficient state for undo
pub trait UndoCommand: Send + Sync {
    /// Execute the command, modifying the world state.
    ///
    /// This method should capture any state needed for undo before making changes.
    /// If execution fails, the command should leave the world in a consistent state.
    ///
    /// # Errors
    /// Returns an error if the command cannot be executed.
    fn execute(&mut self, world: &mut World) -> CommandResult<()>;

    /// Undo the command, restoring the previous world state.
    ///
    /// This method should use the state captured during execute() to restore
    /// the exact previous state.
    ///
    /// # Errors
    /// Returns an error if the command cannot be undone.
    fn undo(&mut self, world: &mut World) -> CommandResult<()>;

    /// Get a human-readable description of this command.
    ///
    /// This is used for displaying command history in the editor UI.
    fn description(&self) -> String;

    /// Check if this command can be merged with another command.
    ///
    /// Command merging is an optimization that combines multiple similar commands
    /// into a single command. For example, multiple consecutive "move entity"
    /// commands can be merged into a single command.
    ///
    /// Default implementation returns false (no merging).
    fn can_merge(&self, _other: &dyn UndoCommand) -> bool {
        false
    }

    /// Merge another command into this command.
    ///
    /// This is only called if `can_merge` returned true.
    ///
    /// # Errors
    /// Returns an error if merging fails.
    fn merge(&mut self, _other: Box<dyn UndoCommand>) -> CommandResult<()> {
        Err(CommandError::CommandError(
            "Command merging not implemented".to_string(),
        ))
    }
}

impl fmt::Debug for dyn UndoCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UndoCommand({})", self.description())
    }
}

/// History of executed commands with undo/redo support.
///
/// CommandHistory maintains a stack of executed commands and supports:
/// - Undo: Reverse the last executed command
/// - Redo: Re-execute a previously undone command
/// - Command merging: Combine similar consecutive commands
///
/// # Requirements
/// - Requirement 9.1: Provides undo/redo functionality
/// - Requirement 9.2: Records command history for undo
pub struct CommandHistory {
    /// Stack of executed commands
    history: Vec<Box<dyn UndoCommand>>,
    /// Current position in history (index of next command to redo)
    current: usize,
    /// Maximum number of commands to keep in history
    max_size: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl CommandHistory {
    /// Create a new command history with the specified maximum size.
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of commands to keep in history
    pub fn new(max_size: usize) -> Self {
        Self {
            history: Vec::new(),
            current: 0,
            max_size,
        }
    }

    /// Execute a command and add it to the history.
    ///
    /// This method:
    /// 1. Executes the command
    /// 2. Discards any commands after the current position (redo history)
    /// 3. Attempts to merge with the previous command if possible
    /// 4. Adds the command to history
    /// 5. Trims history if it exceeds max_size
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `world` - The world to execute the command on
    ///
    /// # Errors
    /// Returns an error if command execution fails.
    ///
    /// # Requirements
    /// - Requirement 9.1: Executes command and records for undo
    /// - Requirement 9.2: Captures state for undo
    pub fn execute(
        &mut self,
        mut command: Box<dyn UndoCommand>,
        world: &mut World,
    ) -> CommandResult<()> {
        // Execute the command
        command.execute(world)?;

        // Discard redo history
        self.history.truncate(self.current);

        // Try to merge with previous command
        if let Some(prev_command) = self.history.last_mut() {
            if prev_command.can_merge(&*command) {
                prev_command.merge(command)?;
                return Ok(());
            }
        }

        // Add command to history
        self.history.push(command);
        self.current = self.history.len();

        // Trim history if needed
        if self.history.len() > self.max_size {
            let remove_count = self.history.len() - self.max_size;
            self.history.drain(0..remove_count);
            self.current = self.history.len();
        }

        Ok(())
    }

    /// Undo the last executed command.
    ///
    /// # Arguments
    /// * `world` - The world to undo the command on
    ///
    /// # Errors
    /// Returns an error if:
    /// - There are no commands to undo
    /// - The undo operation fails
    ///
    /// # Requirements
    /// - Requirement 9.3: Restores exact previous state
    pub fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if self.current == 0 {
            return Err(CommandError::NoUndo);
        }

        self.current -= 1;
        self.history[self.current].undo(world)?;

        Ok(())
    }

    /// Redo the next command in history.
    ///
    /// # Arguments
    /// * `world` - The world to redo the command on
    ///
    /// # Errors
    /// Returns an error if:
    /// - There are no commands to redo
    /// - The redo operation fails
    pub fn redo(&mut self, world: &mut World) -> CommandResult<()> {
        if self.current >= self.history.len() {
            return Err(CommandError::NoRedo);
        }

        self.history[self.current].execute(world)?;
        self.current += 1;

        Ok(())
    }

    /// Clear all command history.
    pub fn clear(&mut self) {
        self.history.clear();
        self.current = 0;
    }

    /// Get the command at the specified index.
    ///
    /// # Arguments
    /// * `index` - Index of the command to retrieve
    ///
    /// # Returns
    /// The command at the specified index, or None if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&dyn UndoCommand> {
        self.history.get(index).map(|cmd| &**cmd)
    }

    /// Get the number of commands in history.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Get the current position in history.
    ///
    /// This is the index of the next command to redo.
    pub fn current_position(&self) -> usize {
        self.current
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.current < self.history.len()
    }

    /// Get an iterator over all commands in history.
    pub fn iter(&self) -> impl Iterator<Item = &dyn UndoCommand> {
        self.history.iter().map(|cmd| &**cmd)
    }

    /// Get the description of the command that would be undone.
    pub fn undo_description(&self) -> Option<String> {
        if self.can_undo() {
            Some(self.history[self.current - 1].description())
        } else {
            None
        }
    }

    /// Get the description of the command that would be redone.
    pub fn redo_description(&self) -> Option<String> {
        if self.can_redo() {
            Some(self.history[self.current].description())
        } else {
            None
        }
    }
}

impl fmt::Debug for CommandHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandHistory")
            .field("len", &self.history.len())
            .field("current", &self.current)
            .field("max_size", &self.max_size)
            .field("can_undo", &self.can_undo())
            .field("can_redo", &self.can_redo())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock command for testing
    struct TestCommand {
        value: i32,
        executed: bool,
    }

    impl TestCommand {
        fn new(value: i32) -> Self {
            Self {
                value,
                executed: false,
            }
        }
    }

    impl UndoCommand for TestCommand {
        fn execute(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = true;
            Ok(())
        }

        fn undo(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = false;
            Ok(())
        }

        fn description(&self) -> String {
            format!("TestCommand({})", self.value)
        }
    }

    #[test]
    fn test_command_history_execute() {
        let mut history = CommandHistory::new(10);
        let mut world = World::new();

        let cmd = Box::new(TestCommand::new(1));
        history.execute(cmd, &mut world).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history.current_position(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_history_undo() {
        let mut history = CommandHistory::new(10);
        let mut world = World::new();

        let cmd = Box::new(TestCommand::new(1));
        history.execute(cmd, &mut world).unwrap();

        history.undo(&mut world).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history.current_position(), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_command_history_redo() {
        let mut history = CommandHistory::new(10);
        let mut world = World::new();

        let cmd = Box::new(TestCommand::new(1));
        history.execute(cmd, &mut world).unwrap();
        history.undo(&mut world).unwrap();

        history.redo(&mut world).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history.current_position(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_history_clear() {
        let mut history = CommandHistory::new(10);
        let mut world = World::new();

        let cmd = Box::new(TestCommand::new(1));
        history.execute(cmd, &mut world).unwrap();

        history.clear();

        assert_eq!(history.len(), 0);
        assert_eq!(history.current_position(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_history_max_size() {
        let mut history = CommandHistory::new(3);
        let mut world = World::new();

        for i in 0..5 {
            let cmd = Box::new(TestCommand::new(i));
            history.execute(cmd, &mut world).unwrap();
        }

        assert_eq!(history.len(), 3);
        assert_eq!(history.current_position(), 3);
    }

    #[test]
    fn test_command_history_discard_redo() {
        let mut history = CommandHistory::new(10);
        let mut world = World::new();

        // Execute 3 commands
        for i in 0..3 {
            let cmd = Box::new(TestCommand::new(i));
            history.execute(cmd, &mut world).unwrap();
        }

        // Undo 2 commands
        history.undo(&mut world).unwrap();
        history.undo(&mut world).unwrap();

        assert_eq!(history.current_position(), 1);
        assert!(history.can_redo());

        // Execute a new command - should discard redo history
        let cmd = Box::new(TestCommand::new(99));
        history.execute(cmd, &mut world).unwrap();

        assert_eq!(history.len(), 2);
        assert_eq!(history.current_position(), 2);
        assert!(!history.can_redo());
    }
}
