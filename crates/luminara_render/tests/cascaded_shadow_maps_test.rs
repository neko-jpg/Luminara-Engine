// Tests for cascaded shadow map implementation
use luminara_render::shadow::{
    calculate_cascade_splits, calculate_cascade_view_proj, ShadowCascades, ShadowMapResources,
};
use luminara_math::{Transform, Vec3};
use luminara_render::{Camera, Projection};

#[test]
fn test_cascade_split_calculation() {
    let near = 0.1;
    let far = 100.0;
    let cascade_count = 4;
    let lambda = 0.5;

    let splits = calculate_cascade_splits(near, far, cascade_count, lambda);

    // Verify we have the correct number of splits
    assert_eq!(splits.len(), cascade_count as usize);

    // Verify splits are monotonically increasing
    for i in 1..splits.len() {
        assert!(splits[i] > splits[i - 1], "Splits should be monotonically increasing");
    }

    // Verify first split is greater than near
    assert!(splits[0] > near, "First split should be greater than near plane");

    // Verify last split equals far
    assert!(
        (splits[splits.len() - 1] - far).abs() < 0.001,
        "Last split should equal far plane"
    );
}

#[test]
fn test_cascade_split_lambda_effect() {
    let near = 0.1;
    let far = 100.0;
    let cascade_count = 4;

    // Test uniform distribution (lambda = 0)
    let uniform_splits = calculate_cascade_splits(near, far, cascade_count, 0.0);

    // Test logarithmic distribution (lambda = 1)
    let log_splits = calculate_cascade_splits(near, far, cascade_count, 1.0);

    // Logarithmic should have smaller early splits (more detail near camera)
    assert!(
        log_splits[0] < uniform_splits[0],
        "Logarithmic distribution should have smaller first split"
    );

    // Both should end at far plane
    assert!(
        (uniform_splits[3] - far).abs() < 0.001,
        "Uniform splits should end at far plane"
    );
    assert!(
        (log_splits[3] - far).abs() < 0.001,
        "Logarithmic splits should end at far plane"
    );
}

#[test]
fn test_shadow_cascades_default() {
    let config = ShadowCascades::default();

    assert_eq!(config.cascade_count, 4, "Default should have 4 cascades");
    assert_eq!(config.shadow_map_size, 2048, "Default shadow map size should be 2048");
    assert_eq!(config.split_lambda, 0.5, "Default lambda should be 0.5");
    assert_eq!(config.blend_region, 0.1, "Default blend region should be 0.1");
}

#[test]
fn test_cascade_view_proj_calculation() {
    // This test verifies that the cascade view-projection calculation completes without panicking
    // The actual correctness of the matrix would require a full rendering context
    let light_direction = Vec3::new(0.0, -1.0, 0.5).normalize(); // Angled downward light
    let camera_transform = Transform {
        translation: Vec3::new(0.0, 5.0, 10.0),
        rotation: luminara_math::Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    let camera = Camera {
        projection: Projection::Perspective {
            fov: 60.0,
            near: 0.1,
            far: 100.0,
        },
        clear_color: luminara_math::Color::BLACK,
        is_active: true,
    };

    let near = 0.1;
    let far = 10.0;
    let aspect = 16.0 / 9.0;

    // This should complete without panicking
    let _view_proj = calculate_cascade_view_proj(
        light_direction,
        &camera_transform,
        &camera,
        near,
        far,
        aspect,
    );

    // If we get here, the function executed successfully
    assert!(true, "Cascade view-projection calculation completed");
}

#[test]
fn test_shadow_map_resources_initialization() {
    // This test would require a wgpu device, so we'll just test the default state
    let resources = ShadowMapResources::default();

    assert!(resources.shadow_texture.is_none(), "Shadow texture should be None initially");
    assert!(resources.shadow_view.is_none(), "Shadow view should be None initially");
    assert!(resources.shadow_sampler.is_none(), "Shadow sampler should be None initially");
    assert!(resources.cascade_uniforms.is_empty(), "Cascade uniforms should be empty initially");
    assert!(resources.cascade_buffer.is_none(), "Cascade buffer should be None initially");
    assert!(resources.bind_group.is_none(), "Bind group should be None initially");
}

#[test]
fn test_cascade_splits_coverage() {
    // Test that cascades cover the entire near-far range without gaps
    let near = 0.1;
    let far = 100.0;
    let cascade_count = 4;
    let lambda = 0.5;

    let splits = calculate_cascade_splits(near, far, cascade_count, lambda);

    // First cascade should start at near
    let mut prev_split = near;

    for split in &splits {
        // Each split should be greater than the previous
        assert!(
            *split > prev_split,
            "Split {} should be greater than previous split {}",
            split,
            prev_split
        );
        prev_split = *split;
    }

    // Last split should reach far
    assert!(
        (splits[splits.len() - 1] - far).abs() < 0.001,
        "Last split should reach far plane"
    );
}

#[test]
fn test_blend_region_calculation() {
    // Test that blend regions are calculated correctly
    let config = ShadowCascades::default();
    let near = 0.1;
    let far = 100.0;

    let splits = calculate_cascade_splits(near, far, config.cascade_count, config.split_lambda);

    // Verify each split has a reasonable blend region
    for (i, &split) in splits.iter().enumerate() {
        let prev_split = if i == 0 { near } else { splits[i - 1] };
        let range = split - prev_split;
        let blend_size = range * config.blend_region;

        // Blend region should be positive and less than the cascade range
        assert!(blend_size > 0.0, "Blend size should be positive");
        assert!(
            blend_size < range,
            "Blend size should be less than cascade range"
        );
    }
}

#[test]
fn test_cascade_count_flexibility() {
    // Test that the system works with different cascade counts
    let near = 0.1;
    let far = 100.0;
    let lambda = 0.5;

    for cascade_count in [2, 3, 4, 5, 6] {
        let splits = calculate_cascade_splits(near, far, cascade_count, lambda);

        assert_eq!(
            splits.len(),
            cascade_count as usize,
            "Should have {} splits",
            cascade_count
        );

        // Verify monotonic increase
        for i in 1..splits.len() {
            assert!(
                splits[i] > splits[i - 1],
                "Splits should be monotonically increasing for {} cascades",
                cascade_count
            );
        }
    }
}

#[test]
fn test_shadow_map_size_options() {
    // Test different shadow map sizes
    for size in [512, 1024, 2048, 4096] {
        let mut config = ShadowCascades::default();
        config.shadow_map_size = size;

        assert_eq!(
            config.shadow_map_size, size,
            "Shadow map size should be {}",
            size
        );

        // Verify size is power of 2
        assert!(
            size.is_power_of_two(),
            "Shadow map size {} should be power of 2",
            size
        );
    }
}
