//! Command dependency tracking and enforcement.
//!
//! This module provides dependency tracking for commands, ensuring that commands
//! execute in the correct order and detecting circular dependencies.
//!
//! # Requirements
//! - Requirement 9.6: WHEN commands have dependencies, THE System SHALL track
//!   command dependencies, enforce execution order, and detect circular dependencies

use crate::undo_command::{CommandError, CommandResult, UndoCommand};
use crate::world::World;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Unique identifier for a command in the dependency graph.
pub type CommandId = usize;

/// A command with dependency tracking support.
///
/// This wraps an UndoCommand and adds dependency information for execution ordering.
pub struct DependentCommand {
    /// Unique identifier for this command
    id: CommandId,
    /// The actual command to execute
    command: Box<dyn UndoCommand>,
    /// IDs of commands that must execute before this one
    dependencies: Vec<CommandId>,
}

impl DependentCommand {
    /// Create a new dependent command.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this command
    /// * `command` - The command to execute
    pub fn new(id: CommandId, command: Box<dyn UndoCommand>) -> Self {
        Self {
            id,
            command,
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to this command.
    ///
    /// This command will not execute until the specified dependency has executed.
    ///
    /// # Arguments
    /// * `dependency_id` - ID of the command that must execute first
    pub fn add_dependency(&mut self, dependency_id: CommandId) {
        if !self.dependencies.contains(&dependency_id) {
            self.dependencies.push(dependency_id);
        }
    }

    /// Get the command ID.
    pub fn id(&self) -> CommandId {
        self.id
    }

    /// Get the dependencies of this command.
    pub fn dependencies(&self) -> &[CommandId] {
        &self.dependencies
    }

    /// Get a reference to the underlying command.
    pub fn command(&self) -> &dyn UndoCommand {
        &*self.command
    }

    /// Get a mutable reference to the underlying command.
    pub fn command_mut(&mut self) -> &mut dyn UndoCommand {
        &mut *self.command
    }
}

impl fmt::Debug for DependentCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DependentCommand")
            .field("id", &self.id)
            .field("description", &self.command.description())
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

/// Manages command dependencies and enforces execution order.
///
/// The DependencyGraph ensures that commands execute in the correct order
/// based on their dependencies, and detects circular dependencies.
///
/// # Requirements
/// - Requirement 9.6: Tracks dependencies, enforces order, detects cycles
pub struct DependencyGraph {
    /// All commands in the graph
    commands: HashMap<CommandId, DependentCommand>,
    /// Next command ID to assign
    next_id: CommandId,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a command to the graph.
    ///
    /// # Arguments
    /// * `command` - The command to add
    ///
    /// # Returns
    /// The ID assigned to the command
    pub fn add_command(&mut self, command: Box<dyn UndoCommand>) -> CommandId {
        let id = self.next_id;
        self.next_id += 1;
        self.commands.insert(id, DependentCommand::new(id, command));
        id
    }

    /// Add a dependency between two commands.
    ///
    /// After this call, `dependent_id` will not execute until `dependency_id` has executed.
    ///
    /// # Arguments
    /// * `dependent_id` - ID of the command that depends on another
    /// * `dependency_id` - ID of the command that must execute first
    ///
    /// # Errors
    /// Returns an error if:
    /// - Either command ID is invalid
    /// - Adding the dependency would create a circular dependency
    pub fn add_dependency(
        &mut self,
        dependent_id: CommandId,
        dependency_id: CommandId,
    ) -> CommandResult<()> {
        // Validate command IDs
        if !self.commands.contains_key(&dependent_id) {
            return Err(CommandError::CommandError(format!(
                "Invalid dependent command ID: {}",
                dependent_id
            )));
        }
        if !self.commands.contains_key(&dependency_id) {
            return Err(CommandError::CommandError(format!(
                "Invalid dependency command ID: {}",
                dependency_id
            )));
        }

        // Check for self-dependency
        if dependent_id == dependency_id {
            return Err(CommandError::CommandError(format!(
                "Command {} cannot depend on itself",
                dependent_id
            )));
        }

        // Add the dependency temporarily to check for cycles
        self.commands
            .get_mut(&dependent_id)
            .unwrap()
            .add_dependency(dependency_id);

        // Check for circular dependencies
        if let Some(cycle) = self.detect_cycle() {
            // Remove the dependency we just added
            self.commands
                .get_mut(&dependent_id)
                .unwrap()
                .dependencies
                .retain(|&id| id != dependency_id);

            return Err(CommandError::CommandError(format!(
                "Adding dependency would create circular dependency: {}",
                format_cycle(&cycle)
            )));
        }

        Ok(())
    }

    /// Detect circular dependencies in the graph.
    ///
    /// # Returns
    /// Some(cycle) if a circular dependency is detected, None otherwise.
    /// The cycle is represented as a vector of command IDs forming the cycle.
    ///
    /// # Requirements
    /// - Requirement 9.6: Detects circular dependencies
    pub fn detect_cycle(&self) -> Option<Vec<CommandId>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &id in self.commands.keys() {
            if !visited.contains(&id) {
                if let Some(cycle) =
                    self.dfs_cycle_detection(id, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    /// Depth-first search for cycle detection.
    fn dfs_cycle_detection(
        &self,
        node: CommandId,
        visited: &mut HashSet<CommandId>,
        rec_stack: &mut HashSet<CommandId>,
        path: &mut Vec<CommandId>,
    ) -> Option<Vec<CommandId>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(command) = self.commands.get(&node) {
            for &dep in command.dependencies() {
                if !visited.contains(&dep) {
                    if let Some(cycle) = self.dfs_cycle_detection(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&dep) {
                    // Found a cycle - extract it from the path
                    let cycle_start = path.iter().position(|&id| id == dep).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(dep); // Close the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        None
    }

    /// Compute a topological ordering of commands.
    ///
    /// This produces an execution order that respects all dependencies.
    ///
    /// # Returns
    /// A vector of command IDs in execution order, or an error if a cycle is detected.
    ///
    /// # Requirements
    /// - Requirement 9.6: Enforces execution order
    pub fn topological_sort(&self) -> CommandResult<Vec<CommandId>> {
        // Check for cycles first
        if let Some(cycle) = self.detect_cycle() {
            return Err(CommandError::CommandError(format!(
                "Cannot sort commands with circular dependency: {}",
                format_cycle(&cycle)
            )));
        }

        // Kahn's algorithm for topological sort
        let mut in_degree: HashMap<CommandId, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Calculate in-degrees
        for &id in self.commands.keys() {
            in_degree.insert(id, 0);
        }
        for command in self.commands.values() {
            for &_dep in command.dependencies() {
                *in_degree.get_mut(&command.id()).unwrap() += 1;
            }
        }

        // Find all nodes with in-degree 0
        for (&id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(id);
            }
        }

        // Process nodes
        while let Some(id) = queue.pop_front() {
            result.push(id);

            // For each command that depends on this one
            for other_command in self.commands.values() {
                if other_command.dependencies().contains(&id) {
                    let other_id = other_command.id();
                    let degree = in_degree.get_mut(&other_id).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(other_id);
                    }
                }
            }
        }

        // If we didn't process all nodes, there's a cycle (shouldn't happen due to earlier check)
        if result.len() != self.commands.len() {
            return Err(CommandError::CommandError(
                "Topological sort failed - unexpected cycle".to_string(),
            ));
        }

        Ok(result)
    }

    /// Execute all commands in dependency order.
    ///
    /// Commands are executed in an order that respects all dependencies.
    /// If any command fails, execution stops and an error is returned.
    ///
    /// # Arguments
    /// * `world` - The world to execute commands on
    ///
    /// # Errors
    /// Returns an error if:
    /// - A circular dependency is detected
    /// - Any command execution fails
    ///
    /// # Requirements
    /// - Requirement 9.6: Enforces execution order
    pub fn execute_all(&mut self, world: &mut World) -> CommandResult<()> {
        let order = self.topological_sort()?;

        for id in order {
            let command = self.commands.get_mut(&id).unwrap();
            command.command_mut().execute(world)?;
        }

        Ok(())
    }

    /// Get the number of commands in the graph.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get a command by ID.
    pub fn get(&self, id: CommandId) -> Option<&DependentCommand> {
        self.commands.get(&id)
    }

    /// Get a mutable reference to a command by ID.
    pub fn get_mut(&mut self, id: CommandId) -> Option<&mut DependentCommand> {
        self.commands.get_mut(&id)
    }

    /// Clear all commands from the graph.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.next_id = 0;
    }
}

impl fmt::Debug for DependencyGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DependencyGraph")
            .field("command_count", &self.commands.len())
            .field("next_id", &self.next_id)
            .finish()
    }
}

/// Format a cycle for error messages.
fn format_cycle(cycle: &[CommandId]) -> String {
    cycle
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(" -> ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AddComponentCommand, Component};

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

    #[test]
    fn test_add_command() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let id = graph.add_command(cmd);

        assert_eq!(id, 0);
        assert_eq!(graph.len(), 1);
        assert!(graph.get(id).is_some());
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);

        // cmd2 depends on cmd1
        graph.add_dependency(id2, id1).unwrap();

        let deps = graph.get(id2).unwrap().dependencies();
        assert_eq!(deps, &[id1]);
    }

    #[test]
    fn test_self_dependency_error() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let id = graph.add_command(cmd);

        // Cannot depend on itself
        let result = graph.add_dependency(id, id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot depend on itself"));
    }

    #[test]
    fn test_detect_simple_cycle() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);

        // Create cycle: cmd1 -> cmd2 -> cmd1
        graph.add_dependency(id2, id1).unwrap();
        let result = graph.add_dependency(id1, id2);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("circular dependency"));
    }

    #[test]
    fn test_detect_complex_cycle() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));
        let cmd3 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 5.0, y: 6.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);
        let id3 = graph.add_command(cmd3);

        // Create cycle: cmd1 -> cmd2 -> cmd3 -> cmd1
        graph.add_dependency(id2, id1).unwrap();
        graph.add_dependency(id3, id2).unwrap();
        let result = graph.add_dependency(id1, id3);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("circular dependency"));
    }

    #[test]
    fn test_topological_sort_no_dependencies() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);

        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 2);
        assert!(order.contains(&id1));
        assert!(order.contains(&id2));
    }

    #[test]
    fn test_topological_sort_linear_chain() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));
        let cmd3 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 5.0, y: 6.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);
        let id3 = graph.add_command(cmd3);

        // Create chain: cmd1 -> cmd2 -> cmd3
        graph.add_dependency(id2, id1).unwrap();
        graph.add_dependency(id3, id2).unwrap();

        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec![id1, id2, id3]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));
        let cmd3 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 5.0, y: 6.0 },
        ));
        let cmd4 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 7.0, y: 8.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);
        let id3 = graph.add_command(cmd3);
        let id4 = graph.add_command(cmd4);

        // Create diamond: cmd1 -> cmd2 -> cmd4
        //                      \-> cmd3 ->/
        graph.add_dependency(id2, id1).unwrap();
        graph.add_dependency(id3, id1).unwrap();
        graph.add_dependency(id4, id2).unwrap();
        graph.add_dependency(id4, id3).unwrap();

        let order = graph.topological_sort().unwrap();

        // Verify cmd1 comes first
        assert_eq!(order[0], id1);
        // Verify cmd4 comes last
        assert_eq!(order[3], id4);
        // Verify cmd2 and cmd3 come before cmd4
        let id2_pos = order.iter().position(|&id| id == id2).unwrap();
        let id3_pos = order.iter().position(|&id| id == id3).unwrap();
        let id4_pos = order.iter().position(|&id| id == id4).unwrap();
        assert!(id2_pos < id4_pos);
        assert!(id3_pos < id4_pos);
    }

    #[test]
    fn test_execute_all_with_dependencies() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));

        let id1 = graph.add_command(cmd1);
        let id2 = graph.add_command(cmd2);

        // cmd2 depends on cmd1
        graph.add_dependency(id2, id1).unwrap();

        // Execute all commands
        graph.execute_all(&mut world).unwrap();

        // Verify final state (cmd2 overwrites cmd1)
        assert_eq!(
            world.get_component::<Position>(entity),
            Some(&Position { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_invalid_dependency_id() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let id = graph.add_command(cmd);

        // Try to add dependency with invalid ID
        let result = graph.add_dependency(id, 999);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid dependency"));
    }

    #[test]
    fn test_clear_graph() {
        let mut graph = DependencyGraph::new();
        let mut world = World::new();
        let entity = world.spawn();

        let cmd1 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 1.0, y: 2.0 },
        ));
        let cmd2 = Box::new(AddComponentCommand::new(
            entity,
            Position { x: 3.0, y: 4.0 },
        ));

        graph.add_command(cmd1);
        graph.add_command(cmd2);

        assert_eq!(graph.len(), 2);

        graph.clear();

        assert_eq!(graph.len(), 0);
        assert!(graph.is_empty());
    }
}
