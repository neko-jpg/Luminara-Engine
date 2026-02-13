// Compute spatial derivatives in spectral domain
// i * k * u_hat

@group(0) @binding(0) var<storage, read> u_hat_real: array<f32>;
@group(0) @binding(1) var<storage, read> u_hat_imag: array<f32>;
@group(0) @binding(2) var<storage, read_write> du_dx_real: array<f32>;
@group(0) @binding(3) var<storage, read_write> du_dx_imag: array<f32>;

struct Uniforms {
    width: u32,
    height: u32,
    direction: u32, // 0 for X, 1 for Y
};
@group(1) @binding(0) var<uniform> params: Uniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.width * params.height) { return; }

    let w = params.width;
    let h = params.height;

    let x = index % w;
    let y = index / w;

    // Wave numbers k = 2 * pi * n / L
    // We assume L = 2*pi for simplicity so k = n.
    // Handling aliasing: if x > w/2, k = x - w.

    var k: f32;
    if (params.direction == 0u) {
        // d/dx -> kx
        if (x <= w / 2u) {
            k = f32(x);
        } else {
            k = f32(x) - f32(w);
        }
    } else {
        // d/dy -> ky
        if (y <= h / 2u) {
            k = f32(y);
        } else {
            k = f32(y) - f32(h);
        }
    }

    // derivative = i * k * (real + i*imag) = i*k*real - k*imag
    // real_out = -k * imag
    // imag_out = k * real

    let u_r = u_hat_real[index];
    let u_i = u_hat_imag[index];

    du_dx_real[index] = -k * u_i;
    du_dx_imag[index] = k * u_r;
}
