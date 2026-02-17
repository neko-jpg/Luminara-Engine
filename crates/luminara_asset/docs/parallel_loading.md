# Parallel Asset Loading

## Overview

The Luminara asset system supports parallel asset loading with priority-based scheduling. This allows multiple assets to load concurrently while ensuring that critical assets are loaded first.

## Features

### 1. Configurable Thread Pool

The asset server uses a configurable thread pool for parallel I/O operations:

```rust
use luminara_asset::AssetServer;

// Create with default thread count (min of CPU cores and 8)
let server = AssetServer::new("assets");

// Create with custom thread count
let server = AssetServer::with_thread_count("assets", 4);

// Check configured thread count
println!("Thread pool size: {}", server.thread_count());
```

### 2. Priority-Based Loading

Assets can be loaded with different priority levels to control loading order:

```rust
use luminara_asset::{AssetServer, LoadPriority};

let server = AssetServer::new("assets");

// Load with default (Normal) priority
let texture = server.load::<Texture>("player.png");

// Load with specific priority
let critical_texture = server.load_with_priority::<Texture>(
    "ui/health_bar.png", 
    LoadPriority::Critical
);

let background = server.load_with_priority::<Texture>(
    "backgrounds/level1.png",
    LoadPriority::Low
);
```

### Priority Levels

- **Critical** (3): Essential assets needed immediately (UI, player character)
- **High** (2): Important assets for current gameplay (enemies, weapons)
- **Normal** (1): Standard assets (default priority)
- **Low** (0): Background assets that can wait (distant scenery, music)

## How It Works

### Architecture

1. **Priority Queue**: Load requests are queued in a priority queue (binary heap)
2. **Dispatcher Thread**: A dedicated thread manages the queue and dispatches to tokio runtime
3. **Tokio Runtime**: Handles async I/O operations with configured worker threads
4. **Non-Blocking**: Main thread never blocks on asset loading

### Loading Pipeline

```
Load Request → Priority Queue → Tokio Runtime → Background Thread Pool
                                      ↓
                                  File I/O (async)
                                      ↓
                                Asset Parsing (blocking)
                                      ↓
                                Result Channel → Main Thread
```

### Sequence Ordering

When multiple assets have the same priority, they are loaded in FIFO order (first requested, first loaded). This is achieved using a sequence counter that increments with each request.

## Performance Characteristics

### Parallel Loading

With N threads and M assets:
- **Sequential**: M × load_time
- **Parallel**: ⌈M / N⌉ × load_time

Example with 50 assets at 20ms each:
- Sequential: ~1000ms
- 8 threads: ~140ms (7x speedup)

### Priority Impact

Higher priority assets start loading first, reducing perceived load time for critical content:

```rust
// Without priority: All assets load in submission order
// Load time to first critical asset: depends on queue position

// With priority: Critical assets jump to front of queue
// Load time to first critical asset: minimal (starts immediately)
```

## Best Practices

### 1. Choose Appropriate Thread Count

```rust
// For I/O-bound loading (most cases)
let threads = num_cpus::get().min(8);

// For CPU-bound parsing (complex formats)
let threads = num_cpus::get() / 2;

// For memory-constrained environments
let threads = 2;
```

### 2. Use Priority Wisely

```rust
// Critical: Assets needed for first frame
server.load_with_priority::<Texture>("ui/loading.png", LoadPriority::Critical);

// High: Assets for current level/scene
server.load_with_priority::<Mesh>("level1/player.gltf", LoadPriority::High);

// Normal: Standard assets (default)
server.load::<Texture>("level1/wall.png");

// Low: Preload for future levels
server.load_with_priority::<Audio>("level2/music.ogg", LoadPriority::Low);
```

### 3. Batch Loading

Load related assets together to maximize parallelism:

```rust
// Good: Load all level assets at once
let handles: Vec<_> = level_assets
    .iter()
    .map(|path| server.load_with_priority::<Texture>(path, LoadPriority::High))
    .collect();

// Less optimal: Load one at a time with delays
for path in level_assets {
    let handle = server.load::<Texture>(path);
    // Don't wait here - let them load in parallel
}
```

### 4. Monitor Load States

```rust
use luminara_asset::LoadState;

// Check if asset is ready
match server.load_state(handle.id()) {
    LoadState::Loaded => {
        // Asset is ready to use
        if let Some(texture) = server.get(&handle) {
            // Use texture
        }
    }
    LoadState::Loading => {
        // Still loading, show placeholder
    }
    LoadState::Failed(err) => {
        // Handle error
        eprintln!("Failed to load: {}", err);
    }
    LoadState::NotLoaded => {
        // Not yet requested
    }
}
```

## Example: Level Loading System

```rust
use luminara_asset::{AssetServer, LoadPriority, LoadState};

struct LevelLoader {
    server: AssetServer,
}

impl LevelLoader {
    fn load_level(&self, level_id: u32) {
        // Critical: UI and player
        let ui_handles = vec![
            self.server.load_with_priority::<Texture>(
                "ui/health_bar.png", 
                LoadPriority::Critical
            ),
            self.server.load_with_priority::<Texture>(
                "ui/minimap.png",
                LoadPriority::Critical
            ),
        ];
        
        let player_handle = self.server.load_with_priority::<Mesh>(
            "characters/player.gltf",
            LoadPriority::Critical
        );
        
        // High: Level geometry and enemies
        let level_handles = vec![
            self.server.load_with_priority::<Mesh>(
                &format!("levels/level{}.gltf", level_id),
                LoadPriority::High
            ),
            self.server.load_with_priority::<Texture>(
                &format!("levels/level{}_diffuse.png", level_id),
                LoadPriority::High
            ),
        ];
        
        // Normal: Props and decorations
        let prop_handles = self.load_props(level_id);
        
        // Low: Background music and distant scenery
        let music_handle = self.server.load_with_priority::<Audio>(
            &format!("music/level{}.ogg", level_id),
            LoadPriority::Low
        );
    }
    
    fn check_loading_progress(&self, handles: &[Handle<Texture>]) -> f32 {
        let loaded = handles.iter()
            .filter(|h| matches!(
                self.server.load_state(h.id()),
                LoadState::Loaded
            ))
            .count();
        
        loaded as f32 / handles.len() as f32
    }
}
```

## Performance Tuning

### Measure Loading Performance

```rust
use std::time::Instant;

let start = Instant::now();

// Load assets
let handles: Vec<_> = (0..100)
    .map(|i| server.load::<Texture>(&format!("texture_{}.png", i)))
    .collect();

// Wait for all to load
loop {
    server.update();
    
    let all_loaded = handles.iter().all(|h| {
        matches!(server.load_state(h.id()), LoadState::Loaded)
    });
    
    if all_loaded {
        break;
    }
    
    std::thread::sleep(Duration::from_millis(10));
}

let elapsed = start.elapsed();
println!("Loaded 100 textures in {:?}", elapsed);
```

### Optimize Thread Count

Test different thread counts to find optimal configuration for your workload:

```rust
for thread_count in [1, 2, 4, 8, 16] {
    let server = AssetServer::with_thread_count("assets", thread_count);
    let elapsed = benchmark_loading(&server);
    println!("{} threads: {:?}", thread_count, elapsed);
}
```

## Limitations

1. **Priority Inversion**: Once an asset starts loading, it runs to completion even if higher priority assets arrive
2. **Memory Usage**: All loaded assets remain in memory until explicitly unloaded
3. **File System**: Performance depends on disk I/O speed and file system characteristics

## Future Enhancements

- [ ] Streaming for large assets (progressive loading)
- [ ] Automatic priority adjustment based on usage patterns
- [ ] Memory budget management with automatic unloading
- [ ] Dependency-aware loading (load dependencies first)
- [ ] Compression support for faster I/O
