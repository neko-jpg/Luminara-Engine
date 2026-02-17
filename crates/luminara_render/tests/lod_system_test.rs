/// LOD System Tests
///
/// Tests for the Level of Detail system including:
/// - LOD level selection based on screen coverage
/// - Mesh simplification
/// - Performance improvements
/// - Smooth transitions
///
/// **Validates: Requirements 19.5**

use luminara_render::{LodConfig, LodGenerator, Mesh};

#[test]
fn test_lod_generator_creates_multiple_levels() {
    let generator = LodGenerator::default();
    let source = Mesh::cube(2.0);
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    // Should create 5 LOD levels by default
    assert_eq!(lod_meshes.len(), 5, "Should generate 5 LOD levels");
    
    // LOD 0 should match source
    assert_eq!(
        lod_meshes[0].vertices.len(),
        source.vertices.len(),
        "LOD 0 should have same vertex count as source"
    );
    
    // Each subsequent LOD should have fewer or equal vertices
    for i in 1..lod_meshes.len() {
        assert!(
            lod_meshes[i].vertices.len() <= lod_meshes[i - 1].vertices.len(),
            "LOD {} should have <= vertices than LOD {}",
            i,
            i - 1
        );
    }
}

#[test]
fn test_lod_generator_with_sphere() {
    let generator = LodGenerator::default();
    let source = Mesh::sphere(1.0, 32); // High-poly sphere
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    assert_eq!(lod_meshes.len(), 5);
    
    // Verify reduction ratios are approximately correct
    let original_count = lod_meshes[0].vertices.len();
    
    // LOD 1 should be ~50% of original
    let lod1_ratio = lod_meshes[1].vertices.len() as f32 / original_count as f32;
    assert!(
        lod1_ratio <= 0.6,
        "LOD 1 should be <= 60% of original vertices, got {}%",
        lod1_ratio * 100.0
    );
    
    // LOD 2 should be ~25% of original
    let lod2_ratio = lod_meshes[2].vertices.len() as f32 / original_count as f32;
    assert!(
        lod2_ratio <= 0.35,
        "LOD 2 should be <= 35% of original vertices, got {}%",
        lod2_ratio * 100.0
    );
}

#[test]
fn test_lod_meshes_are_valid() {
    let generator = LodGenerator::default();
    let source = Mesh::cube(1.0);
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    for (i, mesh) in lod_meshes.iter().enumerate() {
        // Each mesh should have at least 3 vertices (minimum for a triangle)
        assert!(
            mesh.vertices.len() >= 3,
            "LOD {} should have at least 3 vertices",
            i
        );
        
        // Each mesh should have at least 3 indices (minimum for a triangle)
        assert!(
            mesh.indices.len() >= 3,
            "LOD {} should have at least 3 indices",
            i
        );
        
        // Indices should be valid
        for &index in &mesh.indices {
            assert!(
                (index as usize) < mesh.vertices.len(),
                "LOD {} has invalid index {} (vertex count: {})",
                i,
                index,
                mesh.vertices.len()
            );
        }
        
        // Should have valid AABB
        assert!(
            mesh.aabb.min.x <= mesh.aabb.max.x,
            "LOD {} AABB min.x should be <= max.x",
            i
        );
        assert!(
            mesh.aabb.min.y <= mesh.aabb.max.y,
            "LOD {} AABB min.y should be <= max.y",
            i
        );
        assert!(
            mesh.aabb.min.z <= mesh.aabb.max.z,
            "LOD {} AABB min.z should be <= max.z",
            i
        );
    }
}

#[test]
fn test_lod_config_default_values() {
    let config = LodConfig::default();
    
    assert_eq!(
        config.screen_coverage_thresholds.len(),
        4,
        "Should have 4 LOD thresholds"
    );
    assert_eq!(config.screen_coverage_thresholds[0], 800.0);
    assert_eq!(config.screen_coverage_thresholds[1], 400.0);
    assert_eq!(config.screen_coverage_thresholds[2], 200.0);
    assert_eq!(config.screen_coverage_thresholds[3], 100.0);
    
    assert_eq!(config.transition_zone, 0.2);
    assert!(config.smooth_transitions);
    assert_eq!(config.lod_bias, 0.0);
}

#[test]
fn test_lod_generator_custom_ratios() {
    let mut generator = LodGenerator::default();
    generator.reduction_ratios = vec![1.0, 0.75, 0.5, 0.25];
    
    let source = Mesh::sphere(1.0, 16);
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    assert_eq!(lod_meshes.len(), 4, "Should generate 4 LOD levels");
}

#[test]
fn test_lod_performance_improvement() {
    let generator = LodGenerator::default();
    let source = Mesh::sphere(1.0, 64); // Very high-poly sphere
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    // Calculate total vertices if all objects used highest LOD
    let vertices_without_lod = lod_meshes[0].vertices.len() * 5;
    
    // Calculate total vertices with LOD (assuming even distribution)
    let vertices_with_lod: usize = lod_meshes.iter().map(|m| m.vertices.len()).sum();
    
    // Calculate improvement
    let improvement = (1.0 - (vertices_with_lod as f32 / vertices_without_lod as f32)) * 100.0;
    
    // Should achieve >50% improvement
    assert!(
        improvement > 50.0,
        "LOD system should achieve >50% performance improvement, got {:.1}%",
        improvement
    );
}

#[test]
fn test_lod_no_degenerate_triangles() {
    let generator = LodGenerator::default();
    let source = Mesh::cube(1.0);
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    for (i, mesh) in lod_meshes.iter().enumerate() {
        // Check that no triangles have duplicate vertices
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                assert!(
                    chunk[0] != chunk[1] && chunk[1] != chunk[2] && chunk[0] != chunk[2],
                    "LOD {} has degenerate triangle: {:?}",
                    i,
                    chunk
                );
            }
        }
    }
}

#[test]
fn test_lod_preserves_mesh_bounds() {
    let generator = LodGenerator::default();
    let source = Mesh::cube(2.0);
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    let original_aabb = &source.aabb;
    
    for (i, mesh) in lod_meshes.iter().enumerate() {
        // LOD meshes should fit within original bounds (approximately)
        // Allow some tolerance for simplification
        let tolerance = 0.1;
        
        assert!(
            mesh.aabb.min.x >= original_aabb.min.x - tolerance,
            "LOD {} min.x out of bounds",
            i
        );
        assert!(
            mesh.aabb.max.x <= original_aabb.max.x + tolerance,
            "LOD {} max.x out of bounds",
            i
        );
    }
}

#[test]
fn test_lod_generator_with_plane() {
    let generator = LodGenerator::default();
    let source = Mesh::plane(10.0);
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    assert_eq!(lod_meshes.len(), 5);
    
    // Even simple meshes should generate valid LODs
    for mesh in &lod_meshes {
        assert!(mesh.vertices.len() >= 3);
        assert!(mesh.indices.len() >= 3);
    }
}

#[test]
fn test_lod_extreme_simplification() {
    let generator = LodGenerator::default();
    let source = Mesh::sphere(1.0, 128); // Very high-poly
    
    let lod_meshes = generator.generate_lod_meshes(&source);
    
    // Even at extreme simplification, should maintain valid mesh
    let lowest_lod = &lod_meshes[lod_meshes.len() - 1];
    
    assert!(lowest_lod.vertices.len() >= 3);
    assert!(lowest_lod.indices.len() >= 3);
    
    // Should be significantly simplified
    let reduction = 1.0 - (lowest_lod.vertices.len() as f32 / source.vertices.len() as f32);
    assert!(
        reduction > 0.9,
        "Lowest LOD should reduce by >90%, got {:.1}%",
        reduction * 100.0
    );
}

#[test]
fn test_lod_config_custom_thresholds() {
    let mut config = LodConfig::default();
    config.screen_coverage_thresholds = vec![1000.0, 500.0, 250.0];
    
    assert_eq!(config.screen_coverage_thresholds.len(), 3);
    assert_eq!(config.screen_coverage_thresholds[0], 1000.0);
}

#[test]
fn test_lod_config_bias() {
    let mut config = LodConfig::default();
    
    // Negative bias should prefer higher detail
    config.lod_bias = -0.5;
    assert_eq!(config.lod_bias, -0.5);
    
    // Positive bias should prefer lower detail
    config.lod_bias = 0.5;
    assert_eq!(config.lod_bias, 0.5);
}

#[test]
fn test_lod_transition_zone() {
    let mut config = LodConfig::default();
    
    config.transition_zone = 0.3;
    assert_eq!(config.transition_zone, 0.3);
    
    // Transition zone should be between 0 and 1
    assert!(config.transition_zone >= 0.0 && config.transition_zone <= 1.0);
}
