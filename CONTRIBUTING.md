# Contributing to Luminara Engine

## Workspace Structure

- `crates/luminara_core`: ECS and main loop.
- `crates/luminara_math`: Math primitives.
- `crates/luminara_window`: Windowing.
- `crates/luminara_input`: Input system.
- `crates/luminara_render`: Rendering.
- `crates/luminara_asset`: Asset management.
- `crates/luminara_scene`: Scene management.
- `crates/luminara_platform`: Platform abstractions.
- `crates/luminara_diagnostic`: Logging and profiling.
- `crates/luminara`: Meta-crate re-exporting everything.

## Coding Standards

- Run `cargo fmt` before committing.
- Ensure `cargo clippy --workspace` has no warnings.
- All new features should have tests in the `tests/` directory of the respective crate or in the workspace-level `tests/`.
