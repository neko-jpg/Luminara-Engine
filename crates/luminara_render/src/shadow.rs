// Cascaded shadow mapping implementation with smooth transitions
use crate::{Camera, DirectionalLight, GpuContext};
use luminara_core::shared_types::{Query, Res, ResMut, Resource};
use luminara_math::{Mat4, Transform, Vec3};

/// Shadow cascade configuration
pub struct ShadowCascades {
    pub cascade_count: u32,
    pub cascade_splits: Vec<f32>,
    pub shadow_map_size: u32,
    /// Lambda parameter for logarithmic split distribution (0.0 = uniform, 1.0 = logarithmic)
    pub split_lambda: f32,
    /// Blend region size for smooth cascade transitions (0.0 - 1.0)
    pub blend_region: f32,
    /// Shadow bias to prevent shadow acne
    pub depth_bias: f32,
    pub slope_bias: f32,
}

impl Default for ShadowCascades {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            cascade_splits: vec![0.1, 0.25, 0.5, 1.0],
            shadow_map_size: 2048,
            split_lambda: 0.5, // Balanced between uniform and logarithmic
            blend_region: 0.1, // 10% blend region for smooth transitions
            depth_bias: 0.005,
            slope_bias: 2.0,
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
    pub cascade_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Resource for ShadowMapResources {}

impl Default for ShadowMapResources {
    fn default() -> Self {
        Self {
            shadow_texture: None,
            shadow_view: None,
            shadow_sampler: None,
            cascade_uniforms: Vec::new(),
            cascade_buffer: None,
            bind_group: None,
            bind_group_layout: None,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeUniform {
    pub view_proj: [[f32; 4]; 4],
    pub split_depth: f32,
    pub blend_start: f32, // Start of blend region
    pub blend_end: f32,   // End of blend region
    pub _padding: f32,
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

        // Create sampler with PCF (Percentage Closer Filtering) for smooth shadows
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

        // Initialize cascade uniforms with blend regions
        self.cascade_uniforms = vec![
            CascadeUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                split_depth: 0.0,
                blend_start: 0.0,
                blend_end: 0.0,
                _padding: 0.0,
            };
            config.cascade_count as usize
        ];

        // Create uniform buffer for cascade data
        let cascade_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Cascade Buffer"),
            size: (std::mem::size_of::<CascadeUniform>() * config.cascade_count as usize) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Bind Group Layout"),
            entries: &[
                // Shadow map texture array
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shadow sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                // Cascade uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cascade_buffer.as_entire_binding(),
                },
            ],
        });

        self.shadow_texture = Some(shadow_texture);
        self.shadow_view = Some(shadow_view);
        self.shadow_sampler = Some(shadow_sampler);
        self.cascade_buffer = Some(cascade_buffer);
        self.bind_group = Some(bind_group);
        self.bind_group_layout = Some(bind_group_layout);
    }

    /// Update cascade uniform buffer on GPU
    pub fn update_cascade_buffer(&self, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.cascade_buffer {
            let data = bytemuck::cast_slice(&self.cascade_uniforms);
            queue.write_buffer(buffer, 0, data);
        }
    }
}

/// Calculate cascade split depths using logarithmic distribution with lambda blending
pub fn calculate_cascade_splits(near: f32, far: f32, cascade_count: u32, lambda: f32) -> Vec<f32> {
    let mut splits = Vec::with_capacity(cascade_count as usize);

    for i in 0..cascade_count {
        let p = (i + 1) as f32 / cascade_count as f32;
        // Logarithmic distribution (better for large view distances)
        let log = near * (far / near).powf(p);
        // Uniform distribution
        let uniform = near + (far - near) * p;
        // Blend between logarithmic and uniform using lambda
        let d = lambda * log + (1.0 - lambda) * uniform;
        splits.push(d);
    }

    splits
}

/// Calculate blend regions for smooth cascade transitions
fn calculate_blend_regions(splits: &[f32], blend_factor: f32) -> Vec<(f32, f32)> {
    let mut blend_regions = Vec::with_capacity(splits.len());
    
    for (i, &split) in splits.iter().enumerate() {
        let prev_split = if i == 0 { 0.0 } else { splits[i - 1] };
        let range = split - prev_split;
        let blend_size = range * blend_factor;
        
        // Blend region is at the end of each cascade
        let blend_start = split - blend_size;
        let blend_end = split;
        
        blend_regions.push((blend_start, blend_end));
    }
    
    blend_regions
}

/// Calculate light view-projection matrix for a cascade with tight fitting
pub fn calculate_cascade_view_proj(
    light_direction: Vec3,
    camera_transform: &Transform,
    camera: &Camera,
    near: f32,
    far: f32,
    aspect: f32,
) -> Mat4 {
    // Get camera frustum corners in world space for this cascade
    let frustum_corners =
        get_frustum_corners_world_space(camera_transform, camera, near, far, aspect);

    // Calculate frustum center
    let mut center = Vec3::ZERO;
    for corner in &frustum_corners {
        center += *corner;
    }
    center /= frustum_corners.len() as f32;

    // Create light view matrix looking at frustum center
    let light_pos = center - light_direction.normalize() * 50.0; // Offset light position
    let light_view = Mat4::look_at_rh(light_pos, center, Vec3::Y);

    // Transform frustum corners to light space to calculate tight bounds
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for corner in &frustum_corners {
        let light_space = light_view.transform_point3(*corner);
        min = min.min(light_space);
        max = max.max(light_space);
    }

    // Extend the Z range to include potential shadow casters behind the frustum
    let z_extension = (max.z - min.z) * 0.5; // Extend by 50% of frustum depth
    min.z -= z_extension;

    // Snap to texel grid to reduce shimmering when camera moves
    // This ensures shadow map texels align consistently
    let world_units_per_texel = (max.x - min.x) / 2048.0; // Assuming 2048x2048 shadow map
    
    min.x = (min.x / world_units_per_texel).floor() * world_units_per_texel;
    min.y = (min.y / world_units_per_texel).floor() * world_units_per_texel;
    max.x = (max.x / world_units_per_texel).floor() * world_units_per_texel;
    max.y = (max.y / world_units_per_texel).floor() * world_units_per_texel;

    // Create orthographic projection for the light (tight fit to frustum)
    let light_proj = Mat4::orthographic_rh(min.x, max.x, min.y, max.y, min.z, max.z);

    light_proj * light_view
}

/// Get frustum corners in world space
fn get_frustum_corners_world_space(
    camera_transform: &Transform,
    camera: &Camera,
    near: f32,
    far: f32,
    aspect: f32,
) -> Vec<Vec3> {
    // Get projection matrix for this frustum slice
    let mut proj = match camera.projection {
        crate::Projection::Perspective { fov, .. } => {
            Mat4::perspective_rh(fov.to_radians(), aspect, near, far)
        }
        crate::Projection::Orthographic { size, .. } => {
            let half_width = size * aspect * 0.5;
            let half_height = size * 0.5;
            Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, near, far)
        }
    };

    // Convert to [0, 1] depth range for wgpu
    proj.x_axis.z = 0.5 * proj.x_axis.z + 0.5 * proj.x_axis.w;
    proj.y_axis.z = 0.5 * proj.y_axis.z + 0.5 * proj.y_axis.w;
    proj.z_axis.z = 0.5 * proj.z_axis.z + 0.5 * proj.z_axis.w;
    proj.w_axis.z = 0.5 * proj.w_axis.z + 0.5 * proj.w_axis.w;

    // Compute view matrix from camera transform
    let view = Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        camera_transform.rotation,
        camera_transform.translation,
    )
    .inverse();

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

/// System to update shadow cascades with smooth transitions
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

    let splits = calculate_cascade_splits(near, far, config.cascade_count, config.split_lambda);
    let blend_regions = calculate_blend_regions(&splits, config.blend_region);

    // Update cascade uniforms with blend regions for smooth transitions
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

        let (blend_start, blend_end) = blend_regions[i];

        shadow_resources.cascade_uniforms[i] = CascadeUniform {
            view_proj: view_proj.to_cols_array_2d(),
            split_depth: split,
            blend_start,
            blend_end,
            _padding: 0.0,
        };

        prev_split = split;
    }

    // Update GPU buffer with new cascade data
    shadow_resources.update_cascade_buffer(&gpu.queue);
}
