// Debug Overdraw Heatmap Shader
// Visualizes pixel overdraw as a heatmap

@group(0) @binding(0)
var overdraw_texture: texture_2d<u32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Full-screen triangle
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = textureDimensions(overdraw_texture);
    let pixel_coord = vec2<i32>(in.uv * vec2<f32>(texture_size));
    
    // Read overdraw count
    let overdraw_count = textureLoad(overdraw_texture, pixel_coord, 0).r;
    
    // Convert overdraw count to heatmap color
    // 0 draws = black
    // 1 draw = blue
    // 2-3 draws = green
    // 4-5 draws = yellow
    // 6+ draws = red
    
    var color: vec3<f32>;
    
    if (overdraw_count == 0u) {
        color = vec3<f32>(0.0, 0.0, 0.0); // Black - no draws
    } else if (overdraw_count == 1u) {
        color = vec3<f32>(0.0, 0.0, 1.0); // Blue - optimal (1 draw)
    } else if (overdraw_count <= 3u) {
        let t = f32(overdraw_count - 1u) / 2.0;
        color = mix(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), t); // Blue to green
    } else if (overdraw_count <= 5u) {
        let t = f32(overdraw_count - 3u) / 2.0;
        color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), t); // Green to yellow
    } else {
        let t = min(f32(overdraw_count - 5u) / 5.0, 1.0);
        color = mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), t); // Yellow to red
    }
    
    return vec4<f32>(color, 1.0);
}
