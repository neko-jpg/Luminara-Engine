// Time integration (IMEX)
// u_new = exp(-nu * k^2 * dt) * (u + dt * nonlinear)

@group(0) @binding(0) var<storage, read_write> u_hat_real: array<f32>;
@group(0) @binding(1) var<storage, read_write> u_hat_imag: array<f32>;
@group(0) @binding(2) var<storage, read> nonlinear_hat_real: array<f32>;
@group(0) @binding(3) var<storage, read> nonlinear_hat_imag: array<f32>;

struct Uniforms {
    width: u32,
    height: u32,
    dt: f32,
    viscosity: f32,
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

    // Exact integration factor for diffusion
    let decay = exp(-params.viscosity * k2 * params.dt);

    let u_r = u_hat_real[index];
    let u_i = u_hat_imag[index];
    let nl_r = nonlinear_hat_real[index];
    let nl_i = nonlinear_hat_imag[index];

    // Euler step for nonlinear, Exact for linear
    let next_r = (u_r + params.dt * nl_r) * decay;
    let next_i = (u_i + params.dt * nl_i) * decay;

    u_hat_real[index] = next_r;
    u_hat_imag[index] = next_i;
}
