//! Luminara Editor - GPUI-based editor UI for Luminara Engine
//!
//! This crate provides a professional-grade editor interface built on the GPUI framework,
//! integrating seamlessly with Luminara Engine's ECS, Asset System, Database, and Render Pipeline.

pub mod core;
pub mod services;
pub mod ui;
pub mod features;

// Basic exports for application entry points
pub use core::app::EditorApp;
pub use core::window::EditorWindow;

// Re-export command infrastructure for plugins/extensions
pub use core::command_bus::{Command, CommandBus, CommandExecutor};
pub use core::state::EditorStateManager;

// Re-export core services
pub use services::engine_bridge::{EngineHandle, Database, RenderPipeline};
pub use services::asset_server::EditorAssetSource;

// Re-export backend services for activity integration


/// Re-export GPUI for convenience
pub use gpui;
