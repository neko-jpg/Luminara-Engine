# Phase 1 Demo - Luminara Engine

This demo showcases the Phase 1 core engine features of the Luminara Engine.

## Features Demonstrated

- **Scene Loading**: Loads a complete scene from a RON file (`assets/scenes/phase1_demo.scene.ron`)
- **3D PBR Rendering**: Displays objects with physically-based materials (metallic sphere)
- **Lighting**: Directional light with shadow casting
- **Physics Simulation**: Dynamic rigid body (sphere) falling and colliding with static ground
- **Transform Hierarchy**: Proper parent-child transform propagation
- **Asset Pipeline**: Hot-reload support for assets and scenes
- **Audio System**: Background music playback (when audio file is available)

## Requirements Validated

- **10.1**: Engine displays a 3D scene with PBR-rendered objects
- **10.2**: Physics engine simulates falling objects with visible collision responses
- **10.3**: At least one light source casting shadows
- **10.4**: Assets loaded from the asset pipeline
- **10.5**: Audio system plays background music or sound effects (when audio file is present)

## Running the Demo

From the workspace root:

```bash
wsl -d ubuntu /home/arat2/.cargo/bin/cargo run --bin phase1_demo
```

Or from this directory:

```bash
wsl -d ubuntu /home/arat2/.cargo/bin/cargo run
```

## Expected Behavior

When the demo runs, you should see:

1. A camera positioned at (0, 5, 10) looking at the scene
2. A directional light casting shadows from above
3. A ground plane (static collider) at y=0
4. A red metallic sphere falling due to gravity
5. The sphere colliding with the ground and bouncing

## Scene File

The demo scene is defined in `assets/scenes/phase1_demo.scene.ron` and includes:

- **Camera**: Perspective projection with audio listener
- **Sun**: Directional light with cascaded shadow maps
- **Ground**: Static physics body with box collider
- **Sphere**: Dynamic physics body with sphere collider and PBR material

## Adding Background Music

To add background music to the demo:

1. Place an audio file (WAV, OGG, MP3, or FLAC) at `assets/audio/background_music.ogg`
2. The demo will automatically detect and play it on startup

## Notes

- The demo uses the `DefaultPlugins` bundle which includes all core engine plugins
- Component deserialization is handled manually in the demo to avoid circular dependencies
- The scene system currently only auto-deserializes Transform components; other components are added programmatically
