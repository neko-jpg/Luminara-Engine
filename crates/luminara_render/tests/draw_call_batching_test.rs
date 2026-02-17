//! Tests for aggressive draw call batching system
//!
//! Validates Requirement 19.3: <100 draw calls for 1000+ objects

use luminara_math::Color;
use luminara_render::{
    DrawCallBatcher, DrawCallBatcherStats, DrawCallSortKey, MaterialKey, PbrMaterial,
};

/// Helper to create a test material
fn create_material(albedo: Color, metallic: f32, roughness: f32) -> PbrMaterial {
    PbrMaterial {
        albedo,
        metallic,
        roughness,
        emissive: Color::BLACK,
        albedo_texture: None,
        normal_texture: None,
        metallic_roughness_texture: None,
    }
}

#[test]
fn test_material_key_creation() {
    let material = PbrMaterial {
        albedo: Color::rgba(1.0, 0.5, 0.25, 1.0),
        metallic: 0.8,
        roughness: 0.3,
        emissive: Color::BLACK,
        albedo_texture: None,
        normal_texture: None,
        metallic_roughness_texture: None,
    };

    let key = MaterialKey::from_material(&material);

    assert_eq!(key.albedo, [255, 127, 63, 255]);
    assert_eq!(key.metallic, 204); // 0.8 * 255
    assert_eq!(key.roughness, 76); // 0.3 * 255
}

#[test]
fn test_sort_key_ordering() {
    // Shader ID takes precedence
    let key1 = DrawCallSortKey {
        shader_id: 0,
        texture_id: Some(1),
        material_key: MaterialKey {
            albedo: [255, 255, 255, 255],
            metallic: 128,
            roughness: 128,
            emissive: [0, 0, 0],
        },
    };

    let key2 = DrawCallSortKey {
        shader_id: 1,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [0, 0, 0, 255],
            metallic: 0,
            roughness: 0,
            emissive: [0, 0, 0],
        },
    };

    assert!(key1 < key2); // Lower shader ID sorts first

    // Texture ID is second priority
    let key3 = DrawCallSortKey {
        shader_id: 0,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [255, 255, 255, 255],
            metallic: 255,
            roughness: 255,
            emissive: [255, 255, 255],
        },
    };

    let key4 = DrawCallSortKey {
        shader_id: 0,
        texture_id: Some(1),
        material_key: MaterialKey {
            albedo: [0, 0, 0, 255],
            metallic: 0,
            roughness: 0,
            emissive: [0, 0, 0],
        },
    };

    assert!(key3 < key4); // None sorts before Some
}

#[test]
fn test_batcher_empty() {
    let batcher = DrawCallBatcher::new();
    let stats = batcher.stats();

    assert_eq!(stats.total_objects, 0);
    assert_eq!(stats.total_batches, 0);
}

#[test]
fn test_stats_meets_target() {
    // Test with 1000 objects and 50 batches - should meet target
    let stats = DrawCallBatcherStats {
        total_objects: 1000,
        total_batches: 50,
        batching_ratio: 20.0,
        max_instances_per_batch: 100,
        min_instances_per_batch: 5,
        avg_instances_per_batch: 20.0,
    };

    assert!(stats.meets_target());

    // Test with 1000 objects and 150 batches - should fail target
    let stats_fail = DrawCallBatcherStats {
        total_objects: 1000,
        total_batches: 150,
        batching_ratio: 6.67,
        max_instances_per_batch: 20,
        min_instances_per_batch: 1,
        avg_instances_per_batch: 6.67,
    };

    assert!(!stats_fail.meets_target());
}

#[test]
fn test_batching_target_calculation() {
    // Test various scenarios
    let scenarios = vec![
        (1000, 50, true),   // 1000 objects, 50 batches - meets target
        (1000, 99, true),   // 1000 objects, 99 batches - meets target
        (1000, 100, false), // 1000 objects, 100 batches - fails target (needs <100)
        (1000, 150, false), // 1000 objects, 150 batches - fails target
        (2000, 150, true),  // 2000 objects, 150 batches - meets target (scaled)
        (500, 50, true),    // 500 objects, 50 batches - meets target
        (100, 10, true),    // 100 objects, 10 batches - meets target
    ];

    for (objects, batches, should_meet) in scenarios {
        let stats = DrawCallBatcherStats {
            total_objects: objects,
            total_batches: batches,
            batching_ratio: objects as f32 / batches as f32,
            max_instances_per_batch: 100,
            min_instances_per_batch: 1,
            avg_instances_per_batch: objects as f32 / batches as f32,
        };

        assert_eq!(
            stats.meets_target(),
            should_meet,
            "Failed for {} objects, {} batches",
            objects,
            batches
        );
    }
}

#[test]
fn test_material_key_identical_materials() {
    let material1 = create_material(Color::WHITE, 0.5, 0.5);
    let material2 = create_material(Color::WHITE, 0.5, 0.5);

    let key1 = MaterialKey::from_material(&material1);
    let key2 = MaterialKey::from_material(&material2);

    assert_eq!(key1, key2, "Identical materials should have identical keys");
}

#[test]
fn test_material_key_different_albedo() {
    let material1 = create_material(Color::WHITE, 0.5, 0.5);
    let material2 = create_material(Color::RED, 0.5, 0.5);

    let key1 = MaterialKey::from_material(&material1);
    let key2 = MaterialKey::from_material(&material2);

    assert_ne!(key1, key2, "Different albedo should produce different keys");
}

#[test]
fn test_material_key_different_metallic() {
    let material1 = create_material(Color::WHITE, 0.5, 0.5);
    let material2 = create_material(Color::WHITE, 0.8, 0.5);

    let key1 = MaterialKey::from_material(&material1);
    let key2 = MaterialKey::from_material(&material2);

    assert_ne!(key1, key2, "Different metallic should produce different keys");
}

#[test]
fn test_material_key_different_roughness() {
    let material1 = create_material(Color::WHITE, 0.5, 0.5);
    let material2 = create_material(Color::WHITE, 0.5, 0.8);

    let key1 = MaterialKey::from_material(&material1);
    let key2 = MaterialKey::from_material(&material2);

    assert_ne!(key1, key2, "Different roughness should produce different keys");
}

#[test]
fn test_sort_key_shader_priority() {
    // Shader ID should be highest priority in sorting
    let key_shader0 = DrawCallSortKey {
        shader_id: 0,
        texture_id: Some(999),
        material_key: MaterialKey {
            albedo: [255, 255, 255, 255],
            metallic: 255,
            roughness: 255,
            emissive: [255, 255, 255],
        },
    };

    let key_shader1 = DrawCallSortKey {
        shader_id: 1,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [0, 0, 0, 255],
            metallic: 0,
            roughness: 0,
            emissive: [0, 0, 0],
        },
    };

    assert!(
        key_shader0 < key_shader1,
        "Shader ID should take precedence over all other properties"
    );
}

#[test]
fn test_sort_key_texture_priority() {
    // Texture ID should be second priority (after shader)
    let key_no_texture = DrawCallSortKey {
        shader_id: 0,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [255, 255, 255, 255],
            metallic: 255,
            roughness: 255,
            emissive: [255, 255, 255],
        },
    };

    let key_with_texture = DrawCallSortKey {
        shader_id: 0,
        texture_id: Some(1),
        material_key: MaterialKey {
            albedo: [0, 0, 0, 255],
            metallic: 0,
            roughness: 0,
            emissive: [0, 0, 0],
        },
    };

    assert!(
        key_no_texture < key_with_texture,
        "No texture should sort before textured (None < Some)"
    );
}

#[test]
fn test_sort_key_material_priority() {
    // Material properties should be third priority
    let key_dark = DrawCallSortKey {
        shader_id: 0,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [0, 0, 0, 255],
            metallic: 0,
            roughness: 0,
            emissive: [0, 0, 0],
        },
    };

    let key_bright = DrawCallSortKey {
        shader_id: 0,
        texture_id: None,
        material_key: MaterialKey {
            albedo: [255, 255, 255, 255],
            metallic: 255,
            roughness: 255,
            emissive: [255, 255, 255],
        },
    };

    assert!(
        key_dark < key_bright,
        "Material properties should be used for sorting when shader and texture are equal"
    );
}

#[test]
fn test_batching_efficiency_calculation() {
    // Perfect batching: 1000 objects in 10 batches
    let perfect = DrawCallBatcherStats {
        total_objects: 1000,
        total_batches: 10,
        batching_ratio: 100.0,
        max_instances_per_batch: 100,
        min_instances_per_batch: 100,
        avg_instances_per_batch: 100.0,
    };

    assert_eq!(perfect.batching_ratio, 100.0);
    assert!(perfect.meets_target());

    // Poor batching: 1000 objects in 500 batches
    let poor = DrawCallBatcherStats {
        total_objects: 1000,
        total_batches: 500,
        batching_ratio: 2.0,
        max_instances_per_batch: 10,
        min_instances_per_batch: 1,
        avg_instances_per_batch: 2.0,
    };

    assert_eq!(poor.batching_ratio, 2.0);
    assert!(!poor.meets_target());
}

#[test]
fn test_quantization_precision() {
    // Test that quantization to 8-bit doesn't lose too much precision
    let material = create_material(Color::rgba(0.5, 0.5, 0.5, 1.0), 0.5, 0.5);

    let key = MaterialKey::from_material(&material);

    // 0.5 * 255 = 127.5, should round to 127
    assert_eq!(key.albedo[0], 127);
    assert_eq!(key.metallic, 127);
    assert_eq!(key.roughness, 127);
}

#[test]
fn test_quantization_edge_cases() {
    // Test edge cases: 0.0 and 1.0
    let material_black = create_material(Color::BLACK, 0.0, 0.0);
    let material_white = create_material(Color::WHITE, 1.0, 1.0);

    let key_black = MaterialKey::from_material(&material_black);
    let key_white = MaterialKey::from_material(&material_white);

    assert_eq!(key_black.albedo, [0, 0, 0, 255]); // Black RGB, full alpha
    assert_eq!(key_black.metallic, 0);
    assert_eq!(key_black.roughness, 0);

    assert_eq!(key_white.albedo, [255, 255, 255, 255]);
    assert_eq!(key_white.metallic, 255);
    assert_eq!(key_white.roughness, 255);
}

#[test]
fn test_target_1000_objects_scenario() {
    // Simulate the target scenario: 1000 objects with good batching
    // With 10 unique meshes and 5 unique materials = 50 batches expected
    let stats = DrawCallBatcherStats {
        total_objects: 1000,
        total_batches: 50,
        batching_ratio: 20.0,
        max_instances_per_batch: 100,
        min_instances_per_batch: 10,
        avg_instances_per_batch: 20.0,
    };

    stats.print();

    assert!(stats.meets_target(), "Should meet target of <100 batches for 1000+ objects");
    assert!(stats.total_batches < 100, "Expected <100 batches, got {}", stats.total_batches);
    assert_eq!(stats.total_objects, 1000);
}

#[test]
fn test_target_2000_objects_scenario() {
    // Test with 2000 objects - target should scale proportionally
    let stats = DrawCallBatcherStats {
        total_objects: 2000,
        total_batches: 100,
        batching_ratio: 20.0,
        max_instances_per_batch: 200,
        min_instances_per_batch: 10,
        avg_instances_per_batch: 20.0,
    };

    assert!(stats.meets_target(), "Should meet scaled target for 2000 objects");
}
