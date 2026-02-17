# Scripting Integration Architecture

## Overview

Luminara Engine supports two scripting runtimes: Lua for rapid prototyping and WASM for performance-critical logic. Both runtimes feature hot-reload, sandboxing, and seamless integration with the ECS.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Script Layer                             │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │    Lua     │  │    WASM    │  │   Script Manager     │  │
│  │  Runtime   │  │  Runtime   │  │                      │  │
│  └──────┬─────┘  └──────┬─────┘  └──────────┬───────────┘  │
├─────────┴────────────────┴────────────────────┴──────────────┤
│                      Binding Layer                            │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  ECS API   │  │  Math API  │  │  Engine API          │  │
│  └──────┬─────┘  └──────┬─────┘  └──────────┬───────────┘  │
├─────────┴────────────────┴────────────────────┴──────────────┤
│                      ECS Core                                 │
│  World, Entities, Components, Systems, Resources             │
└──────────────────────────────────────────────────────────────┘
```

## Lua Scripting

### Lua Runtime

Luminara uses mlua for Lua integration with LuaJIT for performance.

```rust
pub struct LuaRuntime {
    lua: Lua,
    scripts: HashMap<Entity, LuaScript>,
    error_handler: Box<dyn Fn(LuaError)>,
}

// Features:
// - LuaJIT for near-native performance
// - Sandboxed execution
// - Hot-reload support
// - Error isolation (script errors don't crash engine)
```

### Lua Script Component

```rust
#[derive(Component)]
pub struct LuaScript {
    pub path: String,
    pub code: String,
    pub state: LuaTable,
}

// Attach to entity
world.insert(entity, LuaScript {
    path: "scripts/player.lua".to_string(),
    code: String::new(),
    state: lua.create_table()?,
});
```

### Lua API

Scripts access engine functionality through a clean API.

```lua
-- player.lua

-- Called once when script loads
function on_init(entity)
    print("Player initialized: " .. tostring(entity))
    
    -- Store state
    this.speed = 5.0
    this.health = 100
end

-- Called every frame
function on_update(entity, dt)
    -- Get components
    local transform = entity:get_component("Transform")
    local velocity = entity:get_component("Velocity")
    
    -- Read input
    if input.is_key_pressed("W") then
        velocity.linear.z = -this.speed
    end
    
    -- Update components
    entity:set_component("Velocity", velocity)
    
    -- Spawn entities
    if input.is_key_just_pressed("Space") then
        local bullet = world.spawn()
        bullet:add_component("Transform", {
            position = transform.position,
            rotation = transform.rotation,
            scale = vec3(0.1, 0.1, 0.5)
        })
        bullet:add_component("Velocity", {
            linear = transform.forward() * 20.0
        })
    end
end

-- Called when entity is destroyed
function on_destroy(entity)
    print("Player destroyed")
end

-- Custom event handler
function on_collision(entity, other)
    this.health = this.health - 10
    if this.health <= 0 then
        entity:destroy()
    end
end
```

### Lua Bindings

#### Entity API

```lua
-- Entity operations
local entity = world.spawn()
entity:destroy()
local id = entity:id()

-- Component operations
entity:add_component("Transform", { position = vec3(0, 0, 0) })
entity:remove_component("Transform")
local transform = entity:get_component("Transform")
entity:set_component("Transform", transform)
local has = entity:has_component("Transform")

-- Query entities
local entities = world.query({ "Transform", "Velocity" })
for _, e in ipairs(entities) do
    local transform = e:get_component("Transform")
    print(transform.position)
end
```

#### Math API

```lua
-- Vector operations
local v1 = vec3(1, 2, 3)
local v2 = vec3(4, 5, 6)
local v3 = v1 + v2
local v4 = v1 * 2.0
local len = v1:length()
local norm = v1:normalize()
local dot = v1:dot(v2)
local cross = v1:cross(v2)

-- Quaternion operations
local q1 = quat.from_euler(0, math.pi / 2, 0)
local q2 = quat.from_axis_angle(vec3(0, 1, 0), math.pi / 4)
local q3 = q1 * q2
local v_rotated = q1:rotate_vec3(vec3(1, 0, 0))

-- Transform operations
local transform = Transform.new()
transform.position = vec3(1, 2, 3)
transform.rotation = quat.identity()
transform.scale = vec3(1, 1, 1)
local forward = transform:forward()
local right = transform:right()
local up = transform:up()
```

#### Input API

```lua
-- Keyboard
if input.is_key_pressed("W") then end
if input.is_key_just_pressed("Space") then end
if input.is_key_just_released("Escape") then end

-- Mouse
local mouse_pos = input.mouse_position()
local mouse_delta = input.mouse_delta()
if input.is_mouse_button_pressed("Left") then end

-- Gamepad
if input.is_gamepad_button_pressed(0, "A") then end
local left_stick = input.gamepad_axis(0, "LeftStick")
```

#### Audio API

```lua
-- Play sound
audio.play("sounds/explosion.ogg")

-- Play with options
audio.play("sounds/music.ogg", {
    volume = 0.8,
    looping = true,
    spatial = false
})

-- 3D audio
audio.play_at_position("sounds/footstep.ogg", vec3(10, 0, 5), {
    volume = 1.0,
    max_distance = 50.0,
    rolloff = 1.0
})
```

### Lua Performance

```
Lua Performance (LuaJIT):
- Function call overhead: ~10ns
- Component access: ~50ns
- Vector math: ~20ns per operation
- Typical script update: 10-50μs

Performance vs Rust:
- Simple logic: 2-5x slower
- Math-heavy: 1.5-2x slower (JIT optimization)
- I/O bound: Equivalent
```

### Lua Hot Reload

```rust
// File watcher detects change
// → Reload script file
// → Recompile Lua code
// → Preserve script state (this table)
// → Call on_reload() if defined
// → Resume execution

// Hot reload time: <100ms
// Zero frame drops
// State preserved across reloads
```

## WASM Scripting

### WASM Runtime

Luminara uses wasmer for WASM execution.

```rust
pub struct WasmRuntime {
    store: Store,
    modules: HashMap<Entity, WasmModule>,
    instance_cache: HashMap<ModuleId, Instance>,
}

// Features:
// - Near-native performance
// - Strong sandboxing
// - Memory limits enforced
// - Execution time limits
```

### WASM Script Component

```rust
#[derive(Component)]
pub struct WasmScript {
    pub path: String,
    pub module: Vec<u8>,
    pub memory: WasmMemory,
}

// Compile from Rust, C, C++, AssemblyScript, etc.
// Strict resource limits
// Isolated memory space
```

### WASM API

WASM scripts use exported functions for engine integration.

```rust
// Rust code compiled to WASM

use luminara_wasm_api::*;

#[no_mangle]
pub extern "C" fn on_init(entity: EntityId) {
    log("AI initialized");
}

#[no_mangle]
pub extern "C" fn on_update(entity: EntityId, dt: f32) {
    // Get transform
    let mut transform = get_component::<Transform>(entity);
    
    // Find nearest enemy
    let enemies = query_entities(&["Transform", "Enemy"]);
    let mut nearest = None;
    let mut min_dist = f32::MAX;
    
    for enemy in enemies {
        let enemy_transform = get_component::<Transform>(enemy);
        let dist = transform.position.distance(enemy_transform.position);
        if dist < min_dist {
            min_dist = dist;
            nearest = Some(enemy);
        }
    }
    
    // Move towards enemy
    if let Some(enemy) = nearest {
        let enemy_transform = get_component::<Transform>(enemy);
        let direction = (enemy_transform.position - transform.position).normalize();
        transform.position += direction * 5.0 * dt;
        set_component(entity, transform);
    }
}

#[no_mangle]
pub extern "C" fn on_collision(entity: EntityId, other: EntityId) {
    // Handle collision
    if has_component::<Player>(other) {
        // Attack player
        let mut health = get_component::<Health>(other);
        health.current -= 10.0;
        set_component(other, health);
    }
}
```

### WASM Bindings

WASM bindings are lower-level than Lua for performance.

```rust
// Host functions exported to WASM

#[wasm_bindgen]
pub fn get_component(entity: u64, component_type: u32) -> *const u8;

#[wasm_bindgen]
pub fn set_component(entity: u64, component_type: u32, data: *const u8);

#[wasm_bindgen]
pub fn spawn_entity() -> u64;

#[wasm_bindgen]
pub fn destroy_entity(entity: u64);

#[wasm_bindgen]
pub fn query_entities(components: *const u32, count: usize) -> *const u64;

// Memory management
#[wasm_bindgen]
pub fn alloc(size: usize) -> *mut u8;

#[wasm_bindgen]
pub fn dealloc(ptr: *mut u8, size: usize);
```

### WASM Performance

```
WASM Performance:
- Function call overhead: ~5ns
- Component access: ~20ns
- Vector math: ~5ns per operation
- Typical script update: 2-10μs

Performance vs Rust:
- Simple logic: 1.2-1.5x slower
- Math-heavy: 1.1-1.3x slower
- I/O bound: Equivalent

Performance vs Lua:
- 3-5x faster than Lua
- More predictable performance
- Better for performance-critical code
```

### WASM Resource Limits

```rust
pub struct WasmLimits {
    /// Maximum memory (bytes)
    pub max_memory: usize,
    /// Maximum execution time per frame (ms)
    pub max_execution_time: Duration,
    /// Maximum entity spawns per frame
    pub max_entity_spawns: usize,
}

// Default limits:
// - Memory: 64MB
// - Execution time: 5ms
// - Entity spawns: 1000

// Limits enforced by runtime
// Exceeded limits → script paused/terminated
```

## Script Management

### Script System

```rust
pub fn script_system(
    query: Query<(Entity, &LuaScript)>,
    mut lua_runtime: ResMut<LuaRuntime>,
    time: Res<Time>,
) {
    for (entity, script) in query.iter() {
        // Call on_update for each script
        if let Err(e) = lua_runtime.call_update(entity, script, time.delta_seconds()) {
            error!("Script error in {:?}: {}", entity, e);
            // Script error doesn't crash engine
        }
    }
}

// Execution order:
// 1. on_init (once)
// 2. on_update (every frame)
// 3. Event handlers (on events)
// 4. on_destroy (once)
```

### Error Handling

Scripts are isolated - errors don't crash the engine.

```rust
// Lua error handling
match lua_runtime.call_update(entity, script, dt) {
    Ok(_) => {},
    Err(e) => {
        error!("Lua error in entity {:?}: {}", entity, e);
        // Log error
        // Show error in editor
        // Continue with other scripts
    }
}

// WASM error handling
match wasm_runtime.call_update(entity, script, dt) {
    Ok(_) => {},
    Err(WasmError::Timeout) => {
        warn!("WASM script timeout in entity {:?}", entity);
        // Pause script
    }
    Err(WasmError::MemoryLimit) => {
        error!("WASM script exceeded memory limit in entity {:?}", entity);
        // Terminate script
    }
    Err(e) => {
        error!("WASM error in entity {:?}: {}", entity, e);
    }
}
```

### Hot Reload

Both Lua and WASM support hot-reload.

#### Lua Hot Reload

```rust
// 1. Detect file change
// 2. Reload Lua code
// 3. Preserve script state (this table)
// 4. Call on_reload() if defined
// 5. Resume execution

// State preservation:
// - this.* variables preserved
// - Local variables reset
// - Function definitions updated

// Hot reload time: <100ms
```

#### WASM Hot Reload

```rust
// 1. Detect file change
// 2. Recompile WASM module
// 3. Serialize script state
// 4. Create new instance
// 5. Deserialize state
// 6. Resume execution

// State preservation:
// - Component data preserved
// - Memory state serialized
// - Execution resumes from next frame

// Hot reload time: <200ms
```

## Scripting Best Practices

### When to Use Lua

- **Rapid prototyping**: Quick iteration
- **Game logic**: Player controllers, AI behaviors
- **Level scripting**: Triggers, cutscenes
- **Modding**: User-created content
- **Configuration**: Data-driven design

### When to Use WASM

- **Performance-critical code**: Physics, pathfinding
- **Complex algorithms**: Procedural generation
- **Existing C/C++ code**: Port existing libraries
- **Deterministic simulation**: Multiplayer logic
- **Security-sensitive code**: Anti-cheat, validation

### Performance Tips

**Lua:**
- Cache component references
- Minimize table allocations
- Use local variables
- Avoid string concatenation in loops
- Profile with LuaJIT profiler

**WASM:**
- Minimize host function calls
- Batch component updates
- Use SIMD when possible
- Optimize memory layout
- Profile with browser devtools

### Memory Management

**Lua:**
```lua
-- Good: Reuse tables
local temp_vec = vec3(0, 0, 0)

function on_update(entity, dt)
    temp_vec.x = 1.0
    temp_vec.y = 2.0
    temp_vec.z = 3.0
    -- Use temp_vec
end

-- Bad: Allocate every frame
function on_update(entity, dt)
    local v = vec3(1, 2, 3)  -- Allocation!
end
```

**WASM:**
```rust
// Good: Stack allocation
fn on_update(entity: EntityId, dt: f32) {
    let mut transform = get_component::<Transform>(entity);
    transform.position.x += 1.0;
    set_component(entity, transform);
}

// Bad: Heap allocation
fn on_update(entity: EntityId, dt: f32) {
    let transform = Box::new(get_component::<Transform>(entity));  // Allocation!
}
```

## Advanced Features

### Custom Lua Libraries

```rust
// Register custom Lua library
lua.globals().set("physics", lua.create_table()?)?;

let physics = lua.globals().get::<_, LuaTable>("physics")?;
physics.set("raycast", lua.create_function(raycast_lua)?)?;

// Use in Lua
local hit = physics.raycast(origin, direction, max_distance)
if hit then
    print("Hit: " .. tostring(hit.entity))
end
```

### WASM Imports

```rust
// Import host functions in WASM

#[link(wasm_import_module = "env")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
    fn get_time() -> f64;
    fn random() -> f32;
}

// Use in WASM code
unsafe {
    let msg = "Hello from WASM!";
    log(msg.as_ptr(), msg.len());
}
```

### Script Communication

```lua
-- Lua: Send event
events.send("player_died", { player = entity, cause = "fall" })

-- Lua: Receive event
function on_event(event_type, data)
    if event_type == "player_died" then
        print("Player died: " .. data.cause)
    end
end
```

```rust
// WASM: Send event
send_event("enemy_spawned", &EnemySpawnedEvent {
    entity,
    position: transform.position,
});

// WASM: Receive event
#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: *const u8, len: usize) {
    match event_type {
        EVENT_ENEMY_SPAWNED => {
            let event = unsafe { parse_event::<EnemySpawnedEvent>(data, len) };
            // Handle event
        }
        _ => {}
    }
}
```

## Testing

### Lua Testing

```lua
-- test_player.lua

function test_movement()
    local entity = world.spawn()
    entity:add_component("Transform", { position = vec3(0, 0, 0) })
    entity:add_component("Velocity", { linear = vec3(1, 0, 0) })
    
    -- Simulate one frame
    on_update(entity, 1.0)
    
    local transform = entity:get_component("Transform")
    assert(transform.position.x > 0, "Player should move")
end

-- Run tests
test_movement()
```

### WASM Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ai_behavior() {
        let mut world = World::new();
        let entity = world.spawn();
        
        // Setup
        world.insert(entity, Transform::default());
        world.insert(entity, WasmScript::load("ai.wasm"));
        
        // Execute
        on_update(entity, 1.0);
        
        // Verify
        let transform = world.get::<Transform>(entity).unwrap();
        assert!(transform.position.length() > 0.0);
    }
}
```

## Further Reading

- [Lua API Reference](../api/lua_api.md) - Complete Lua API documentation
- [WASM API Reference](../api/wasm_api.md) - Complete WASM API documentation
- [Scripting Tutorial](../workflows/scripting_tutorial.md) - Step-by-step scripting guide
- [Performance Profiling](../audit/script_stress_test_results.md) - Script performance analysis

## References

- [mlua Documentation](https://docs.rs/mlua/) - Lua bindings
- [wasmer Documentation](https://docs.wasmer.io/) - WASM runtime
- [LuaJIT](https://luajit.org/) - JIT compiler
- [WebAssembly](https://webassembly.org/) - WASM specification
