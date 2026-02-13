// Project vector field to be divergence free in spectral domain
// u_hat = u_hat - k (k . u_hat) / |k|^2

@group(0) @binding(0) var<storage, read_write> u_hat_x_real: array<f32>;
@group(0) @binding(1) var<storage, read_write> u_hat_x_imag: array<f32>;
@group(0) @binding(2) var<storage, read_write> u_hat_y_real: array<f32>;
@group(0) @binding(3) var<storage, read_write> u_hat_y_imag: array<f32>;

struct Uniforms {
    width: u32,
    height: u32,
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

    var kx: f32;
    if (x <= w / 2u) { kx = f32(x); } else { kx = f32(x) - f32(w); }

    var ky: f32;
    if (y <= h / 2u) { ky = f32(y); } else { ky = f32(y) - f32(h); }

    let k2 = kx*kx + ky*ky;
    if (k2 < 1e-6) { return; } // Mean flow (k=0) is unchanged or zeroed depending on BC

    // dot = kx * ux + ky * uy (complex dot product)
    // dot_r = kx * ux_r + ky * uy_r
    // dot_i = kx * ux_i + ky * uy_i

    let ux_r = u_hat_x_real[index];
    let ux_i = u_hat_x_imag[index];
    let uy_r = u_hat_y_real[index];
    let uy_i = u_hat_y_imag[index];

    let dot_r = kx * ux_r + ky * uy_r;
    let dot_i = kx * ux_i + ky * uy_i;

    // correction = k * dot / k2
    let corr_x_r = kx * dot_r / k2;
    let corr_x_i = kx * dot_i / k2;
    let corr_y_r = ky * dot_r / k2;
    let corr_y_i = ky * dot_i / k2;

    u_hat_x_real[index] = ux_r - corr_x_r;
    u_hat_x_imag[index] = ux_i - corr_x_i;
    u_hat_y_real[index] = uy_r - corr_y_r;
    u_hat_y_imag[index] = uy_i - corr_y_i;
}
