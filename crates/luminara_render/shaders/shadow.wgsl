// Shadow Map Rendering Shader
// Optimized for cascaded shadow maps with minimal GPU overhead

// Cascade uniform
struct CascadeUniform {
    view_proj: mat4x4<f32>,
    split_depth: f32,
    blend_start: f32,
    blend_end: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> cascade: CascadeUniform;

// Model transform
struct ModelTransform {
    model: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> model_transform: ModelTransform;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

// Vertex shader - minimal processing for shadow map generation
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform to world space
    let world_pos = model_transform.model * vec4<f32>(input.position, 1.0);
    
    // Transform to light space (cascade view-projection)
    output.clip_position = cascade.view_proj * world_pos;
    
    return output;
}

// Fragment shader - no output needed for depth-only rendering
// The depth buffer is automatically written
@fragment
fn fs_main(input: VertexOutput) {
    // Empty fragment shader - depth is written automatically
    // This is optimal for shadow map generation
}
