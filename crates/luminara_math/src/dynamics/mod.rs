//! Dynamics module for fluid simulation and FFT utilities.
//!
//! This module provides a spectral method fluid solver using FFT for
//! solving the incompressible Navier-Stokes equations on GPU.

pub mod fft;
pub mod fluid_viz;
pub mod spectral_fluid;

pub use fft::*;
pub use fluid_viz::*;
pub use spectral_fluid::*;
