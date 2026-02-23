//! Luminara Editor - Vizia-based editor UI for Luminara Engine
//!
//! This crate provides a professional-grade editor interface built on the Vizia framework,
//! integrating seamlessly with Luminara Engine's ECS, Asset System, Database, and Render Pipeline.

pub mod core;
pub mod features;
pub mod rendering;
pub mod services;
pub mod ui;

pub use core::app::EditorApp;
pub use core::window::EditorWindowState;

pub use core::command_bus::{Command, CommandBus, CommandExecutor};
pub use core::state::EditorStateManager;

pub use services::asset_server::EditorAssetSource;
pub use services::engine_bridge::{Database, EngineHandle, RenderPipeline};

pub use vizia;
