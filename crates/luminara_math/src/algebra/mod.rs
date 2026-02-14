//! Algebra module for geometric algebra and Lie group operations.
//!
//! This module provides PGA (Projective Geometric Algebra) Motor for unified
//! rotation and translation, Lie group integrators, and dual quaternions.

pub mod bivector;
pub mod dual_quat;
pub mod lie_integrator;
pub mod motor;
pub mod transform_motor;
pub mod rotor;
pub mod point;
pub mod plane;
pub mod vector;
pub mod traits;

pub use bivector::*;
pub use dual_quat::*;
pub use lie_integrator::*;
pub use motor::*;
pub use transform_motor::*;
pub use rotor::*;
pub use point::*;
pub use plane::*;
pub use vector::*;
pub use traits::*;
