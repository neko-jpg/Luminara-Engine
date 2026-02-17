# Placeholder Asset System

## Overview

The placeholder asset system provides temporary assets that are displayed while real assets are loading asynchronously. This ensures that the rendering pipeline never encounters missing assets and maintains smooth frame rates during asset loading.

## Key Features

- **Immediate Availability**: Placeholder assets are inserted immediately when loading starts
- **Hot-Swap**: Real assets replace placeholders seamlessly when loading completes
- **Zero Frame Drops**: Assets are always available, preventing rendering gaps
- **Type-Safe**: Placeholders are registered per asset type using Rust's type system

## Architecture

### PlaceholderRegistry

The `PlaceholderRegistry` stores placeholder assets indexed by `TypeId`:

```rust
pub struct PlaceholderRegistry {
    placeholders: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}
```

### PlaceholderAsset Trait

Asset types can implement the `PlaceholderAsset` trait to provide default placeholders:

```rust
pub trait PlaceholderAsset: Asset {
    fn create_placeholder() -> Self where Self: Sized;
}
```

## Usage

### Registering Placeholders

Register placeholders with the `AssetServer` before loading assets:

```rust
use luminara_asset::{AssetServer, PlaceholderAsset};
use luminara_render::{Mesh, Texture};

let mut server = AssetServer::new("assets");

// Register default placeholders
server.register_placeholder(Mesh::create_placeholder());
server.register_placeholder(Texture::create_placeholder());

// Or register custom placeholders
server.register_placeholder(Texture::solid_color(64, 64, [255, 0, 255, 255]));
```

### Loading with Placeholders

When you load an asset, the placeholder is immediately available:

```rust
// Load texture - placeholder is available immediately
let handle = server.load::<Texture>("textures/albedo.png");

// Get asset - returns placeholder while loading
if let Some(texture) = server.get(&handle) {
    // Use texture (placeholder or real)
    // Rendering continues without interruption
}

// Update asset server each frame
server.update();

// After loading completes, get() returns the real asset
// Hot-swap happens automatically without frame drops
```

### Asset Loading Timeline

```
Frame 0: load() called
         ├─ Placeholder inserted immediately
         ├─ Handle returned
         └─ Async loading starts

Frame 1-N: get() returns placeholder
           └─ Rendering uses placeholder

Frame N+1: update() processes load completion
           ├─ Real asset replaces placeholder (hot-swap)
           └─ Generation incremented

Frame N+2+: get() returns real asset
            └─ Rendering uses real asset
```

## Default Placeholders

### Mesh

The default mesh placeholder is a simple 1x1x1 cube:

```rust
impl PlaceholderAsset for Mesh {
    fn create_placeholder() -> Self {
        Mesh::cube(1.0)
    }
}
```

### Texture

The default texture placeholder is a 64x64 magenta/black checkerboard:

```rust
impl PlaceholderAsset for Texture {
    fn create_placeholder() -> Self {
        Texture::checkerboard(
            64,
            [255, 0, 255, 255], // Magenta
            [0, 0, 0, 255],     // Black
        )
    }
}
```

Magenta is commonly used in game engines to indicate missing or loading textures.

## Implementation Details

### Hot-Swap Mechanism

Hot-swapping is implemented using generation counters:

1. Placeholder is inserted with generation 0
2. When real asset loads, generation is incremented
3. Handles track generation to detect updates
4. Rendering systems can detect hot-swaps if needed

```rust
// In AssetServer::update()
let current_gen = if let Some(entry) = assets.get(&result.id) {
    entry.generation + 1  // Increment for hot-swap
} else {
    0
};

assets.insert(id, AssetEntry {
    asset: real_asset,
    generation: current_gen,
});
```

### Thread Safety

The placeholder system is thread-safe:

- `PlaceholderRegistry` uses `Arc<RwLock<...>>` for concurrent access
- Placeholders are `Arc<T>` for cheap cloning
- Asset insertion is atomic per asset ID

### Performance

- **Placeholder insertion**: O(1) hash map lookup and insert
- **Hot-swap**: O(1) hash map update
- **Memory overhead**: One placeholder instance per asset type (shared via Arc)
- **No frame drops**: Assets are always available, preventing rendering gaps

## Best Practices

### 1. Register Placeholders Early

Register placeholders during engine initialization:

```rust
fn setup_asset_system(server: &mut AssetServer) {
    server.register_placeholder(Mesh::create_placeholder());
    server.register_placeholder(Texture::create_placeholder());
    // Register other asset type placeholders
}
```

### 2. Use Distinctive Placeholders

Make placeholders visually distinctive so developers can identify loading assets:

```rust
// Good: Magenta checkerboard is clearly a placeholder
server.register_placeholder(Texture::checkerboard(64, [255, 0, 255, 255], [0, 0, 0, 255]));

// Bad: White texture looks like a real asset
server.register_placeholder(Texture::solid_color(64, 64, [255, 255, 255, 255]));
```

### 3. Call update() Every Frame

Ensure `AssetServer::update()` is called every frame to process completed loads:

```rust
fn update_system(server: Res<AssetServer>) {
    server.update();
}
```

### 4. Handle Missing Placeholders

If no placeholder is registered, the asset won't be available until loading completes:

```rust
let handle = server.load::<CustomAsset>("data.custom");

// Returns None if no placeholder registered and asset not loaded
if let Some(asset) = server.get(&handle) {
    // Use asset
} else {
    // Skip rendering or use fallback
}
```

## Testing

The placeholder system includes comprehensive tests:

- `test_placeholder_registry_basic`: Verifies placeholder registration and retrieval
- `test_placeholder_in_asset_server`: Tests integration with AssetServer
- `test_placeholder_display_during_loading`: Verifies immediate placeholder availability
- `test_hot_swap_without_frame_drops`: Ensures assets are always available during hot-swap

Run tests with:

```bash
cargo test -p luminara_asset --test placeholder_test
```

## Future Enhancements

Potential improvements to the placeholder system:

1. **Automatic Placeholder Generation**: Generate simple placeholders automatically for common asset types
2. **Loading Progress**: Track loading progress and update placeholder appearance
3. **Priority Loading**: Load high-priority assets first, keep low-priority placeholders longer
4. **Placeholder Caching**: Cache generated placeholders to disk for faster startup
5. **Streaming LOD**: Use low-resolution placeholders that progressively improve

## Related Systems

- **Asset Loading**: See `crates/luminara_asset/src/server.rs`
- **Hot Reload**: See `crates/luminara_asset/src/hot_reload.rs`
- **Handle System**: See `crates/luminara_asset/src/handle.rs`

## Requirements

This system implements:

- **Requirement 14.2**: Display placeholders during loading
- **Property 16**: Placeholder Asset Display

From the Pre-Editor Engine Audit specification.
