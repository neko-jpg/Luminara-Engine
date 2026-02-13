//! Geometry module for spatial data structures and differential geometry.
//!
//! This module provides BVH with SAH construction, Reeb graph navigation,
//! manifold surfaces, and the Heat Method for geodesic distance computation.

pub mod bvh;
pub mod heat_method;
pub mod manifold;
pub mod reeb_graph;
pub mod sparse_matrix;

pub use bvh::*;
pub use heat_method::*;
pub use manifold::*;
pub use reeb_graph::*;
pub use sparse_matrix::*;
