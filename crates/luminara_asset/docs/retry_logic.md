# Asset Loading Retry Logic

## Overview

The asset loading system includes automatic retry logic with exponential backoff to handle transient failures gracefully. This ensures that temporary issues (like file locks, network interruptions, or resource contention) don't cause permanent asset loading failures.

## Features

### 1. Exponential Backoff

When an asset fails to load due to a transient error, the system automatically retries with increasing delays:

- **First retry**: Initial delay (default: 100ms)
- **Second retry**: Initial delay × backoff multiplier (default: 200ms)
- **Third retry**: Previous delay × backoff multiplier (default: 400ms)
- **Maximum delay**: Capped at max delay (default: 5 seconds)

### 2. Transient Error Detection

The system automatically detects transient errors that should be retried:

**I/O Errors:**
- `Interrupted`: Operation was interrupted
- `WouldBlock`: Resource temporarily unavailable
- `TimedOut`: Operation timed out
- `ConnectionReset`: Connection was reset
- `ConnectionAborted`: Connection was aborted
- `BrokenPipe`: Broken pipe

**Parse Errors:**
- I/O errors during parsing are also retried

### 3. Fallback to Error Placeholder

After exhausting all retry attempts, the system falls back to:
1. **Registered fallback asset**: If a fallback was registered for the asset type
2. **Error state**: If no fallback is available

### 4. Progress Tracking

The system provides real-time progress tracking for asset loading:

```rust
let progress = asset_server.load_progress();
println!("Loading: {}/{} assets", progress.loaded, progress.total);
println!("Currently loading: {}", progress.loading);
println!("Failed: {}", progress.failed);
```

## Configuration

### Default Configuration

```rust
let asset_server = AssetServer::new("assets");
// Uses default retry config:
// - max_retries: 3
// - initial_delay: 100ms
// - max_delay: 5 seconds
// - backoff_multiplier: 2.0
```

### Custom Configuration

```rust
use luminara_asset::{AssetServer, RetryConfig};
use std::time::Duration;

let retry_config = RetryConfig {
    max_retries: 5,
    initial_delay: Duration::from_millis(50),
    max_delay: Duration::from_secs(10),
    backoff_multiplier: 2.5,
};

let asset_server = AssetServer::with_config("assets", 4, retry_config);
```

### Configuration Parameters

- **max_retries**: Maximum number of retry attempts (default: 3)
- **initial_delay**: Delay before first retry (default: 100ms)
- **max_delay**: Maximum delay between retries (default: 5 seconds)
- **backoff_multiplier**: Multiplier for exponential backoff (default: 2.0)

## Usage Examples

### Basic Usage with Automatic Retry

```rust
// Load asset - retries automatically on transient failures
let texture_handle = asset_server.load::<Texture>("textures/player.png");

// The system will:
// 1. Try to load immediately
// 2. If transient error, wait 100ms and retry
// 3. If still failing, wait 200ms and retry
// 4. If still failing, wait 400ms and retry
// 5. If all retries exhausted, use fallback or mark as failed
```

### With Fallback Asset

```rust
// Register a fallback texture for when loading fails
asset_server.register_fallback(Texture::error_texture());

// Now if loading fails after all retries, the error texture is used
let texture_handle = asset_server.load::<Texture>("textures/missing.png");
```

### With Placeholder Asset

```rust
// Register a placeholder that shows while loading
asset_server.register_placeholder(Texture::loading_texture());

// The placeholder is shown immediately, then hot-swapped when real asset loads
let texture_handle = asset_server.load::<Texture>("textures/large_texture.png");
```

### Monitoring Progress

```rust
// Load multiple assets
for i in 0..100 {
    asset_server.load::<Texture>(&format!("textures/tile_{}.png", i));
}

// Monitor progress
loop {
    asset_server.update();
    
    let progress = asset_server.load_progress();
    println!(
        "Progress: {}/{} loaded, {} loading, {} failed",
        progress.loaded,
        progress.total,
        progress.loading,
        progress.failed
    );
    
    if progress.loaded + progress.failed == progress.total {
        break;
    }
    
    std::thread::sleep(Duration::from_millis(100));
}
```

## Implementation Details

### Retry Flow

```
┌─────────────────┐
│  Load Request   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Read File      │
└────────┬────────┘
         │
         ├─ Success ──────────────┐
         │                        │
         └─ Transient Error       │
                  │               │
                  ▼               │
         ┌─────────────────┐     │
         │ Check Retry     │     │
         │ Attempts        │     │
         └────────┬────────┘     │
                  │               │
         ├─ < Max Retries        │
         │        │               │
         │        ▼               │
         │ ┌─────────────────┐   │
         │ │ Exponential     │   │
         │ │ Backoff Delay   │   │
         │ └────────┬────────┘   │
         │          │             │
         │          └─ Retry ─────┘
         │                        │
         └─ >= Max Retries        │
                  │               │
                  ▼               ▼
         ┌─────────────────┐     │
         │ Use Fallback    │     │
         │ or Mark Failed  │     │
         └─────────────────┘     │
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Parse Asset    │
                        └────────┬────────┘
                                 │
                        ├─ Success ──────┐
                        │                │
                        └─ Parse Error   │
                                 │       │
                                 ▼       │
                        (Retry if       │
                         transient)     │
                                        │
                                        ▼
                               ┌─────────────────┐
                               │  Asset Loaded   │
                               └─────────────────┘
```

### Thread Safety

- All retry logic is thread-safe
- Multiple assets can be loading and retrying concurrently
- Retry state is managed per-asset
- No global locks during retry delays

### Performance Considerations

- Retry delays use async sleep (non-blocking)
- Failed assets don't block other assets from loading
- Priority queue ensures high-priority assets are processed first
- Exponential backoff prevents retry storms

## Best Practices

### 1. Configure Retry for Your Use Case

**For local file loading:**
```rust
// Shorter delays, fewer retries
RetryConfig {
    max_retries: 2,
    initial_delay: Duration::from_millis(50),
    max_delay: Duration::from_millis(500),
    backoff_multiplier: 2.0,
}
```

**For network loading:**
```rust
// Longer delays, more retries
RetryConfig {
    max_retries: 5,
    initial_delay: Duration::from_millis(200),
    max_delay: Duration::from_secs(10),
    backoff_multiplier: 2.0,
}
```

### 2. Always Register Fallbacks

```rust
// Register fallbacks for critical asset types
asset_server.register_fallback(Texture::error_texture());
asset_server.register_fallback(Mesh::error_mesh());
asset_server.register_fallback(Material::error_material());
```

### 3. Use Placeholders for Better UX

```rust
// Show loading placeholders for better user experience
asset_server.register_placeholder(Texture::loading_texture());
asset_server.register_placeholder(Mesh::loading_mesh());
```

### 4. Monitor Progress for Loading Screens

```rust
fn update_loading_screen(asset_server: &AssetServer) {
    let progress = asset_server.load_progress();
    let percentage = (progress.loaded as f32 / progress.total as f32) * 100.0;
    
    ui.label(format!("Loading: {:.1}%", percentage));
    ui.progress_bar(percentage / 100.0);
    
    if progress.failed > 0 {
        ui.label(format!("⚠ {} assets failed to load", progress.failed));
    }
}
```

## Testing

The retry logic is thoroughly tested:

- **Retry count verification**: Ensures correct number of attempts
- **Exponential backoff timing**: Verifies delays increase exponentially
- **Fallback behavior**: Tests fallback after max retries
- **Progress tracking**: Validates progress reporting accuracy
- **Transient error handling**: Tests various error conditions

Run tests:
```bash
cargo test --test retry_logic_test --package luminara_asset
```

## Troubleshooting

### Assets Not Retrying

**Problem**: Assets fail immediately without retrying

**Solution**: Check that the error is classified as transient. Only specific I/O errors trigger retries.

### Excessive Retry Delays

**Problem**: Assets take too long to load due to retry delays

**Solution**: Reduce `initial_delay` and `max_delay` in retry config:
```rust
RetryConfig {
    initial_delay: Duration::from_millis(50),
    max_delay: Duration::from_millis(500),
    ..Default::default()
}
```

### Too Many Failed Assets

**Problem**: Many assets failing after retries

**Solution**: 
1. Check file permissions and paths
2. Increase `max_retries` if errors are truly transient
3. Investigate root cause of failures (disk issues, network problems)

## Future Enhancements

Potential improvements for future versions:

1. **Adaptive retry delays**: Adjust delays based on error patterns
2. **Per-asset-type retry configs**: Different configs for textures vs. meshes
3. **Retry statistics**: Track retry success rates and patterns
4. **Circuit breaker**: Temporarily stop retrying if too many failures
5. **Custom retry predicates**: Allow users to define custom retry conditions
