// Penalization for obstacles
// u = u * (1 - chi) + u_obs * chi
// Here assuming stationary obstacles (u_obs=0): u = u * (1 - chi)

@group(0) @binding(0) var<storage, read_write> u_real: array<f32>; // Spatial domain
@group(0) @binding(1) var<storage, read_write> u_imag: array<f32>; // Spatial domain (usually 0 if real transform, but we use complex)
@group(0) @binding(2) var<storage, read> mask: array<f32>; // 1.0 inside obstacle, 0.0 outside

struct Uniforms {
    width: u32,
    height: u32,
};
@group(1) @binding(0) var<uniform> params: Uniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.width * params.height) { return; }

    let chi = mask[index];
    let factor = 1.0 - chi;

    u_real[index] = u_real[index] * factor;
    u_imag[index] = u_imag[index] * factor;
}
