# Atomic Command Execution

This document describes the atomic command execution system, which ensures all-or-nothing semantics for operations affecting multiple entities.

## Overview

The `AtomicCommand` provides transactional semantics for multi-entity operations. When executing multiple sub-commands, either all succeed or all fail - there is no partial success. If any sub-command fails, all previously executed sub-commands are automatically rolled back.

This is essential for editor operations like:
- **Delete Selection**: Remove multiple selected entities atomically
- **Duplicate Selection**: Spawn and configure multiple entities atomically
- **Apply Material**: Change materials on multiple objects atomically
- **Batch Transform**: Move/rotate/scale multiple entities atomically

## Requirements

**Requirement 9.5**: WHEN commands affect multiple entities, THE System SHALL ensure atomic execution (all or nothing)

## Architecture

```
AtomicCommand
├── description: String
├── commands: Vec<Box<dyn UndoCommand>>
└── executed_count: usize (for rollback tracking)

Execution Flow:
1. Execute each sub-command in order
2. Track how many succeeded (executed_count)
3. If any fails:
   a. Undo all previously executed commands in reverse order
   b. Return error with rollback confirmation
4. If all succeed:
   a. Return success
```

## Usage

### Basic Example

```rust
use luminara_core::{AtomicCommand, AddComponentCommand, World};

let mut world = World::new();
let entity1 = world.spawn();
let entity2 = world.spawn();

// Create atomic command
let mut atomic_cmd = AtomicCommand::new("Add positions to entities");
atomic_cmd.add_command(Box::new(AddComponentCommand::new(
    entity1,
    Position { x: 1.0, y: 2.0 }
)));
atomic_cmd.add_command(Box::new(AddComponentCommand::new(
    entity2,
    Position { x: 3.0, y: 4.0 }
)));

// Execute atomically - either both succeed or both fail
atomic_cmd.execute(&mut world)?;
```

### Delete Selection Example

```rust
// Delete multiple selected entities atomically
let mut delete_cmd = AtomicCommand::new("Delete Selection");
for entity in selected_entities {
    delete_cmd.add_command(Box::new(DestroyEntityCommand::new(entity)));
}

// If any entity can't be deleted, none are deleted
history.execute(Box::new(delete_cmd), &mut world)?;
```

### Batch Configuration Example

```rust
// Configure multiple entities atomically
let mut config_cmd = AtomicCommand::new("Configure spawned enemies");
for entity in enemy_entities {
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity,
        Health { value: 100 }
    )));
    config_cmd.add_command(Box::new(AddComponentCommand::new(
        entity,
        Enemy { aggression: 0.8 }
    )));
}

// All enemies get configured, or none do
history.execute(Box::new(config_cmd), &mut world)?;
```

### Complex Multi-Entity Operation

```rust
// Initialize a party of characters atomically
let mut init_party = AtomicCommand::new("Initialize party");

// Player
init_party.add_command(Box::new(AddComponentCommand::new(
    player,
    Position { x: 0.0, y: 0.0 }
)));
init_party.add_command(Box::new(AddComponentCommand::new(
    player,
    Health { value: 100 }
)));

// Ally 1
init_party.add_command(Box::new(AddComponentCommand::new(
    ally1,
    Position { x: -5.0, y: 0.0 }
)));
init_party.add_command(Box::new(AddComponentCommand::new(
    ally1,
    Health { value: 80 }
)));

// Ally 2
init_party.add_command(Box::new(AddComponentCommand::new(
    ally2,
    Position { x: 5.0, y: 0.0 }
)));
init_party.add_command(Box::new(AddComponentCommand::new(
    ally2,
    Health { value: 80 }
)));

// Execute atomically
history.execute(Box::new(init_party), &mut world)?;
```

## Rollback Behavior

When a sub-command fails, `AtomicCommand` automatically rolls back all previously executed commands:

```rust
let mut world = World::new();
let entity1 = world.spawn();
let entity2 = world.spawn();

// Add Position to entity1 so we can remove it
world.add_component(entity1, Position { x: 0.0, y: 0.0 })?;

let mut atomic_cmd = AtomicCommand::new("Remove positions");
atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity1)));
// This will fail because entity2 doesn't have Position to remove
atomic_cmd.add_command(Box::new(RemoveComponentCommand::<Position>::new(entity2)));

// Execute - will fail and rollback
let result = atomic_cmd.execute(&mut world);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("rolled back"));

// entity1's Position is restored (rollback succeeded)
assert_eq!(world.get_component::<Position>(entity1), Some(&Position { x: 0.0, y: 0.0 }));
```

## Undo/Redo

`AtomicCommand` implements the `UndoCommand` trait, so it works seamlessly with `CommandHistory`:

```rust
let mut history = CommandHistory::new(100);

// Execute atomic command
let mut atomic_cmd = AtomicCommand::new("Batch operation");
// ... add sub-commands ...
history.execute(Box::new(atomic_cmd), &mut world)?;

// Undo - all sub-commands are undone in reverse order
history.undo(&mut world)?;

// Redo - all sub-commands are re-executed in original order
history.redo(&mut world)?;
```

The entire atomic operation is treated as a single command in the history, so undo/redo operates on the whole group.

## Error Handling

### Execution Errors

When a sub-command fails during execution:

```rust
match atomic_cmd.execute(&mut world) {
    Ok(()) => println!("All commands succeeded"),
    Err(err) => {
        // Error message includes which command failed and confirms rollback
        eprintln!("Atomic command failed: {}", err);
        // Example: "Atomic command failed at sub-command 2/5: Entity not found. All changes rolled back."
    }
}
```

### Rollback Failures

If rollback itself fails (rare, but possible in edge cases):
- A warning is printed to stderr
- Rollback continues for remaining commands
- The world may be in an inconsistent state

This is a serious error that should be logged and investigated. In production, consider:
- Implementing a backup/restore mechanism
- Validating world state after rollback failures
- Providing recovery tools for users

## Performance Considerations

### Memory

Each `AtomicCommand` stores:
- A `Vec` of boxed sub-commands
- A description string
- An execution counter (usize)

For large batch operations (1000+ entities), consider:
- Breaking into smaller atomic batches
- Using progress indicators for user feedback
- Implementing command streaming for very large operations

### Execution Time

Atomic commands execute sub-commands sequentially:
- Time = sum of all sub-command execution times
- Rollback time = sum of all undo times for executed commands

For performance-critical operations:
- Minimize the number of sub-commands
- Use efficient command implementations
- Consider parallel execution (future enhancement)

## Testing

Comprehensive tests are provided in:
- `crates/luminara_core/src/atomic_command.rs` - Unit tests
- `crates/luminara_core/tests/atomic_command_test.rs` - Integration tests

Run tests:
```bash
# Unit tests
cargo test --package luminara_core --lib atomic_command

# Integration tests
cargo test --package luminara_core --test atomic_command_test
```

### Test Coverage

The test suite validates:
- ✅ All sub-commands succeed
- ✅ Partial failure triggers rollback
- ✅ First command failure (no rollback needed)
- ✅ Middle command failure (partial rollback)
- ✅ Last command failure (full rollback)
- ✅ Undo/redo functionality
- ✅ Integration with CommandHistory
- ✅ Empty atomic commands
- ✅ Complex multi-entity operations
- ✅ Nested atomic commands in history

## API Reference

### AtomicCommand

```rust
pub struct AtomicCommand {
    description: String,
    commands: Vec<Box<dyn UndoCommand>>,
    executed_count: usize,
}
```

#### Methods

##### `new(description: impl Into<String>) -> Self`

Create a new atomic command with the given description.

```rust
let atomic_cmd = AtomicCommand::new("Delete selection");
```

##### `add_command(&mut self, command: Box<dyn UndoCommand>)`

Add a sub-command to this atomic command. Sub-commands are executed in the order they are added.

```rust
atomic_cmd.add_command(Box::new(DestroyEntityCommand::new(entity)));
```

##### `command_count(&self) -> usize`

Get the number of sub-commands in this atomic command.

```rust
let count = atomic_cmd.command_count();
```

##### `is_empty(&self) -> bool`

Check if this atomic command has no sub-commands.

```rust
if atomic_cmd.is_empty() {
    println!("No operations to execute");
}
```

### UndoCommand Implementation

##### `execute(&mut self, world: &mut World) -> CommandResult<()>`

Execute all sub-commands atomically. If any fails, all previously executed commands are rolled back.

##### `undo(&mut self, world: &mut World) -> CommandResult<()>`

Undo all sub-commands in reverse order.

##### `description(&self) -> String`

Returns a description including the number of operations:
- Empty: `"Description (empty)"`
- With commands: `"Description (N operations)"`

## Design Patterns

### Builder Pattern

```rust
fn create_enemy_spawn(position: Vec3, count: usize) -> AtomicCommand {
    let mut cmd = AtomicCommand::new(format!("Spawn {} enemies", count));
    
    for i in 0..count {
        let offset = Vec3::new(i as f32 * 2.0, 0.0, 0.0);
        // Add spawn and configuration commands
        // ...
    }
    
    cmd
}
```

### Command Factory

```rust
struct CommandFactory;

impl CommandFactory {
    fn delete_selection(entities: &[Entity]) -> Box<dyn UndoCommand> {
        let mut cmd = AtomicCommand::new("Delete Selection");
        for &entity in entities {
            cmd.add_command(Box::new(DestroyEntityCommand::new(entity)));
        }
        Box::new(cmd)
    }
    
    fn duplicate_selection(entities: &[Entity], world: &World) -> Box<dyn UndoCommand> {
        let mut cmd = AtomicCommand::new("Duplicate Selection");
        // Spawn new entities and copy components
        // ...
        Box::new(cmd)
    }
}
```

## Future Enhancements

### Parallel Execution

For independent sub-commands, parallel execution could improve performance:

```rust
// Future API (not yet implemented)
let mut atomic_cmd = AtomicCommand::new("Batch operation");
atomic_cmd.set_parallel(true); // Enable parallel execution
// Sub-commands that don't conflict can execute in parallel
```

### Progress Tracking

For long-running atomic operations:

```rust
// Future API (not yet implemented)
atomic_cmd.set_progress_callback(|current, total| {
    println!("Progress: {}/{}", current, total);
});
```

### Conditional Execution

Skip certain sub-commands based on conditions:

```rust
// Future API (not yet implemented)
atomic_cmd.add_conditional_command(
    Box::new(some_command),
    |world| world.entity_exists(entity) // Only execute if condition is true
);
```

## See Also

- [Core Commands](./core_commands.md) - Individual command implementations
- [Undo Command System](./undo_command.md) - Base command trait and history
- [Command Dependencies](./command_dependencies.md) - Command ordering and dependencies
