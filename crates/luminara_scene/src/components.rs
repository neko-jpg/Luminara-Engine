//! Component implementations for math types
//!
//! This module provides Component trait implementations for types from luminara_math,
//! allowing them to be used in the ECS without creating a circular dependency.
//!
//! NOTE: Component implementations for Transform and Color have been moved to luminara_math
//! to avoid violating Rust's orphan rules. This module is kept for potential future
//! scene-specific component types.
