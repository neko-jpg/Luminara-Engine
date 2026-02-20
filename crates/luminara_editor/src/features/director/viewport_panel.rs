//! Director Viewport Panel Component
//!
//! Displays a preview of the current timeline/cutscene with:
//! - Grid background pattern
//! - Camera path preview
//! - Viewport toolbar (Move, Rotate, Scale gizmos)
//! - Time indicator overlay
//! - Gizmo visualization

use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Gizmo mode for viewport interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    None,
    Move,
    Rotate,
    Scale,
}

/// The Director Viewport Panel component
pub struct DirectorViewportPanel {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current gizmo mode
    gizmo_mode: GizmoMode,
    /// Current playback time
    current_time: f32,
}

impl DirectorViewportPanel {
    /// Create a new Director Viewport Panel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            gizmo_mode: GizmoMode::Move,
            current_time: 1.5,
        }
    }

    /// Set the gizmo mode
    pub fn set_gizmo_mode(&mut self, mode: GizmoMode) {
        self.gizmo_mode = mode;
    }

    /// Get the current gizmo mode
    pub fn gizmo_mode(&self) -> GizmoMode {
        self.gizmo_mode
    }

    /// Render the viewport toolbar
    fn render_viewport_toolbar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let modes = vec![
            (GizmoMode::Move, "Move", "↔"),
            (GizmoMode::Rotate, "Rotate", "↻"),
            (GizmoMode::Scale, "Scale", "⤢"),
        ];
        
        div()
            .absolute()
            .top(px(12.0))
            .left(px(12.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(6.0))
            .bg(theme.colors.surface_active)
            .border_1()
            .border_color(theme.colors.border)
            .rounded(px(30.0))
            .children(
                modes.into_iter().map(move |(mode, label, icon)| {
                    let theme = theme.clone();
                    let is_active = self.gizmo_mode == mode;
                    
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded(px(16.0))
                        .bg(if is_active { 
                            theme.colors.accent 
                        } else { 
                            gpui::transparent_black() 
                        })
                        .cursor_pointer()
                        .hover(|this| {
                            if !is_active {
                                this.bg(theme.colors.surface_hover)
                            } else {
                                this
                            }
                        })
                        .child(
                            div()
                                .text_color(if is_active {
                                    theme.colors.background
                                } else {
                                    theme.colors.text
                                })
                                .text_size(theme.typography.sm)
                                .child(format!("{} {}", icon, label))
                        )
                })
            )
    }

    /// Render the gizmo in the center
    fn render_gizmo(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(4.0))
            .child(
                // Large gizmo icon
                div()
                    .text_color(theme.colors.accent)
                    .text_size(px(32.0))
                    .child("↔")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("Camera path preview")
            )
    }

    /// Render the time indicator overlay
    fn render_time_indicator(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .absolute()
            .bottom(px(12.0))
            .left(px(12.0))
            .px(px(12.0))
            .py(px(4.0))
            .bg(theme.colors.surface_hover)
            .rounded(px(20.0))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.sm)
                    .child(format!("t = {:.3}s", self.current_time))
            )
    }
}

impl Render for DirectorViewportPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.canvas_background)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_t(px(4.0))
            .relative()
            // Panel header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .items_center()
                    .px(theme.spacing.md)
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("👁")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.md)
                                    .child("Viewport Preview")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("✂")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("🎥")
                            )
                    )
            )
            // Viewport content
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .relative()
                    // Grid background pattern (using CSS-like radial gradient simulation)
                    .bg(theme.colors.background)
                    .child(
                        div()
                            .absolute()
                            .inset(px(0.0))
                            // Simulated grid pattern with small dots
                            .child(
                                div()
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(self.render_gizmo())
                            )
                    )
                    // Viewport toolbar overlay
                    .child(self.render_viewport_toolbar())
                    // Time indicator
                    .child(self.render_time_indicator())
            )
    }
}
