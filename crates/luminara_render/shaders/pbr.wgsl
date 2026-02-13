// PBR Forward+ Shader for Luminara Engine
// Implements physically-based rendering with tile-based light culling

// Camera uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
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
}

// Model transform (push constant or instance data)
struct ModelTransform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(3) @binding(0)
var<uniform> model_transform: ModelTransform;

// Vertex shader
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = model_transform.model * vec4<f32>(input.position, 1.0);
    output.world_position = world_pos.xyz;
    output.clip_position = camera.view_proj * world_pos;
    
    output.world_normal = normalize((model_transform.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    output.world_tangent = normalize((model_transform.model * vec4<f32>(input.tangent.xyz, 0.0)).xyz);
    output.world_bitangent = cross(output.world_normal, output.world_tangent) * input.tangent.w;
    
    output.uv = input.uv;
    
    return output;
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
    
    // Directional lights
    for (var i = 0u; i < lights.directional_count; i = i + 1u) {
        let light = lights.directional_lights[i];
        let L = normalize(-light.direction);
        let H = normalize(V + L);
        
        let radiance = light.color * light.intensity;
        
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
    
    // Point lights
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
    
    // Gamma correction (simple)
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, albedo_color.a);
}
