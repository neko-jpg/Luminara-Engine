//! Version Migration Demo
//!
//! This example demonstrates how to use the version migration system
//! to handle changes in data formats over time.
//!
//! Run with: cargo run --example version_migration_demo

use luminara_math::migration::{
    from_ron_versioned, migrate_ron_files, to_ron_versioned, BatchMigrationResult, Migratable,
    MigrationError, MigrationErrorKind, Versioned, CURRENT_VERSION,
};
use luminara_math::{Color, Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

fn main() {
    println!("=== Luminara Engine - Version Migration Demo ===\n");

    // Demo 1: Basic versioned serialization
    demo_basic_versioning();

    // Demo 2: Automatic migration
    demo_automatic_migration();

    // Demo 3: Custom type migration
    demo_custom_migration();

    // Demo 4: Batch migration
    demo_batch_migration();

    // Demo 5: Error handling
    demo_error_handling();

    println!("\n=== Demo Complete ===");
}

fn demo_basic_versioning() {
    println!("--- Demo 1: Basic Versioned Serialization ---");

    let transform = Transform {
        translation: Vec3::new(10.0, 20.0, 30.0),
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        scale: Vec3::splat(2.0),
    };

    // Wrap with version
    let versioned = Versioned::new(transform);
    println!("Version: {}", versioned.version());
    println!("Is current version: {}", versioned.is_current_version());

    // Serialize
    let ron = ron::ser::to_string_pretty(&versioned, ron::ser::PrettyConfig::default()).unwrap();
    println!("\nSerialized (with version):\n{}", ron);

    // Deserialize
    let loaded: Versioned<Transform> = ron::from_str(&ron).unwrap();
    println!("\nDeserialized version: {}", loaded.version());
    println!("Translation: {:?}", loaded.data.translation);

    println!();
}

fn demo_automatic_migration() {
    println!("--- Demo 2: Automatic Migration ---");

    let color = Color::rgba(0.8, 0.6, 0.4, 0.9);

    // Save with version
    let ron = to_ron_versioned(&color).unwrap();
    println!("Saved with version {}", CURRENT_VERSION);

    // Load with automatic migration
    let loaded = from_ron_versioned::<Color>(&ron).unwrap();
    println!("Loaded successfully: {:?}", loaded);

    // Simulate loading legacy data (no version field)
    let legacy_ron = "(r: 1.0, g: 0.5, b: 0.0, a: 1.0)";
    println!("\nLoading legacy data (no version field)...");
    let legacy_loaded = from_ron_versioned::<Color>(legacy_ron).unwrap();
    println!("Legacy data loaded successfully: {:?}", legacy_loaded);

    println!();
}

fn demo_custom_migration() {
    println!("--- Demo 3: Custom Type Migration ---");

    // Define a custom component with migration support
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct PhysicsComponent {
        position: Vec3,
        velocity: Vec3,
        mass: f32,
    }

    impl Migratable for PhysicsComponent {
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

            match from_version {
                1 => {
                    // Current version
                    serde_json::from_value(data).map_err(|e| MigrationError {
                        from_version,
                        to_version: CURRENT_VERSION,
                        kind: MigrationErrorKind::DataCorruption {
                            reason: format!("Failed to deserialize: {}", e),
                        },
                    })
                }
                0 => {
                    // Migrate from version 0 (had different field names)
                    #[derive(Deserialize)]
                    struct V0 {
                        pos: Vec3,
                        vel: Vec3,
                        // mass didn't exist in v0
                    }

                    let v0: V0 = serde_json::from_value(data).map_err(|e| MigrationError {
                        from_version,
                        to_version: CURRENT_VERSION,
                        kind: MigrationErrorKind::DataCorruption {
                            reason: format!("Failed to deserialize v0: {}", e),
                        },
                    })?;

                    Ok(PhysicsComponent {
                        position: v0.pos,
                        velocity: v0.vel,
                        mass: 1.0, // Default value for new field
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
            0
        }
    }

    // Create and save current version
    let component = PhysicsComponent {
        position: Vec3::new(1.0, 2.0, 3.0),
        velocity: Vec3::new(0.5, 0.0, -0.5),
        mass: 2.5,
    };

    let ron = to_ron_versioned(&component).unwrap();
    println!("Saved PhysicsComponent (v{}):", CURRENT_VERSION);
    println!("{}", ron);

    // Load current version
    let loaded = from_ron_versioned::<PhysicsComponent>(&ron).unwrap();
    println!("\nLoaded: {:?}", loaded);

    // Simulate loading v0 data
    println!("\nSimulating v0 data migration...");
    let v0_json = serde_json::json!({
        "pos": [5.0, 10.0, 15.0],
        "vel": [1.0, 0.0, -1.0]
    });

    let migrated = PhysicsComponent::migrate(0, v0_json).unwrap();
    println!("Migrated from v0: {:?}", migrated);
    println!("Note: mass defaulted to 1.0 (didn't exist in v0)");

    println!();
}

fn demo_batch_migration() {
    println!("--- Demo 4: Batch Migration ---");

    // Create test data
    let vec1 = Vec3::new(1.0, 2.0, 3.0);
    let vec2 = Vec3::new(4.0, 5.0, 6.0);
    let vec3 = Vec3::new(7.0, 8.0, 9.0);

    let ron1 = to_ron_versioned(&vec1).unwrap();
    let ron2 = to_ron_versioned(&vec2).unwrap();
    let ron3 = to_ron_versioned(&vec3).unwrap();

    // Prepare files
    let files = vec![
        ("vec1.ron".to_string(), ron1),
        ("vec2.ron".to_string(), ron2),
        ("vec3.ron".to_string(), ron3),
    ];

    println!("Migrating {} files...", files.len());

    // Migrate all files
    let result: BatchMigrationResult = migrate_ron_files::<Vec3>(&files);

    println!("\nMigration Results:");
    println!("  Total: {}", result.total);
    println!("  Succeeded: {}", result.succeeded);
    println!("  Failed: {}", result.failed);
    println!("  Success Rate: {:.1}%", result.success_rate() * 100.0);

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for (name, error) in &result.errors {
            println!("  {}: {}", name, error);
        }
    }

    println!();
}

fn demo_error_handling() {
    println!("--- Demo 5: Error Handling ---");

    // Test 1: Future version
    println!("Test 1: Loading data from future version");
    let future_version = CURRENT_VERSION + 1;
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::with_version(future_version, vec);
    let ron = ron::to_string(&versioned).unwrap();

    match from_ron_versioned::<Vec3>(&ron) {
        Ok(_) => println!("  ❌ Should have failed!"),
        Err(e) => println!("  ✓ Error (expected): {}", e),
    }

    // Test 2: Corrupted data
    println!("\nTest 2: Loading corrupted data");
    let corrupted_ron = "(version: 1, data: (x: \"not_a_number\", y: 2.0, z: 3.0))";

    match from_ron_versioned::<Vec3>(corrupted_ron) {
        Ok(_) => println!("  ❌ Should have failed!"),
        Err(e) => println!("  ✓ Error (expected): {}", e),
    }

    // Test 3: Invalid binary
    println!("\nTest 3: Loading invalid binary data");
    let invalid_binary = vec![1, 2, 3]; // Too short

    match luminara_math::migration::from_binary_versioned::<Vec3>(&invalid_binary) {
        Ok(_) => println!("  ❌ Should have failed!"),
        Err(e) => println!("  ✓ Error (expected): {}", e),
    }

    println!();
}

