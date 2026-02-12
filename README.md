# Luminara Engine

High-performance parallel AI-driven game engine written in Rust.

## Architecture Overview

Luminara is designed as a modular, high-performance game engine with a focus on parallel execution and clean architecture.

### Key Components

- **Luminara Core**: A data-oriented ECS framework.
- **Luminara Math**: Fast math operations powered by `glam`.
- **Luminara Window**: Cross-platform windowing using `winit`.
- **Luminara Input**: Unified input system for keyboard, mouse, and gamepads.
- **Luminara Render**: `wgpu`-based modern rendering pipeline.
- **Luminara Asset**: Synchronous asset loading with hot-reloading support.
- **Luminara Scene**: RON/JSON-based scene management and entity hierarchy.

## Getting Started

Add Luminara to your `Cargo.toml`:

```toml
[dependencies]
luminara = { path = "path/to/luminara" }
```

### Basic Example

```rust
use luminara::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // A collection of core plugins
        .add_system(CoreStage::Update, hello_world_system)
        .run();
}

fn hello_world_system() {
    println!("Hello from Luminara!");
}
```

## Development and Contributions

Luminara uses a workspace-based structure. Core logic resides in `crates/`, tools in `tools/`, and examples in `examples/`.

To run tests:
```bash
cargo test --workspace
```

To run clippy:
```bash
cargo clippy --workspace
```
