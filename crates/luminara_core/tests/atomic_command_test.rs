/// Integration tests for atomic command execution.
///
/// This test file verifies that atomic commands properly implement all-or-nothing
/// semantics when executing multiple sub-commands.
///
/// **Validates: Requirement 9.5**
/// - WHEN commands affect multiple entities, THE System SHALL ensure atomic execution (all or nothing)
use luminara_core::{
    AddComponentCommand, AtomicCommand, CommandHistory, Component, ModifyComponentCommand,
    RemoveComponentCommand, SpawnEntityCommand, UndoCommand, World,
};

// Test components
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

#[derive(Debug, Clone, PartialEq)]
struct Health {
    value: i32,
}

impl Component for Health {
    fn type_name() -> &'static str {
        "Health"
    }
}

#[test]
fn test_requirement_9_5_atomic_execution_all_succeed() {
    // Requirement 9.5: WHEN commands affect multiple entities,
    // THE System SHALL ensure atomic execution (all or nothing)

    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Create multiple entities
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    let entity3 = world.spawn();

    // Create atomic command affecting all three entities
    let mut atomic_cmd = AtomicCommand::new("Initialize entities");
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 1.0, y: 2.0 },
    )));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Position { x: 3.0, y: 4.0 },
    )));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity3,
        Position { x: 5.0, y: 6.0 },
    )));

    // Execute atomically
    history.execute(Box::new(atomic_cmd), &mut world).unwrap();

    // Verify all entities have their components
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 1.0, y: 2.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 3.0, y: 4.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity3),
        Some(&Position { x: 5.0, y: 6.0 })
    );
}

#[test]
fn test_requirement_9_5_atomic_execution_partial_failure() {
    // Requirement 9.5: WHEN commands affect multiple entities,
    // THE System SHALL ensure atomic execution (all or nothing)
    // This test verifies the "nothing" part - if one fails, all rollback

    let mut world = World::new();

    // Create entities
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    let entity3 = world.spawn();

    // Add Position to entity1 and entity2
    world
        .add_component(entity1, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 0.0, y: 0.0 })
        .unwrap();
    // entity3 has no Position component

    // Create atomic command that will fail on the third entity
    let mut atomic_cmd = AtomicCommand::new("Remove positions");
    atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity1)));
    atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity2)));
    // This will fail because entity3 doesn't have Position to remove
    atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity3)));

    // Execute - should fail
    let result = atomic_cmd.execute(&mut world);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("rolled back"));

    // Verify all entities were rolled back to original state
    // entity1 and entity2 should still have their Position components
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 0.0, y: 0.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 0.0, y: 0.0 })
    );
    assert_eq!(world.get_component::<Position>(entity3), None);
}

#[test]
fn test_atomic_command_with_command_history() {
    // Test that atomic commands integrate properly with CommandHistory
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    let entity1 = world.spawn();
    let entity2 = world.spawn();

    // Execute atomic command through history
    let mut atomic_cmd = AtomicCommand::new("Add components");
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 1.0, y: 2.0 },
    )));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Velocity { x: 3.0, y: 4.0 },
    )));

    history.execute(Box::new(atomic_cmd), &mut world).unwrap();

    // Verify components were added
    assert!(world.get_component::<Position>(entity1).is_some());
    assert!(world.get_component::<Velocity>(entity2).is_some());

    // Undo through history
    history.undo(&mut world).unwrap();

    // Verify all components were removed
    assert_eq!(world.get_component::<Position>(entity1), None);
    assert_eq!(world.get_component::<Velocity>(entity2), None);

    // Redo through history
    history.redo(&mut world).unwrap();

    // Verify components were re-added
    assert!(world.get_component::<Position>(entity1).is_some());
    assert!(world.get_component::<Velocity>(entity2).is_some());
}

#[test]
fn test_atomic_command_complex_multi_entity_operation() {
    // Test a complex scenario with multiple entities and multiple component types
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Create a party of entities
    let player = world.spawn();
    let ally1 = world.spawn();
    let ally2 = world.spawn();

    // Initialize all entities atomically
    let mut init_cmd = AtomicCommand::new("Initialize party");
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        player,
        Position { x: 0.0, y: 0.0 },
    )));
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        player,
        Health { value: 100 },
    )));
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        ally1,
        Position { x: -5.0, y: 0.0 },
    )));
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        ally1,
        Health { value: 80 },
    )));
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        ally2,
        Position { x: 5.0, y: 0.0 },
    )));
    init_cmd.add_command(Box::new(AddComponentCommand::new(
        ally2,
        Health { value: 80 },
    )));

    history.execute(Box::new(init_cmd), &mut world).unwrap();

    // Verify all entities are initialized
    assert!(world.get_component::<Position>(player).is_some());
    assert!(world.get_component::<Health>(player).is_some());
    assert!(world.get_component::<Position>(ally1).is_some());
    assert!(world.get_component::<Health>(ally1).is_some());
    assert!(world.get_component::<Position>(ally2).is_some());
    assert!(world.get_component::<Health>(ally2).is_some());

    // Apply damage to all entities atomically
    let mut damage_cmd = AtomicCommand::new("Apply area damage");
    damage_cmd.add_command(Box::new(ModifyComponentCommand::new(
        player,
        Health { value: 70 },
    )));
    damage_cmd.add_command(Box::new(ModifyComponentCommand::new(
        ally1,
        Health { value: 50 },
    )));
    damage_cmd.add_command(Box::new(ModifyComponentCommand::new(
        ally2,
        Health { value: 50 },
    )));

    history.execute(Box::new(damage_cmd), &mut world).unwrap();

    // Verify damage was applied
    assert_eq!(
        world.get_component::<Health>(player),
        Some(&Health { value: 70 })
    );
    assert_eq!(
        world.get_component::<Health>(ally1),
        Some(&Health { value: 50 })
    );
    assert_eq!(
        world.get_component::<Health>(ally2),
        Some(&Health { value: 50 })
    );

    // Undo damage
    history.undo(&mut world).unwrap();

    // Verify health was restored
    assert_eq!(
        world.get_component::<Health>(player),
        Some(&Health { value: 100 })
    );
    assert_eq!(
        world.get_component::<Health>(ally1),
        Some(&Health { value: 80 })
    );
    assert_eq!(
        world.get_component::<Health>(ally2),
        Some(&Health { value: 80 })
    );
}

#[test]
fn test_atomic_command_first_command_fails() {
    // Test that if the first command fails, nothing is executed
    let mut world = World::new();

    let entity1 = world.spawn();
    let entity2 = world.spawn();

    let mut atomic_cmd = AtomicCommand::new("Remove components");
    // This will fail immediately because entity1 has no Position to remove
    atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity1)));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Position { x: 3.0, y: 4.0 },
    )));

    // Execute - should fail
    let result = atomic_cmd.execute(&mut world);
    assert!(result.is_err());

    // Verify entity2 doesn't have Position (second command never executed)
    assert_eq!(world.get_component::<Position>(entity2), None);
}

#[test]
fn test_atomic_command_middle_command_fails() {
    // Test that if a middle command fails, previous commands are rolled back
    let mut world = World::new();

    let entity1 = world.spawn();
    let entity2 = world.spawn();
    let entity3 = world.spawn();

    let mut atomic_cmd = AtomicCommand::new("Add components");
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 1.0, y: 2.0 },
    )));
    // This will fail because we try to remove a component that doesn't exist
    atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Velocity>::new(entity2)));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity3,
        Position { x: 5.0, y: 6.0 },
    )));

    // Execute - should fail
    let result = atomic_cmd.execute(&mut world);
    assert!(result.is_err());

    // Verify entity1's position was rolled back
    assert_eq!(world.get_component::<Position>(entity1), None);
    // Verify entity3 doesn't have position (third command never executed)
    assert_eq!(world.get_component::<Position>(entity3), None);
}

#[test]
fn test_atomic_command_nested_in_history() {
    // Test multiple atomic commands in history
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    let entity1 = world.spawn();
    let entity2 = world.spawn();

    // First atomic command
    let mut cmd1 = AtomicCommand::new("Setup entities");
    cmd1.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 0.0, y: 0.0 },
    )));
    cmd1.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Position { x: 0.0, y: 0.0 },
    )));
    history.execute(Box::new(cmd1), &mut world).unwrap();

    // Second atomic command
    let mut cmd2 = AtomicCommand::new("Move entities");
    cmd2.add_command(Box::new(ModifyComponentCommand::new(
        entity1,
        Position { x: 10.0, y: 20.0 },
    )));
    cmd2.add_command(Box::new(ModifyComponentCommand::new(
        entity2,
        Position { x: 30.0, y: 40.0 },
    )));
    history.execute(Box::new(cmd2), &mut world).unwrap();

    // Verify final state
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 10.0, y: 20.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 30.0, y: 40.0 })
    );

    // Undo second command
    history.undo(&mut world).unwrap();
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 0.0, y: 0.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 0.0, y: 0.0 })
    );

    // Undo first command
    history.undo(&mut world).unwrap();
    assert_eq!(world.get_component::<Position>(entity1), None);
    assert_eq!(world.get_component::<Position>(entity2), None);

    // Redo both commands
    history.redo(&mut world).unwrap();
    history.redo(&mut world).unwrap();
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 10.0, y: 20.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 30.0, y: 40.0 })
    );
}

#[test]
fn test_atomic_command_spawn_and_configure() {
    // Test a realistic scenario: spawn entities and configure them atomically
    let mut world = World::new();
    let mut history = CommandHistory::new(10);

    // Create atomic command that spawns and configures entities
    let mut atomic_cmd = AtomicCommand::new("Spawn configured entities");

    // Spawn first entity
    atomic_cmd.add_command(Box::new(SpawnEntityCommand::new()));

    // Note: In a real implementation, we'd need a way to reference the just-spawned entity
    // For this test, we'll spawn entities first, then configure them

    let entity1 = world.spawn();
    let entity2 = world.spawn();

    let mut config_cmd = AtomicCommand::new("Configure spawned entities");
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 1.0, y: 2.0 },
    )));
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity1,
        Health { value: 100 },
    )));
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Position { x: 3.0, y: 4.0 },
    )));
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity2,
        Health { value: 100 },
    )));

    history.execute(Box::new(config_cmd), &mut world).unwrap();

    // Verify entities are configured
    assert!(world.get_component::<Position>(entity1).is_some());
    assert!(world.get_component::<Health>(entity1).is_some());
    assert!(world.get_component::<Position>(entity2).is_some());
    assert!(world.get_component::<Health>(entity2).is_some());

    // Undo configuration
    history.undo(&mut world).unwrap();

    // Verify all components were removed
    assert_eq!(world.get_component::<Position>(entity1), None);
    assert_eq!(world.get_component::<Health>(entity1), None);
    assert_eq!(world.get_component::<Position>(entity2), None);
    assert_eq!(world.get_component::<Health>(entity2), None);
}

#[test]
fn test_atomic_command_description() {
    let mut atomic_cmd = AtomicCommand::new("Test operation");

    // Empty command
    assert!(atomic_cmd.description().contains("Test operation"));
    assert!(atomic_cmd.description().contains("empty"));

    // Add commands
    let mut world = World::new();
    let entity = world.spawn();
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity,
        Position { x: 0.0, y: 0.0 },
    )));
    atomic_cmd.add_command(Box::new(AddComponentCommand::new(
        entity,
        Velocity { x: 0.0, y: 0.0 },
    )));

    assert!(atomic_cmd.description().contains("Test operation"));
    assert!(atomic_cmd.description().contains("2 operations"));
}
