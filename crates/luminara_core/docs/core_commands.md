# Core Commands

This document describes the core command implementations for editor operations in Luminara Engine.

## Overview

The core commands module provides concrete implementations of the `UndoCommand` trait for common editor operations. These commands enable undo/redo functionality for all entity and component manipulations.

## Requirements

**Validates: Requirement 9.4**
- THE System SHALL provide commands for: SpawnEntity, DestroyEntity, AddComponent, RemoveComponent, ModifyComponent, ModifyTransform

## Command Types

### SpawnEntityCommand

Creates a new entity in the world.

**Usage:**
```rust
use luminara_core::{SpawnEntityCommand, CommandHistory, World};

let mut world = World::new();
let mut history = CommandHistory::new(100);

let cmd = Box::new(SpawnEntityCommand::new());
history.execute(cmd, &mut world).unwrap();

// Undo to remove the entity
history.undo(&mut world).unwrap();
```

**Behavior:**
- **Execute**: Spawns a new entity and captures its ID
- **Undo**: Despawns the entity

### DestroyEntityCommand

Removes an entity from the world.

**Usage:**
```rust
let entity = world.spawn();
let cmd = Box::new(DestroyEntityCommand::new(entity));
history.execute(cmd, &mut world).unwrap();

// Undo to restore the entity
history.undo(&mut world).unwrap();
```

**Behavior:**
- **Execute**: Despawns the entity (TODO: capture component data for full restoration)
- **Undo**: Spawns a new entity (TODO: restore all components using reflection)

**Note**: Due to current limitations, the entity ID may change after undo. Full component restoration requires reflection system integration.

### AddComponentCommand<T>

Adds a component to an entity.

**Usage:**
```rust
#[derive(Clone)]
struct Position { x: f32, y: f32 }
impl Component for Position { fn type_name() -> &'static str { "Position" } }

let entity = world.spawn();
let cmd = Box::new(AddComponentCommand::new(entity, Position { x: 10.0, y: 20.0 }));
history.execute(cmd, &mut world).unwrap();

// Undo to remove the component
history.undo(&mut world).unwrap();
```

**Behavior:**
- **Execute**: Adds the component to the entity, captures whether it already existed
- **Undo**: Removes the component only if it didn't exist before

### RemoveComponentCommand<T>

Removes a component from an entity.

**Usage:**
```rust
world.add_component(entity, Position { x: 5.0, y: 15.0 }).unwrap();

let cmd = Box::new(RemoveComponentCommand::<Position>::new(entity));
history.execute(cmd, &mut world).unwrap();

// Undo to restore the component
history.undo(&mut world).unwrap();
```

**Behavior:**
- **Execute**: Removes the component and captures its previous value
- **Undo**: Restores the component with its previous value

**Error Handling**: Returns an error if the entity doesn't have the component.

### ModifyComponentCommand<T>

Modifies a component's value.

**Usage:**
```rust
world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();

let cmd = Box::new(ModifyComponentCommand::new(entity, Position { x: 100.0, y: 200.0 }));
history.execute(cmd, &mut world).unwrap();

// Undo to restore the original value
history.undo(&mut world).unwrap();
```

**Behavior:**
- **Execute**: Sets the component to the new value, captures the old value
- **Undo**: Restores the old value (or removes the component if it didn't exist)

**Special Case**: If the component doesn't exist, it will be created. Undo will then remove it.

### ModifyTransformCommand

Specialized command for modifying Transform components (requires `math` feature).

**Usage:**
```rust
use luminara_math::Transform;

let cmd = Box::new(ModifyTransformCommand::new(entity, Transform::from_xyz(10.0, 20.0, 30.0)));
history.execute(cmd, &mut world).unwrap();
```

**Behavior**: Same as `ModifyComponentCommand<Transform>`, provided as a convenience since transform modifications are very common in editors.

## Integration with CommandHistory

All commands integrate seamlessly with `CommandHistory`:

```rust
let mut world = World::new();
let mut history = CommandHistory::new(100);

// Execute a sequence of commands
let entity = world.spawn();
history.execute(Box::new(AddComponentCommand::new(entity, Position { x: 0.0, y: 0.0 })), &mut world).unwrap();
history.execute(Box::new(ModifyComponentCommand::new(entity, Position { x: 10.0, y: 20.0 })), &mut world).unwrap();

// Undo all operations
while history.can_undo() {
    history.undo(&mut world).unwrap();
}

// Redo all operations
while history.can_redo() {
    history.redo(&mut world).unwrap();
}
```

## Command Merging

Command merging is currently not supported. The `can_merge` method returns `false` for all commands because proper implementation requires downcasting support (Any trait).

Future enhancement: Implement command merging for consecutive modifications of the same component to reduce history size.

## Testing

Comprehensive tests are provided in:
- `crates/luminara_core/src/commands.rs` - Unit tests for each command
- `crates/luminara_core/tests/core_commands_test.rs` - Integration tests validating Requirement 9.4

Run tests with:
```bash
cargo test --package luminara_core --lib commands::tests
cargo test --package luminara_core --test core_commands_test
```

## Future Enhancements

1. **Full Component Restoration**: Integrate with reflection system to capture and restore all components in `DestroyEntityCommand`
2. **Command Merging**: Implement proper downcasting to enable merging of consecutive modifications
3. **Batch Commands**: Add support for atomic multi-entity operations
4. **Command Dependencies**: Track and enforce execution order for dependent commands

## See Also

- [Undo Command System](./undo_command.md) - Base command pattern implementation
- [Reflection System](../reflect.rs) - Runtime type information for component capture
- [World API](../world.rs) - Entity and component management
