//! # Luminara Core
//!
//! The core crate of the Luminara Engine, providing the Entity-Component-System (ECS)
//! framework and main application loop management.

// Re-export the Reflect derive macro
pub use luminara_reflect_derive::Reflect;

pub mod app;
pub mod archetype;
pub mod atomic_command;
pub mod bundle;
pub mod change_detection;
pub mod command_dependencies;
pub mod commands;
pub mod component;
pub mod console;
pub mod entity;
pub mod error;
pub mod event;
pub mod plugin;
pub mod query;
pub mod reflect;
pub mod resource;
pub mod schedule;
pub mod shared_types;
pub mod system;
pub mod time;
pub mod undo_command;
pub mod world;

// Re-export core types for convenience
pub use app::App;
pub use atomic_command::AtomicCommand;
pub use bundle::Bundle;
pub use command_dependencies::{CommandId, DependencyGraph, DependentCommand};
#[cfg(feature = "math")]
pub use commands::ModifyTransformCommand;
pub use commands::{
    AddComponentCommand, DestroyEntityCommand, ModifyComponentCommand, RemoveComponentCommand,
    SpawnEntityCommand,
};
pub use component::Component;
pub use entity::Entity;
pub use error::WorldError;
pub use event::{EventReader, EventWriter, Events};
pub use plugin::Plugin;
pub use query::{Added, Changed, Query, With, Without};
pub use reflect::{FieldInfo, Reflect, ReflectError, ReflectRegistry, TypeInfo, TypeKind};
pub use resource::{Res, ResMut, Resource};
pub use shared_types::{AppInterface, CoreStage};
pub use system::{ExclusiveMarker, IntoSystem, System, SystemParam};
pub use time::Time;
pub use undo_command::{CommandError, CommandHistory, CommandResult, UndoCommand};
pub use world::World;
