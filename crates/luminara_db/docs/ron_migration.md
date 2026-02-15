# RON Scene Migration Tool

This document describes how to migrate existing RON scene files to the database format.

## Overview

The RON Migration Tool converts scene files from the RON (Rusty Object Notation) format to the database format, preserving all entity relationships, components, and metadata.

## Features

- **Complete Migration**: Migrates entities, components, and relationships
- **Hierarchy Preservation**: Maintains parent-child relationships
- **Batch Processing**: Migrate multiple files at once
- **Verification**: Validates migration integrity
- **Statistics**: Provides detailed migration metrics

## Usage

### Basic Migration

```rust
use luminara_db::{LuminaraDatabase, RonMigrationTool};

async fn migrate_scene() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = LuminaraDatabase::new_memory().await?;
    
    // Create migration tool
    let tool = RonMigrationTool::new(db);
    
    // Migrate a scene file
    let stats = tool.migrate_file("assets/scenes/my_scene.scene.ron").await?;
    
    println!("Migrated {} entities", stats.entities_migrated);
    println!("Migrated {} components", stats.components_migrated);
    println!("Preserved {} relationships", stats.relationships_preserved);
    
    Ok(())
}
```

### Batch Migration

Migrate multiple scene files at once:

```rust
use luminara_db::{LuminaraDatabase, RonMigrationTool};

async fn migrate_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let db = LuminaraDatabase::new_memory().await?;
    let tool = RonMigrationTool::new(db);
    
    let files = vec![
        "assets/scenes/scene1.scene.ron",
        "assets/scenes/scene2.scene.ron",
        "assets/scenes/scene3.scene.ron",
    ];
    
    let stats = tool.migrate_batch(&files).await?;
    
    println!("Total entities: {}", stats.entities_migrated);
    println!("Total time: {}ms", stats.duration_ms);
    
    Ok(())
}
```

### Migration Verification

Verify that migration completed successfully:

```rust
use luminara_db::{LuminaraDatabase, RonMigrationTool, migration::Scene};

async fn verify() -> Result<(), Box<dyn std::error::Error>> {
    let db = LuminaraDatabase::new_memory().await?;
    let tool = RonMigrationTool::new(db);
    
    // Migrate scene
    tool.migrate_file("assets/scenes/my_scene.scene.ron").await?;
    
    // Load original scene for verification
    let content = std::fs::read_to_string("assets/scenes/my_scene.scene.ron")?;
    let scene: Scene = ron::from_str(&content)?;
    
    // Verify migration
    let is_valid = tool.verify_migration(&scene).await?;
    
    if is_valid {
        println!("✓ Migration verified successfully!");
    } else {
        println!("✗ Migration verification failed!");
    }
    
    Ok(())
}
```

## Command Line Tool

Use the provided example as a command-line tool:

```bash
# Migrate a single scene
cargo run --example migrate_ron_scene -- assets/scenes/phase1_demo.scene.ron

# Migrate with custom database path
cargo run --example migrate_ron_scene -- assets/scenes/my_scene.scene.ron data/migrated.db
```

## RON Scene Format

The migration tool expects RON files in this format:

```ron
Scene(
    meta: SceneMeta(
        name: "My Scene",
        description: "A test scene",
        version: "1.0.0",
        tags: ["demo", "test"],
    ),
    entities: [
        EntityData(
            name: "Camera",
            id: Some(1),
            parent: None,
            components: {
                "Transform": {
                    "translation": [0.0, 3.0, 8.0],
                    "rotation": [0.0, 0.0, 0.0, 1.0],
                    "scale": [1.0, 1.0, 1.0]
                },
                "Camera": {
                    "projection": {
                        "Perspective": {
                            "fov": 60.0,
                            "near": 0.1,
                            "far": 1000.0
                        }
                    },
                    "is_active": true
                }
            },
            children: [],
            tags: ["camera"],
        ),
    ],
)
```

## Migration Process

The migration happens in two passes:

### Pass 1: Entity and Component Creation

1. Parse RON file
2. Create entity records in database
3. Create component records for each entity
4. Map old entity IDs to new RecordIds
5. Process children recursively

### Pass 2: Relationship Establishment

1. Load entities from database
2. Set parent references using ID map
3. Set children references using ID map
4. Update entities with relationships
5. Process children recursively

This two-pass approach ensures all entities exist before establishing relationships.

## Migration Statistics

The tool provides detailed statistics:

```rust
pub struct MigrationStatistics {
    /// Number of entities migrated
    pub entities_migrated: usize,
    /// Number of components migrated
    pub components_migrated: usize,
    /// Number of relationships preserved
    pub relationships_preserved: usize,
    /// Migration duration in milliseconds
    pub duration_ms: u64,
}
```

## Verification

The verification process checks:

1. **Entity Count**: Ensures all entities were migrated
2. **Entity Names**: Verifies entities can be found by name
3. **Tags**: Checks all tags are preserved
4. **Components**: Validates component count matches
5. **Hierarchy**: Recursively verifies children

## Error Handling

The migration tool handles various error cases:

- **File Not Found**: Returns error if RON file doesn't exist
- **Parse Error**: Returns error if RON syntax is invalid
- **Database Error**: Returns error if database operations fail
- **Missing References**: Logs warnings for broken relationships

## Performance Considerations

### Large Scenes

For scenes with many entities:

```rust
// Use batch processing for better performance
let tool = RonMigrationTool::new(db);

// Process in chunks if needed
let chunk_size = 100;
for chunk in scene.entities.chunks(chunk_size) {
    // Process chunk
}
```

### Memory Usage

The migration tool loads the entire scene into memory. For very large scenes:

1. Split into multiple smaller scene files
2. Migrate each file separately
3. Use batch migration for efficiency

### Database Backend

Choose appropriate backend for your use case:

```rust
// In-memory: Fast but not persistent
let db = LuminaraDatabase::new_memory().await?;

// RocksDB: Persistent and fast (recommended for production)
// let db = LuminaraDatabase::new("data/luminara.db").await?;

// WASM/IndexedDB: For browser environments
#[cfg(target_arch = "wasm32")]
let db = LuminaraDatabase::new_indexeddb("luminara_db").await?;
```

## Best Practices

1. **Backup Original Files**: Keep RON files as backup
2. **Verify Migration**: Always run verification after migration
3. **Test First**: Test migration on a copy before production
4. **Monitor Statistics**: Check statistics for anomalies
5. **Handle Errors**: Implement proper error handling
6. **Log Progress**: Use logging for large migrations

## Example: Complete Migration Workflow

```rust
use luminara_db::{LuminaraDatabase, RonMigrationTool, migration::Scene};
use std::path::Path;

async fn complete_migration_workflow(
    scene_path: impl AsRef<Path>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting migration workflow...");
    
    // 1. Initialize database
    println!("1. Initializing database...");
    let db = LuminaraDatabase::new_memory().await?;
    
    // 2. Create migration tool
    println!("2. Creating migration tool...");
    let tool = RonMigrationTool::new(db.clone());
    
    // 3. Migrate scene
    println!("3. Migrating scene...");
    let stats = tool.migrate_file(&scene_path).await?;
    
    println!("   - Entities: {}", stats.entities_migrated);
    println!("   - Components: {}", stats.components_migrated);
    println!("   - Relationships: {}", stats.relationships_preserved);
    println!("   - Duration: {}ms", stats.duration_ms);
    
    // 4. Verify migration
    println!("4. Verifying migration...");
    let content = std::fs::read_to_string(&scene_path)?;
    let scene: Scene = ron::from_str(&content)?;
    
    if !tool.verify_migration(&scene).await? {
        return Err("Migration verification failed".into());
    }
    println!("   ✓ Verification passed");
    
    // 5. Query migrated data
    println!("5. Querying migrated data...");
    let entities = db.query_entities("SELECT * FROM entity").await?;
    println!("   - Found {} entities in database", entities.len());
    
    // 6. Show database statistics
    println!("6. Database statistics:");
    let db_stats = db.get_statistics().await?;
    println!("   - Entities: {}", db_stats.entity_count);
    println!("   - Components: {}", db_stats.component_count);
    
    println!("\n✓ Migration workflow completed successfully!");
    
    Ok(())
}
```

## Troubleshooting

### "Failed to parse RON"

**Problem**: RON syntax error in scene file.

**Solution**:
- Validate RON syntax
- Check for missing commas or brackets
- Use a RON validator tool

### "Entity ID not found in map"

**Problem**: Broken parent-child reference.

**Solution**:
- Ensure all referenced entity IDs exist
- Check for circular references
- Verify parent IDs are valid

### "Migration verification failed"

**Problem**: Data mismatch after migration.

**Solution**:
- Check logs for specific failures
- Verify component data is valid JSON
- Ensure all relationships are correct

### Slow Migration

**Problem**: Migration takes too long.

**Solution**:
- Use batch processing
- Optimize database backend
- Split large scenes into smaller files

## Integration with Editor

The migration tool can be integrated into the editor:

```rust
// In editor code
use luminara_db::{LuminaraDatabase, RonMigrationTool};

async fn import_scene_to_editor(
    scene_path: &str,
    db: &LuminaraDatabase
) -> Result<(), Box<dyn std::error::Error>> {
    let tool = RonMigrationTool::new(db.clone());
    
    // Migrate scene
    let stats = tool.migrate_file(scene_path).await?;
    
    // Show progress to user
    println!("Imported {} entities", stats.entities_migrated);
    
    // Load entities into editor
    let entities = db.query_entities("SELECT * FROM entity").await?;
    
    // Spawn entities into world
    for entity in entities {
        // Spawn entity in editor...
    }
    
    Ok(())
}
```

## Future Enhancements

Planned improvements:

- **Incremental Migration**: Update only changed entities
- **Conflict Resolution**: Handle duplicate entities
- **Asset Migration**: Migrate asset references
- **Progress Callbacks**: Report migration progress
- **Parallel Processing**: Migrate multiple scenes in parallel
- **Compression**: Compress large component data

## See Also

- [Database Documentation](../README.md)
- [Entity Storage](entity_storage.md)
- [Asset Dependency Tracking](asset_dependency_tracking.md)
- [WASM Support](wasm_indexeddb_support.md)
