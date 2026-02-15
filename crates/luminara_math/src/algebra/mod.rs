//! Algebra module for geometric algebra and Lie group operations.
//!
//! This module provides PGA (Projective Geometric Algebra) Motor for unified
//! rotation and translation, Lie group integrators, and dual quaternions.

pub mod bivector;
pub mod dual_quat;
pub mod lie_integrator;
pub mod motor;
pub mod plane;
pub mod point;
pub mod rotor;
pub mod traits;
pub mod transform_motor;
pub mod vector;

pub use bivector::*;
pub use dual_quat::*;
pub use lie_integrator::*;
pub use motor::*;
pub use plane::*;
pub use point::*;
pub use rotor::*;
pub use traits::*;
pub use transform_motor::*;
pub use vector::*;
