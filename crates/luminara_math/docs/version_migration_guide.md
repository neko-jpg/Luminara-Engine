# Version Migration Guide

## Overview

This guide explains how to use the version migration system in Luminara Engine to handle changes in serialization formats over time.

**Validates: Requirements 8.3**

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Usage](#basic-usage)
3. [Implementing Migration](#implementing-migration)
4. [Migration Tools](#migration-tools)
5. [Best Practices](#best-practices)
6. [Examples](#examples)
7. [Troubleshooting](#troubleshooting)

## Introduction

### Why Version Migration?

As the engine evolves, data structures change:
- Fields are renamed or removed
- New fields are added
- Data types change
- Relationships are restructured

Version migration ensures that old save files, scenes, and assets continue to work with newer versions of the engine.

### How It Works

1. **Version Tracking**: All serialized data includes a version number
2. **Automatic Detection**: The system detects the version when loading
3. **Migration Path**: Old data is automatically upgraded to the current format
4. **Error Handling**: Clear errors for corrupted or future-version data

## Basic Usage

### Saving with Version

```rust
use luminara_math::migration::{to_ron_versioned, to_binary_versioned};
use luminara_math::Transform;

let transform = Transform::default();

// Save as RON with version
let ron = to_ron_versioned(&transform)?;
std::fs::write("transform.ron", ron)?;

// Save as binary with version
let binary = to_binary_versioned(&transform)?;
std::fs::write("transform.bin", binary)?;
```

### Loading with Migration

```rust
use luminara_math::migration::{from_ron_versioned, from_binary_versioned};
use luminara_math::Transform;

// Load RON (automatically migrates if needed)
let ron = std::fs::read_to_string("transform.ron")?;
let transform = from_ron_versioned::<Transform>(&ron)?;

// Load binary (automatically migrates if needed)
let binary = std::fs::read("transform.bin")?;
let transform = from_binary_versioned::<Transform>(&binary)?;
```

### Manual Version Wrapping

```rust
use luminara_math::migration::{Versioned, CURRENT_VERSION};

let transform = Transform::default();

// Wrap with current version
let versioned = Versioned::new(transform);
assert_eq!(versioned.version(), CURRENT_VERSION);

// Serialize
let ron = ron::to_string(&versioned)?;

// Deserialize
let loaded: Versioned<Transform> = ron::from_str(&ron)?;
assert_eq!(loaded.version(), CURRENT_VERSION);
let transform = loaded.into_inner();
```

## Implementing Migration

### Step 1: Define Migration Logic

Implement the `Migratable` trait for your type:

```rust
use luminara_math::migration::{Migratable, MigrationError, MigrationErrorKind, CURRENT_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MyComponent {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
}

impl Migratable for MyComponent {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        // Handle future versions
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Handle each version
        match from_version {
            1 => {
                // Version 1 is current format
                serde_json::from_value(data).map_err(|e| MigrationError {
                    from_version,
                    to_version: CURRENT_VERSION,
                    kind: MigrationErrorKind::DataCorruption {
                        reason: format!("Failed to deserialize: {}", e),
                    },
                })
            }
            0 => {
                // Migrate from version 0
                #[derive(Deserialize)]
                struct V0 {
                    pos: Vec3,  // Old field name
                    vel: Vec3,  // Old field name
                    // mass didn't exist in v0
                }

                let v0: V0 = serde_json::from_value(data).map_err(|e| MigrationError {
                    from_version,
                    to_version: CURRENT_VERSION,
                    kind: MigrationErrorKind::DataCorruption {
                        reason: format!("Failed to deserialize v0: {}", e),
                    },
                })?;

                Ok(MyComponent {
                    position: v0.pos,
                    velocity: v0.vel,
                    mass: 1.0,  // Default value for new field
                })
            }
            _ => Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::UnsupportedVersion {
                    reason: format!("Version {} is too old", from_version),
                },
            }),
        }
    }

    fn min_supported_version() -> u32 {
        0  // We support migration from version 0
    }
}
```

### Step 2: Test Migration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use luminara_math::migration::{from_ron_versioned, to_ron_versioned};

    #[test]
    fn test_migration_from_v0() {
        // Create v0 data manually
        let v0_json = serde_json::json!({
            "pos": {"x": 1.0, "y": 2.0, "z": 3.0},
            "vel": {"x": 0.5, "y": 0.0, "z": -0.5}
        });

        // Migrate
        let migrated = MyComponent::migrate(0, v0_json).unwrap();

        // Verify
        assert_eq!(migrated.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(migrated.velocity, Vec3::new(0.5, 0.0, -0.5));
        assert_eq!(migrated.mass, 1.0);  // Default value
    }

    #[test]
    fn test_roundtrip_current_version() {
        let original = MyComponent {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::new(0.5, 0.0, -0.5),
            mass: 2.5,
        };

        // Serialize with version
        let ron = to_ron_versioned(&original).unwrap();

        // Deserialize with migration
        let loaded = from_ron_versioned::<MyComponent>(&ron).unwrap();

        // Verify
        assert_eq!(loaded.position, original.position);
        assert_eq!(loaded.velocity, original.velocity);
        assert_eq!(loaded.mass, original.mass);
    }
}
```

### Step 3: Document Migration

Add comments documenting the migration path:

```rust
/// MyComponent - Physics component
///
/// # Version History
///
/// ## Version 1 (Current)
/// - Fields: position, velocity, mass
/// - All fields are Vec3 or f32
///
/// ## Version 0 (Legacy)
/// - Fields: pos, vel (no mass)
/// - Migration: pos -> position, vel -> velocity, mass defaults to 1.0
///
/// # Migration Support
///
/// Supports migration from version 0 to current.
/// Minimum supported version: 0
```

## Migration Tools

### Batch Migration

Migrate multiple files at once:

```rust
use luminara_math::migration::{migrate_ron_files, BatchMigrationResult};

// Collect files
let files = vec![
    ("file1.ron".to_string(), std::fs::read_to_string("file1.ron")?),
    ("file2.ron".to_string(), std::fs::read_to_string("file2.ron")?),
    ("file3.ron".to_string(), std::fs::read_to_string("file3.ron")?),
];

// Migrate all
let result: BatchMigrationResult = migrate_ron_files::<MyComponent>(&files);

// Report results
println!("Migration Results:");
println!("  Total: {}", result.total);
println!("  Succeeded: {}", result.succeeded);
println!("  Failed: {}", result.failed);
println!("  Success Rate: {:.1}%", result.success_rate() * 100.0);

// Handle errors
if !result.errors.is_empty() {
    eprintln!("\nErrors:");
    for (name, error) in &result.errors {
        eprintln!("  {}: {}", name, error);
    }
}
```

### Scene Migration

For complete scenes, use the database migration tool:

```rust
use luminara_db::{LuminaraDatabase, migration::RonMigrationTool};

async fn migrate_scene(scene_path: &str) -> Result<()> {
    // Initialize database
    let db = LuminaraDatabase::new_memory().await?;
    
    // Create migration tool
    let tool = RonMigrationTool::new(db);
    
    // Migrate scene
    let stats = tool.migrate_file(scene_path).await?;
    
    println!("Scene Migration Results:");
    println!("  Entities: {}", stats.entities_migrated);
    println!("  Components: {}", stats.components_migrated);
    println!("  Relationships: {}", stats.relationships_preserved);
    println!("  Duration: {}ms", stats.duration_ms);
    
    Ok(())
}
```

See `crates/luminara_db/docs/ron_migration.md` for complete scene migration documentation.

## Best Practices

### 1. Always Version Persistent Data

```rust
// ✅ Good: Versioned
let ron = to_ron_versioned(&data)?;
std::fs::write("data.ron", ron)?;

// ❌ Bad: Unversioned (for persistent data)
let ron = ron::to_string(&data)?;
std::fs::write("data.ron", ron)?;
```

### 2. Keep Migration Paths Simple

```rust
// ✅ Good: Direct migration
match from_version {
    1 => deserialize_current(data),
    0 => migrate_v0_to_v1(data),
}

// ❌ Bad: Complex chain
match from_version {
    3 => deserialize_current(data),
    2 => migrate_v2_to_v3(data),
    1 => migrate_v1_to_v2(migrate_v2_to_v3(data)),
    0 => migrate_v0_to_v1(migrate_v1_to_v2(migrate_v2_to_v3(data))),
}
```

### 3. Test All Migration Paths

```rust
#[test]
fn test_all_versions() {
    // Test v0 -> current
    let v0_data = create_v0_test_data();
    let migrated = MyType::migrate(0, v0_data).unwrap();
    verify_migrated_data(&migrated);

    // Test v1 -> current
    let v1_data = create_v1_test_data();
    let migrated = MyType::migrate(1, v1_data).unwrap();
    verify_migrated_data(&migrated);

    // Test current version (no migration)
    let current_data = create_current_test_data();
    let migrated = MyType::migrate(CURRENT_VERSION, current_data).unwrap();
    verify_migrated_data(&migrated);
}
```

### 4. Keep Old Test Data

Store test data for each version:

```
tests/
  fixtures/
    v0_transform.ron
    v1_transform.ron
    v2_transform.ron
```

Use these to verify migration:

```rust
#[test]
fn test_migration_from_v0_fixture() {
    let v0_ron = include_str!("fixtures/v0_transform.ron");
    let migrated = from_ron_versioned::<Transform>(v0_ron).unwrap();
    // Verify migrated data
}
```

### 5. Document Breaking Changes

```rust
/// # Version History
///
/// ## Version 2 (Current)
/// - Added `parent` field (defaults to None)
/// - Changed `color` from RGB to RGBA (alpha defaults to 1.0)
///
/// ## Version 1
/// - Renamed `pos` to `position`
/// - Renamed `rot` to `rotation`
///
/// ## Version 0 (Legacy)
/// - Original format with `pos` and `rot` fields
```

### 6. Provide Clear Error Messages

```rust
Err(MigrationError {
    from_version,
    to_version: CURRENT_VERSION,
    kind: MigrationErrorKind::DataCorruption {
        reason: format!(
            "Failed to deserialize Transform: expected 'translation' field, \
             found '{}'. This may be a version 0 file that needs migration.",
            field_name
        ),
    },
})
```

## Examples

### Example 1: Adding a New Field

**Version 0:**
```rust
struct Player {
    name: String,
    health: f32,
}
```

**Version 1 (Current):**
```rust
struct Player {
    name: String,
    health: f32,
    max_health: f32,  // New field
}
```

**Migration:**
```rust
impl Migratable for Player {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        match from_version {
            1 => serde_json::from_value(data).map_err(|e| /* ... */),
            0 => {
                #[derive(Deserialize)]
                struct V0 {
                    name: String,
                    health: f32,
                }
                let v0: V0 = serde_json::from_value(data)?;
                Ok(Player {
                    name: v0.name,
                    health: v0.health,
                    max_health: 100.0,  // Default value
                })
            }
            _ => Err(/* ... */),
        }
    }
}
```

### Example 2: Renaming Fields

**Version 0:**
```rust
struct Transform {
    pos: Vec3,
    rot: Quat,
}
```

**Version 1 (Current):**
```rust
struct Transform {
    translation: Vec3,
    rotation: Quat,
}
```

**Migration:**
```rust
impl Migratable for Transform {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        match from_version {
            1 => serde_json::from_value(data).map_err(|e| /* ... */),
            0 => {
                #[derive(Deserialize)]
                struct V0 {
                    pos: Vec3,
                    rot: Quat,
                }
                let v0: V0 = serde_json::from_value(data)?;
                Ok(Transform {
                    translation: v0.pos,
                    rotation: v0.rot,
                })
            }
            _ => Err(/* ... */),
        }
    }
}
```

### Example 3: Changing Data Types

**Version 0:**
```rust
struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

**Version 1 (Current):**
```rust
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
```

**Migration:**
```rust
impl Migratable for Color {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        match from_version {
            1 => serde_json::from_value(data).map_err(|e| /* ... */),
            0 => {
                #[derive(Deserialize)]
                struct V0 {
                    r: u8,
                    g: u8,
                    b: u8,
                }
                let v0: V0 = serde_json::from_value(data)?;
                Ok(Color {
                    r: v0.r as f32 / 255.0,
                    g: v0.g as f32 / 255.0,
                    b: v0.b as f32 / 255.0,
                    a: 1.0,
                })
            }
            _ => Err(/* ... */),
        }
    }
}
```

## Troubleshooting

### "Data is from a newer version"

**Problem:** Trying to load data saved with a newer version of the engine.

**Solution:**
- Upgrade the engine to the version that saved the data
- Or export the data from the newer version in a compatible format

### "Failed to deserialize"

**Problem:** Data corruption or incompatible format.

**Solution:**
- Check the error message for specific field issues
- Verify the file is valid RON/binary
- Check if migration logic handles all fields correctly

### "Unsupported version"

**Problem:** Data is from a version too old to migrate.

**Solution:**
- Implement migration from that version
- Or use an intermediate version to migrate in steps

### Migration Produces Wrong Values

**Problem:** Migrated data doesn't match expected values.

**Solution:**
- Add debug logging to migration code
- Verify default values are correct
- Check field mappings are correct
- Test with known good data

### Slow Migration

**Problem:** Migrating large files takes too long.

**Solution:**
- Use batch migration for multiple files
- Consider binary format (faster than RON)
- Profile migration code to find bottlenecks

## See Also

- [Serialization Documentation](serialization.md)
- [RON Scene Migration](../../luminara_db/docs/ron_migration.md)
- [Requirements 8.3](.kiro/specs/pre-editor-engine-audit/requirements.md)
- [RON Specification](https://github.com/ron-rs/ron)

