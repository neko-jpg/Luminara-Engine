# Plugin Execution Order Verification

**Task 21.3: Verify Plugin Execution Order**

**Validates: Requirements 16.2**

## Overview

This document describes the test suite that verifies the plugin system correctly handles execution order, stage placement, and prevents race conditions when plugins register systems.

## Requirement 16.2

> WHEN plugins register systems, THE System SHALL respect declared execution order and stage placement

## Test Implementation

### Location

`crates/luminara_core/tests/plugin_execution_order_test.rs`

### Test Coverage

The test suite provides comprehensive coverage of three critical aspects:

#### 1. Declared Execution Order Respected

**Tests:**
- `test_systems_execute_in_registration_order_same_stage`
- `test_multiple_plugins_respect_registration_order`
- `test_deterministic_execution_across_multiple_runs`

**Verification:**
- Systems registered in a specific order execute in that order within the same stage
- When multiple plugins register systems, the systems execute in the order the plugins were registered
- Execution order is consistent across multiple update cycles

**Example:**
```rust
// Plugin 1 registers system_a, system_b
// Plugin 2 registers system_c, system_d
// Execution order: system_a → system_b → system_c → system_d
```

#### 2. Stage Placement Correct

**Tests:**
- `test_systems_execute_in_correct_stages`
- `test_all_stages_execute_in_correct_order`
- `test_stage_placement_with_multiple_plugins`
- `test_startup_stage_executes_before_other_stages`

**Verification:**
- Systems execute in their declared stages
- Stages execute in the correct order:
  - Startup (once, before all others)
  - PreUpdate
  - Update
  - FixedUpdate
  - PostUpdate
  - PreRender
  - Render
  - PostRender
- Multiple plugins adding systems to different stages maintain correct stage execution order

**Example:**
```rust
// Plugin registers:
// - PreUpdate: pre_system
// - Update: update_system
// - PostUpdate: post_system
// Execution order: pre_system → update_system → post_system
```

#### 3. No Race Conditions

**Tests:**
- `test_systems_with_shared_resource_no_race_condition`
- `test_complex_plugin_ordering_scenario`

**Verification:**
- Systems accessing shared resources don't have race conditions
- The schedule ensures proper synchronization
- Complex scenarios with multiple plugins, stages, and systems work correctly

**Example:**
```rust
// 10 systems all increment a shared counter
// 10 update cycles
// Expected result: counter = 100 (no race conditions)
```

## Test Infrastructure

### ExecutionTracker

A resource that tracks system execution order:

```rust
struct ExecutionTracker {
    executions: Arc<Mutex<Vec<(String, CoreStage)>>>,
}
```

This allows tests to verify:
- Which systems executed
- In what order they executed
- In which stage they executed

### OrderedSystemPlugin

A test plugin that registers systems in a specific order:

```rust
struct OrderedSystemPlugin {
    name: String,
    systems: Vec<(CoreStage, String)>,
}
```

This allows tests to:
- Register multiple systems in different stages
- Track when each system executes
- Verify execution order matches registration order

## Test Results

All 9 tests pass successfully:

```
test test_systems_execute_in_registration_order_same_stage ... ok
test test_multiple_plugins_respect_registration_order ... ok
test test_systems_execute_in_correct_stages ... ok
test test_all_stages_execute_in_correct_order ... ok
test test_stage_placement_with_multiple_plugins ... ok
test test_systems_with_shared_resource_no_race_condition ... ok
test test_deterministic_execution_across_multiple_runs ... ok
test test_complex_plugin_ordering_scenario ... ok
test test_startup_stage_executes_before_other_stages ... ok
```

## Key Findings

### ✅ Execution Order Respected

The plugin system correctly maintains execution order:
- Systems execute in registration order within the same stage
- Plugin registration order determines system execution order
- Order is deterministic across multiple runs

### ✅ Stage Placement Correct

The schedule correctly handles stage placement:
- Systems execute in their declared stages
- Stages execute in the correct order
- Startup stage executes before all other stages
- Multiple plugins can add to the same stage without conflicts

### ✅ No Race Conditions

The system ensures thread safety:
- Systems with shared resource access don't have race conditions
- The schedule properly synchronizes system execution
- Complex scenarios with multiple plugins work correctly

## Implementation Details

### How Execution Order is Maintained

1. **Plugin Registration**: When a plugin is registered, its `build()` method is called immediately
2. **System Registration**: Systems are added to the schedule in the order they're registered
3. **Stage Execution**: The schedule runs stages in a fixed order
4. **Within-Stage Order**: Systems within a stage execute in registration order (with parallelization where safe)

### How Race Conditions are Prevented

1. **System Access Analysis**: Each system declares what components/resources it reads/writes
2. **Conflict Detection**: The schedule detects when systems have conflicting access
3. **Batching**: Systems with non-conflicting access can run in parallel
4. **Synchronization**: Systems with conflicting access run sequentially

## Conclusion

The test suite comprehensively verifies that the plugin system correctly handles execution order, stage placement, and prevents race conditions. All tests pass, confirming that Requirement 16.2 is fully satisfied.

## Related Documentation

- [Plugin System](./plugin_dependencies.md)
- [Plugin Dependencies](./plugin_dependencies.md)
- [Core Commands](./core_commands.md)
