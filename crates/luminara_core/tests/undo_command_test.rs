use luminara_core::{
    CommandError, CommandHistory, CommandResult, Component, Entity, UndoCommand, World,
};

// Test component for verification
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

// Command to spawn an entity
struct SpawnEntityCommand {
    entity: Option<Entity>,
}

impl SpawnEntityCommand {
    fn new() -> Self {
        Self { entity: None }
    }
}

impl UndoCommand for SpawnEntityCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        let entity = world.spawn();
        self.entity = Some(entity);
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Some(entity) = self.entity {
            world.despawn(entity);
        }
        Ok(())
    }

    fn description(&self) -> String {
        "Spawn Entity".to_string()
    }
}

// Command to add a component
struct AddComponentCommand {
    entity: Entity,
    component: Position,
    had_component: bool,
}

impl AddComponentCommand {
    fn new(entity: Entity, component: Position) -> Self {
        Self {
            entity,
            component,
            had_component: false,
        }
    }
}

impl UndoCommand for AddComponentCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        self.had_component = world.get_component::<Position>(self.entity).is_some();
        world.add_component(self.entity, self.component.clone())?;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if !self.had_component {
            world.remove_component::<Position>(self.entity)?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add Position to {:?}", self.entity)
    }
}

// Command to modify a component
struct ModifyComponentCommand {
    entity: Entity,
    old_value: Option<Position>,
    new_value: Position,
}

impl ModifyComponentCommand {
    fn new(entity: Entity, new_value: Position) -> Self {
        Self {
            entity,
            old_value: None,
            new_value,
        }
    }
}

impl UndoCommand for ModifyComponentCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        self.old_value = world.get_component::<Position>(self.entity).cloned();
        world.add_component(self.entity, self.new_value.clone())?;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Some(old_value) = &self.old_value {
            world.add_component(self.entity, old_value.clone())?;
        } else {
            world.remove_component::<Position>(self.entity)?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        format!("Modify Position on {:?}", self.entity)
    }

    fn can_merge(&self, other: &dyn UndoCommand) -> bool {
        // Check if other is also a ModifyComponentCommand for the same entity
        other.description().starts_with("Modify Position on")
            && other.description().contains(&format!("{:?}", self.entity))
    }

    fn merge(&mut self, other: Box<dyn UndoCommand>) -> CommandResult<()> {
        // Downcast to ModifyComponentCommand
        // In a real implementation, we'd use Any trait for proper downcasting
        // For this test, we'll just update the new_value
        // This is a simplified version - proper implementation would need type-safe downcasting
        Ok(())
    }
}

#[test]
fn test_requirement_9_1_command_trait_with_execute_and_undo() {
    // Requirement 9.1: WHEN defining operations, THE System SHALL provide a Command trait
    // with execute and undo methods

    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Create and execute a command
    let cmd = Box::new(SpawnEntityCommand::new());
    history.execute(cmd, &mut world).unwrap();

    // Verify command was executed
    assert_eq!(history.len(), 1);
    assert!(history.can_undo());

    // Undo the command
    history.undo(&mut world).unwrap();

    // Verify undo worked
    assert!(!history.can_undo());
    assert!(history.can_redo());
}

#[test]
fn test_requirement_9_2_record_sufficient_state_for_undo() {
    // Requirement 9.2: WHEN executing commands, THE System SHALL record sufficient state
    // to enable undo

    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Spawn an entity
    let entity = world.spawn();

    // Add a component
    let initial_pos = Position { x: 1.0, y: 2.0 };
    let cmd = Box::new(AddComponentCommand::new(entity, initial_pos.clone()));
    history.execute(cmd, &mut world).unwrap();

    // Verify component was added
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&initial_pos)
    );

    // Undo the command
    history.undo(&mut world).unwrap();

    // Verify component was removed (state was restored)
    assert_eq!(world.get_component::<Position>(entity), None);

    // Redo the command
    history.redo(&mut world).unwrap();

    // Verify component was re-added
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&initial_pos)
    );
}

#[test]
fn test_command_history_basic_operations() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Execute multiple commands
    for i in 0..3 {
        let cmd = Box::new(SpawnEntityCommand::new());
        history.execute(cmd, &mut world).unwrap();
    }

    assert_eq!(history.len(), 3);
    assert_eq!(history.current_position(), 3);

    // Undo all commands
    for _ in 0..3 {
        history.undo(&mut world).unwrap();
    }

    assert_eq!(history.current_position(), 0);
    assert!(!history.can_undo());

    // Redo all commands
    for _ in 0..3 {
        history.redo(&mut world).unwrap();
    }

    assert_eq!(history.current_position(), 3);
    assert!(!history.can_redo());
}

#[test]
fn test_command_history_discard_redo_on_new_command() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Execute 3 commands
    for _ in 0..3 {
        let cmd = Box::new(SpawnEntityCommand::new());
        history.execute(cmd, &mut world).unwrap();
    }

    // Undo 2 commands
    history.undo(&mut world).unwrap();
    history.undo(&mut world).unwrap();

    assert_eq!(history.current_position(), 1);
    assert_eq!(history.len(), 3);

    // Execute a new command - should discard redo history
    let cmd = Box::new(SpawnEntityCommand::new());
    history.execute(cmd, &mut world).unwrap();

    assert_eq!(history.len(), 2);
    assert_eq!(history.current_position(), 2);
    assert!(!history.can_redo());
}

#[test]
fn test_command_history_max_size_enforcement() {
    let mut world = World::new();
    let mut history = CommandHistory::new(3);

    // Execute 5 commands
    for _ in 0..5 {
        let cmd = Box::new(SpawnEntityCommand::new());
        history.execute(cmd, &mut world).unwrap();
    }

    // Should only keep the last 3 commands
    assert_eq!(history.len(), 3);
    assert_eq!(history.current_position(), 3);

    // Should be able to undo 3 times
    for _ in 0..3 {
        assert!(history.can_undo());
        history.undo(&mut world).unwrap();
    }

    assert!(!history.can_undo());
}

#[test]
fn test_command_descriptions() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    let cmd = Box::new(SpawnEntityCommand::new());
    history.execute(cmd, &mut world).unwrap();

    assert_eq!(history.undo_description(), Some("Spawn Entity".to_string()));
    assert_eq!(history.redo_description(), None);

    history.undo(&mut world).unwrap();

    assert_eq!(history.undo_description(), None);
    assert_eq!(history.redo_description(), Some("Spawn Entity".to_string()));
}

#[test]
fn test_command_error_handling() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Try to undo with no commands
    let result = history.undo(&mut world);
    assert!(matches!(result, Err(CommandError::NoUndo)));

    // Try to redo with no commands
    let result = history.redo(&mut world);
    assert!(matches!(result, Err(CommandError::NoRedo)));
}

#[test]
fn test_modify_component_command_state_preservation() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Spawn entity and add initial component
    let entity = world.spawn();
    let initial_pos = Position { x: 1.0, y: 2.0 };
    world.add_component(entity, initial_pos.clone()).unwrap();

    // Modify the component
    let new_pos = Position { x: 3.0, y: 4.0 };
    let cmd = Box::new(ModifyComponentCommand::new(entity, new_pos.clone()));
    history.execute(cmd, &mut world).unwrap();

    // Verify modification
    assert_eq!(world.get_component::<Position>(entity), Some(&new_pos));

    // Undo modification
    history.undo(&mut world).unwrap();

    // Verify original state restored
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&initial_pos)
    );

    // Redo modification
    history.redo(&mut world).unwrap();

    // Verify new state restored
    assert_eq!(world.get_component::<Position>(entity), Some(&new_pos));
}

#[test]
fn test_command_history_clear() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Execute some commands
    for _ in 0..3 {
        let cmd = Box::new(SpawnEntityCommand::new());
        history.execute(cmd, &mut world).unwrap();
    }

    assert_eq!(history.len(), 3);

    // Clear history
    history.clear();

    assert_eq!(history.len(), 0);
    assert_eq!(history.current_position(), 0);
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_command_history_iterator() {
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Execute some commands
    for _ in 0..3 {
        let cmd = Box::new(SpawnEntityCommand::new());
        history.execute(cmd, &mut world).unwrap();
    }

    // Iterate over commands
    let descriptions: Vec<String> = history.iter().map(|cmd| cmd.description()).collect();

    assert_eq!(descriptions.len(), 3);
    assert!(descriptions.iter().all(|d| d == "Spawn Entity"));
}

#[test]
fn test_complex_command_sequence() {
    // Test a realistic sequence of editor operations
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // 1. Spawn an entity
    let spawn_cmd = Box::new(SpawnEntityCommand::new());
    history.execute(spawn_cmd, &mut world).unwrap();

    // Get the spawned entity (we need to track it for subsequent commands)
    // In a real implementation, commands would return the entity ID
    let entity = world.spawn(); // Temporary workaround for test

    // 2. Add a component
    let add_cmd = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 0.0, y: 0.0 },
    ));
    history.execute(add_cmd, &mut world).unwrap();

    // 3. Modify the component
    let modify_cmd = Box::new(ModifyComponentCommand::new(
        entity,
        Position { x: 10.0, y: 20.0 },
    ));
    history.execute(modify_cmd, &mut world).unwrap();

    // Verify final state
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position { x: 10.0, y: 20.0 })
    );

    // Undo all operations
    history.undo(&mut world).unwrap(); // Undo modify
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position { x: 0.0, y: 0.0 })
    );

    history.undo(&mut world).unwrap(); // Undo add
    assert_eq!(world.get_component::<Position>(entity), None);

    // Redo operations
    history.redo(&mut world).unwrap(); // Redo add
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position { x: 0.0, y: 0.0 })
    );

    history.redo(&mut world).unwrap(); // Redo modify
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position { x: 10.0, y: 20.0 })
    );
}
