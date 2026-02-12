//! # Luminara Core
//!
//! The core crate of the Luminara Engine, providing the Entity-Component-System (ECS)
//! framework and main application loop management.

pub mod app;
pub mod archetype;
pub mod bundle;
pub mod change_detection;
pub mod commands;
pub mod component;
pub mod entity;
pub mod event;
pub mod plugin;
pub mod query;
pub mod resource;
pub mod schedule;
pub mod shared_types;
pub mod system;
pub mod time;
pub mod world;

// Re-export core types for convenience
pub use app::App;
pub use bundle::Bundle;
pub use commands::Commands;
pub use component::Component;
pub use entity::Entity;
pub use event::{EventReader, EventWriter, Events};
pub use plugin::Plugin;
pub use query::{Added, Changed, Query, With, Without};
pub use resource::{Res, ResMut, Resource};
pub use shared_types::{AppInterface, CoreStage};
pub use system::{ExclusiveMarker, IntoSystem, System, SystemParam};
pub use time::Time;
pub use world::World;
