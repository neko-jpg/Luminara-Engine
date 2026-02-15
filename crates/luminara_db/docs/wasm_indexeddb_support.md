# WASM/IndexedDB Support

This document describes how to use Luminara Database in WASM environments with IndexedDB as the storage backend.

## Overview

When compiled for the `wasm32` target, Luminara Database can use the browser's IndexedDB API as its storage backend. This enables persistent storage in web applications without requiring a server-side database.

## Features

- **Persistent Storage**: Data persists across browser sessions
- **Same API**: Identical API to native backends (RocksDB, in-memory)
- **Async Operations**: All operations are async and non-blocking
- **Graph Queries**: Full SurrealQL support for complex queries
- **Concurrent Access**: Safe concurrent access from multiple parts of your application

## Building for WASM

### Prerequisites

Install the WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

Install wasm-pack for building and testing:

```bash
cargo install wasm-pack
```

### Building

Build the crate with the `wasm` feature:

```bash
wasm-pack build --target web --features wasm
```

Or add it to your Cargo.toml:

```toml
[dependencies]
luminara_db = { version = "0.1", features = ["wasm"] }
```

## Usage

### Initialization

```rust
use luminara_db::LuminaraDatabase;

#[cfg(target_arch = "wasm32")]
async fn init_database() -> Result<LuminaraDatabase, Box<dyn std::error::Error>> {
    // Initialize with IndexedDB backend
    // The database name will be used as the IndexedDB database name
    let db = LuminaraDatabase::new_indexeddb("my_game_db").await?;
    Ok(db)
}
```

### Basic Operations

All operations work identically to native backends:

```rust
use luminara_db::EntityRecord;

async fn example(db: &LuminaraDatabase) -> Result<(), Box<dyn std::error::Error>> {
    // Create an entity
    let entity = EntityRecord::new(Some("Player".to_string()))
        .with_tag("player");
    
    let entity_id = db.store_entity(entity).await?;
    
    // Load the entity
    let loaded = db.load_entity(&entity_id).await?;
    
    // Query entities
    let players = db.query_entities(
        "SELECT * FROM entity WHERE 'player' IN tags"
    ).await?;
    
    Ok(())
}
```

### Persistence

Data stored in IndexedDB persists across browser sessions:

```rust
// First session
{
    let db = LuminaraDatabase::new_indexeddb("my_game").await?;
    let entity = EntityRecord::new(Some("SavedEntity".to_string()));
    db.store_entity(entity).await?;
}

// Later session (after page reload)
{
    let db = LuminaraDatabase::new_indexeddb("my_game").await?;
    // The entity is still there!
    let entities = db.query_entities("SELECT * FROM entity").await?;
    assert_eq!(entities.len(), 1);
}
```

## Testing

### Running WASM Tests

Use wasm-pack to run tests in a headless browser:

```bash
wasm-pack test --headless --chrome --features wasm
```

Or in Firefox:

```bash
wasm-pack test --headless --firefox --features wasm
```

### Test in Real Browser

For interactive testing:

```bash
wasm-pack test --chrome --features wasm
```

This will open a browser window where you can see test results and use the browser's developer tools.

## Browser Compatibility

IndexedDB is supported in all modern browsers:

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support (iOS 10+)
- Opera: ✅ Full support

## Storage Limits

IndexedDB storage limits vary by browser:

- **Chrome/Edge**: ~60% of available disk space
- **Firefox**: ~50% of available disk space (with user prompt for more)
- **Safari**: ~1GB (with user prompt for more)

For most game applications, these limits are sufficient. Monitor storage usage with:

```rust
let stats = db.get_statistics().await?;
println!("Entities: {}", stats.entity_count);
println!("Components: {}", stats.component_count);
println!("Assets: {}", stats.asset_count);
```

## Performance Considerations

### Async Operations

All database operations are async. Use them with async/await:

```rust
// Good: Await operations
let entity = db.load_entity(&id).await?;

// Bad: Don't block
// let entity = block_on(db.load_entity(&id))?; // Don't do this in WASM!
```

### Batching

For better performance, batch operations when possible:

```rust
// Store multiple entities
let mut ids = Vec::new();
for i in 0..100 {
    let entity = EntityRecord::new(Some(format!("Entity{}", i)));
    let id = db.store_entity(entity).await?;
    ids.push(id);
}
```

### Query Optimization

Use indexes and efficient queries:

```rust
// Good: Use indexed fields
let entities = db.query_entities(
    "SELECT * FROM entity WHERE 'player' IN tags"
).await?;

// Less efficient: Full table scan
let entities = db.query_entities(
    "SELECT * FROM entity WHERE name CONTAINS 'Player'"
).await?;
```

## Debugging

### Browser DevTools

Use browser developer tools to inspect IndexedDB:

1. Open DevTools (F12)
2. Go to "Application" tab (Chrome) or "Storage" tab (Firefox)
3. Expand "IndexedDB"
4. Find your database (e.g., "my_game_db")

You can inspect tables, view records, and even run queries directly.

### Logging

Enable logging to see database operations:

```rust
// In your WASM initialization
console_log::init_with_level(log::Level::Debug).unwrap();
```

## Limitations

### No File System Access

IndexedDB is not a file system. You cannot:
- Store files directly (use Blob storage instead)
- Access files outside the browser sandbox
- Share data between different origins

### Single Origin

Each origin (domain) has its own IndexedDB storage. Data cannot be shared between:
- Different domains
- Different protocols (http vs https)
- Different ports

### Quota Management

Browsers may clear IndexedDB data if:
- Storage quota is exceeded
- User clears browser data
- Browser is in private/incognito mode

Always handle potential data loss gracefully.

## Migration from Native

If you're migrating from a native backend (RocksDB) to WASM/IndexedDB:

1. Export data from native backend
2. Serialize to JSON or RON
3. Import into IndexedDB on first load

Example:

```rust
#[cfg(not(target_arch = "wasm32"))]
async fn export_data(db: &LuminaraDatabase) -> Result<String, Box<dyn std::error::Error>> {
    let entities = db.query_entities("SELECT * FROM entity").await?;
    Ok(serde_json::to_string(&entities)?)
}

#[cfg(target_arch = "wasm32")]
async fn import_data(db: &LuminaraDatabase, json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entities: Vec<EntityRecord> = serde_json::from_str(json)?;
    for entity in entities {
        db.store_entity(entity).await?;
    }
    Ok(())
}
```

## Best Practices

1. **Use Meaningful Database Names**: Choose descriptive names for your IndexedDB databases
2. **Handle Errors Gracefully**: Network issues and quota errors can occur
3. **Implement Retry Logic**: Transient errors may require retries
4. **Monitor Storage Usage**: Check available space before large operations
5. **Provide User Feedback**: Show loading states during async operations
6. **Test Across Browsers**: Behavior may vary slightly between browsers
7. **Clear Old Data**: Implement data cleanup for old/unused records

## Example: Complete WASM Application

```rust
use luminara_db::{LuminaraDatabase, EntityRecord};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameDatabase {
    db: LuminaraDatabase,
}

#[wasm_bindgen]
impl GameDatabase {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<GameDatabase, JsValue> {
        // Initialize logging
        console_log::init_with_level(log::Level::Info)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Initialize database
        let db = LuminaraDatabase::new_indexeddb("my_game")
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(GameDatabase { db })
    }
    
    #[wasm_bindgen]
    pub async fn create_player(&self, name: String) -> Result<String, JsValue> {
        let entity = EntityRecord::new(Some(name))
            .with_tag("player");
        
        let id = self.db.store_entity(entity)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(id.to_string())
    }
    
    #[wasm_bindgen]
    pub async fn get_player_count(&self) -> Result<i64, JsValue> {
        let stats = self.db.get_statistics()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(stats.entity_count)
    }
}
```

## Troubleshooting

### "QuotaExceededError"

**Problem**: Browser storage quota exceeded.

**Solution**:
- Implement data cleanup
- Ask user to clear browser data
- Reduce data storage requirements

### "VersionError"

**Problem**: Database schema version mismatch.

**Solution**:
- Clear IndexedDB and reinitialize
- Implement migration logic
- Use versioned database names

### Slow Performance

**Problem**: Operations are slow.

**Solution**:
- Batch operations
- Use indexes
- Optimize queries
- Reduce data size

## Further Reading

- [IndexedDB API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API)
- [SurrealDB WASM Documentation](https://surrealdb.com/docs/integration/sdks/rust)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
