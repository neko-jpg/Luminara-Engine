# Contributing to Luminara Engine

Thank you for your interest in contributing to Luminara Engine! This guide will help you get started with contributing code, documentation, or other improvements.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Architecture Overview](#architecture-overview)
5. [Code Style](#code-style)
6. [Testing Requirements](#testing-requirements)
7. [Pull Request Process](#pull-request-process)
8. [Areas for Contribution](#areas-for-contribution)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

### Our Standards

- **Be respectful**: Treat everyone with respect and kindness
- **Be constructive**: Provide helpful feedback and suggestions
- **Be collaborative**: Work together towards common goals
- **Be patient**: Remember that everyone is learning

## Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Version control
- **wgpu-compatible GPU**: For rendering (most modern GPUs work)
- **WSL** (Windows users): Recommended for development

### Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/luminara.git
cd luminara

# Add upstream remote
git remote add upstream https://github.com/luminara-engine/luminara.git
```

### Build and Test

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p luminara_core

# Run examples
cargo run --example phase1_demo
```

## Development Setup

### Recommended Tools

- **IDE**: VS Code with rust-analyzer extension
- **Debugger**: rust-lldb (macOS/Linux) or rust-gdb (Linux)
- **Profiler**: cargo-flamegraph for performance profiling
- **Coverage**: cargo-tarpaulin for code coverage

### VS Code Extensions

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates",
    "tamasfe.even-better-toml"
  ]
}
```

### Environment Setup (WSL)

For Windows users using WSL:

```bash
# Cargo location
export PATH="/home/YOUR_USERNAME/.cargo/bin:$PATH"

# Project location
cd /mnt/c/dev/Luminara-Engine
```

## Architecture Overview

### Crate Structure

```
luminara/
├── crates/
│   ├── luminara/              # Main crate (re-exports)
│   ├── luminara_core/         # ECS, plugins, scheduling
│   ├── luminara_render/       # Rendering pipeline
│   ├── luminara_asset/        # Asset management
│   ├── luminara_physics/      # Physics simulation
│   ├── luminara_audio/        # Audio system
│   ├── luminara_input/        # Input handling
│   ├── luminara_window/       # Window management
│   ├── luminara_scene/        # Scene management
│   ├── luminara_math/         # Math library (PGA, etc.)
│   ├── luminara_script_lua/   # Lua scripting
│   ├── luminara_script_wasm/  # WASM scripting
│   ├── luminara_ai_agent/     # AI integration
│   ├── luminara_db/           # Database integration
│   └── luminara_diagnostic/   # Profiling and diagnostics
├── examples/                  # Example projects
└── docs/                      # Documentation
```

### Key Concepts

- **ECS**: Entity Component System architecture
- **Plugins**: Modular functionality
- **Systems**: Logic that operates on components
- **Resources**: Global singletons
- **Assets**: Loaded resources (textures, meshes, etc.)

See [Architecture Documentation](architecture/) for details.

## Code Style

### Rust Style

Follow standard Rust conventions:

```rust
// Use descriptive names
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// Document public APIs
/// Spawns a new entity with the given components.
///
/// # Examples
///
/// ```
/// let entity = world.spawn()
///     .insert(Transform::default())
///     .insert(Mesh::default())
///     .id();
/// ```
pub fn spawn(&mut self) -> EntityBuilder {
    // Implementation
}

// Use Result for fallible operations
pub fn load_asset(&self, path: &str) -> Result<Handle<Asset>, AssetError> {
    // Implementation
}

// Prefer iterators over loops
let sum: i32 = entities.iter()
    .filter(|e| e.is_active())
    .map(|e| e.value())
    .sum();
```

### Formatting

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace -- -D warnings
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `TransformComponent`)
- **Functions**: `snake_case` (e.g., `spawn_entity`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_ENTITIES`)
- **Modules**: `snake_case` (e.g., `asset_loader`)

### Documentation

- Document all public APIs with `///` comments
- Include examples in documentation
- Explain non-obvious behavior
- Link to related functions/types

```rust
/// Loads an asset asynchronously.
///
/// This function returns immediately with a handle. The asset loads
/// in the background and the handle becomes valid once loading completes.
///
/// # Examples
///
/// ```
/// let texture = asset_server.load::<Texture>("player.png");
/// // Use texture immediately (placeholder shown until loaded)
/// ```
///
/// # Errors
///
/// Returns `AssetError::FileNotFound` if the file doesn't exist.
///
/// # See Also
///
/// - [`AssetServer::is_loaded`] - Check if asset is loaded
/// - [`Handle`] - Asset handle documentation
pub fn load<T: Asset>(&self, path: &str) -> Result<Handle<T>, AssetError> {
    // Implementation
}
```

## Testing Requirements

### Test Coverage

- **Core crates**: Minimum 70% line coverage
- **New features**: Must include tests
- **Bug fixes**: Add regression test

### Test Types

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_composition() {
        let t1 = Transform::from_position(Vec3::X);
        let t2 = Transform::from_position(Vec3::Y);
        let result = t1.compose(&t2);
        
        assert_eq!(result.position, Vec3::new(1.0, 1.0, 0.0));
    }
}
```

#### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transform_inverse(
        position in prop::array::uniform3(-100.0f32..100.0),
    ) {
        let transform = Transform::from_position(Vec3::from(position));
        let inverse = transform.inverse();
        let identity = transform.compose(&inverse);
        
        prop_assert!((identity.position.length() < 0.001));
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use luminara::prelude::*;

#[test]
fn test_complete_workflow() {
    let mut app = App::new();
    app.add_plugin(CorePlugin);
    
    // Setup
    let entity = app.world.spawn()
        .insert(Transform::default())
        .id();
    
    // Execute
    app.update();
    
    // Verify
    let transform = app.world.get::<Transform>(entity).unwrap();
    assert!(transform.position.length() >= 0.0);
}
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_transform_composition

# Run with output
cargo test -- --nocapture

# Run property tests (more iterations)
cargo test --release

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage
```

## Pull Request Process

### Before Submitting

1. **Create an issue**: Discuss major changes first
2. **Create a branch**: `git checkout -b feature/my-feature`
3. **Write tests**: Ensure good test coverage
4. **Run tests**: `cargo test --workspace`
5. **Format code**: `cargo fmt --all`
6. **Run clippy**: `cargo clippy --workspace`
7. **Update docs**: Document new features

### PR Guidelines

#### Title Format

```
[Category] Brief description

Examples:
[Feature] Add GPU instancing support
[Fix] Resolve memory leak in asset loader
[Docs] Improve ECS architecture documentation
[Perf] Optimize frustum culling
```

#### Description Template

```markdown
## Description
Brief description of changes.

## Motivation
Why is this change needed?

## Changes
- List of changes
- Another change

## Testing
How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy passed (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
```

### Review Process

1. **Automated checks**: CI runs tests and linting
2. **Code review**: Maintainers review code
3. **Feedback**: Address review comments
4. **Approval**: At least one maintainer approval required
5. **Merge**: Maintainer merges PR

### After Merge

- Delete your branch
- Update your fork: `git pull upstream main`
- Celebrate! 🎉

## Areas for Contribution

### High Priority

- **Performance optimization**: Profiling and optimization
- **Documentation**: Tutorials, guides, examples
- **Testing**: Increase test coverage
- **Bug fixes**: Fix reported issues

### Feature Development

- **Rendering**: Advanced rendering techniques
- **Physics**: Physics improvements
- **Audio**: 3D audio, effects
- **Networking**: Multiplayer support
- **Editor**: Visual editor development

### Documentation

- **Tutorials**: Step-by-step guides
- **Examples**: Sample projects
- **API docs**: Improve API documentation
- **Translations**: Translate documentation

### Community

- **Discord moderation**: Help manage community
- **Issue triage**: Help categorize issues
- **Support**: Answer questions
- **Showcase**: Share your projects

## Getting Help

### Resources

- **Documentation**: [docs/](.)
- **Examples**: [examples/](../examples/)
- **Discord**: [Join our Discord](https://discord.gg/luminara) (coming soon)
- **Forum**: [Community Forum](https://forum.luminara-engine.org) (coming soon)

### Asking Questions

When asking for help:

1. **Search first**: Check existing issues and docs
2. **Be specific**: Provide details and context
3. **Include code**: Share relevant code snippets
4. **Show effort**: Explain what you've tried
5. **Be patient**: Maintainers are volunteers

### Reporting Bugs

Use the bug report template:

```markdown
## Description
Clear description of the bug.

## Steps to Reproduce
1. Step one
2. Step two
3. Bug occurs

## Expected Behavior
What should happen?

## Actual Behavior
What actually happens?

## Environment
- OS: Windows 11 / macOS 14 / Ubuntu 22.04
- Rust version: 1.75.0
- Luminara version: 0.1.0

## Additional Context
Any other relevant information.
```

## Recognition

Contributors are recognized in:

- **CONTRIBUTORS.md**: List of all contributors
- **Release notes**: Mention in release notes
- **Discord**: Contributor role
- **Website**: Featured on website (coming soon)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT/Apache-2.0 dual license).

---

Thank you for contributing to Luminara Engine! Your contributions help make game development more accessible and enjoyable for everyone.
