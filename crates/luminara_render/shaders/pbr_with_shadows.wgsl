// PBR Forward+ Shader with Cascaded Shadow Maps
// Implements physically-based rendering with smooth shadow transitions

// Camera uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    near: f32,
    far: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Material uniform
struct Material {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    emissive: vec3<f32>,
    _padding: f32,
}

@group(1) @binding(0)
var<uniform> material: Material;

@group(1) @binding(1)
var albedo_texture: texture_2d<f32>;

@group(1) @binding(2)
var albedo_sampler: sampler;

@group(1) @binding(3)
var normal_texture: texture_2d<f32>;

@group(1) @binding(4)
var normal_sampler: sampler;

@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;

@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

// Light data
struct DirectionalLightData {
    direction: vec3<f32>,
    _padding1: f32,
    color: vec3<f32>,
    intensity: f32,
}

struct PointLightData {
    position: vec3<f32>,
    range: f32,
    color: vec3<f32>,
    intensity: f32,
}

struct LightBuffer {
    directional_count: u32,
    point_count: u32,
    _padding: vec2<u32>,
    directional_lights: array<DirectionalLightData, 4>,
    point_lights: array<PointLightData, 256>,
}

@group(2) @binding(0)
var<storage, read> lights: LightBuffer;

// Shadow cascade data
struct CascadeUniform {
    view_proj: mat4x4<f32>,
    split_depth: f32,
    blend_start: f32,
    blend_end: f32,
    _padding: f32,
}

struct ShadowCascades {
    cascades: array<CascadeUniform, 4>,
}

@group(3) @binding(0)
var shadow_map: texture_depth_2d_array;

@group(3) @binding(1)
var shadow_sampler: sampler_comparison;

@group(3) @binding(2)
var<uniform> shadow_cascades: ShadowCascades;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
    @location(5) view_depth: f32,
}

// Model transform
struct ModelTransform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(4) @binding(0)
var<uniform> model_transform: ModelTransform;

// Vertex shader
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = model_transform.model * vec4<f32>(input.position, 1.0);
    output.world_position = world_pos.xyz;
    output.clip_position = camera.view_proj * world_pos;
    
    // Calculate view space depth for cascade selection
    let view_pos = camera.view * world_pos;
    output.view_depth = -view_pos.z; // Positive depth in view space
    
    output.world_normal = normalize((model_transform.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    output.world_tangent = normalize((model_transform.model * vec4<f32>(input.tangent.xyz, 0.0)).xyz);
    output.world_bitangent = cross(output.world_normal, output.world_tangent) * input.tangent.w;
    
    output.uv = input.uv;
    
    return output;
}

// Select cascade index based on view depth with smooth blending
fn select_cascade(view_depth: f32) -> vec2<u32> {
    // Returns (cascade_index, next_cascade_index) for blending
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (view_depth < shadow_cascades.cascades[i].split_depth) {
            return vec2<u32>(i, min(i + 1u, 3u));
        }
    }
    return vec2<u32>(3u, 3u);
}

// Calculate blend factor for smooth cascade transitions
fn calculate_blend_factor(view_depth: f32, cascade_idx: u32) -> f32 {
    let cascade = shadow_cascades.cascades[cascade_idx];
    
    // Check if we're in the blend region
    if (view_depth >= cascade.blend_start && view_depth < cascade.blend_end) {
        // Linear blend from 0 (start) to 1 (end)
        return (view_depth - cascade.blend_start) / (cascade.blend_end - cascade.blend_start);
    }
    
    return 0.0;
}

// PCF (Percentage Closer Filtering) shadow sampling with 3x3 kernel
fn sample_shadow_pcf(world_pos: vec3<f32>, cascade_idx: u32, bias: f32) -> f32 {
    let cascade = shadow_cascades.cascades[cascade_idx];
    
    // Transform world position to light space
    let light_space_pos = cascade.view_proj * vec4<f32>(world_pos, 1.0);
    let ndc = light_space_pos.xyz / light_space_pos.w;
    
    // Convert to texture coordinates [0, 1]
    let shadow_coord = vec2<f32>(
        ndc.x * 0.5 + 0.5,
        -ndc.y * 0.5 + 0.5  // Flip Y for texture coordinates
    );
    
    // Check if position is within shadow map bounds
    if (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 ||
        shadow_coord.y < 0.0 || shadow_coord.y > 1.0) {
        return 1.0; // Outside shadow map, fully lit
    }
    
    let depth = ndc.z - bias;
    
    // 3x3 PCF kernel for smooth shadows
    let texel_size = 1.0 / 2048.0; // Shadow map resolution
    var shadow = 0.0;
    
    for (var x = -1; x <= 1; x = x + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_coord = shadow_coord + offset;
            
            shadow += textureSampleCompareLevel(
                shadow_map,
                shadow_sampler,
                sample_coord,
                cascade_idx,
                depth
            );
        }
    }
    
    return shadow / 9.0; // Average of 9 samples
}

// Sample shadow with smooth cascade transitions
fn sample_shadow_cascaded(world_pos: vec3<f32>, view_depth: f32, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    // Select cascades for blending
    let cascade_indices = select_cascade(view_depth);
    let cascade_idx = cascade_indices.x;
    let next_cascade_idx = cascade_indices.y;
    
    // Calculate slope-based bias to prevent shadow acne
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let bias = max(0.005 * (1.0 - n_dot_l), 0.001);
    
    // Sample primary cascade
    let shadow1 = sample_shadow_pcf(world_pos, cascade_idx, bias);
    
    // Calculate blend factor
    let blend = calculate_blend_factor(view_depth, cascade_idx);
    
    // If in blend region, sample next cascade and blend
    if (blend > 0.0 && cascade_idx != next_cascade_idx) {
        let shadow2 = sample_shadow_pcf(world_pos, next_cascade_idx, bias);
        return mix(shadow1, shadow2, blend);
    }
    
    return shadow1;
}

// PBR functions
const PI: f32 = 3.14159265359;

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let nom = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return nom / denom;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    
    let nom = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    
    return nom / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// Fragment shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    let albedo_sample = textureSample(albedo_texture, albedo_sampler, input.uv);
    let albedo_color = material.albedo * albedo_sample;
    
    let normal_sample = textureSample(normal_texture, normal_sampler, input.uv);
    let metallic_roughness_sample = textureSample(metallic_roughness_texture, metallic_roughness_sampler, input.uv);
    
    // Extract material properties
    let metallic = material.metallic * metallic_roughness_sample.b;
    let roughness = material.roughness * metallic_roughness_sample.g;
    
    // Normal mapping
    let tangent_normal = normal_sample.xyz * 2.0 - 1.0;
    let TBN = mat3x3<f32>(
        normalize(input.world_tangent),
        normalize(input.world_bitangent),
        normalize(input.world_normal)
    );
    let N = normalize(TBN * tangent_normal);
    
    let V = normalize(camera.camera_pos - input.world_position);
    
    // Calculate reflectance at normal incidence
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo_color.rgb, metallic);
    
    // Reflectance equation
    var Lo = vec3<f32>(0.0);
    
    // Directional lights with cascaded shadows
    for (var i = 0u; i < lights.directional_count; i = i + 1u) {
        let light = lights.directional_lights[i];
        let L = normalize(-light.direction);
        let H = normalize(V + L);
        
        // Sample cascaded shadow map with smooth transitions
        let shadow = sample_shadow_cascaded(input.world_position, input.view_depth, N, L);
        
        let radiance = light.color * light.intensity * shadow;
        
        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        let kS = F;
        var kD = vec3<f32>(1.0) - kS;
        kD = kD * (1.0 - metallic);
        
        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;
        
        let NdotL = max(dot(N, L), 0.0);
        Lo = Lo + (kD * albedo_color.rgb / PI + specular) * radiance * NdotL;
    }
    
    // Point lights (no shadows for now - can be added later)
    for (var i = 0u; i < lights.point_count; i = i + 1u) {
        let light = lights.point_lights[i];
        let L = normalize(light.position - input.world_position);
        let H = normalize(V + L);
        let distance = length(light.position - input.world_position);
        
        // Attenuation
        let attenuation = 1.0 / (distance * distance);
        let radiance = light.color * light.intensity * attenuation;
        
        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        let kS = F;
        var kD = vec3<f32>(1.0) - kS;
        kD = kD * (1.0 - metallic);
        
        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;
        
        let NdotL = max(dot(N, L), 0.0);
        Lo = Lo + (kD * albedo_color.rgb / PI + specular) * radiance * NdotL;
    }
    
    // Ambient lighting (simple approximation)
    let ambient = vec3<f32>(0.03) * albedo_color.rgb;
    var color = ambient + Lo + material.emissive;
    
    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, albedo_color.a);
}
