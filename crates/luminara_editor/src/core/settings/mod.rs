//! Settings Panel Module
//!
//! A VSCode-like full-screen settings overlay that can be accessed from the Activity Bar.
//! Features a left sidebar with categories and a right panel with settings content.

pub mod settings_panel;
pub mod settings_category;
pub mod settings_content;

pub use settings_panel::{SettingsPanel, SettingsCategory};
