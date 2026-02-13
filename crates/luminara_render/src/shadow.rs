// Cascaded shadow mapping implementation
use crate::{Camera, DirectionalLight, GpuContext};
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::{Mat4, Transform, Vec3};

/// Shadow cascade configuration
pub struct ShadowCascades {
    pub cascade_count: u32,
    pub cascade_splits: Vec<f32>,
    pub shadow_map_size: u32,
}

impl Default for ShadowCascades {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            cascade_splits: vec![0.1, 0.25, 0.5, 1.0],
            shadow_map_size: 2048,
        }
    }
}

impl Resource for ShadowCascades {}

/// Shadow map resources
pub struct ShadowMapResources {
    pub shadow_texture: Option<wgpu::Texture>,
    pub shadow_view: Option<wgpu::TextureView>,
    pub shadow_sampler: Option<wgpu::Sampler>,
    pub cascade_uniforms: Vec<CascadeUniform>,
}

impl Resource for ShadowMapResources {}

impl Default for ShadowMapResources {
    fn default() -> Self {
        Self {
            shadow_texture: None,
            shadow_view: None,
            shadow_sampler: None,
            cascade_uniforms: Vec::new(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeUniform {
    pub view_proj: [[f32; 4]; 4],
    pub split_depth: f32,
    pub _padding: [f32; 3],
}

impl ShadowMapResources {
    pub fn initialize(&mut self, device: &wgpu::Device, config: &ShadowCascades) {
        // Create shadow map texture array
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: config.shadow_map_size,
                height: config.shadow_map_size,
                depth_or_array_layers: config.cascade_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Map View"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(config.cascade_count),
        });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        self.shadow_texture = Some(shadow_texture);
        self.shadow_view = Some(shadow_view);
        self.shadow_sampler = Some(shadow_sampler);
        self.cascade_uniforms = vec![
            CascadeUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                split_depth: 0.0,
                _padding: [0.0; 3],
            };
            config.cascade_count as usize
        ];
    }
}

/// Calculate cascade split depths using logarithmic distribution
pub fn calculate_cascade_splits(near: f32, far: f32, cascade_count: u32, lambda: f32) -> Vec<f32> {
    let mut splits = Vec::with_capacity(cascade_count as usize);

    for i in 0..cascade_count {
        let p = (i + 1) as f32 / cascade_count as f32;
        let log = near * (far / near).powf(p);
        let uniform = near + (far - near) * p;
        let d = lambda * log + (1.0 - lambda) * uniform;
        splits.push(d);
    }

    splits
}

/// Calculate light view-projection matrix for a cascade
pub fn calculate_cascade_view_proj(
    light_direction: Vec3,
    camera_transform: &Transform,
    camera: &Camera,
    near: f32,
    far: f32,
    aspect: f32,
) -> Mat4 {
    // Get camera frustum corners in world space
    let frustum_corners =
        get_frustum_corners_world_space(camera_transform, camera, near, far, aspect);

    // Calculate frustum center
    let mut center = Vec3::ZERO;
    for corner in &frustum_corners {
        center += *corner;
    }
    center /= frustum_corners.len() as f32;

    // Create light view matrix
    let light_view = Mat4::look_at_rh(center - light_direction.normalize() * 10.0, center, Vec3::Y);

    // Transform frustum corners to light space
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for corner in &frustum_corners {
        let light_space = light_view.transform_point3(*corner);
        min = min.min(light_space);
        max = max.max(light_space);
    }

    // Extend the Z range to include potential shadow casters
    min.z -= 10.0;
    max.z += 10.0;

    // Create orthographic projection for the light
    let light_proj = Mat4::orthographic_rh(min.x, max.x, min.y, max.y, min.z, max.z);

    light_proj * light_view
}

/// Get frustum corners in world space
fn get_frustum_corners_world_space(
    camera_transform: &Transform,
    camera: &Camera,
    _near: f32,
    _far: f32,
    aspect: f32,
) -> Vec<Vec3> {
    let proj = camera.projection_matrix(aspect);
    let view = camera_transform.compute_matrix().inverse();
    let inv_view_proj = (proj * view).inverse();

    let mut corners = Vec::with_capacity(8);

    // NDC corners of the frustum
    for x in &[-1.0, 1.0] {
        for y in &[-1.0, 1.0] {
            for z in &[0.0, 1.0] {
                // Transform from NDC to world space
                let ndc = Vec3::new(*x, *y, *z);
                let world = inv_view_proj.project_point3(ndc);
                corners.push(world);
            }
        }
    }

    corners
}

/// System to update shadow cascades
pub fn update_shadow_cascades_system(
    gpu: Res<GpuContext>,
    config: Res<ShadowCascades>,
    mut shadow_resources: ResMut<ShadowMapResources>,
    cameras: Query<(&Camera, &Transform)>,
    directional_lights: Query<(&DirectionalLight, &Transform)>,
) {
    // Initialize shadow resources if needed
    if shadow_resources.shadow_texture.is_none() {
        shadow_resources.initialize(&gpu.device, &config);
    }

    // Get active camera
    let Some((camera, camera_transform)) = cameras.iter().next() else {
        return;
    };

    // Get first directional light that casts shadows
    let Some((_light, light_transform)) = directional_lights.iter().find(|(l, _)| l.cast_shadows)
    else {
        return;
    };

    let light_direction = light_transform.forward();
    let aspect = gpu.surface_config.width as f32 / gpu.surface_config.height as f32;

    // Calculate cascade splits
    let (near, far) = match camera.projection {
        crate::Projection::Perspective { near, far, .. } => (near, far),
        crate::Projection::Orthographic { near, far, .. } => (near, far),
    };

    let splits = calculate_cascade_splits(near, far, config.cascade_count, 0.5);

    // Update cascade uniforms
    let mut prev_split = near;
    for (i, &split) in splits.iter().enumerate() {
        let view_proj = calculate_cascade_view_proj(
            light_direction,
            camera_transform,
            camera,
            prev_split,
            split,
            aspect,
        );

        shadow_resources.cascade_uniforms[i] = CascadeUniform {
            view_proj: view_proj.to_cols_array_2d(),
            split_depth: split,
            _padding: [0.0; 3],
        };

        prev_split = split;
    }
}
