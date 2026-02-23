# Luminara Editor

Luminara Editor is the official integrated development environment for the Luminara Engine. It provides a modern, high-performance interface for scene composition, logic editing, and asset management.

## Architecture

The editor is built using **Vizia** (v0.3) for the UI layer, communicating with the **Bevy**-based backend engine.

### Key Components

- **Vizia UI**: Handles all windowing, input, and widget rendering.
- **Engine Bridge**: Communicates with the background Bevy engine instance.
- **WGPU Viewport**: Embeds the 3D engine output directly into the Vizia interface.

## Getting Started

To run the editor:

```bash
cargo run -p luminara_editor
```

## Features

- **Scene Builder**: 3D viewport and hierarchy management.
- **Logic Graph**: Visual node-based scripting.
- **Director**: Cinematic timeline editor.
- **Backend & AI**: Integrated script editor with AI assistance.
- **Asset Vault**: Asset management and preview.
