# Operation Timeline

The Operation Timeline provides Git-like operation history with persistent undo/redo functionality and branch management. This system enables experimental workflows, selective undo, and AI-driven context generation.

## Features

### 1. Operation Recording

Operations are recorded with complete metadata:
- **Operation Type**: Classification of the operation (e.g., "SpawnEntity", "ModifyComponent")
- **Description**: Human-readable description
- **Commands**: Forward commands to execute the operation
- **Inverse Commands**: Commands to undo the operation
- **Affected Entities**: List of entities modified by the operation
- **Timestamp**: Unix timestamp for ordering
- **Branch**: Git-like branch name for organization
- **Intent** (optional): AI intent that generated the operation

```rust
use luminara_db::{LuminaraDatabase, OperationTimeline};
use serde_json::json;

let db = LuminaraDatabase::new_memory().await?;
let mut timeline = OperationTimeline::new(db, None);

// Record an operation
let op_id = timeline.record_operation(
    "SpawnEntity",
    "Spawned player entity",
    vec![json!({"type": "spawn", "entity": "player"})],
    vec![json!({"type": "despawn", "entity": "player"})],
    vec![],
).await?;

// Record with AI intent
let op_id = timeline.record_operation_with_intent(
    "ModifyComponent",
    "Increased player health",
    vec![json!({"health": 150})],
    vec![json!({"health": 100})],
    vec![player_entity_id],
    Some("Make the player stronger for the boss fight".to_string()),
).await?;
```

### 2. Undo/Redo

Standard undo/redo functionality with inverse command execution:

```rust
// Undo the last operation
if let Some((op_id, inverse_commands)) = timeline.undo().await? {
    println!("Undoing operation: {:?}", op_id);
    // Execute inverse commands to restore previous state
    for command in inverse_commands {
        execute_command(command)?;
    }
}

// Redo the next operation
if let Some((op_id, commands)) = timeline.redo().await? {
    println!("Redoing operation: {:?}", op_id);
    // Execute forward commands
    for command in commands {
        execute_command(command)?;
    }
}
```

### 3. Selective Undo

Undo specific operations in the history, even if they're not the most recent. The system automatically detects conflicts with dependent operations:

```rust
use surrealdb::RecordId;

// Try to undo a specific operation
match timeline.selective_undo(&operation_id).await? {
    Some((conflicts, inverse_commands)) if conflicts.is_empty() => {
        println!("Safe to undo, executing inverse commands");
        for command in inverse_commands {
            execute_command(command)?;
        }
    }
    Some((conflicts, _)) => {
        println!("Cannot undo due to {} conflicts:", conflicts.len());
        for conflict in conflicts {
            println!("  - {}", conflict);
        }
    }
    None => {
        println!("Operation not found");
    }
}
```

**Conflict Detection:**
- Detects if later operations modified the same entities
- Detects if later operations depend on this operation as a parent
- Provides detailed conflict descriptions

### 4. Branch Management

Git-like branch workflow for experimental changes:

```rust
// Create a new branch
timeline.create_branch("experimental-feature").await?;
println!("Now on branch: {}", timeline.current_branch());

// Record operations on the new branch
timeline.record_operation(
    "ExperimentalChange",
    "Trying a new approach",
    vec![],
    vec![],
    vec![],
).await?;

// Switch back to main branch
timeline.switch_branch("main").await?;

// List all branches
let branches = timeline.list_branches().await?;
for branch in branches {
    println!("Branch: {} ({} operations)", branch.name, branch.operation_count);
}

// Delete a branch (cannot delete current branch)
timeline.delete_branch("experimental-feature").await?;
```

### 5. Persistent Storage

All operations are persisted to the SurrealDB database, enabling:
- Recovery across sessions
- Long-term operation history
- Cross-session undo/redo

```rust
// Create timeline and record operations
let db = LuminaraDatabase::new("path/to/db").await?;
let mut timeline = OperationTimeline::new(db.clone(), None);

timeline.record_operation(/* ... */).await?;

// Simulate session restart
let db2 = LuminaraDatabase::new("path/to/db").await?;
let mut timeline2 = OperationTimeline::new(db2, None);

// Operations are still available
let history = timeline2.get_history(100).await?;
println!("Found {} operations from previous session", history.len());
```

### 6. AI Context Generation

Generate human-readable summaries for AI context:

```rust
// Detailed context with all metadata
let context = timeline.generate_ai_context(10, true).await?;
println!("{}", context);
// Output:
// Operation History (Branch: main, 3 operations):
//
// 1. [2024-12-16 10:30:45] SpawnEntity - Spawned player entity
//    Intent: Create the player character
//    Affected 1 entities
//    Commands: 1 operations
//
// 2. [2024-12-16 10:31:12] AddComponent - Added health component
//    Intent: Give the player health tracking
//    Affected 1 entities
//    Commands: 1 operations

// Compact summary for token efficiency
let summary = timeline.generate_compact_summary(5).await?;
println!("{}", summary);
// Output:
// Recent ops (main): SpawnEntity: Create the player character → AddComponent: Give the player health tracking
```

### 7. Timeline Statistics

Get comprehensive statistics about the timeline:

```rust
let stats = timeline.get_statistics().await?;
println!("Total operations: {}", stats.total_operations);
println!("Branch operations: {}", stats.branch_operations);
println!("Current branch: {}", stats.current_branch);
println!("Can undo: {} operations", stats.undoable_operations);
println!("Can redo: {} operations", stats.redoable_operations);
```

## Architecture

### Data Model

```rust
pub struct OperationRecord {
    pub id: Option<RecordId>,
    pub operation_type: String,
    pub description: String,
    pub commands: Vec<serde_json::Value>,
    pub inverse_commands: Vec<serde_json::Value>,
    pub affected_entities: Vec<RecordId>,
    pub timestamp: i64,
    pub parent: Option<RecordId>,
    pub branch: Option<String>,
    pub intent: Option<String>,
}
```

### Timeline Structure

Operations form a directed acyclic graph (DAG):
- Each operation has an optional parent
- Branches diverge from a common ancestor
- Undo/redo navigates the parent-child relationships

```
main branch:
  Op1 → Op2 → Op3 → Op4
         ↓
    feature branch:
      FeatureOp1 → FeatureOp2
```

## Requirements Validation

This implementation satisfies the following requirements:

### Requirement 27.1: Operation Recording
✅ **WHEN recording operations, THE System SHALL capture: intent, commands, inverse commands, timestamp, change summary**

- `record_operation_with_intent()` captures all required metadata
- Intent field stores AI-generated intent
- Commands and inverse commands stored as JSON
- Timestamp captured automatically
- Description provides change summary

### Requirement 27.2: Undo/Redo Correctness
✅ **WHEN undoing operations, THE System SHALL execute inverse commands and restore exact previous state**

- `undo()` returns inverse commands for execution
- `redo()` returns forward commands for execution
- Tests verify round-trip correctness
- State restoration is exact (no data loss)

### Additional Requirements Met:
✅ **Selective undo with conflict detection** - `selective_undo()` detects entity and dependency conflicts
✅ **Branch management** - Full Git-like workflow with create, switch, list, delete
✅ **Persistent storage** - All operations stored in SurrealDB
✅ **AI context generation** - `generate_ai_context()` and `generate_compact_summary()`
✅ **Round-trip testing** - Comprehensive test suite validates correctness

## Testing

The operation timeline includes comprehensive tests:

- **Basic operation recording** - Verifies operations are stored correctly
- **Undo/redo cycles** - Tests multiple undo/redo operations
- **Branch management** - Tests branch creation, switching, and deletion
- **Selective undo** - Tests conflict detection and safe undo
- **AI context generation** - Verifies context formatting
- **Persistent storage** - Tests cross-session operation recovery
- **Metadata completeness** - Validates all fields are captured
- **State preservation** - Verifies exact state restoration

Run tests:
```bash
cargo test --package luminara_db --test operation_timeline_test
```

## Performance Considerations

- **Query Optimization**: Operations are indexed by branch and timestamp
- **Batch Loading**: History queries support configurable limits
- **Memory Efficiency**: Operations stored in database, not memory
- **Conflict Detection**: O(n) where n is operations after target operation

## Future Enhancements

Potential improvements for future versions:

1. **Operation Compression**: Merge consecutive operations on the same entity
2. **Conflict Resolution**: Automatic conflict resolution strategies
3. **Branch Merging**: Merge operations from one branch into another
4. **Operation Tagging**: Tag operations for easier filtering
5. **Visual Timeline**: Generate visual representation of operation history
6. **Diff Generation**: Show detailed diffs between operations
7. **Operation Search**: Full-text search across operation descriptions and intents

## See Also

- [SurrealDB Integration](./README.md)
- [ECS Synchronization](./ecs_synchronization.md)
- [Asset Dependency Tracking](./asset_dependency_tracking.md)
