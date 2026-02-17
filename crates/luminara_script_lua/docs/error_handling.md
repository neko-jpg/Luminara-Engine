# Enhanced Lua Error Handling

## Overview

The Luminara Lua scripting system provides comprehensive error handling with detailed stack traces, error isolation, and graceful degradation. This ensures that script errors never crash the engine and provide developers with actionable debugging information.

## Features

### 1. Detailed Stack Traces

All Lua errors now include:
- **Script path**: The file path of the script that caused the error
- **Error message**: A clear description of what went wrong
- **Stack trace**: Full Lua call stack showing where the error occurred

Example error output:
```
Runtime error in assets/scripts/player.lua:
attempt to index a nil value (local 'x')

Stack trace:
stack traceback:
    assets/scripts/player.lua:7: in function 'on_start'
    [C]: in function 'call_lifecycle'
```

### 2. Error Isolation

Errors in one script do not affect other scripts:
- Each script's execution is isolated
- Errors are caught and logged without propagating
- Other scripts continue to run normally
- The engine remains stable even with buggy scripts

### 3. Error Types

The system distinguishes between different error types:

#### Compilation Errors
Syntax errors detected when loading or reloading a script:
```rust
ScriptError::Compilation {
    script_path: String,
    message: String,
    stack_trace: String,
}
```

Example:
- Missing `end` keyword
- Invalid syntax
- Malformed expressions

#### Runtime Errors
Errors that occur during script execution:
```rust
ScriptError::Runtime {
    script_path: String,
    message: String,
    stack_trace: String,
}
```

Example:
- Attempting to index nil
- Calling undefined functions
- Type mismatches

#### Script Not Found
When trying to access a script that doesn't exist:
```rust
ScriptError::ScriptNotFound(String)
```

### 4. Hot Reload Error Handling

When reloading scripts, errors are handled gracefully:
- Compilation errors during reload are logged but don't crash
- The old script version remains active if reload fails
- State preservation is attempted even if errors occur
- Detailed error messages help fix issues quickly

## Usage Examples

### Loading a Script with Error Handling

```rust
use luminara_script_lua::LuaScriptRuntime;

let mut runtime = LuaScriptRuntime::new()?;

match runtime.load_script(Path::new("assets/scripts/player.lua")) {
    Ok(script_id) => {
        println!("Script loaded successfully: {:?}", script_id);
    }
    Err(e) => {
        eprintln!("Failed to load script: {}", e);
        // Error includes full path, message, and stack trace
    }
}
```

### Calling Lifecycle Hooks with Error Handling

```rust
// Errors are caught and returned as Result
match runtime.call_lifecycle(script_id, "on_start") {
    Ok(_) => println!("Lifecycle hook executed successfully"),
    Err(e) => eprintln!("Error in lifecycle hook: {}", e),
}
```

### Update Loop with Error Isolation

```rust
// Errors in individual scripts are isolated
// The update loop continues even if one script fails
runtime.update(dt, &mut world, &input)?;

// If script A errors, script B still runs
// Errors are logged to stderr with full context
```

## Error Recovery Strategies

### 1. Graceful Degradation
When a script errors:
1. The error is logged with full context
2. Other scripts continue to execute
3. The engine remains stable
4. The problematic script can be fixed and hot-reloaded

### 2. Hot Reload Recovery
If a script reload fails:
1. The old version remains active
2. The error is logged with details
3. You can fix the script and try again
4. No need to restart the engine

### 3. State Preservation
During hot reload:
1. Script state is saved via `on_save()` hook
2. If reload succeeds, state is restored via `on_restore()` hook
3. If hooks error, the error is logged but reload continues
4. Non-function fields are automatically copied

## Best Practices

### 1. Always Check Return Values
```lua
local module = {}

function module.on_start()
    -- Check for nil before accessing
    if self.player then
        self.player:move()
    else
        print("Warning: player not initialized")
    end
end

return module
```

### 2. Use pcall for Risky Operations
```lua
function module.on_update(dt, input, world)
    local success, err = pcall(function()
        -- Risky operation
        self:complex_calculation()
    end)
    
    if not success then
        print("Error in calculation: " .. tostring(err))
        -- Fallback behavior
    end
end
```

### 3. Implement State Hooks for Hot Reload
```lua
function module.on_save()
    return {
        position = self.position,
        health = self.health,
        -- Save important state
    }
end

function module.on_restore(state)
    self.position = state.position
    self.health = state.health
    -- Restore saved state
end
```

### 4. Log Errors Appropriately
```lua
function module.on_start()
    if not self:initialize() then
        error("Failed to initialize: missing required components")
    end
end
```

## Implementation Details

### Stack Trace Extraction
The system uses Lua's debug library to extract detailed stack traces:
```rust
fn extract_stack_trace(&self, error: &mlua::Error) -> String {
    match error {
        mlua::Error::CallbackError { traceback, cause } => {
            format!("Callback error:\n{}\nCause: {}", traceback, cause)
        }
        mlua::Error::RuntimeError(msg) => {
            // Try to get debug traceback
            if let Ok(debug_traceback) = self.lua.load("debug.traceback()").eval::<String>() {
                format!("{}\n\nStack trace:\n{}", msg, debug_traceback)
            } else {
                msg.clone()
            }
        }
        // ... other error types
    }
}
```

### Error Isolation in Update Loop
```rust
pub fn update(&mut self, dt: f32, world: &mut World, input: &Input) -> Result<(), ScriptError> {
    for script in self.scripts.values() {
        // Each script is executed in isolation
        if let Err(e) = func.call::<_, ()>((dt, input_ud.clone(), world_ud.clone())) {
            let stack_trace = self.extract_stack_trace(&e);
            eprintln!(
                "Error in script {:?} ({}) on_update:\n{}\n\nStack trace:\n{}",
                script.id,
                script.path.display(),
                e,
                stack_trace
            );
            // Continue processing other scripts instead of propagating error
        }
    }
    Ok(())
}
```

## Testing

The error handling system is thoroughly tested:
- `test_compilation_error_with_stack_trace`: Verifies compilation errors include full context
- `test_runtime_error_with_stack_trace`: Verifies runtime errors include stack traces
- `test_error_isolation_in_update`: Verifies one script's error doesn't affect others
- `test_script_not_found_error`: Verifies proper handling of missing scripts
- `test_reload_with_compilation_error`: Verifies reload error handling
- `test_reload_with_runtime_error`: Verifies runtime error handling during reload

## Performance Considerations

- Stack trace extraction has minimal overhead (only on error)
- Error isolation adds no overhead to successful script execution
- Error logging is asynchronous (uses eprintln!)
- No performance impact on production builds when scripts are error-free

## Future Enhancements

Potential improvements for future versions:
1. Error rate limiting to prevent log spam
2. Error recovery callbacks for custom handling
3. Script error metrics and monitoring
4. Automatic script disabling after repeated errors
5. Integration with external error tracking services
