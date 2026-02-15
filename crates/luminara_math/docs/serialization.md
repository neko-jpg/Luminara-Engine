# Serialization and Version Migration

This document explains how to use the serialization and version migration features in Luminara Math.

## Overview

Luminara Math provides robust serialization support for all core types (Vec3, Quat, Transform, Color) with:
- **Multiple formats**: Human-readable RON and compact binary formats
- **Validation**: Automatic validation during deserialization with clear error messages
- **Version migration**: Support for loading older data formats and migrating them to the current version

## Basic Serialization

### RON Format (Human-Readable)

```rust
use luminara_math::{Transform, Vec3, Quat};

// Create a transform
let transform = Transform {
    translation: Vec3::new(1.0, 2.0, 3.0),
    rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
    scale: Vec3::ONE,
};

// Serialize to RON
let ron_str = ron::to_string(&transform).expect("Failed to serialize");
println!("{}", ron_str);

// Deserialize from RON
let loaded: Transform = ron::from_str(&ron_str).expect("Failed to deserialize");
```

### Binary Format (Compact)

```rust
use luminara_math::Transform;

// Serialize to binary
let binary = bincode::serialize(&transform).expect("Failed to serialize");

// Deserialize from binary
let loaded: Transform = bincode::deserialize(&binary).expect("Failed to deserialize");
```

## Validation

All core types implement validation that runs during deserialization:

```rust
use luminara_math::validation::{Validate, from_ron_validated};

// This will fail validation because the quaternion is not normalized
let invalid_ron = r#"
Transform(
    translation: (1.0, 2.0, 3.0),
    rotation: (0.5, 0.5, 0.5, 0.5),  // Not normalized!
    scale: (1.0, 1.0, 1.0),
)
"#;

let result = from_ron_validated::<Transform>(invalid_ron);
assert!(result.is_err());
// Error message will include:
// - What's wrong (quaternion not normalized)
// - How to fix it (normalize the quaternion)
```

## Version Migration

### Using Versioned Serialization

To ensure forward compatibility, use the versioned serialization functions:

```rust
use luminara_math::migration::{to_ron_versioned, from_ron_versioned};
use luminara_math::Vec3;

let vec = Vec3::new(1.0, 2.0, 3.0);

// Serialize with version information
let ron_str = to_ron_versioned(&vec).expect("Failed to serialize");
// Output: (version: 1, data: (1.0, 2.0, 3.0))

// Deserialize with automatic migration
let loaded = from_ron_versioned::<Vec3>(&ron_str).expect("Failed to load");
```

### Binary Format with Version

```rust
use luminara_math::migration::{to_binary_versioned, from_binary_versioned};
use luminara_math::Transform;

// Serialize with version (version stored as first 4 bytes)
let binary = to_binary_versioned(&transform).expect("Failed to serialize");

// Deserialize with automatic migration
let loaded = from_binary_versioned::<Transform>(&binary).expect("Failed to load");
```

### Legacy Format Support

The migration system can load data without version information (assumes version 1):

```rust
use luminara_math::migration::from_ron_versioned;
use luminara_math::Vec3;

// Legacy data without version field
let legacy_ron = "(1.0, 2.0, 3.0)";

// Still loads correctly (assumes version 1)
let vec = from_ron_versioned::<Vec3>(legacy_ron).expect("Failed to load");
```

### Future Version Handling

If you try to load data from a newer version, you'll get a clear error:

```rust
// Data from version 2 (future version)
let future_data = "(version: 2, data: (1.0, 2.0, 3.0))";

let result = from_ron_versioned::<Vec3>(future_data);
assert!(result.is_err());
// Error: "Data is from a newer version (v2) than current (v1). Please upgrade the engine."
```

## Batch Migration

For migrating multiple files at once:

```rust
use luminara_math::migration::migrate_ron_files;
use luminara_math::Transform;

let files = vec![
    ("scene1.ron".to_string(), scene1_content),
    ("scene2.ron".to_string(), scene2_content),
    ("scene3.ron".to_string(), scene3_content),
];

let result = migrate_ron_files::<Transform>(&files);

println!("Migration complete:");
println!("  Total: {}", result.total);
println!("  Succeeded: {}", result.succeeded);
println!("  Failed: {}", result.failed);
println!("  Success Rate: {:.1}%", result.success_rate() * 100.0);

// Check for errors
for (filename, error) in &result.errors {
    println!("  {}: {}", filename, error);
}
```

## Implementing Migration for Custom Types

To add migration support to your own types:

```rust
use luminara_math::migration::{Migratable, MigrationError, MigrationErrorKind, CURRENT_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyCustomType {
    pub value: f32,
}

impl Migratable for MyCustomType {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Version 1: Current format
        if from_version == 1 {
            return serde_json::from_value(data).map_err(|e| MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::DataCorruption {
                    reason: format!("Failed to deserialize: {}", e),
                },
            });
        }

        // If you add version 2 in the future, add migration logic here:
        // if from_version == 1 {
        //     // Migrate from v1 to v2
        //     let old_data: OldFormat = serde_json::from_value(data)?;
        //     let new_data = MyCustomType {
        //         value: old_data.old_value * 2.0,  // Example migration
        //     };
        //     return Ok(new_data);
        // }

        Err(MigrationError {
            from_version,
            to_version: CURRENT_VERSION,
            kind: MigrationErrorKind::UnsupportedVersion {
                reason: format!("Version {} is not supported", from_version),
            },
        })
    }
}
```

## Best Practices

1. **Always use versioned serialization for persistent data** (scenes, assets, save files)
2. **Use validation for all deserialized data** to catch corruption early
3. **Test migration paths** when adding new versions
4. **Document breaking changes** in serialization format
5. **Keep old migration code** to support loading legacy data
6. **Use RON for human-editable files** (scenes, configs)
7. **Use binary for performance-critical data** (large assets, network packets)

## Error Handling

All serialization functions return `Result` types with descriptive errors:

```rust
use luminara_math::migration::from_ron_versioned;
use luminara_math::Transform;

match from_ron_versioned::<Transform>(data) {
    Ok(transform) => {
        // Use the loaded transform
    }
    Err(e) => {
        eprintln!("Failed to load transform: {}", e);
        // Error message includes:
        // - What went wrong
        // - Where it went wrong (if applicable)
        // - How to fix it (suggestions)
    }
}
```

## Current Version

The current serialization format version is defined in `luminara_math::migration::CURRENT_VERSION`.

As of this writing, the current version is **1**.

When you need to make breaking changes to the serialization format:
1. Increment `CURRENT_VERSION`
2. Add migration logic in the `Migratable` implementations
3. Add tests for the migration path
4. Document the changes in this file
