//! Atomic command execution for multi-entity operations.
//!
//! This module provides support for atomic command execution, ensuring that
//! commands affecting multiple entities either fully succeed or fully fail
//! (all-or-nothing semantics).
//!
//! # Requirements
//! - Requirement 9.5: WHEN commands affect multiple entities, THE System SHALL
//!   ensure atomic execution (all or nothing)

use crate::undo_command::{CommandError, CommandResult, UndoCommand};
use crate::world::World;

/// A command that executes multiple sub-commands atomically.
///
/// If any sub-command fails during execution, all previously executed sub-commands
/// are automatically rolled back, ensuring the world remains in a consistent state.
///
/// # Requirements
/// - Requirement 9.5: Ensures atomic execution (all or nothing)
///
/// # Example
/// ```
/// use luminara_core::{AtomicCommand, AddComponentCommand, World};
///
/// let mut world = World::new();
/// let entity1 = world.spawn();
/// let entity2 = world.spawn();
///
/// // Create an atomic command that adds components to multiple entities
/// let mut atomic_cmd = AtomicCommand::new("Add positions to entities");
/// // atomic_cmd.add_command(Box::new(AddComponentCommand::new(entity1, Position { x: 1.0, y: 2.0 })));
/// // atomic_cmd.add_command(Box::new(AddComponentCommand::new(entity2, Position { x: 3.0, y: 4.0 })));
///
/// // Execute atomically - either both succeed or both fail
/// // atomic_cmd.execute(&mut world).unwrap();
/// ```
#[derive(Debug)]
pub struct AtomicCommand {
    /// Human-readable description of this atomic operation
    description: String,
    /// Sub-commands to execute atomically
    commands: Vec<Box<dyn UndoCommand>>,
    /// Number of commands that were successfully executed (for rollback)
    executed_count: usize,
}

impl AtomicCommand {
    /// Create a new atomic command with the given description.
    ///
    /// # Arguments
    /// * `description` - Human-readable description of the atomic operation
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            commands: Vec::new(),
            executed_count: 0,
        }
    }

    /// Add a sub-command to this atomic command.
    ///
    /// Sub-commands will be executed in the order they are added.
    ///
    /// # Arguments
    /// * `command` - The command to add
    pub fn add_command(&mut self, command: Box<dyn UndoCommand>) {
        self.commands.push(command);
    }

    /// Get the number of sub-commands in this atomic command.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Check if this atomic command is empty (has no sub-commands).
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl UndoCommand for AtomicCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        // Reset executed count
        self.executed_count = 0;

        // Execute each command in order
        for (index, command) in self.commands.iter_mut().enumerate() {
            match command.execute(world) {
                Ok(()) => {
                    self.executed_count += 1;
                }
                Err(err) => {
                    // Execution failed - rollback all previously executed commands
                    self.rollback(world);
                    return Err(CommandError::CommandError(format!(
                        "Atomic command failed at sub-command {}/{}: {}. All changes rolled back.",
                        index + 1,
                        self.commands.len(),
                        err
                    )));
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        // Undo all commands in reverse order
        for command in self.commands.iter_mut().rev() {
            command.undo(world)?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        if self.commands.is_empty() {
            format!("{} (empty)", self.description)
        } else {
            format!("{} ({} operations)", self.description, self.commands.len())
        }
    }
}

impl AtomicCommand {
    /// Rollback all successfully executed commands.
    ///
    /// This is called when a sub-command fails during execution to ensure
    /// all-or-nothing semantics.
    fn rollback(&mut self, world: &mut World) {
        // Undo all successfully executed commands in reverse order
        for i in (0..self.executed_count).rev() {
            if let Err(err) = self.commands[i].undo(world) {
                // Rollback failure is serious - log but continue trying to rollback others
                eprintln!(
                    "WARNING: Failed to rollback sub-command {} during atomic command failure: {}",
                    i, err
                );
            }
        }
        self.executed_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AddComponentCommand, Component, RemoveComponentCommand};

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        fn type_name() -> &'static str {
            "Position"
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    impl Component for Velocity {
        fn type_name() -> &'static str {
            "Velocity"
        }
    }

    #[test]
    fn test_atomic_command_all_succeed() {
        let mut world = World::new();
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        let mut atomic_cmd = AtomicCommand::new("Add positions to entities");
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity1,
            Position { x: 1.0, y: 2.0 },
        )));
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity2,
            Position { x: 3.0, y: 4.0 },
        )));

        // Execute - should succeed
        atomic_cmd.execute(&mut world).unwrap();

        // Verify both components were added
        assert_eq!(
            world.get_component::<Position>(entity1),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get_component::<Position>(entity2),
            Some(&Position { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_atomic_command_partial_failure_rollback() {
        let mut world = World::new();
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        // Add Position to entity1 so we can remove it
        world
            .add_component(entity1, Position { x: 0.0, y: 0.0 })
            .unwrap();

        let mut atomic_cmd = AtomicCommand::new("Remove positions");
        atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity1)));
        // This will fail because entity2 doesn't have a Position component to remove
        atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity2)));

        // Execute - should fail
        let result = atomic_cmd.execute(&mut world);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rolled back"));

        // Verify entity1's position was rolled back (restored)
        assert_eq!(
            world.get_component::<Position>(entity1),
            Some(&Position { x: 0.0, y: 0.0 })
        );

        // Verify entity2 still has no position
        assert_eq!(world.get_component::<Position>(entity2), None);
    }

    #[test]
    fn test_atomic_command_undo() {
        let mut world = World::new();
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        let mut atomic_cmd = AtomicCommand::new("Add components");
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity1,
            Position { x: 1.0, y: 2.0 },
        )));
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity2,
            Velocity { x: 3.0, y: 4.0 },
        )));

        // Execute
        atomic_cmd.execute(&mut world).unwrap();

        // Verify components were added
        assert!(world.get_component::<Position>(entity1).is_some());
        assert!(world.get_component::<Velocity>(entity2).is_some());

        // Undo
        atomic_cmd.undo(&mut world).unwrap();

        // Verify components were removed
        assert_eq!(world.get_component::<Position>(entity1), None);
        assert_eq!(world.get_component::<Velocity>(entity2), None);
    }

    #[test]
    fn test_atomic_command_empty() {
        let mut world = World::new();
        let mut atomic_cmd = AtomicCommand::new("Empty command");

        // Execute empty command - should succeed
        atomic_cmd.execute(&mut world).unwrap();

        // Undo empty command - should succeed
        atomic_cmd.undo(&mut world).unwrap();

        assert!(atomic_cmd.is_empty());
        assert_eq!(atomic_cmd.command_count(), 0);
    }

    #[test]
    fn test_atomic_command_description() {
        let mut atomic_cmd = AtomicCommand::new("Test operation");
        assert!(atomic_cmd.description().contains("Test operation"));
        assert!(atomic_cmd.description().contains("empty"));

        let mut world = World::new();
        let entity = world.spawn();
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity,
            Position { x: 0.0, y: 0.0 },
        )));

        assert!(atomic_cmd.description().contains("Test operation"));
        assert!(atomic_cmd.description().contains("1 operations"));
    }

    #[test]
    fn test_atomic_command_multiple_entities() {
        let mut world = World::new();
        let entities: Vec<_> = (0..5).map(|_| world.spawn()).collect();

        let mut atomic_cmd = AtomicCommand::new("Add positions to 5 entities");
        for (i, &entity) in entities.iter().enumerate() {
            atomic_cmd.add_command(Box::new(AddComponentCommand::new(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32 * 2.0,
                },
            )));
        }

        // Execute
        atomic_cmd.execute(&mut world).unwrap();

        // Verify all entities have positions
        for (i, &entity) in entities.iter().enumerate() {
            assert_eq!(
                world.get_component::<Position>(entity),
                Some(&Position {
                    x: i as f32,
                    y: i as f32 * 2.0
                })
            );
        }

        // Undo
        atomic_cmd.undo(&mut world).unwrap();

        // Verify all positions were removed
        for &entity in &entities {
            assert_eq!(world.get_component::<Position>(entity), None);
        }
    }

    #[test]
    fn test_atomic_command_first_fails() {
        let mut world = World::new();
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        let mut atomic_cmd = AtomicCommand::new("Remove components");
        // This will fail immediately because entity1 has no Position to remove
        atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity1)));
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity2,
            Position { x: 30.0, y: 40.0 },
        )));

        // Execute - should fail immediately
        let result = atomic_cmd.execute(&mut world);
        assert!(result.is_err());

        // Verify entity2 doesn't have position (second command never executed)
        assert_eq!(world.get_component::<Position>(entity2), None);
    }

    #[test]
    fn test_atomic_command_last_fails() {
        let mut world = World::new();
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        let mut atomic_cmd = AtomicCommand::new("Add and remove");
        atomic_cmd.add_command(Box::new(AddComponentCommand::new(
            entity1,
            Position { x: 1.0, y: 2.0 },
        )));
        // This will fail because entity2 doesn't have Position to remove
        atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity2)));

        // Execute - should fail and rollback
        let result = atomic_cmd.execute(&mut world);
        assert!(result.is_err());

        // Verify entity1's position was rolled back
        assert_eq!(world.get_component::<Position>(entity1), None);
    }
}
