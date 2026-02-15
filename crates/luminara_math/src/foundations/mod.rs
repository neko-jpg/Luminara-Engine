//! Foundations module for exact predicates and adaptive precision arithmetic.
//!
//! This module provides robust geometric predicates that never fail due to
//! floating-point errors, using adaptive precision arithmetic.

pub mod error_bounds;
pub mod exact_predicates;
pub mod expansion;

pub use error_bounds::*;
pub use exact_predicates::*;
pub use expansion::*;
