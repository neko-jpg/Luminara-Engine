# Luminara Engine Architecture Documentation

This directory contains comprehensive architecture documentation for the Luminara Engine, covering all major subsystems and design decisions.

## Overview Documents

- [ECS Architecture](ecs_architecture.md) - Entity Component System design and implementation
- [Rendering Pipeline](rendering_pipeline.md) - Graphics rendering architecture and optimization
- [Asset Pipeline](asset_pipeline.md) - Asset loading, management, and hot-reload system
- [Scripting Integration](scripting_integration.md) - Lua and WASM scripting systems
- [AI Agent System](ai_agent_system.md) - AI-driven development assistance architecture

## Design Documents

- [Design Decisions](../design_decisions.md) - Rationale behind key architectural choices
- [Performance Optimization](../audit/physics_optimization_guide.md) - Performance optimization strategies
- [Cross-Platform Support](cross_platform.md) - Platform compatibility and abstractions

## API Documentation

API documentation is generated from source code using `cargo doc`. To generate:

```bash
cargo doc --workspace --no-deps --open
```

## Getting Started

For new contributors, start with:
1. [Getting Started Guide](../getting_started.md)
2. [ECS Architecture](ecs_architecture.md)
3. [Contribution Guide](../CONTRIBUTING.md)

## Additional Resources

- [Workflow Guides](../workflows/) - Step-by-step development workflows
- [Migration Guides](../migration/) - Migrating from other engines
- [Audit Reports](../audit/) - Performance and quality audit results
