# Hot-Reload State Preservation

## Overview

The Lua scripting runtime supports hot-reloading scripts while preserving state across reloads. This enables rapid iteration during development without losing runtime data.

## How It Works

When a script is reloaded:

1. **Compilation**: The new script file is read and compiled
2. **Execution**: The new script body is executed to create a new module table
3. **State Preservation**: Non-function fields from the old module table are copied to the new table
4. **Custom Hooks**: If present, `on_save()` and `on_restore()` hooks are called for custom state management
5. **Swap**: The new module table replaces the old one

## State Preservation Behavior

### Automatic Field Preservation

All non-function fields are automatically preserved across reloads:

```lua
-- v1.lua
local module = {
    health = 100,
    position = { x = 10, y = 20 },
    target_entity = 42
}
return module
```

After reload to:

```lua
-- v2.lua
local module = {
    health = 0,  -- Will be overwritten to 100
    position = { x = 0, y = 0 },  -- Will be overwritten to { x = 10, y = 20 }
    target_entity = 0  -- Will be overwritten to 42
}
return module
```

The old values are preserved because non-function fields are copied from the old table to the new table.

### Function Updates

Functions are always taken from the new script:

```lua
-- v1.lua
local module = { x = 10 }
function module.on_update()
    print("Old behavior")
end
return module
```

After reload to:

```lua
-- v2.lua
local module = { x = 0 }  -- x will be 10 (preserved)
function module.on_update()
    print("New behavior")  -- This function will be used
end
return module
```

Result: `x` remains 10, but `on_update()` uses the new implementation.

### Custom State Management

For fine-grained control over what gets preserved, implement `on_save()` and `on_restore()`:

```lua
local module = {
    important_data = "must preserve",
    cache = {}  -- Don't want to preserve this
}

function module.on_save()
    -- Return only the data you want to preserve
    return {
        important = module.important_data
    }
end

function module.on_restore(state)
    -- Restore the saved data
    module.important_data = state.important
    -- cache will be empty (new table's initial value)
end

return module
```

**Note**: Even with `on_save`/`on_restore`, all non-function fields are still copied. The hooks provide additional control but don't prevent the automatic field copy.

## Entity References

Entity IDs (stored as numbers) are preserved across reloads:

```lua
local module = {
    target = 123,  -- Entity ID
    enemies = { 456, 789 }  -- Array of entity IDs
}
```

These references remain valid after reload, allowing scripts to maintain relationships with game entities.

## Component Data

Nested tables (like component data) are preserved:

```lua
local module = {
    stats = {
        health = 100,
        mana = 50,
        level = 5
    },
    inventory = {
        items = { "sword", "shield" },
        gold = 1000
    }
}
```

All nested data structures are preserved across reloads.

## Multiple Consecutive Reloads

State preservation works across multiple reloads:

```
Initial: x = 10
Reload 1: x = 0 (in new script) → x = 10 (preserved)
Reload 2: x = 0 (in new script) → x = 10 (preserved)
Reload 3: x = 0 (in new script) → x = 10 (preserved)
```

The original value persists through all reloads.

## Error Handling

If a reload fails (syntax error, runtime error), the old script remains active:

```lua
-- v1.lua (working)
local module = { x = 10 }
return module
```

Reload to:

```lua
-- v2.lua (broken)
return { syntax error here
```

Result: `reload_script()` returns `Err`, and v1 continues running with `x = 10`.

## Best Practices

### 1. Initialize All Fields

Always initialize all fields in your module table, even if they'll be overwritten:

```lua
local module = {
    health = 100,  -- Good: explicit initialization
    mana = 50
}
```

### 2. Use on_save/on_restore for Complex State

For state that requires special handling:

```lua
function module.on_save()
    return {
        player_pos = module.player_position,
        quest_state = module.serialize_quests()
    }
end

function module.on_restore(state)
    module.player_position = state.player_pos
    module.deserialize_quests(state.quest_state)
end
```

### 3. Avoid Storing Functions in Data

Don't store functions in data fields that should be preserved:

```lua
-- Bad: callback will be preserved from old script
local module = {
    callback = function() print("old") end
}

-- Good: define functions as methods
local module = {}
function module.callback()
    print("new")  -- Will update on reload
end
```

### 4. Test Hot-Reload Behavior

Test that your scripts work correctly after reload:

```lua
function module.on_update()
    -- Verify state is correct
    assert(module.health > 0, "Health should be preserved")
end
```

## Limitations

1. **Metatables**: Metatables are not automatically preserved
2. **Userdata**: Lua userdata (C objects) may not survive reload
3. **Coroutines**: Running coroutines are not preserved
4. **Upvalues**: Closure upvalues are not preserved

## Testing

The hot-reload system is tested with property-based tests covering:

- Single reload state preservation
- Multiple consecutive reloads
- Entity reference preservation
- Component data preservation
- Custom on_save/on_restore hooks
- Error fallback behavior

See `tests/hot_reload_test.rs` for comprehensive test coverage.
