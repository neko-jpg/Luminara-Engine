//! Lie group integrators for structure-preserving integration on SE(3).
//!
//! Implements Munthe-Kaas RK4 method for rigid body dynamics.

use super::{Bivector, Motor};
use glam::Vec3;

/// Compute the Lie bracket [u, v] of two bivectors.
///
/// In PGA(3,0,1), the bivector Lie algebra is isomorphic to se(3).
/// The bracket is defined as the commutator: [u, v] = uv - vu.
///
/// Given the structure constants of PGA bivectors (where rotational basis elements
/// satisfy [e23, e13] = 2e12 etc.), this corresponds to twice the standard
/// vector cross product in R3.
pub fn lie_bracket(u: &Bivector, v: &Bivector) -> Bivector {
    // Extract rotational and translational parts as vectors
    let w_u = Vec3::new(u.e23, u.e13, u.e12); // (x, y, z)
    let v_u = Vec3::new(u.e01, u.e02, u.e03);

    let w_v = Vec3::new(v.e23, v.e13, v.e12);
    let v_v = Vec3::new(v.e01, v.e02, v.e03);

    // Rotational part: 2 * (w_u x w_v)
    let w_res = w_u.cross(w_v) * 2.0;

    // Translational part: 2 * (w_u x v_v - w_v x v_u)
    // Note: translations commute, so v_u x v_v term is zero
    let v_res = (w_u.cross(v_v) - w_v.cross(v_u)) * 2.0;

    Bivector {
        e23: w_res.x,
        e13: w_res.y,
        e12: w_res.z,
        e01: v_res.x,
        e02: v_res.y,
        e03: v_res.z,
    }
}

/// Compute the inverse of the derivative of exponential map (dexpinv).
///
/// Used to correct the RK stages in Munthe-Kaas methods.
/// Approximation using truncated BCH series:
/// dexpinv(u, v) ≈ v - 0.5[u, v] + (1/12)[u, [u, v]]
pub fn dexpinv(u: &Bivector, v: &Bivector) -> Bivector {
    let comm1 = lie_bracket(u, v); // [u, v]
    let comm2 = lie_bracket(u, &comm1); // [u, [u, v]]

    // v - 0.5 * comm1 + (1.0/12.0) * comm2
    let term1 = comm1.scale(0.5);
    let term2 = comm2.scale(1.0 / 12.0);

    v.sub(&term1).add(&term2)
}

/// Integrator for Lie groups using Munthe-Kaas Runge-Kutta method.
pub struct LieGroupIntegrator;

impl LieGroupIntegrator {
    /// Perform a single step of MK4 integration.
    ///
    /// # Arguments
    /// * `y` - Current state (Motor)
    /// * `h` - Time step
    /// * `f` - Differential equation y' = y * f(y), where f returns a Bivector (body velocity)
    ///
    /// # Returns
    /// The next state
    pub fn step<F>(y: Motor, h: f32, f: F) -> Motor
    where
        F: Fn(Motor) -> Bivector,
    {
        // K1, K2, K3, K4 calculation using Munthe-Kaas method
        // Note: For standard MK4, we evaluate vector field at predicted points on the group
        // The argument to dexpinv is the accumulated "u" so far.
        // Actually, for MK4, the stages are usually defined as:
        // k1 = f(y)
        // k2 = f(y * exp(h/2 * k1))
        // k3 = f(y * exp(h/2 * k2)) ... this is simple Lie Group RK.
        //
        // Munthe-Kaas form usually involves solving u' = dexpinv(u, f(y*exp(u))).
        // Let's implement the standard RKMK4.
        // Coordinates in algebra: k_i.

        // Stage 1
        let u1 = Bivector::ZERO; // u(tn) = 0
        let f1 = f(y); // f(y * exp(0))
        let k1 = dexpinv(&u1, &f1); // = f1

        // Stage 2
        let u2 = k1.scale(h * 0.5);
        let y2 = y.geometric_product(&Motor::exp(&u2));
        let f2 = f(y2);
        let k2 = dexpinv(&u2, &f2);

        // Stage 3
        let u3 = k2.scale(h * 0.5);
        let y3 = y.geometric_product(&Motor::exp(&u3));
        let f3 = f(y3);
        let k3 = dexpinv(&u3, &f3);

        // Stage 4
        let u4 = k3.scale(h);
        let y4 = y.geometric_product(&Motor::exp(&u4));
        let f4 = f(y4);
        let k4 = dexpinv(&u4, &f4);

        // Combine
        // u_next = h/6 * (k1 + 2k2 + 2k3 + k4)
        let sum = k1.add(&k2.scale(2.0)).add(&k3.scale(2.0)).add(&k4);
        let u_next = sum.scale(h / 6.0);

        // Update
        y.geometric_product(&Motor::exp(&u_next))
    }
}
