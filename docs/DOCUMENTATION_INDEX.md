# Luminara Engine Documentation Index

Complete documentation for the Luminara Engine, covering architecture, APIs, workflows, and migration guides.

## 📚 Architecture Documentation

Comprehensive architecture overviews for all major subsystems:

- **[ECS Architecture](architecture/ecs_architecture.md)** - Entity Component System design, archetype storage, queries, and best practices
- **[Rendering Pipeline](architecture/rendering_pipeline.md)** - Forward+ rendering, PBR materials, shadows, culling, and optimization
- **[Asset Pipeline](architecture/asset_pipeline.md)** - Asynchronous loading, hot-reload, caching, and asset management
- **[Scripting Integration](architecture/scripting_integration.md)** - Lua and WASM runtimes, APIs, and hot-reload
- **[AI Agent System](architecture/ai_agent_system.md)** - AI-driven development assistance, context engine, and verification

## 🎯 Design Decisions

- **[Design Decisions and Rationale](design_decisions.md)** - Key architectural choices, trade-offs, and alternatives considered

## 📖 API Documentation

### Generated API Docs

Generate complete API documentation from source code:

```bash
cargo doc --workspace --no-deps --open
```

This generates comprehensive documentation for all public APIs with examples.

### API Guides

- **Lua Scripting API** - See [Scripting Integration](architecture/scripting_integration.md#lua-api)
- **WASM Scripting API** - See [Scripting Integration](architecture/scripting_integration.md#wasm-api)
- **ECS API** - See [ECS Architecture](architecture/ecs_architecture.md)
- **Rendering API** - See [Rendering Pipeline](architecture/rendering_pipeline.md)

### Core API Examples

#### Entity and Component Management

```rust
// Spawn entity with components
let entity = world.spawn()
    .insert(Transform::default())
    .insert(Mesh::from_path("models/player.gltf"))
    .insert(PbrMaterial::default())
    .id();

// Query entities
for (entity, transform, mesh) in world.query::<(Entity, &Transform, &Mesh)>().iter() {
    // Process entities
}

// Modify components
if let Some(mut transform) = world.get_mut::<Transform>(entity) {
    transform.position += Vec3::X * dt;
}
```

#### Asset Loading

```rust
// Load assets asynchronously
let texture = asset_server.load::<Texture>("textures/player.png");
let mesh = asset_server.load::<Mesh>("models/character.gltf");
let audio = asset_server.load::<AudioClip>("sounds/explosion.ogg");

// Use immediately (placeholder shown until loaded)
material.base_color_texture = Some(texture);

// Check load status
if asset_server.is_loaded(&texture) {
    // Asset ready
}
```

#### Scripting

```lua
-- Lua script example
function on_update(entity, dt)
    local transform = entity:get_component("Transform")
    local velocity = entity:get_component("Velocity")
    
    if input.is_key_pressed("W") then
        velocity.linear.z = -5.0
    end
    
    entity:set_component("Velocity", velocity)
end
```

```rust
// WASM script example
#[no_mangle]
pub extern "C" fn on_update(entity: EntityId, dt: f32) {
    let mut transform = get_component::<Transform>(entity);
    transform.position += Vec3::X * 5.0 * dt;
    set_component(entity, transform);
}
```

## 🚀 Workflow Guides

### Getting Started

1. **[Installation and Setup](getting_started.md)** - Install Rust, clone repository, build engine
2. **[First Project](workflows/first_project.md)** - Create your first Luminara project
3. **[First Game Tutorial](workflows/first_game.md)** - Build a simple game step-by-step

### Development Workflows

- **[Hot Reload Workflow](workflows/hot_reload.md)** - Using hot-reload for rapid iteration
- **[Scripting Tutorial](workflows/scripting_tutorial.md)** - Writing Lua and WASM scripts
- **[AI Integration Guide](workflows/ai_integration.md)** - Using AI assistance features
- **[Debugging Guide](workflows/debugging.md)** - Debugging techniques and tools
- **[Profiling Guide](workflows/profiling.md)** - Performance profiling and optimization

### Advanced Topics

- **[Custom Shaders](workflows/custom_shaders.md)** - Writing custom WGSL shaders
- **[Plugin Development](workflows/plugin_development.md)** - Creating engine plugins
- **[Custom Asset Loaders](workflows/custom_asset_loaders.md)** - Supporting new asset formats
- **[Networking](workflows/networking.md)** - Multiplayer and networking

### Optimization Guides

- **[Performance Optimization](audit/build_optimization_guide.md)** - General optimization strategies
- **[Rendering Optimization](audit/rendering_pipeline_analysis.md)** - Graphics optimization
- **[Physics Optimization](audit/physics_optimization_guide.md)** - Physics performance tuning
- **[Build Time Optimization](audit/compilation_performance_analysis.md)** - Faster compilation

## 🔄 Migration Guides

### From Other Engines

- **[Migrating from Unity](migration/from_unity.md)** - Unity to Luminara migration guide
- **[Migrating from Godot](migration/from_godot.md)** - Godot to Luminara migration guide
- **[Migrating from Bevy](migration/from_bevy.md)** - Bevy to Luminara migration guide
- **[Migrating from Unreal](migration/from_unreal.md)** - Unreal to Luminara migration guide

### API Changes

- **[API Migration Guide](migration/api_changes.md)** - Breaking changes and migration paths
- **[Version Migration](migration/version_migration.md)** - Upgrading between Luminara versions

## 🤝 Contributing

### For Contributors

- **[Contribution Guide](CONTRIBUTING.md)** - How to contribute to Luminara Engine
- **[Code Style Guide](contributing/code_style.md)** - Coding standards and conventions
- **[Testing Guide](contributing/testing.md)** - Writing and running tests
- **[PR Process](contributing/pr_process.md)** - Pull request workflow
- **[Architecture Guide](contributing/architecture.md)** - Understanding the codebase

### Development Setup

```bash
# Clone repository
git clone https://github.com/luminara-engine/luminara.git
cd luminara

# Build engine
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example phase1_demo
```

## 📊 Audit Reports

Performance and quality audit results:

- **[Final Audit Report](audit/checkpoint_30_final_report.md)** - Comprehensive audit summary
- **[Test Coverage Analysis](audit/test_coverage_analysis.md)** - Code coverage metrics
- **[Performance Benchmarks](audit/checkpoint_30_verification_summary.md)** - Benchmark results
- **[Technical Debt Tracking](audit/technical_debt_tracking.md)** - Known issues and improvements

## 🔍 Reference

### Quick References

- **[Quick Reference](audit/quick_reference.md)** - Common commands and patterns
- **[Keyboard Shortcuts](reference/keyboard_shortcuts.md)** - Editor and debug shortcuts
- **[Error Messages](reference/error_messages.md)** - Common errors and solutions

### Specifications

- **[Requirements](../.kiro/specs/pre-editor-engine-audit/requirements.md)** - Engine requirements
- **[Design Specification](../.kiro/specs/pre-editor-engine-audit/design.md)** - Detailed design
- **[Implementation Tasks](../.kiro/specs/pre-editor-engine-audit/tasks.md)** - Development tasks

## 📝 Additional Resources

### Examples

All examples are in the `examples/` directory:

- `phase1_demo` - Complete demo showcasing engine features
- `fluid_viz_demo` - Spectral fluid simulation visualization
- `gizmo_system_demo` - Debug visualization examples

Run examples:
```bash
cargo run --example phase1_demo
cargo run --example fluid_viz_demo
```

### Community

- **Discord**: [Join our Discord](https://discord.gg/luminara) (coming soon)
- **Forum**: [Community Forum](https://forum.luminara-engine.org) (coming soon)
- **GitHub**: [GitHub Repository](https://github.com/luminara-engine/luminara)

### External Resources

- **[Rust Book](https://doc.rust-lang.org/book/)** - Learn Rust programming
- **[wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)** - Learn WebGPU/wgpu
- **[Game Programming Patterns](https://gameprogrammingpatterns.com/)** - Design patterns for games
- **[Real-Time Rendering](https://www.realtimerendering.com/)** - Graphics programming

## 🎓 Learning Path

### Beginner

1. Read [Getting Started](getting_started.md)
2. Follow [First Game Tutorial](workflows/first_game.md)
3. Explore [ECS Architecture](architecture/ecs_architecture.md)
4. Try [Scripting Tutorial](workflows/scripting_tutorial.md)

### Intermediate

1. Study [Rendering Pipeline](architecture/rendering_pipeline.md)
2. Learn [Asset Pipeline](architecture/asset_pipeline.md)
3. Practice [Hot Reload Workflow](workflows/hot_reload.md)
4. Read [Performance Optimization](audit/build_optimization_guide.md)

### Advanced

1. Deep dive into [AI Agent System](architecture/ai_agent_system.md)
2. Study [Design Decisions](design_decisions.md)
3. Explore [Custom Shaders](workflows/custom_shaders.md)
4. Contribute via [Contribution Guide](CONTRIBUTING.md)

## 📞 Support

- **Documentation Issues**: [Report on GitHub](https://github.com/luminara-engine/luminara/issues)
- **Questions**: Ask on Discord or Forum
- **Bug Reports**: [GitHub Issues](https://github.com/luminara-engine/luminara/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/luminara-engine/luminara/discussions)

---

**Last Updated**: 2024
**Engine Version**: 0.1.0 (Pre-Editor Audit Phase)
**Documentation Version**: 1.0
