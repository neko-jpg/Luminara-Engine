//! Foundations module for exact predicates and adaptive precision arithmetic.
//!
//! This module provides robust geometric predicates that never fail due to
//! floating-point errors, using adaptive precision arithmetic.

pub mod error_bounds;
pub mod expansion;
pub mod exact_predicates;

pub use error_bounds::*;
pub use expansion::*;
pub use exact_predicates::*;
