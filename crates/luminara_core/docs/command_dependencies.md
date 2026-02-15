# Command Dependencies

This document describes the command dependency tracking system in Luminara Core, which ensures commands execute in the correct order and prevents circular dependencies.

## Overview

The command dependency system allows you to specify that certain commands must execute before others. This is essential for complex editor operations where the order of execution matters.

**Requirements Addressed:**
- Requirement 9.6: WHEN commands have dependencies, THE System SHALL track command dependencies, enforce execution order, and detect circular dependencies

## Key Components

### DependencyGraph

The `DependencyGraph` manages a collection of commands with dependencies between them. It provides:

- **Dependency Tracking**: Track which commands depend on which other commands
- **Execution Order Enforcement**: Automatically compute a valid execution order using topological sorting
- **Circular Dependency Detection**: Detect and prevent circular dependencies that would cause infinite loops

### DependentCommand

A `DependentCommand` wraps an `UndoCommand` and adds dependency information:

```rust
pub struct DependentCommand {
    id: CommandId,
    command: Box<dyn UndoCommand>,
    dependencies: Vec<CommandId>,
}
```

### CommandId

A unique identifier for each command in the dependency graph:

```rust
pub type CommandId = usize;
```

## Usage Examples

### Basic Dependency Chain

Create a chain of commands where each depends on the previous:

```rust
use luminara_core::{DependencyGraph, AddComponentCommand, World};

let mut graph = DependencyGraph::new();
let mut world = World::new();
let entity = world.spawn();

// Add three commands
let cmd1 = Box::new(AddComponentCommand::new(entity, Position { x: 1.0, y: 2.0 }));
let cmd2 = Box::new(AddComponentCommand::new(entity, Velocity { x: 3.0, y: 4.0 }));
let cmd3 = Box::new(AddComponentCommand::new(entity, Mass { value: 5.0 }));

let id1 = graph.add_command(cmd1);
let id2 = graph.add_command(cmd2);
let id3 = graph.add_command(cmd3);

// Create dependency chain: cmd1 -> cmd2 -> cmd3
graph.add_dependency(id2, id1).unwrap();
graph.add_dependency(id3, id2).unwrap();

// Execute all commands in dependency order
graph.execute_all(&mut world).unwrap();
```

### Diamond Dependency Pattern

Handle complex dependency patterns like diamonds:

```rust
// Create diamond dependency:
//     cmd1
//    /    \
//  cmd2  cmd3
//    \    /
//     cmd4

let id1 = graph.add_command(cmd1);
let id2 = graph.add_command(cmd2);
let id3 = graph.add_command(cmd3);
let id4 = graph.add_command(cmd4);

graph.add_dependency(id2, id1).unwrap();
graph.add_dependency(id3, id1).unwrap();
graph.add_dependency(id4, id2).unwrap();
graph.add_dependency(id4, id3).unwrap();

// Topological sort will ensure cmd1 executes first,
// then cmd2 and cmd3 (in any order), then cmd4 last
let order = graph.topological_sort().unwrap();
```

### Circular Dependency Detection

The system automatically detects and prevents circular dependencies:

```rust
let id1 = graph.add_command(cmd1);
let id2 = graph.add_command(cmd2);

// Create dependency: cmd1 -> cmd2
graph.add_dependency(id2, id1).unwrap();

// Try to create cycle: cmd2 -> cmd1
let result = graph.add_dependency(id1, id2);

// This will fail with an error describing the cycle
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("circular dependency"));
```

## Topological Sorting

The dependency graph uses topological sorting to compute a valid execution order. The algorithm guarantees:

1. **Dependency Respect**: If command A depends on command B, then B will execute before A
2. **Cycle Detection**: If a cycle exists, the sort will fail with an error
3. **Deterministic Order**: For commands with no dependencies between them, the order is deterministic based on insertion order

### Algorithm

The implementation uses Kahn's algorithm:

1. Calculate in-degree (number of dependencies) for each command
2. Start with all commands that have no dependencies (in-degree = 0)
3. Process each command, removing it from the graph and decreasing the in-degree of dependent commands
4. Repeat until all commands are processed or a cycle is detected

## Error Handling

The dependency system provides clear error messages for common issues:

### Self-Dependency

```rust
let id = graph.add_command(cmd);
let result = graph.add_dependency(id, id);
// Error: "Command 0 cannot depend on itself"
```

### Invalid Command ID

```rust
let id = graph.add_command(cmd);
let result = graph.add_dependency(id, 999);
// Error: "Invalid dependency command ID: 999"
```

### Circular Dependency

```rust
graph.add_dependency(id2, id1).unwrap();
let result = graph.add_dependency(id1, id2);
// Error: "Adding dependency would create circular dependency: 0 -> 1 -> 0"
```

## Performance Considerations

- **Cycle Detection**: O(V + E) where V is the number of commands and E is the number of dependencies
- **Topological Sort**: O(V + E) using Kahn's algorithm
- **Memory**: O(V + E) to store the graph structure

For typical editor operations with hundreds of commands, performance is negligible.

## Integration with Command History

The dependency graph is designed to work alongside `CommandHistory`:

```rust
let mut history = CommandHistory::new(100);
let mut graph = DependencyGraph::new();

// Build dependency graph
let id1 = graph.add_command(cmd1);
let id2 = graph.add_command(cmd2);
graph.add_dependency(id2, id1).unwrap();

// Execute in dependency order
graph.execute_all(&mut world).unwrap();

// Commands can still be undone through history
// (Note: You'd need to add them to history separately)
```

## Best Practices

1. **Use Dependencies Sparingly**: Only add dependencies when execution order truly matters
2. **Document Dependencies**: Comment why dependencies exist for complex operations
3. **Test for Cycles**: Write tests that verify your dependency patterns don't create cycles
4. **Consider Atomic Commands**: For operations that must execute together, consider using `AtomicCommand` instead of dependencies

## Future Enhancements

Potential future improvements:

- **Automatic Dependency Inference**: Analyze commands to automatically detect dependencies
- **Parallel Execution**: Execute independent commands in parallel
- **Dependency Visualization**: Generate graphs showing command dependencies
- **Conditional Dependencies**: Support dependencies that only apply under certain conditions

## See Also

- [Undo/Redo Command Pattern](undo_command.md)
- [Atomic Command Execution](atomic_command.md)
- [Core Commands](core_commands.md)
