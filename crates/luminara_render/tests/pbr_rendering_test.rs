// Unit tests for PBR rendering components
use luminara_math::{Color, Mat4, Quat, Vec3};
use luminara_scene::Transform;
use luminara_render::{
    DirectionalLight, ForwardPlusRenderer, PbrMaterial, PointLight, PostProcessResources,
    ShadowCascades, ShadowMapResources,
};

#[test]
fn test_pbr_material_creation() {
    let material = PbrMaterial {
        albedo: Color::WHITE,
        albedo_texture: None,
        normal_texture: None,
        metallic: 0.5,
        roughness: 0.5,
        metallic_roughness_texture: None,
        emissive: Color::BLACK,
    };

    assert_eq!(material.albedo, Color::WHITE);
    assert_eq!(material.metallic, 0.5);
    assert_eq!(material.roughness, 0.5);
    assert_eq!(material.emissive, Color::BLACK);
}

#[test]
fn test_directional_light_creation() {
    let light = DirectionalLight {
        color: Color::WHITE,
        intensity: 1.0,
        cast_shadows: true,
        shadow_cascade_count: 4,
    };

    assert_eq!(light.color, Color::WHITE);
    assert_eq!(light.intensity, 1.0);
    assert!(light.cast_shadows);
    assert_eq!(light.shadow_cascade_count, 4);
}

#[test]
fn test_point_light_creation() {
    let light = PointLight {
        color: Color::rgb(1.0, 0.8, 0.6),
        intensity: 2.0,
        range: 10.0,
        cast_shadows: false,
    };

    assert_eq!(light.color, Color::rgb(1.0, 0.8, 0.6));
    assert_eq!(light.intensity, 2.0);
    assert_eq!(light.range, 10.0);
    assert!(!light.cast_shadows);
}

#[test]
fn test_forward_plus_renderer_initialization() {
    let renderer = ForwardPlusRenderer::new();

    assert!(renderer.pipeline.is_none());
    assert!(renderer.camera_buffer.is_none());
    assert!(renderer.camera_bind_group.is_none());
    assert!(renderer.light_buffer.is_none());
    assert!(renderer.light_bind_group.is_none());
}

#[test]
fn test_shadow_cascades_default() {
    let cascades = ShadowCascades::default();

    assert_eq!(cascades.cascade_count, 4);
    assert_eq!(cascades.shadow_map_size, 2048);
    assert_eq!(cascades.cascade_splits.len(), 4);
}

#[test]
fn test_shadow_map_resources_initialization() {
    let resources = ShadowMapResources::default();

    assert!(resources.shadow_texture.is_none());
    assert!(resources.shadow_view.is_none());
    assert!(resources.shadow_sampler.is_none());
    assert!(resources.cascade_uniforms.is_empty());
}

#[test]
fn test_post_process_resources_initialization() {
    let resources = PostProcessResources::default();

    assert!(resources.pipeline.is_none());
    assert!(resources.bind_group_layout.is_none());
    assert!(resources.sampler.is_none());
}

#[test]
fn test_transform_forward_direction() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    let forward = transform.forward();
    // Forward should be -Z in default orientation
    assert!((forward - Vec3::NEG_Z).length() < 0.001);
}

#[test]
fn test_transform_rotated_forward() {
    let mut transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    // Rotate 90 degrees around Y axis
    transform.rotate_y(std::f32::consts::FRAC_PI_2);

    let forward = transform.forward();
    // After rotating 90 degrees around Y (right-handed), forward should point in -X direction
    assert!((forward - Vec3::NEG_X).length() < 0.001);
}

#[test]
fn test_pbr_material_metallic_range() {
    let material = PbrMaterial {
        albedo: Color::WHITE,
        albedo_texture: None,
        normal_texture: None,
        metallic: 1.0,
        roughness: 0.0,
        metallic_roughness_texture: None,
        emissive: Color::BLACK,
    };

    // Metallic should be in valid range [0, 1]
    assert!(material.metallic >= 0.0 && material.metallic <= 1.0);
    assert!(material.roughness >= 0.0 && material.roughness <= 1.0);
}

#[test]
fn test_directional_light_intensity() {
    let light = DirectionalLight {
        color: Color::WHITE,
        intensity: 3.0,
        cast_shadows: true,
        shadow_cascade_count: 4,
    };

    // Intensity should be positive
    assert!(light.intensity > 0.0);
}

#[test]
fn test_point_light_range() {
    let light = PointLight {
        color: Color::WHITE,
        intensity: 1.0,
        range: 15.0,
        cast_shadows: true,
    };

    // Range should be positive
    assert!(light.range > 0.0);
}

#[test]
fn test_shadow_cascade_splits_ordering() {
    let cascades = ShadowCascades::default();

    // Cascade splits should be in ascending order
    for i in 1..cascades.cascade_splits.len() {
        assert!(
            cascades.cascade_splits[i] > cascades.cascade_splits[i - 1],
            "Cascade splits should be in ascending order"
        );
    }
}

#[test]
fn test_pbr_material_emissive() {
    let material = PbrMaterial {
        albedo: Color::WHITE,
        albedo_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        emissive: Color::rgb(1.0, 0.5, 0.0),
    };

    // Emissive color should be preserved
    assert_eq!(material.emissive, Color::rgb(1.0, 0.5, 0.0));
}

#[test]
fn test_light_color_components() {
    let light = DirectionalLight {
        color: Color::rgb(0.9, 0.95, 1.0),
        intensity: 1.0,
        cast_shadows: false,
        shadow_cascade_count: 1,
    };

    // Color components should be in valid range
    assert!(light.color.r >= 0.0 && light.color.r <= 1.0);
    assert!(light.color.g >= 0.0 && light.color.g <= 1.0);
    assert!(light.color.b >= 0.0 && light.color.b <= 1.0);
}

#[test]
fn test_transform_right_direction() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    let right = transform.right();
    // Right should be +X in default orientation
    assert!((right - Vec3::X).length() < 0.001);
}

#[test]
fn test_transform_up_direction() {
    let transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    let up = transform.up();
    // Up should be +Y in default orientation
    assert!((up - Vec3::Y).length() < 0.001);
}
