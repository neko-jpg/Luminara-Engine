//! Workspace Layout Constants & Helpers (Vizia v0.3)
//!
//! Standard layout structure for editor feature panels.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

/// Standard dimensions
pub const MENU_BAR_HEIGHT: f32 = 32.0;
pub const TOOLBAR_HEIGHT: f32 = 44.0;
pub const BOTTOM_PANEL_HEIGHT: f32 = 200.0;
pub const LEFT_PANEL_WIDTH: f32 = 260.0;
pub const RIGHT_PANEL_WIDTH: f32 = 320.0;

pub struct WorkspaceConstants;

impl WorkspaceConstants {
    pub fn menu_bar_height() -> f32 {
        MENU_BAR_HEIGHT
    }
    pub fn toolbar_height() -> f32 {
        TOOLBAR_HEIGHT
    }
    pub fn bottom_panel_height() -> f32 {
        BOTTOM_PANEL_HEIGHT
    }
}
