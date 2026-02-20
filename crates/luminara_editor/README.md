# Luminara Editor

GPUI-based editor UI for Luminara Engine.

## Overview

This crate provides a professional-grade editor interface built on the GPUI framework, integrating seamlessly with Luminara Engine's ECS, Asset System, Database, and Render Pipeline.

## Architecture

The editor follows a component-based architecture with clear separation between UI presentation, state management, and engine integration:

- **EditorApp**: Root GPUI application managing the editor lifecycle
- **EditorWindow**: Main window containing all UI elements
- **EngineHandle**: Bridge between GPUI UI and Luminara Engine

## Features

- **GPU-Accelerated Rendering**: Uses GPUI for high-performance UI rendering
- **ECS Integration**: Seamless integration with Luminara's Entity Component System
- **Asset Management**: Direct integration with Luminara's Asset System
- **Database Integration**: Real-time queries and persistence with SurrealDB
- **3D Viewport**: Embedded 3D viewport using Luminara's render pipeline

## Requirements

This implementation satisfies the following requirements from the GPUI Editor UI spec:

- **Requirement 1.1**: Integrate GPUI as a dependency
- **Requirement 1.2**: Initialize GPUI runtime with GPU acceleration
- **Requirement 1.3**: Provide a root window with proper event handling
- **Requirement 1.4**: Integrate GPUI's rendering loop with Luminara's render pipeline

## Usage

```rust
use luminara_editor::{EditorApp, EngineHandle};
use luminara_core::World;
use luminara_asset::AssetServer;
use luminara_db::Database;
use luminara_render::RenderPipeline;
use parking_lot::RwLock;
use std::sync::Arc;
use gpui::App as GpuiApp;

fn main() -> anyhow::Result<()> {
    // Initialize Luminara Engine
    let world = Arc::new(RwLock::new(World::new()));
    let asset_server = Arc::new(AssetServer::new());
    let database = Arc::new(Database::memory()?);
    let render_pipeline = Arc::new(RwLock::new(RenderPipeline::mock()));
    
    // Create engine handle
    let engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));
    
    // Initialize GPUI application
    GpuiApp::new().run(|cx| {
        let editor_app = EditorApp::new(engine_handle);
        editor_app.run(cx)?;
        Ok(())
    })
}
```

## Testing

Run tests with:

```bash
cargo test -p luminara_editor
```

## Examples

See `examples/basic_editor.rs` for a minimal editor setup.

## Status

This is Task 1 of the GPUI Editor UI implementation. The basic infrastructure is complete:

- ✅ Created `crates/luminara_editor` crate with GPUI dependency
- ✅ Initialized GPUI runtime with GPU acceleration
- ✅ Set up basic window and event loop
- ✅ Integrated with Luminara's render pipeline

## Next Steps

- Task 2: Implement Core Theme System
- Task 3: Implement Activity Bar Component
- Task 4: Implement Resizable Panel Component
