// ============================================================================
// Version Migration Tests
// ============================================================================
//
// Tests for version migration support in serialization.
//
// **Validates: Requirements 8.3**

use luminara_math::migration::{
    from_binary_versioned, from_ron_versioned, to_binary_versioned, to_ron_versioned,
    BatchMigrationResult, Migratable, MigrationError, Versioned, CURRENT_VERSION,
};
use luminara_math::{Color, Quat, Transform, Vec3};

// ============================================================================
// Version Field Tests
// ============================================================================

#[test]
fn test_versioned_wrapper_creation() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::new(vec);

    assert_eq!(versioned.version(), CURRENT_VERSION);
    assert!(versioned.is_current_version());
    assert_eq!(versioned.data, vec);
}

#[test]
fn test_versioned_wrapper_with_specific_version() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::with_version(1, vec);

    assert_eq!(versioned.version(), 1);
    assert_eq!(versioned.data, vec);
}

#[test]
fn test_versioned_wrapper_into_inner() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::new(vec);
    let inner = versioned.into_inner();

    assert_eq!(inner, vec);
}

#[test]
fn test_versioned_serialization_ron() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::new(vec);

    // Serialize
    let ron_str = ron::to_string(&versioned).expect("Failed to serialize");
    println!("Versioned RON: {}", ron_str);

    // Should contain version field
    assert!(ron_str.contains("version"));

    // Deserialize
    let deserialized: Versioned<Vec3> =
        ron::from_str(&ron_str).expect("Failed to deserialize");

    assert_eq!(deserialized.version(), CURRENT_VERSION);
    assert_eq!(deserialized.data, vec);
}

#[test]
fn test_versioned_serialization_binary() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let versioned = Versioned::new(vec);

    // Serialize
    let binary = bincode::serialize(&versioned).expect("Failed to serialize");

    // Deserialize
    let deserialized: Versioned<Vec3> =
        bincode::deserialize(&binary).expect("Failed to deserialize");

    assert_eq!(deserialized.version(), CURRENT_VERSION);
    assert_eq!(deserialized.data, vec);
}

// ============================================================================
// Migration Tests - Current Version
// ============================================================================

#[test]
fn test_migrate_vec3_current_version() {
    let vec = Vec3::new(1.0, 2.0, 3.0);

    // Serialize with version
    let ron_str = to_ron_versioned(&vec).expect("Failed to serialize");
    println!("Versioned Vec3: {}", ron_str);

    // Deserialize and migrate (should be no-op for current version)
    let migrated = from_ron_versioned::<Vec3>(&ron_str).expect("Failed to migrate");

    assert_eq!(migrated, vec);
}

#[test]
fn test_migrate_quat_current_version() {
    let quat = Quat::from_rotation_y(std::f32::consts::PI / 4.0);

    // Serialize with version
    let ron_str = to_ron_versioned(&quat).expect("Failed to serialize");

    // Deserialize and migrate
    let migrated = from_ron_versioned::<Quat>(&ron_str).expect("Failed to migrate");

    // Use approximate comparison
    assert!((quat.x - migrated.x).abs() < 1e-6);
    assert!((quat.y - migrated.y).abs() < 1e-6);
    assert!((quat.z - migrated.z).abs() < 1e-6);
    assert!((quat.w - migrated.w).abs() < 1e-6);
}

#[test]
fn test_migrate_transform_current_version() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };

    // Serialize with version
    let ron_str = to_ron_versioned(&transform).expect("Failed to serialize");

    // Deserialize and migrate
    let migrated = from_ron_versioned::<Transform>(&ron_str).expect("Failed to migrate");

    assert_eq!(migrated.translation, transform.translation);
    assert_eq!(migrated.scale, transform.scale);
    assert!((migrated.rotation.x - transform.rotation.x).abs() < 1e-6);
    assert!((migrated.rotation.y - transform.rotation.y).abs() < 1e-6);
    assert!((migrated.rotation.z - transform.rotation.z).abs() < 1e-6);
    assert!((migrated.rotation.w - transform.rotation.w).abs() < 1e-6);
}

#[test]
fn test_migrate_color_current_version() {
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);

    // Serialize with version
    let ron_str = to_ron_versioned(&color).expect("Failed to serialize");

    // Deserialize and migrate
    let migrated = from_ron_versioned::<Color>(&ron_str).expect("Failed to migrate");

    assert_eq!(migrated, color);
}

// ============================================================================
// Migration Tests - Binary Format
// ============================================================================

#[test]
fn test_migrate_vec3_binary_current_version() {
    let vec = Vec3::new(1.0, 2.0, 3.0);

    // Serialize with version
    let binary = to_binary_versioned(&vec).expect("Failed to serialize");

    // First 4 bytes should be version
    assert_eq!(binary.len() > 4, true);
    let version = u32::from_le_bytes([binary[0], binary[1], binary[2], binary[3]]);
    assert_eq!(version, CURRENT_VERSION);

    // Deserialize and migrate
    let migrated = from_binary_versioned::<Vec3>(&binary).expect("Failed to migrate");

    assert_eq!(migrated, vec);
}

#[test]
fn test_migrate_transform_binary_current_version() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };

    // Serialize with version
    let binary = to_binary_versioned(&transform).expect("Failed to serialize");

    // Deserialize and migrate
    let migrated = from_binary_versioned::<Transform>(&binary).expect("Failed to migrate");

    assert_eq!(migrated.translation, transform.translation);
    assert_eq!(migrated.scale, transform.scale);
}

// ============================================================================
// Migration Tests - Future Version Handling
// ============================================================================

#[test]
fn test_migrate_future_version_error() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let future_version = CURRENT_VERSION + 1;

    // Create data with future version
    let versioned = Versioned::with_version(future_version, vec);
    let ron_str = ron::to_string(&versioned).expect("Failed to serialize");

    // Attempt to migrate should fail
    let result = from_ron_versioned::<Vec3>(&ron_str);
    assert!(result.is_err());

    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("newer version"));
    assert!(error_msg.contains(&format!("v{}", future_version)));
}

#[test]
fn test_migrate_future_version_binary_error() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let future_version = CURRENT_VERSION + 1;

    // Create binary data with future version
    let mut binary = Vec::new();
    binary.extend_from_slice(&future_version.to_le_bytes());
    let data_bytes = bincode::serialize(&vec).expect("Failed to serialize");
    binary.extend_from_slice(&data_bytes);

    // Attempt to migrate should fail
    let result = from_binary_versioned::<Vec3>(&binary);
    assert!(result.is_err());

    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("newer version"));
}

// ============================================================================
// Migration Tests - Legacy Format (No Version Field)
// ============================================================================

#[test]
fn test_migrate_legacy_format_no_version() {
    // Simulate legacy data without version field
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let legacy_ron = ron::to_string(&vec).expect("Failed to serialize");

    // Should still be able to load (assumes version 1)
    let migrated = from_ron_versioned::<Vec3>(&legacy_ron).expect("Failed to migrate");

    assert_eq!(migrated, vec);
}

// ============================================================================
// Migration Tests - Corrupted Data
// ============================================================================

#[test]
fn test_migrate_corrupted_ron_data() {
    let corrupted_ron = "(version: 1, x: \"not_a_number\", y: 2.0, z: 3.0)";

    let result = from_ron_versioned::<Vec3>(corrupted_ron);
    assert!(result.is_err());
}

#[test]
fn test_migrate_corrupted_binary_data() {
    let corrupted_binary = vec![1, 2, 3]; // Too short

    let result = from_binary_versioned::<Vec3>(&corrupted_binary);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too short"));
}

#[test]
fn test_migrate_invalid_binary_version() {
    // Create binary with version but no data
    let binary = vec![1, 0, 0, 0]; // Version 1, but no data

    let result = from_binary_versioned::<Vec3>(&binary);
    assert!(result.is_err());
}

// ============================================================================
// Batch Migration Tests
// ============================================================================

#[test]
fn test_batch_migration_result_creation() {
    let mut result = BatchMigrationResult::new();

    assert_eq!(result.total, 0);
    assert_eq!(result.succeeded, 0);
    assert_eq!(result.failed, 0);
    assert_eq!(result.success_rate(), 0.0);
}

#[test]
fn test_batch_migration_result_tracking() {
    let mut result = BatchMigrationResult::new();

    result.add_success();
    result.add_success();
    result.add_failure(
        "test.ron".to_string(),
        MigrationError {
            from_version: 1,
            to_version: CURRENT_VERSION,
            kind: luminara_math::migration::MigrationErrorKind::DataCorruption {
                reason: "Test error".to_string(),
            },
        },
    );

    assert_eq!(result.total, 3);
    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 1);
    assert!((result.success_rate() - 0.666).abs() < 0.01);
}

#[test]
fn test_batch_migration_result_display() {
    let mut result = BatchMigrationResult::new();
    result.add_success();
    result.add_success();

    let display = format!("{}", result);
    assert!(display.contains("Total: 2"));
    assert!(display.contains("Succeeded: 2"));
    assert!(display.contains("Failed: 0"));
    assert!(display.contains("100.0%"));
}

#[test]
fn test_batch_migration_ron_files() {
    use luminara_math::migration::migrate_ron_files;

    let vec1 = Vec3::new(1.0, 2.0, 3.0);
    let vec2 = Vec3::new(4.0, 5.0, 6.0);

    let ron1 = to_ron_versioned(&vec1).expect("Failed to serialize");
    let ron2 = to_ron_versioned(&vec2).expect("Failed to serialize");

    let files = vec![
        ("vec1.ron".to_string(), ron1),
        ("vec2.ron".to_string(), ron2),
    ];

    let result = migrate_ron_files::<Vec3>(&files);

    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 0);
    assert_eq!(result.success_rate(), 1.0);
}

#[test]
fn test_batch_migration_ron_files_with_errors() {
    use luminara_math::migration::migrate_ron_files;

    let vec1 = Vec3::new(1.0, 2.0, 3.0);
    let ron1 = to_ron_versioned(&vec1).expect("Failed to serialize");
    let corrupted_ron = "(version: 1, x: \"invalid\", y: 2.0, z: 3.0)";

    let files = vec![
        ("vec1.ron".to_string(), ron1),
        ("corrupted.ron".to_string(), corrupted_ron.to_string()),
    ];

    let result = migrate_ron_files::<Vec3>(&files);

    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 1);
    assert_eq!(result.failed, 1);
    assert_eq!(result.success_rate(), 0.5);
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_batch_migration_binary_files() {
    use luminara_math::migration::migrate_binary_files;

    let vec1 = Vec3::new(1.0, 2.0, 3.0);
    let vec2 = Vec3::new(4.0, 5.0, 6.0);

    let binary1 = to_binary_versioned(&vec1).expect("Failed to serialize");
    let binary2 = to_binary_versioned(&vec2).expect("Failed to serialize");

    let files = vec![
        ("vec1.bin".to_string(), binary1),
        ("vec2.bin".to_string(), binary2),
    ];

    let result = migrate_binary_files::<Vec3>(&files);

    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 0);
    assert_eq!(result.success_rate(), 1.0);
}

// ============================================================================
// Migratable Trait Tests
// ============================================================================

#[test]
fn test_migratable_can_migrate() {
    assert!(Vec3::can_migrate(1));
    assert!(Vec3::can_migrate(CURRENT_VERSION));
    assert!(!Vec3::can_migrate(CURRENT_VERSION + 1));
}

#[test]
fn test_migratable_min_supported_version() {
    assert_eq!(Vec3::min_supported_version(), 1);
    assert_eq!(Quat::min_supported_version(), 1);
    assert_eq!(Transform::min_supported_version(), 1);
    assert_eq!(Color::min_supported_version(), 1);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_roundtrip_with_version_ron() {
    let original = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };

    // Serialize with version
    let ron_str = to_ron_versioned(&original).expect("Failed to serialize");

    // Deserialize with migration
    let migrated = from_ron_versioned::<Transform>(&ron_str).expect("Failed to migrate");

    // Verify roundtrip
    assert_eq!(migrated.translation, original.translation);
    assert_eq!(migrated.scale, original.scale);
}

#[test]
fn test_roundtrip_with_version_binary() {
    let original = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };

    // Serialize with version
    let binary = to_binary_versioned(&original).expect("Failed to serialize");

    // Deserialize with migration
    let migrated = from_binary_versioned::<Transform>(&binary).expect("Failed to migrate");

    // Verify roundtrip
    assert_eq!(migrated.translation, original.translation);
    assert_eq!(migrated.scale, original.scale);
}

#[test]
fn test_multiple_types_migration() {
    // Test that different types can all be migrated
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let quat = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    let transform = Transform {
        translation: vec,
        rotation: quat,
        scale: Vec3::ONE,
    };

    // Serialize all
    let vec_ron = to_ron_versioned(&vec).expect("Failed to serialize Vec3");
    let quat_ron = to_ron_versioned(&quat).expect("Failed to serialize Quat");
    let color_ron = to_ron_versioned(&color).expect("Failed to serialize Color");
    let transform_ron = to_ron_versioned(&transform).expect("Failed to serialize Transform");

    // Migrate all
    let vec_migrated = from_ron_versioned::<Vec3>(&vec_ron).expect("Failed to migrate Vec3");
    let quat_migrated = from_ron_versioned::<Quat>(&quat_ron).expect("Failed to migrate Quat");
    let color_migrated =
        from_ron_versioned::<Color>(&color_ron).expect("Failed to migrate Color");
    let transform_migrated =
        from_ron_versioned::<Transform>(&transform_ron).expect("Failed to migrate Transform");

    // Verify all
    assert_eq!(vec_migrated, vec);
    assert_eq!(color_migrated, color);
    assert_eq!(transform_migrated.translation, transform.translation);
}
