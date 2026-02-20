//! Toolbar for Backend & AI Box
//!
//! Contains:
//! - Run button for executing scripts
//! - Mode selector (Script, Query, AI)
//! - Status bar (FPS, Entities, DB status, AI status)

use gpui::{
    div, px, IntoElement, ParentElement, Styled, svg, InteractiveElement,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Editor mode for the Backend & AI box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Script editing mode (Rust/WASM)
    Script,
    /// Database query mode (SurrealQL)
    Query,
    /// AI assistant mode
    AI,
}

impl EditorMode {
    /// Get the display label for the mode
    pub fn label(&self) -> &'static str {
        match self {
            EditorMode::Script => "Script",
            EditorMode::Query => "Query",
            EditorMode::AI => "AI",
        }
    }
    
    /// Get the icon path for the mode
    pub fn icon_path(&self) -> &'static str {
        match self {
            EditorMode::Script => "icons/code.svg",
            EditorMode::Query => "icons/database.svg",
            EditorMode::AI => "icons/robot.svg",
        }
    }
}

/// Toolbar component for Backend & AI Box
pub struct Toolbar {
    /// Current editor mode
    current_mode: EditorMode,
    /// Theme for styling
    theme: Arc<Theme>,
}

impl Toolbar {
    /// Create a new Toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            current_mode: EditorMode::Script,
            theme,
        }
    }
    
    /// Set the current editor mode
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.current_mode = mode;
    }
    
    /// Get the current editor mode
    pub fn current_mode(&self) -> EditorMode {
        self.current_mode
    }
    
    /// Render the toolbar
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let current_mode = self.current_mode;
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(40.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.sm)
            // Run button group
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .bg(theme.colors.surface)
                    .rounded(theme.borders.sm)
                    .px(theme.spacing.xs)
                    .py(px(2.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(theme.spacing.xs)
                            .px(theme.spacing.sm)
                            .py(theme.spacing.xs)
                            .rounded(theme.borders.xs)
                            .bg(theme.colors.success)
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path("icons/play.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.background)
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.background)
                                    .text_size(theme.typography.sm)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Run")
                            )
                    )
            )
            // Separator
            .child(
                div()
                    .w(px(1.0))
                    .h(px(24.0))
                    .bg(theme.colors.border)
            )
            // Mode selector
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .bg(theme.colors.surface)
                    .rounded(theme.borders.sm)
                    .px(theme.spacing.xs)
                    .py(px(2.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("Mode:")
                    )
                    .child(
                        self.render_mode_button(EditorMode::Script, current_mode == EditorMode::Script)
                    )
                    .child(
                        self.render_mode_button(EditorMode::Query, current_mode == EditorMode::Query)
                    )
                    .child(
                        self.render_mode_button(EditorMode::AI, current_mode == EditorMode::AI)
                    )
            )
            // Spacer
            .child(div().flex_1())
            // Status bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.md)
                    .bg(theme.colors.surface)
                    .rounded(theme.borders.rounded)
                    .px(theme.spacing.md)
                    .py(theme.spacing.xs)
                    .child(
                        self.render_status_item("icons/chart-line.svg", "120 FPS", theme.colors.success)
                    )
                    .child(
                        self.render_status_item("icons/cubes.svg", "32 Entities", theme.colors.text_secondary)
                    )
                    .child(
                        self.render_status_item("icons/database.svg", "DB: backend", theme.colors.accent)
                    )
                    .child(
                        self.render_status_item("icons/robot.svg", "AI ready", theme.colors.success)
                    )
            )
    }
    
    /// Render a mode selector button
    fn render_mode_button(&self, mode: EditorMode, is_active: bool) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = mode.label().to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .px(theme.spacing.sm)
            .py(theme.spacing.xs)
            .rounded(theme.borders.xs)
            .bg(if is_active {
                theme.colors.toolbar_active
            } else {
                gpui::transparent_black()
            })
            .hover(|this| {
                if !is_active {
                    this.bg(theme.colors.surface_hover)
                } else {
                    this
                }
            })
            .cursor_pointer()
            .child(
                div()
                    .text_color(if is_active {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .text_size(theme.typography.sm)
                    .child(label)
            )
    }
    
    /// Render a status bar item
    fn render_status_item(&self, icon_path: &str, text: &str, color: gpui::Hsla) -> impl IntoElement {
        let theme = self.theme.clone();
        let text = text.to_string();
        let icon_path = icon_path.to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .child(
                svg()
                    .path(icon_path)
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(color)
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child(text)
            )
    }
}
