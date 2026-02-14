# Phase 1 Demo - Luminara Engine

This demo showcases the Phase 1 core engine features of the Luminara Engine with enhanced interactivity and visual effects.

## Features Demonstrated

- **Scene Loading**: Loads a complete scene from a RON file (`assets/scenes/phase1_demo_enhanced.scene.ron`)
- **3D PBR Rendering**: Displays multiple objects with physically-based materials (metallic spheres, cubes, platforms)
- **Lighting**: Directional light with shadow casting and enhanced intensity
- **Physics Simulation**: Multiple dynamic rigid bodies falling and colliding with static platforms
- **Transform Hierarchy**: Proper parent-child transform propagation
- **Asset Pipeline**: Hot-reload support for assets and scenes
- **Audio System**: Background music playback (when audio file is available)
- **Animations**: Floating and rotating objects (Energy Core)
- **Interactive Controls**: Real-time gravity, time scale, and object spawning

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

## Controls

### Camera Controls
- **W/A/S/D** or **Arrow Keys**: Move camera forward/left/backward/right
- **Space/E**: Move camera up
- **Shift/Q**: Move camera down
- **Right Mouse Button**: Hold to look around
- **C**: Toggle between first-person and third-person camera modes

### Demo Interaction Controls
- **1**: Spawn a random colored sphere
- **2**: Spawn a random colored cube
- **+/=**: Increase gravity scale
- **-**: Decrease gravity scale
- **]**: Increase time scale (speed up simulation)
- **[**: Decrease time scale (slow down simulation)
- **R**: Reset scene (remove all spawned objects)
- **G**: Toggle debug gizmos (velocity vectors, BVH, etc.)

## Expected Behavior

When the demo runs, you should see:

1. A camera positioned at (0, 5, 15) with smooth movement controls
2. A directional light casting shadows from above
3. A large ground plane (static collider) at y=0
4. Multiple platforms at different positions
5. An animated "Energy Core" sphere floating and rotating at the center
6. Red, green, and golden spheres falling due to gravity
7. Objects colliding with the ground, platforms, and each other with realistic physics
8. Ability to spawn new objects dynamically and control physics parameters

## Scene File

The demo scene is defined in `assets/scenes/phase1_demo_enhanced.scene.ron` and includes:

- **Camera**: Perspective projection with audio listener and smooth controller
- **Sun**: Directional light with cascaded shadow maps
- **Ground**: Large static physics body with box collider
- **EnergyCore**: Animated floating and rotating sphere with emissive material
- **Multiple Spheres**: Dynamic physics bodies with different materials (red metallic, golden)
- **Green Cube**: Dynamic physics body with matte material
- **Platforms**: Static elevated platforms for complex physics interactions

## Adding Background Music

To add background music to the demo:

1. Place an audio file (WAV, OGG, MP3, or FLAC) at `assets/audio/background_music.ogg`
2. The demo will automatically detect and play it on startup

## Performance Tips

- The demo supports spawning many objects dynamically
- Use time scale controls to slow down or speed up the simulation
- Reset the scene (R key) if performance degrades with too many objects
- Debug gizmos (G key) can impact performance when enabled

## Notes

- The demo uses the `DefaultPlugins` bundle which includes all core engine plugins
- Component deserialization is handled manually in the demo to avoid circular dependencies
- Animations are applied programmatically to demonstrate the animation system
- Physics parameters can be adjusted in real-time to experiment with different behaviors
