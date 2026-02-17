// Debug Normal Visualization Shader
// Visualizes surface normals as RGB colors

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert normal from [-1, 1] to [0, 1] for RGB visualization
    // X -> Red, Y -> Green, Z -> Blue
    let color = (in.normal + vec3<f32>(1.0)) * 0.5;
    return vec4<f32>(color, 1.0);
}
