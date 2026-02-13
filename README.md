# Luminara Engine

High-performance parallel AI-driven game engine written in Rust.

## Features

### Phase 1 — Core Engine (Completed)

- **ECS (Entity Component System)**: Data-oriented archetype-based ECS with type-safe queries, exclusive systems, and plugin architecture
- **3D PBR Rendering**: Forward+ rendering pipeline with PBR-lite materials (albedo, metallic, roughness, emissive), Blinn-Phong specular, Fresnel, and tone mapping via `wgpu`
- **Physics Simulation**: 3D rigid body physics with gravity, collision detection, and restitution via Rapier3D
- **Scene System**: RON/JSON-based scene serialization with entity hierarchy, component deserialization, and prefab support
- **Transform System**: Full 3D transforms with translation, rotation (quaternion), and scale
- **Camera System**: Perspective/orthographic projection with automatic aspect ratio updates
- **Asset Pipeline**: Synchronous asset loading with hot-reload support, caching, and reference-counted handles
- **Audio System**: Audio playback with spatial audio support, volume/pitch control, and looping (via `cpal` + `rodio`)
- **Input System**: Unified keyboard, mouse, and gamepad input handling
- **Windowing**: Cross-platform window management via `winit`
- **Diagnostics**: Frame timing, system profiling, and performance metrics
- **Plugin System**: Modular plugin architecture for engine extensibility

## Architecture

```
luminara (facade crate)
├── luminara_core       — ECS, App, World, Schedule, Plugin system
├── luminara_math       — glam-based math (Vec3, Mat4, Quat, Transform, Color)
├── luminara_window     — Cross-platform windowing (winit)
├── luminara_input      — Keyboard, mouse, gamepad input
├── luminara_render     — wgpu rendering pipeline, PBR materials, camera, sprites
├── luminara_asset      — Asset loading, hot-reload, caching
├── luminara_scene      — Scene serialization (RON/JSON), entity hierarchy
├── luminara_physics    — 3D physics simulation (Rapier3D)
├── luminara_audio      — Audio playback (cpal + rodio)
├── luminara_diagnostic — Performance diagnostics
└── luminara_platform   — Platform abstraction
```

## Getting Started

### Prerequisites

- Rust 1.85+ (see [rust-toolchain.toml](rust-toolchain.toml))
- Vulkan-compatible GPU (for rendering)

### Build

```bash
cargo build --workspace
```

### Run the Phase 1 Demo

```bash
cargo run -p phase1_demo
```

The demo renders a 3D scene with:
- A grey ground plane (static rigid body)
- A red metallic sphere falling under gravity and colliding with the ground
- PBR lighting with directional sun light
- Camera at an elevated angle looking down at the scene

### Basic Example

```rust
use luminara::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system::<ExclusiveMarker>(setup)
        .run();
}

fn setup(world: &mut World) {
    // Spawn a camera
    let camera = world.spawn();
    world.add_component(camera, Transform::from_translation(Vec3::new(0.0, 3.0, 8.0)));
    world.add_component(camera, Camera {
        projection: Projection::Perspective { fov: 60.0, near: 0.1, far: 1000.0 },
        clear_color: Color::rgb(0.1, 0.1, 0.15),
        is_active: true,
    });
    world.add_component(camera, Camera3d);

    // Spawn a mesh with PBR material
    let sphere = world.spawn();
    world.add_component(sphere, Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)));
    world.add_component(sphere, Mesh::sphere(0.5, 32));
    world.add_component(sphere, luminara::render::PbrMaterial {
        albedo: Color::rgb(0.8, 0.2, 0.2),
        metallic: 0.9,
        roughness: 0.3,
        ..Default::default()
    });
}
```

## Development

```bash
# Run all tests
cargo test --workspace

# Run clippy
cargo clippy --workspace

# Run the CLI tool
cargo run -p luminara_cli
```

## License

Dual-licensed under MIT ([LICENSE](LICENSE)) and Commercial ([LICENSE-COMMERCIAL](LICENSE-COMMERCIAL)).
