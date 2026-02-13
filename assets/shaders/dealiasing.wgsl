// Dealiasing using 2/3 rule
// Zero out high frequencies |k| >= 2/3 k_max

@group(0) @binding(0) var<storage, read_write> u_hat_real: array<f32>;
@group(0) @binding(1) var<storage, read_write> u_hat_imag: array<f32>;

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

    let kx_max = f32(w) / 2.0;
    let ky_max = f32(h) / 2.0;

    var kx: f32;
    if (x <= w / 2u) { kx = f32(x); } else { kx = f32(x) - f32(w); }

    var ky: f32;
    if (y <= h / 2u) { ky = f32(y); } else { ky = f32(y) - f32(h); }

    // 2/3 rule
    if (abs(kx) > kx_max * 0.66666 || abs(ky) > ky_max * 0.66666) {
        u_hat_real[index] = 0.0;
        u_hat_imag[index] = 0.0;
    }
}
