# WASM Hot-Reload

## Overview

The WASM scripting runtime supports hot-reloading modules by replacing the module instance with a newly compiled version. Unlike Lua scripts, WASM modules are stateless by design, so state preservation must be handled externally.

## How It Works

When a WASM module is reloaded:

1. **Compilation**: The new WASM bytecode is compiled into a `Module`
2. **Instantiation**: The new module is instantiated with the same linker (host functions)
3. **Replacement**: The old module instance is replaced with the new one
4. **State Loss**: All internal WASM state (globals, memory) is reset

## State Management

### WASM Modules Are Stateless

WASM modules do not preserve internal state across reloads:

```wat
(module
    (global $counter (mut i32) (i32.const 0))
    
    (func (export "increment")
        global.get $counter
        i32.const 1
        i32.add
        global.set $counter
    )
    
    (func (export "get_counter") (result i32)
        global.get $counter
    )
)
```

After calling `increment()` three times, `$counter` will be 3. But after `reload_module()`, `$counter` resets to 0.

### External State Management

To preserve state across reloads, store it in ECS components:

```rust
// Store state in ECS component
#[derive(Component)]
struct ScriptState {
    counter: i32,
    health: f32,
    position: Vec3,
}

// Before reload: read state from WASM
let counter = wasm_runtime.call_json_func(script_id, "get_counter", ())?;
world.insert(entity, ScriptState { counter, .. })?;

// Reload module
wasm_runtime.reload_module(script_id, new_bytecode)?;

// After reload: restore state to WASM
let state = world.get::<ScriptState>(entity)?;
wasm_runtime.call_json_func(script_id, "set_counter", state.counter)?;
```

## Error Handling

If a reload fails (invalid WASM, instantiation error), the old module remains active:

```rust
let result = runtime.reload_module(script_id, invalid_wasm);

if result.is_err() {
    // Old module still works
    let old_result = runtime.call_json_func(script_id, "test", ())?;
}
```

## Multiple Reloads

Multiple consecutive reloads work correctly, with each reload resetting state:

```rust
// Load v1
let id = runtime.load_module(wasm_v1)?;

// Reload to v2
runtime.reload_module(id, wasm_v2)?;

// Reload to v3
runtime.reload_module(id, wasm_v3)?;

// Each reload resets internal state
```

## Function Signature Changes

You can change function signatures across reloads:

```wat
;; v1: add takes two i32 parameters
(func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
)
```

After reload:

```wat
;; v2: add takes three i32 parameters
(func (export "add") (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
)
```

The new signature is immediately available after reload.

## Host Function Compatibility

Host functions (registered in the linker) remain the same across reloads. The new module must be compatible with the existing host functions:

```rust
// Host functions registered once
linker.func_wrap("host", "log", |ptr: i32, len: i32| { ... })?;

// All reloaded modules can use this host function
```

If a reloaded module tries to import a non-existent host function, instantiation will fail and the old module will remain active.

## Best Practices

### 1. Store State Externally

Never rely on WASM internal state persisting across reloads:

```rust
// Bad: Assuming WASM state persists
wasm.call("increment", ())?;
wasm.reload(new_bytecode)?;
let count = wasm.call("get_counter", ())?; // Will be 0, not 1

// Good: Store state in ECS
let count = wasm.call("get_counter", ())?;
world.insert(entity, Counter(count))?;
wasm.reload(new_bytecode)?;
let counter = world.get::<Counter>(entity)?;
wasm.call("set_counter", counter.0)?;
```

### 2. Provide State Serialization Functions

Export functions for getting/setting state:

```wat
(func (export "get_state") (result i32 i32)
    ;; Return (ptr, len) to JSON state
)

(func (export "set_state") (param i32 i32)
    ;; Restore from JSON state
)
```

### 3. Test Hot-Reload Behavior

Test that your WASM modules work correctly after reload:

```rust
#[test]
fn test_my_module_reload() {
    let mut runtime = WasmScriptRuntime::new(limits)?;
    let id = runtime.load_module(wasm_v1)?;
    
    // Set up state
    runtime.call_json_func(id, "init", initial_state)?;
    
    // Reload
    runtime.reload_module(id, wasm_v2)?;
    
    // Verify state needs to be restored
    let state = runtime.call_json_func(id, "get_state", ())?;
    assert_eq!(state, default_state); // Not initial_state
}
```

### 4. Handle Reload Failures

Always check reload results and handle failures:

```rust
match runtime.reload_module(script_id, new_wasm) {
    Ok(()) => {
        // Restore state to new module
        restore_state(&mut runtime, script_id, &state)?;
    }
    Err(e) => {
        // Old module still works, log error
        eprintln!("Reload failed: {}", e);
    }
}
```

## Comparison with Lua Hot-Reload

| Feature | Lua | WASM |
|---------|-----|------|
| State Preservation | Automatic (non-function fields) | Manual (external storage) |
| Custom Hooks | `on_save`/`on_restore` | Export get/set functions |
| Function Updates | Automatic | Automatic |
| Error Fallback | Old script remains | Old module remains |
| Performance | Slower (interpreted) | Faster (compiled) |

## Limitations

1. **No Automatic State Preservation**: All state must be managed externally
2. **Memory Reset**: Linear memory is reset to initial state
3. **Globals Reset**: Mutable globals are reset to initial values
4. **No Coroutines**: WASM doesn't support coroutines/continuations
5. **Host Function Compatibility**: New module must be compatible with existing host functions

## Testing

The hot-reload system is tested with:

- Basic reload functionality
- Error handling and fallback
- Multiple consecutive reloads
- Stateless behavior verification

See `tests/hot_reload_test.rs` for comprehensive test coverage.
