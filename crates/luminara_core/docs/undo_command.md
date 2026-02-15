# Undo/Redo Command Pattern

## Overview

The undo/redo command pattern provides a foundation for editor operations that can be reversed. This implementation satisfies Requirements 9.1 and 9.2 from the Pre-Editor Engine Audit specification.

## Architecture

### UndoCommand Trait

The `UndoCommand` trait defines the interface for all undoable operations:

```rust
pub trait UndoCommand: Send + Sync {
    fn execute(&mut self, world: &mut World) -> CommandResult<()>;
    fn undo(&mut self, world: &mut World) -> CommandResult<()>;
    fn description(&self) -> String;
    fn can_merge(&self, other: &dyn UndoCommand) -> bool;
    fn merge(&mut self, other: Box<dyn UndoCommand>) -> CommandResult<()>;
}
```

**Key Features:**
- **execute**: Performs the operation and captures state needed for undo
- **undo**: Reverses the operation using captured state
- **description**: Provides human-readable description for UI
- **can_merge/merge**: Enables optimization by combining similar commands

### CommandHistory

The `CommandHistory` struct manages the undo/redo stack:

```rust
pub struct CommandHistory {
    history: Vec<Box<dyn UndoCommand>>,
    current: usize,
    max_size: usize,
}
```

**Key Features:**
- Maintains a stack of executed commands
- Tracks current position for undo/redo
- Automatically discards redo history when new commands are executed
- Enforces maximum history size to prevent unbounded memory growth
- Supports command merging for optimization

## Usage Example

```rust
use luminara_core::{CommandHistory, UndoCommand, World};

// Define a custom command
struct MoveEntityCommand {
    entity: Entity,
    old_position: Option<Position>,
    new_position: Position,
}

impl UndoCommand for MoveEntityCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        // Capture old state
        self.old_position = world.get_component::<Position>(self.entity).cloned();
        
        // Apply new state
        world.add_component(self.entity, self.new_position.clone())?;
        Ok(())
    }
    
    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        // Restore old state
        if let Some(old_pos) = &self.old_position {
            world.add_component(self.entity, old_pos.clone())?;
        }
        Ok(())
    }
    
    fn description(&self) -> String {
        format!("Move Entity {:?}", self.entity)
    }
}

// Use the command
let mut world = World::new();
let mut history = CommandHistory::new(100);

let cmd = Box::new(MoveEntityCommand { /* ... */ });
history.execute(cmd, &mut world)?;

// Undo
history.undo(&mut world)?;

// Redo
history.redo(&mut world)?;
```

## Requirements Validation

### Requirement 9.1: Command Trait with Execute and Undo

✅ **Satisfied**: The `UndoCommand` trait provides both `execute` and `undo` methods.

**Evidence:**
- `execute` method performs operations and captures state
- `undo` method reverses operations using captured state
- All tests in `undo_command_test.rs` verify this functionality

### Requirement 9.2: Record Sufficient State for Undo

✅ **Satisfied**: Commands capture all necessary state during execution to enable complete reversal.

**Evidence:**
- `ModifyComponentCommand` captures old component values before modification
- `AddComponentCommand` tracks whether component existed before addition
- `SpawnEntityCommand` stores entity ID for despawning
- Test `test_requirement_9_2_record_sufficient_state_for_undo` verifies state preservation

## Command Merging

Command merging is an optimization that combines multiple similar commands into a single command. This is useful for operations like dragging an entity, where many small position updates can be merged into a single "move" command.

**Example:**
```rust
impl UndoCommand for MoveEntityCommand {
    fn can_merge(&self, other: &dyn UndoCommand) -> bool {
        // Check if other is also a MoveEntityCommand for the same entity
        // (Simplified - real implementation would use proper type checking)
        other.description().contains(&format!("{:?}", self.entity))
    }
    
    fn merge(&mut self, other: Box<dyn UndoCommand>) -> CommandResult<()> {
        // Update new_position to the other command's new_position
        // Keep the original old_position
        Ok(())
    }
}
```

## Error Handling

The command system uses a custom `CommandError` type:

```rust
pub enum CommandError {
    WorldError(WorldError),
    CommandError(String),
    NoUndo,
    NoRedo,
}
```

All command operations return `CommandResult<T>` which is `Result<T, CommandError>`.

## Testing

Comprehensive tests are provided in:
- `crates/luminara_core/src/undo_command.rs` - Unit tests for basic functionality
- `crates/luminara_core/tests/undo_command_test.rs` - Integration tests with World

**Test Coverage:**
- Basic execute/undo/redo operations
- Command history management
- Max size enforcement
- Redo history discarding
- State preservation across undo/redo
- Error handling
- Complex command sequences

## Future Enhancements

The following enhancements are planned for subsequent tasks:

1. **Concrete Command Implementations** (Task 12.3):
   - SpawnEntityCommand
   - DestroyEntityCommand
   - AddComponentCommand
   - RemoveComponentCommand
   - ModifyComponentCommand
   - ModifyTransformCommand

2. **Atomic Command Execution** (Task 12.4):
   - Multi-entity commands
   - All-or-nothing execution
   - Partial failure handling

3. **Command Dependencies** (Task 12.5):
   - Dependency tracking
   - Execution order enforcement
   - Circular dependency detection

## Integration with Editor

The command pattern is designed to integrate seamlessly with the editor:

1. **UI Integration**: Command descriptions can be displayed in undo/redo menus
2. **Keyboard Shortcuts**: Ctrl+Z/Ctrl+Y can trigger undo/redo
3. **Command Palette**: All commands can be listed and executed from a command palette
4. **History Panel**: Command history can be visualized in a timeline panel
5. **Branching**: Future enhancement could support branching undo history

## Performance Considerations

- **Memory**: History is bounded by `max_size` to prevent unbounded growth
- **Command Merging**: Reduces memory usage for repetitive operations
- **Cloning**: Commands may need to clone component data - consider using `Arc` for large data
- **Execution Time**: Commands should be fast - avoid expensive operations in execute/undo

## Thread Safety

All commands must implement `Send + Sync` to enable:
- Execution on background threads
- Parallel command processing (future enhancement)
- Integration with async editor operations


## Atomic Command Execution

The `AtomicCommand` provides all-or-nothing semantics for operations affecting multiple entities. If any sub-command fails during execution, all previously executed sub-commands are automatically rolled back.

### Usage

```rust
use luminara_core::{AtomicCommand, AddComponentCommand, World};

let mut world = World::new();
let entity1 = world.spawn();
let entity2 = world.spawn();

// Create an atomic command that adds components to multiple entities
let mut atomic_cmd = AtomicCommand::new("Initialize entities");
atomic_cmd.add_command(Box::new(AddComponentCommand::new(
    entity1,
    Position { x: 1.0, y: 2.0 },
)));
atomic_cmd.add_command(Box::new(AddComponentCommand::new(
    entity2,
    Position { x: 3.0, y: 4.0 },
)));

// Execute atomically - either both succeed or both fail
atomic_cmd.execute(&mut world)?;
```

### Requirements Validation

#### Requirement 9.5: Atomic Execution (All or Nothing)

✅ **Satisfied**: The `AtomicCommand` ensures that commands affecting multiple entities either fully succeed or fully fail.

**Evidence:**
- If any sub-command fails, all previously executed sub-commands are rolled back
- The world remains in a consistent state even when partial failures occur
- Test `test_requirement_9_5_atomic_execution_partial_failure` verifies rollback behavior
- Test `test_requirement_9_5_atomic_execution_all_succeed` verifies successful execution

### Implementation Details

The `AtomicCommand` tracks the number of successfully executed sub-commands. If a sub-command fails:

1. The failure is detected immediately
2. All previously executed sub-commands are undone in reverse order
3. An error is returned indicating which sub-command failed
4. The world state is restored to its pre-execution state

### Error Handling

When a sub-command fails, the error message includes:
- The index of the failed sub-command
- The total number of sub-commands
- The original error message
- Confirmation that all changes were rolled back

Example error message:
```
Atomic command failed at sub-command 2/3: Entity does not have component Position. All changes rolled back.
```

### Integration with CommandHistory

`AtomicCommand` integrates seamlessly with `CommandHistory`:

```rust
let mut history = CommandHistory::new(100);
let mut atomic_cmd = AtomicCommand::new("Multi-entity operation");
// ... add sub-commands ...

// Execute through history
history.execute(Box::new(atomic_cmd), &mut world)?;

// Undo all sub-commands atomically
history.undo(&mut world)?;

// Redo all sub-commands atomically
history.redo(&mut world)?;
```

### Best Practices

1. **Group Related Operations**: Use atomic commands for operations that must succeed or fail together
2. **Descriptive Names**: Provide clear descriptions that explain what the atomic operation does
3. **Error Recovery**: Handle atomic command failures gracefully in your application
4. **Testing**: Always test both success and failure paths for atomic commands

### Performance Considerations

- **Rollback Cost**: If a command fails late in the sequence, many undo operations may be required
- **Memory**: Each sub-command must store state for undo, which can add up for large atomic operations
- **Ordering**: Place commands most likely to fail early in the sequence to minimize rollback cost
