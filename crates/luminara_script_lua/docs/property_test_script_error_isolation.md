# Property Test: Script Error Isolation

## Overview

This document describes the property-based test for **Property 19: Script Error Isolation**, which validates **Requirements 17.1** from the pre-editor-engine-audit spec.

## Property Statement

**For any Lua or WASM script error, the error should be caught and logged without crashing the engine or affecting other scripts.**

## Test Implementation

The property test is implemented in `tests/property_script_error_isolation_test.rs` using the `quickcheck` framework.

### Test Coverage

The test suite includes 6 main property tests and 4 additional unit tests:

#### Property Tests (100+ iterations each)

1. **`property_runtime_error_does_not_crash_engine`**
   - Generates random runtime errors (8 different types)
   - Verifies the engine doesn't crash when scripts error
   - Tests: nil indexing, undefined functions, arithmetic errors, explicit errors, type mismatches, division by zero, table access errors, concatenation errors

2. **`property_error_in_one_script_does_not_affect_others`**
   - Creates 1-5 valid scripts alongside 1 error script
   - Verifies that errors in one script don't prevent other scripts from running
   - Runs 10 update cycles to ensure stability

3. **`property_compilation_error_does_not_crash_engine`**
   - Generates scripts with syntax errors
   - Verifies compilation errors are caught and returned as errors (not panics)

4. **`property_multiple_errors_in_sequence`**
   - Creates 1-10 scripts that all produce errors
   - Runs 5 update cycles with all error scripts
   - Verifies the engine handles multiple simultaneous errors gracefully

5. **`property_error_then_reload_with_fix`**
   - Loads a script with an error
   - Runs update (should not crash)
   - Fixes the script and reloads it
   - Verifies the engine recovers after script is fixed

6. **`property_lifecycle_hook_error_isolation`**
   - Creates scripts with errors in `on_start` hook
   - Verifies errors in one lifecycle hook don't prevent other hooks from running
   - Tests that `on_update` still works even if `on_start` errors

#### Unit Tests

1. **`test_nil_index_error_isolation`**
   - Specific test for nil indexing error
   - Verifies it doesn't panic

2. **`test_multiple_scripts_with_mixed_errors`**
   - 1 error script + 2 good scripts
   - Runs 10 updates
   - Verifies good scripts continue to work

3. **`test_compilation_error_provides_details`**
   - Verifies compilation errors include useful information
   - Checks error message contains "Compilation error" or "syntax"

4. **`test_engine_stability_after_100_errors`**
   - Runs 100 update cycles with a script that errors every time
   - Verifies the engine remains stable after repeated errors

### Error Types Tested

The test generates 8 different types of Lua runtime errors:

1. **IndexNil**: `local x = nil; local y = x.field`
2. **UndefinedFunction**: `undefined_function()`
3. **ArithmeticError**: `local x = nil; local y = x + 5`
4. **ExplicitError**: `error("Intentional error")`
5. **TypeMismatch**: `local x = "string"; x()`
6. **DivisionByZero**: `local x = nil; local y = x + (1/0)`
7. **TableAccessError**: `local t = {}; local v = t.a.b`
8. **ConcatError**: `local x = nil; local s = "test" .. x`

### Test Strategy

The property tests use `quickcheck` to:
- Generate random error types
- Generate random numbers of scripts (1-5 good scripts, 1-10 error scripts)
- Run multiple update cycles (5-100 iterations)
- Test error recovery scenarios

Each property test runs 100 times by default (configurable via `QUICKCHECK_TESTS` environment variable).

## Validation

The tests validate the following properties:

1. **No Crashes**: The engine never panics or crashes due to script errors
2. **Error Isolation**: Errors in one script don't affect other scripts
3. **Graceful Degradation**: The engine continues to function after errors
4. **Error Reporting**: Compilation errors are caught and reported with details
5. **Recovery**: The engine can recover after a script is fixed and reloaded
6. **Lifecycle Isolation**: Errors in one lifecycle hook don't prevent other hooks from running
7. **Repeated Errors**: The engine remains stable even after 100+ consecutive errors

## Running the Tests

```bash
# Run all property tests
cargo test --package luminara_script_lua --test property_script_error_isolation_test

# Run with more iterations (default is 100)
QUICKCHECK_TESTS=1000 cargo test --package luminara_script_lua --test property_script_error_isolation_test

# Run specific property test
cargo test --package luminara_script_lua --test property_script_error_isolation_test property_runtime_error_does_not_crash_engine

# Run with output
cargo test --package luminara_script_lua --test property_script_error_isolation_test -- --nocapture
```

## Implementation Notes

### Error Handling in LuaScriptRuntime

The test relies on the error handling implementation in `LuaScriptRuntime`:

1. **`safe_call`**: Wraps Lua function calls and catches errors
2. **`extract_stack_trace`**: Extracts detailed stack traces from Lua errors
3. **`update` loop**: Isolates errors per script - if one script errors, others continue

### Key Implementation Details

```rust
// In update loop - errors are isolated per script
for script in self.scripts.values() {
    if let Err(e) = func.call::<_, ()>((dt, input_ud.clone(), world_ud.clone())) {
        let stack_trace = self.extract_stack_trace(&e);
        eprintln!("Error in script {:?}: {}\n{}", script.id, e, stack_trace);
        // Continue processing other scripts instead of propagating error
    }
}
```

This ensures that:
- Each script's execution is isolated
- Errors are logged but don't propagate
- The update loop continues even if scripts error
- The engine remains stable

## Success Criteria

The property test passes if:
- All 6 property tests pass with 100+ iterations each
- All 4 unit tests pass
- No panics or crashes occur during any test
- The engine remains stable after all error scenarios

## Related Documentation

- [Error Handling Documentation](./error_handling.md)
- [Pre-Editor Engine Audit Spec](.kiro/specs/pre-editor-engine-audit/requirements.md)
- [Design Document](.kiro/specs/pre-editor-engine-audit/design.md)

## Future Enhancements

Potential improvements for future versions:
1. Test WASM script error isolation (currently only tests Lua)
2. Test error rate limiting
3. Test memory leak detection after repeated errors
4. Test concurrent script execution with errors
5. Test error recovery callbacks
