# Asset Pipeline Architecture

## Overview

Luminara Engine features a fully asynchronous asset loading system with hot-reload support, placeholder assets, and intelligent caching. The pipeline is designed to never block the main thread, ensuring smooth gameplay even during asset loading.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Asset Request                            │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Handle    │→ │  Asset     │→ │  Load Request        │  │
│  │  Creation  │  │  Server    │  │  Queue               │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│                      Background Loading                       │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  I/O       │→ │  Parsing   │→ │  GPU Upload          │  │
│  │  Thread    │  │  Thread    │  │  (Main Thread)       │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│                      Asset Management                         │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Cache     │  │  Hot       │  │  Dependency          │  │
│  │  Manager   │  │  Reload    │  │  Tracking            │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Asset Handles

Handles are lightweight references to assets that remain valid even while assets are loading.

```rust
pub struct Handle<T> {
    id: AssetId,
    _marker: PhantomData<T>,
}

// Handles are:
// - Cheap to clone (just an ID)
// - Type-safe (Handle<Texture> != Handle<Mesh>)
// - Valid across hot-reloads
// - Automatically reference-counted
```

**Usage:**
```rust
// Request asset (returns immediately)
let texture: Handle<Texture> = asset_server.load("textures/player.png");

// Use handle immediately (placeholder shown if not loaded)
material.base_color_texture = Some(texture.clone());

// Check load status
if asset_server.is_loaded(&texture) {
    // Asset ready
}
```

### Asset Server

Central hub for all asset operations.

```rust
pub struct AssetServer {
    /// Asset storage
    assets: HashMap<AssetId, Box<dyn Asset>>,
    /// Load queue
    load_queue: Arc<Mutex<VecDeque<LoadRequest>>>,
    /// Thread pool for I/O
    io_pool: ThreadPool,
    /// Cache for loaded assets
    cache: AssetCache,
}

impl AssetServer {
    /// Load asset asynchronously
    pub fn load<T: Asset>(&self, path: &str) -> Handle<T>;
    
    /// Get asset if loaded
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<&T>;
    
    /// Check if asset is loaded
    pub fn is_loaded<T: Asset>(&self, handle: &Handle<T>) -> bool;
    
    /// Unload asset
    pub fn unload<T: Asset>(&self, handle: Handle<T>);
}
```

### Asset Trait

All assets implement the Asset trait.

```rust
pub trait Asset: Send + Sync + 'static {
    /// Load asset from bytes
    fn load_from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;
    
    /// Asset type name
    fn type_name() -> &'static str where Self: Sized;
    
    /// Asset dependencies
    fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }
}
```

## Supported Asset Types

### Textures

```rust
#[derive(Asset)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
}

// Supported formats:
// - PNG, JPEG, BMP, TGA, DDS, KTX2
// - Automatic mipmap generation
// - Compression support (BC, ASTC, ETC2)

// Usage:
let texture = asset_server.load::<Texture>("textures/player.png");
```

### Meshes

```rust
#[derive(Asset)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub primitive_topology: PrimitiveTopology,
}

// Supported formats:
// - glTF 2.0 (.gltf, .glb)
// - OBJ (.obj)
// - FBX (.fbx) - via external converter

// Usage:
let mesh = asset_server.load::<Mesh>("models/character.gltf");
```

### Audio

```rust
#[derive(Asset)]
pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

// Supported formats:
// - WAV, MP3, OGG, FLAC
// - Streaming for long audio files
// - Spatial audio support

// Usage:
let audio = asset_server.load::<AudioClip>("sounds/explosion.ogg");
```

### Scenes

```rust
#[derive(Asset)]
pub struct Scene {
    pub entities: Vec<EntityData>,
    pub resources: Vec<ResourceData>,
}

// Format: RON (Rusty Object Notation)
// Human-readable and version-control friendly
// Supports entity hierarchies and references

// Usage:
let scene = asset_server.load::<Scene>("scenes/level1.scene.ron");
```

### Scripts

```rust
// Lua scripts
let script = asset_server.load::<LuaScript>("scripts/player.lua");

// WASM modules
let wasm = asset_server.load::<WasmModule>("scripts/ai.wasm");

// Hot-reload supported for both
```

## Asynchronous Loading

### Non-Blocking Design

The asset pipeline never blocks the main thread.

```rust
// This returns immediately, even for large assets
let texture = asset_server.load::<Texture>("huge_texture.png");

// Game continues running
// Placeholder shown until asset loads
// Automatic swap when ready
```

**Loading Pipeline:**

1. **Request Phase** (Main Thread, <1μs)
   - Create handle
   - Queue load request
   - Return handle immediately

2. **I/O Phase** (Background Thread)
   - Read file from disk
   - Decompress if needed
   - Parse file format

3. **Processing Phase** (Background Thread)
   - Decode image/mesh/audio
   - Generate mipmaps
   - Optimize data layout

4. **Upload Phase** (Main Thread, <1ms)
   - Upload to GPU
   - Update asset storage
   - Notify systems

### Placeholder Assets

Placeholders are shown while assets load.

```rust
pub struct PlaceholderAssets {
    pub texture: Handle<Texture>,      // Checkerboard pattern
    pub mesh: Handle<Mesh>,            // Cube
    pub audio: Handle<AudioClip>,      // Silence
}

// Automatically used until real asset loads
// Configurable per asset type
// No visual "pop" when swapping
```

**Placeholder Strategy:**
- Textures: Pink/black checkerboard (indicates missing texture)
- Meshes: Simple cube or sphere
- Audio: Silence
- Materials: Default gray material

### Parallel Loading

Multiple assets load concurrently.

```rust
// All load in parallel
let textures = vec![
    asset_server.load::<Texture>("tex1.png"),
    asset_server.load::<Texture>("tex2.png"),
    asset_server.load::<Texture>("tex3.png"),
];

// Thread pool size configurable
asset_server.set_thread_count(8);

// Priority queue for important assets
asset_server.load_with_priority("player.png", Priority::High);
```

**Performance:**
- 8 worker threads by default
- Priority queue for critical assets
- Automatic load balancing
- Minimal main thread overhead (<0.1ms/frame)

## Hot Reload

Assets automatically reload when files change.

### File Watching

```rust
pub struct AssetWatcher {
    watcher: RecommendedWatcher,
    events: Receiver<DebouncedEvent>,
}

// Watches asset directories
// Debounces rapid changes
// Triggers reload on file modification
```

**Supported Platforms:**
- Windows: ReadDirectoryChangesW
- macOS: FSEvents
- Linux: inotify
- WASM: Polling (configurable interval)

### Reload Process

```
1. File Change Detected
   └─> Debounce (300ms)

2. Reload Asset
   ├─> Load new version
   ├─> Parse and process
   └─> Upload to GPU

3. Swap Assets
   ├─> Update handle mapping
   ├─> Preserve references
   └─> Notify systems

4. Cleanup
   └─> Free old asset memory
```

**State Preservation:**
- Entity references preserved
- Component data unchanged
- Material properties maintained
- Only asset data updated

### Hot Reload Performance

```
Texture reload: <50ms
Mesh reload: <100ms
Script reload: <100ms
Scene reload: <200ms

Zero frame drops during reload
```

## Caching

### Memory Cache

Recently used assets stay in memory.

```rust
pub struct AssetCache {
    /// LRU cache
    cache: LruCache<AssetId, Box<dyn Asset>>,
    /// Maximum memory usage
    max_size: usize,
}

// Default: 512MB cache
// Configurable per asset type
// Automatic eviction of unused assets
```

**Cache Strategy:**
- LRU (Least Recently Used) eviction
- Reference counting prevents premature eviction
- Configurable size limits
- Per-type cache policies

### Disk Cache

Processed assets cached on disk.

```rust
// Processed assets stored in:
// - Windows: %LOCALAPPDATA%/luminara/cache
// - macOS: ~/Library/Caches/luminara
// - Linux: ~/.cache/luminara

// Cache includes:
// - Compressed textures
// - Optimized meshes
// - Parsed scenes
// - Compiled shaders

// Cache invalidation:
// - Source file modification time
// - Asset version number
// - Engine version
```

## Dependency Tracking

Assets can depend on other assets.

```rust
impl Asset for Material {
    fn dependencies(&self) -> Vec<AssetId> {
        let mut deps = Vec::new();
        
        if let Some(tex) = &self.base_color_texture {
            deps.push(tex.id());
        }
        if let Some(tex) = &self.normal_map {
            deps.push(tex.id());
        }
        
        deps
    }
}

// Dependency graph automatically maintained
// Dependent assets reload when dependencies change
// Circular dependencies detected and reported
```

### Dependency Resolution

```rust
// Load material (automatically loads textures)
let material = asset_server.load::<Material>("materials/wood.mat");

// Dependency graph:
// wood.mat
//   ├─> wood_albedo.png
//   ├─> wood_normal.png
//   └─> wood_roughness.png

// All textures loaded automatically
// Material ready when all dependencies loaded
```

## Error Handling

### Retry Logic

Failed loads automatically retry with exponential backoff.

```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f32,
}

// Default policy:
// - 3 attempts
// - Initial delay: 100ms
// - Max delay: 5s
// - Backoff: 2x

// After max attempts: fallback to error placeholder
```

### Error Reporting

```rust
pub enum AssetError {
    FileNotFound(PathBuf),
    ParseError { path: PathBuf, error: String },
    UnsupportedFormat { path: PathBuf, format: String },
    IoError(io::Error),
}

// Errors logged with context
// Error placeholder shown in-game
// Detailed error in console/log file
```

**Error Placeholder:**
- Textures: Bright magenta (easy to spot)
- Meshes: Error cube with "ERROR" text
- Audio: Beep sound (indicates missing audio)

## Progress Tracking

Track loading progress for loading screens.

```rust
pub struct LoadProgress {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: usize,
}

// Query progress
let progress = asset_server.get_progress();
let percent = (progress.loaded_assets as f32 / progress.total_assets as f32) * 100.0;

// Display loading bar
ui.progress_bar(percent);
```

## Asset Bundles

Group related assets for batch loading.

```rust
#[derive(Asset)]
pub struct AssetBundle {
    pub assets: Vec<AssetId>,
}

// Create bundle
let level1_bundle = AssetBundle {
    assets: vec![
        asset_server.load::<Texture>("level1/ground.png").id(),
        asset_server.load::<Mesh>("level1/building.gltf").id(),
        asset_server.load::<AudioClip>("level1/music.ogg").id(),
    ],
};

// Load entire bundle
asset_server.load_bundle(level1_bundle);

// Unload entire bundle
asset_server.unload_bundle(level1_bundle);
```

## Best Practices

### Asset Organization

```
assets/
├── textures/
│   ├── characters/
│   ├── environment/
│   └── ui/
├── models/
│   ├── characters/
│   └── props/
├── audio/
│   ├── music/
│   ├── sfx/
│   └── voice/
├── scenes/
│   └── levels/
└── scripts/
    ├── lua/
    └── wasm/
```

### Loading Strategies

**Eager Loading:**
```rust
// Load critical assets at startup
fn setup(asset_server: Res<AssetServer>) {
    asset_server.load::<Texture>("ui/logo.png");
    asset_server.load::<AudioClip>("music/menu.ogg");
}
```

**Lazy Loading:**
```rust
// Load assets when needed
fn spawn_enemy(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    let mesh = asset_server.load::<Mesh>("enemies/goblin.gltf");
    commands.spawn().insert(mesh);
}
```

**Streaming:**
```rust
// Load level sections as player approaches
fn stream_level(
    player_pos: Vec3,
    asset_server: Res<AssetServer>,
) {
    let chunk = get_chunk_at(player_pos);
    asset_server.load_bundle(chunk.asset_bundle);
}
```

### Memory Management

```rust
// Unload unused assets
asset_server.unload_unused();

// Set memory limits
asset_server.set_cache_size(512 * 1024 * 1024); // 512MB

// Monitor memory usage
let usage = asset_server.memory_usage();
println!("Asset memory: {}MB", usage / 1024 / 1024);
```

### Performance Tips

- **Batch loads**: Load related assets together
- **Use bundles**: Group level assets
- **Preload critical assets**: Load menu/UI assets early
- **Unload unused assets**: Free memory when changing levels
- **Use compressed textures**: Reduce memory and load time
- **Optimize mesh LODs**: Load appropriate detail level

## Advanced Features

### Custom Asset Loaders

```rust
pub trait AssetLoader: Send + Sync {
    type Asset: Asset;
    
    fn load(&self, bytes: &[u8]) -> Result<Self::Asset>;
    fn extensions(&self) -> &[&str];
}

// Register custom loader
asset_server.register_loader(MyCustomLoader);
```

### Asset Preprocessing

```rust
// Preprocess assets at build time
// - Compress textures
// - Optimize meshes
// - Compile shaders
// - Generate mipmaps

// Run preprocessor:
// cargo run --bin asset_preprocessor
```

### Virtual File System

```rust
// Support for:
// - Zip archives
// - Pak files
// - Embedded assets
// - Network loading

asset_server.mount("data.pak", "/");
let texture = asset_server.load::<Texture>("/textures/player.png");
```

## Performance Benchmarks

```
Asset Loading Performance:
- Texture (2K PNG): 15ms
- Mesh (10K vertices): 8ms
- Audio (1min OGG): 25ms
- Scene (100 entities): 12ms

Hot Reload Performance:
- Texture: 45ms
- Mesh: 85ms
- Script: 95ms
- Scene: 180ms

Memory Usage:
- Handle: 16 bytes
- Cache overhead: <5% of asset size
- Typical game: 200-500MB assets in memory
```

## Further Reading

- [Asset Formats](asset_formats.md) - Supported file formats
- [Hot Reload Guide](../workflows/hot_reload.md) - Using hot reload effectively
- [Performance Optimization](../audit/asset_loading_analysis.md) - Asset loading optimization
- [Custom Loaders](custom_asset_loaders.md) - Creating custom asset loaders

## References

- [glTF Specification](https://www.khronos.org/gltf/) - 3D model format
- [KTX2 Specification](https://www.khronos.org/ktx/) - Texture format
- [RON Format](https://github.com/ron-rs/ron) - Scene serialization
