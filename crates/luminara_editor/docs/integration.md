# Luminara Engine Integration

This document describes how the GPUI Editor integrates with Luminara Engine's subsystems.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     GPUI Editor UI                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              EditorWindow (Root View)                  │  │
│  │  ┌──────────┬────────────────────────────────────┐   │  │
│  │  │ Activity │  Active Box (Scene Builder, etc)   │   │  │
│  │  │   Bar    │                                     │   │  │
│  │  └──────────┴────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼ EngineHandle
┌─────────────────────────────────────────────────────────────┐
│                  Luminara Engine Core                        │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │   ECS    │  Asset   │ Database │  Render  │   Input  │  │
│  │  World   │  System  │ SurrealDB│ Pipeline │  System  │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## EngineHandle

The `EngineHandle` struct provides thread-safe access to all major engine subsystems:

```rust
pub struct EngineHandle {
    world: Arc<RwLock<World>>,
    asset_server: Arc<AssetServer>,
    database: Arc<Database>,
    render_pipeline: Arc<RwLock<RenderPipeline>>,
}
```

### ECS Integration (Requirement 12.1)

The editor accesses the ECS World through `EngineHandle`:

```rust
// Read access
let world = engine.world();
let entity = world.get_entity(entity_id)?;

// Write access
let mut world = engine.world_mut();
let entity = world.spawn_empty();
world.entity_mut(entity).insert(Transform::default());
```

**Requirements Satisfied:**
- 12.1.1: Query entities from ECS
- 12.1.2: Update components in ECS
- 12.1.3: Spawn entities in ECS

### Asset System Integration (Requirement 12.2)

The editor loads assets through the `AssetServer`:

```rust
let asset_server = engine.asset_server();
let texture_handle = asset_server.load::<Texture>("textures/icon.png");
```

**Requirements Satisfied:**
- 12.2.1: Use AssetServer for all asset loading
- 12.2.2: Generate asset handles
- 12.2.7: Support async asset loading without blocking UI

### Database Integration (Requirement 12.3)

The editor persists and queries data through SurrealDB:

```rust
let database = engine.database();

// Save scene
database.save_scene(&scene).await?;

// Query entities
let results = database.query("SELECT * FROM entities WHERE type = 'Light'").await?;
```

**Requirements Satisfied:**
- 12.3.1: Use SurrealDB for all data persistence
- 12.3.2: Serialize scenes to database
- 12.3.5: Implement optimistic UI updates with DB sync

### Render Pipeline Integration (Requirement 12.4)

The editor shares the WGPU device and queue with Luminara's renderer:

```rust
let render_pipeline = engine.render_pipeline();
let device = render_pipeline.device();
let queue = render_pipeline.queue();

// Create shared render target for viewport
let render_target = SharedRenderTarget::new(device, (1920, 1080));
```

**Requirements Satisfied:**
- 12.4.1: Share WGPU device and queue with Luminara's renderer
- 12.4.2: Use Luminara's render graph
- 12.4.6: Synchronize camera transforms between UI and renderer

## Thread Safety

All engine subsystems are wrapped in `Arc` for shared ownership and `RwLock` for thread-safe access:

- **World**: `Arc<RwLock<World>>` - Allows multiple readers or single writer
- **AssetServer**: `Arc<AssetServer>` - Internally thread-safe
- **Database**: `Arc<Database>` - Internally thread-safe
- **RenderPipeline**: `Arc<RwLock<RenderPipeline>>` - Allows multiple readers or single writer

## Event Flow

1. **UI Event** → GPUI captures user input (mouse, keyboard)
2. **Command Creation** → UI creates a command (e.g., SpawnEntity)
3. **Engine Execution** → Command is executed on the engine through EngineHandle
4. **State Update** → Engine state is modified (ECS, Assets, DB)
5. **UI Refresh** → GPUI re-renders affected UI components

## Testing

The `EngineHandle::mock()` method creates a minimal engine setup for testing:

```rust
#[test]
fn test_editor_with_mock_engine() {
    let engine = Arc::new(EngineHandle::mock());
    let app = EditorApp::new(engine);
    // Test editor functionality
}
```

## Performance Considerations

- **Read-Heavy Operations**: Use `world()` for read-only access to avoid blocking writers
- **Batch Updates**: Group multiple ECS updates into a single write lock
- **Async Loading**: Assets are loaded asynchronously to avoid blocking the UI thread
- **Optimistic UI**: UI updates immediately, syncing with DB in the background

## Future Enhancements

- **Command Queue**: Implement a command queue for undo/redo support
- **Event Bus**: Add an event bus for communication between UI and engine
- **Hot Reload**: Integrate with Luminara's hot-reload system for live editing
