//! Lie group integrators for structure-preserving integration on SE(3).
//!
//! Implements Munthe-Kaas RK4 method for rigid body dynamics.

use super::traits::Scalar;
use super::vector::Vector3;
use super::{Bivector, Motor};

/// Compute the Lie bracket [u, v] of two bivectors.
///
/// In PGA(3,0,1), the bivector Lie algebra is isomorphic to se(3).
/// The bracket is defined as the commutator: [u, v] = uv - vu.
///
/// Given the structure constants of PGA bivectors (where rotational basis elements
/// satisfy [e23, e13] = 2e12 etc.), this corresponds to twice the standard
/// vector cross product in R3.
pub fn lie_bracket<T: Scalar>(u: &Bivector<T>, v: &Bivector<T>) -> Bivector<T> {
    // Extract rotational and translational parts as vectors
    let w_u = Vector3::new(u.e23, u.e13, u.e12); // (x, y, z)
    let v_u = Vector3::new(u.e01, u.e02, u.e03);

    let w_v = Vector3::new(v.e23, v.e13, v.e12);
    let v_v = Vector3::new(v.e01, v.e02, v.e03);

    let two = T::one() + T::one();

    // Rotational part: 2 * (w_u x w_v)
    let w_res = w_u.cross(w_v) * two;

    // Translational part: 2 * (w_u x v_v - w_v x v_u)
    // Note: translations commute, so v_u x v_v term is zero
    let v_res = (w_u.cross(v_v) - w_v.cross(v_u)) * two;

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
pub fn dexpinv<T: Scalar>(u: &Bivector<T>, v: &Bivector<T>) -> Bivector<T> {
    let comm1 = lie_bracket(u, v); // [u, v]
    let comm2 = lie_bracket(u, &comm1); // [u, [u, v]]

    // v - 0.5 * comm1 + (1.0/12.0) * comm2
    let two = T::one() + T::one();
    let half = T::one() / two;
    let twelve = two * two * (two + T::one());
    let one_twelfth = T::one() / twelve;

    let term1 = comm1.scale(half);
    let term2 = comm2.scale(one_twelfth);

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
    pub fn step<F, T>(y: Motor<T>, h: T, f: F) -> Motor<T>
    where
        T: Scalar,
        F: Fn(Motor<T>) -> Bivector<T>,
    {
        // K1, K2, K3, K4 calculation using Munthe-Kaas method

        let two = T::one() + T::one();
        let six = two * (two + T::one());
        let half_h = h / two;

        // Stage 1
        // u1 = 0
        let zero = T::zero();
        let u1 = Bivector::new(zero, zero, zero, zero, zero, zero);
        let f1 = f(y); // f(y * exp(0))
        let k1 = dexpinv(&u1, &f1); // = f1

        // Stage 2
        let u2 = k1.scale(half_h);
        let y2 = y.geometric_product(&Motor::exp(&u2));
        let f2 = f(y2);
        let k2 = dexpinv(&u2, &f2);

        // Stage 3
        let u3 = k2.scale(half_h);
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
        let sum = k1.add(&k2.scale(two)).add(&k3.scale(two)).add(&k4);
        let u_next = sum.scale(h / six);

        // Update
        y.geometric_product(&Motor::exp(&u_next))
    }
}

/// Symplectic Euler integrator for Lie Groups.
///
/// Implements a first-order symplectic integrator that preserves the symplectic structure
/// of the phase space, making it suitable for long-term simulation of conservative systems.
///
/// The update scheme is:
/// 1. v_{n+1} = v_n + h * a(q_n)
/// 2. q_{n+1} = q_n * exp(h * v_{n+1})
pub struct SymplecticEuler;

impl SymplecticEuler {
    /// Perform a single step of Symplectic Euler integration.
    ///
    /// # Arguments
    /// * `q` - Current configuration (Motor)
    /// * `v` - Current velocity (Bivector, body frame)
    /// * `h` - Time step
    /// * `acceleration` - Function computing acceleration (Bivector) from configuration
    ///
    /// # Returns
    /// The next configuration and velocity `(q_next, v_next)`
    pub fn step<F, T>(q: Motor<T>, v: Bivector<T>, h: T, acceleration: F) -> (Motor<T>, Bivector<T>)
    where
        T: Scalar,
        F: Fn(Motor<T>) -> Bivector<T>,
    {
        // 1. Update velocity (momentum)
        // v_{n+1} = v_n + h * a(q_n)
        let acc = acceleration(q);
        let v_next = v.add(&acc.scale(h));

        // 2. Update configuration
        // q_{n+1} = q_n * exp(h * v_{n+1})
        // Note: The exponential map moves along the geodesic determined by the body velocity
        let delta = v_next.scale(h);
        let q_next = q.geometric_product(&Motor::exp(&delta));

        (q_next, v_next)
    }
}
