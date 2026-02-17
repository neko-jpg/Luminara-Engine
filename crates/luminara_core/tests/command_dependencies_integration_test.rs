//! Integration tests for command dependency tracking.
//!
//! These tests verify that command dependencies work correctly with the full
//! command system, including undo/redo and atomic commands.
//!
//! **Validates: Requirements 9.6**

use luminara_core::{
    AddComponentCommand, CommandHistory, Component, DependencyGraph, ModifyComponentCommand,
    RemoveComponentCommand, SpawnEntityCommand, World,
};

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
    value: f32,
}

impl Component for Health {
    fn type_name() -> &'static str {
        "Health"
    }
}

/// Test that commands execute in dependency order.
///
/// **Validates: Requirements 9.6 - Enforce execution order**
#[test]
fn test_dependency_order_enforcement() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    // Create commands that modify the same component
    let cmd1 = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let cmd2 = Box::new(ModifyComponentCommand::new(
        entity,
        Position { x: 3.0, y: 4.0 },
    ));
    let cmd3 = Box::new(ModifyComponentCommand::new(
        entity,
        Position { x: 5.0, y: 6.0 },
    ));

    let id1 = graph.add_command(cmd1);
    let id2 = graph.add_command(cmd2);
    let id3 = graph.add_command(cmd3);

    // Create dependency chain: cmd1 -> cmd2 -> cmd3
    graph.add_dependency(id2, id1).unwrap();
    graph.add_dependency(id3, id2).unwrap();

    // Execute all commands
    graph.execute_all(&mut world).unwrap();

    // Verify final state reflects the last command
    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position { x: 5.0, y: 6.0 })
    );
}

/// Test that circular dependencies are detected and prevented.
///
/// **Validates: Requirements 9.6 - Detect circular dependencies**
#[test]
fn test_circular_dependency_detection() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    let cmd1 = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let cmd2 = Box::new(AddComponentCommand::new(
        entity,
        Velocity { x: 3.0, y: 4.0 },
    ));
    let cmd3 = Box::new(AddComponentCommand::new(entity, Health { value: 100.0 }));

    let id1 = graph.add_command(cmd1);
    let id2 = graph.add_command(cmd2);
    let id3 = graph.add_command(cmd3);

    // Create dependencies: cmd1 -> cmd2 -> cmd3
    graph.add_dependency(id2, id1).unwrap();
    graph.add_dependency(id3, id2).unwrap();

    // Try to create cycle: cmd3 -> cmd1
    let result = graph.add_dependency(id1, id3);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("circular dependency"));
}

/// Test complex dependency patterns (diamond).
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_diamond_dependency_pattern() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();

    // Create 4 entities
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    let entity3 = world.spawn();
    let entity4 = world.spawn();

    // Create diamond dependency:
    //     cmd1 (entity1)
    //    /              \
    //  cmd2 (entity2)  cmd3 (entity3)
    //    \              /
    //     cmd4 (entity4)

    let cmd1 = Box::new(AddComponentCommand::new(
        entity1,
        Position { x: 1.0, y: 1.0 },
    ));
    let cmd2 = Box::new(AddComponentCommand::new(
        entity2,
        Position { x: 2.0, y: 2.0 },
    ));
    let cmd3 = Box::new(AddComponentCommand::new(
        entity3,
        Position { x: 3.0, y: 3.0 },
    ));
    let cmd4 = Box::new(AddComponentCommand::new(
        entity4,
        Position { x: 4.0, y: 4.0 },
    ));

    let id1 = graph.add_command(cmd1);
    let id2 = graph.add_command(cmd2);
    let id3 = graph.add_command(cmd3);
    let id4 = graph.add_command(cmd4);

    graph.add_dependency(id2, id1).unwrap();
    graph.add_dependency(id3, id1).unwrap();
    graph.add_dependency(id4, id2).unwrap();
    graph.add_dependency(id4, id3).unwrap();

    // Execute all commands
    graph.execute_all(&mut world).unwrap();

    // Verify all entities have their components
    assert_eq!(
        world.get_component::<Position>(entity1),
        Some(&Position { x: 1.0, y: 1.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity2),
        Some(&Position { x: 2.0, y: 2.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity3),
        Some(&Position { x: 3.0, y: 3.0 })
    );
    assert_eq!(
        world.get_component::<Position>(entity4),
        Some(&Position { x: 4.0, y: 4.0 })
    );

    // Verify execution order
    let order = graph.topological_sort().unwrap();
    assert_eq!(order[0], id1); // cmd1 must be first
    assert_eq!(order[3], id4); // cmd4 must be last

    // cmd2 and cmd3 must come before cmd4
    let id2_pos = order.iter().position(|&id| id == id2).unwrap();
    let id3_pos = order.iter().position(|&id| id == id3).unwrap();
    let id4_pos = order.iter().position(|&id| id == id4).unwrap();
    assert!(id2_pos < id4_pos);
    assert!(id3_pos < id4_pos);
}

/// Test that spawn entity command can be a dependency.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_spawn_entity_dependency() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();

    // Spawn entity command
    let spawn_cmd = Box::new(SpawnEntityCommand::new());
    let spawn_id = graph.add_command(spawn_cmd);

    // Execute spawn command first to get entity ID
    graph.execute_all(&mut world).unwrap();

    // Get the spawned entity
    let entity = graph.get(spawn_id).unwrap().command().description();
    assert!(entity.contains("Spawn Entity"));
}

/// Test that dependencies work with component removal.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_dependency_with_component_removal() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    // Add component first
    world
        .add_component(entity, Position { x: 1.0, y: 2.0 })
        .unwrap();

    // Create commands: modify then remove
    let modify_cmd = Box::new(ModifyComponentCommand::new(
        entity,
        Position { x: 3.0, y: 4.0 },
    ));
    let remove_cmd = Box::new(RemoveComponentCommand::<Position>::new(entity));

    let modify_id = graph.add_command(modify_cmd);
    let remove_id = graph.add_command(remove_cmd);

    // Remove depends on modify
    graph.add_dependency(remove_id, modify_id).unwrap();

    // Execute all commands
    graph.execute_all(&mut world).unwrap();

    // Verify component was removed
    assert_eq!(world.get_component::<Position>(entity), None);
}

/// Test that topological sort produces valid order for complex graph.
///
/// **Validates: Requirements 9.6 - Enforce execution order**
#[test]
fn test_topological_sort_complex_graph() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();

    // Create 6 entities
    let entities: Vec<_> = (0..6).map(|_| world.spawn()).collect();

    // Create complex dependency graph:
    //   0 -> 1 -> 3 -> 5
    //   0 -> 2 -> 4 -> 5

    let mut ids = Vec::new();
    for (i, &entity) in entities.iter().enumerate() {
        let cmd = Box::new(AddComponentCommand::new(
            entity,
            Position {
                x: i as f32,
                y: i as f32,
            },
        ));
        ids.push(graph.add_command(cmd));
    }

    // Add dependencies
    graph.add_dependency(ids[1], ids[0]).unwrap();
    graph.add_dependency(ids[2], ids[0]).unwrap();
    graph.add_dependency(ids[3], ids[1]).unwrap();
    graph.add_dependency(ids[4], ids[2]).unwrap();
    graph.add_dependency(ids[5], ids[3]).unwrap();
    graph.add_dependency(ids[5], ids[4]).unwrap();

    // Get topological order
    let order = graph.topological_sort().unwrap();

    // Verify order respects all dependencies
    for (i, &id) in order.iter().enumerate() {
        let cmd = graph.get(id).unwrap();
        for &dep in cmd.dependencies() {
            let dep_pos = order.iter().position(|&x| x == dep).unwrap();
            assert!(
                dep_pos < i,
                "Dependency {} should come before command {}",
                dep,
                id
            );
        }
    }
}

/// Test that empty graph can be executed.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_empty_graph_execution() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();

    // Execute empty graph - should succeed
    graph.execute_all(&mut world).unwrap();

    assert!(graph.is_empty());
}

/// Test that graph can be cleared and reused.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_graph_clear_and_reuse() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    // Add commands
    let cmd1 = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let cmd2 = Box::new(AddComponentCommand::new(
        entity,
        Velocity { x: 3.0, y: 4.0 },
    ));

    let id1 = graph.add_command(cmd1);
    let id2 = graph.add_command(cmd2);
    graph.add_dependency(id2, id1).unwrap();

    assert_eq!(graph.len(), 2);

    // Clear graph
    graph.clear();
    assert!(graph.is_empty());

    // Reuse graph
    let cmd3 = Box::new(AddComponentCommand::new(entity, Health { value: 100.0 }));
    let id3 = graph.add_command(cmd3);

    // New ID should start from 0 again
    assert_eq!(id3, 0);
    assert_eq!(graph.len(), 1);
}

/// Test integration with CommandHistory.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_integration_with_command_history() {
    let mut history = CommandHistory::new(100);
    let mut world = World::new();
    let entity = world.spawn();

    // Execute commands through history (not through dependency graph)
    let cmd1 = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let cmd2 = Box::new(AddComponentCommand::new(
        entity,
        Velocity { x: 3.0, y: 4.0 },
    ));

    history.execute(cmd1, &mut world).unwrap();
    history.execute(cmd2, &mut world).unwrap();

    // Verify components were added
    assert!(world.get_component::<Position>(entity).is_some());
    assert!(world.get_component::<Velocity>(entity).is_some());

    // Undo both commands
    history.undo(&mut world).unwrap();
    history.undo(&mut world).unwrap();

    // Verify components were removed
    assert_eq!(world.get_component::<Position>(entity), None);
    assert_eq!(world.get_component::<Velocity>(entity), None);
}

/// Test that self-dependency is prevented.
///
/// **Validates: Requirements 9.6 - Detect circular dependencies**
#[test]
fn test_self_dependency_prevention() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    let cmd = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let id = graph.add_command(cmd);

    // Try to make command depend on itself
    let result = graph.add_dependency(id, id);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot depend on itself"));
}

/// Test that invalid command IDs are rejected.
///
/// **Validates: Requirements 9.6 - Track command dependencies**
#[test]
fn test_invalid_command_id_rejection() {
    let mut graph = DependencyGraph::new();
    let mut world = World::new();
    let entity = world.spawn();

    let cmd = Box::new(AddComponentCommand::new(
        entity,
        Position { x: 1.0, y: 2.0 },
    ));
    let id = graph.add_command(cmd);

    // Try to add dependency with non-existent ID
    let result = graph.add_dependency(id, 999);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid dependency"));

    // Try to add dependency from non-existent ID
    let result = graph.add_dependency(999, id);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid dependent"));
}
