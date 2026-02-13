//! Symbolic computation engine for expression manipulation and code generation.
//!
//! This module provides symbolic expression representation, differentiation,
//! simplification, and code generation to WGSL and Rust.

pub mod differentiation;
pub mod expr;
pub mod rust_codegen;
pub mod simplification;
pub mod wgsl_codegen;

pub use differentiation::*;
pub use expr::*;
pub use rust_codegen::*;
pub use simplification::*;
pub use wgsl_codegen::*;
