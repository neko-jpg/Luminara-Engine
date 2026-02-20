//! Director Toolbar Component
//!
//! The Director toolbar contains:
//! - Director title/icon
//! - Timeline selector dropdown
//! - Transport controls (step backward, play, pause, stop, step forward)
//! - Time display (current / total)
//! - Loop and speed controls
//! - Status indicators (track count, etc.)

use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// The Director Toolbar component
pub struct DirectorToolbar {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Currently selected timeline name
    selected_timeline: String,
    /// Current time display
    current_time: f32,
    /// Total duration
    duration: f32,
    /// Whether playback is looping
    #[allow(dead_code)]
    is_looping: bool,
    /// Playback speed multiplier
    playback_speed: f32,
    /// Number of tracks
    track_count: u32,
}

impl DirectorToolbar {
    /// Create a new Director Toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            selected_timeline: "Intro Cutscene".to_string(),
            current_time: 1.5,
            duration: 5.0,
            is_looping: true,
            playback_speed: 1.0,
            track_count: 12,
        }
    }

    /// Render the Director title/icon section
    fn render_title(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .child(
                // Film icon (using text as placeholder for SVG)
                div()
                    .text_color(theme.colors.accent)
                    .text_size(theme.typography.lg)
                    .child("▶")
            )
            .child(
                div()
                    .text_color(theme.colors.accent)
                    .text_size(theme.typography.ml)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Director")
            )
    }

    /// Render the timeline selector dropdown
    fn render_timeline_selector(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(12.0))
            .py(px(4.0))
            .bg(theme.colors.surface_active)
            .border_1()
            .border_color(theme.colors.border)
            .rounded(px(16.0))
            .cursor_pointer()
            .hover(|this| this.bg(theme.colors.surface_hover))
            .child(
                // Timeline icon
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("◫")
            )
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.sm)
                    .child(format!("Timeline: {}", self.selected_timeline))
            )
            .child(
                // Chevron down
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("▼")
            )
    }

    /// Render the transport controls (play, pause, stop, etc.)
    fn render_transport_controls(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let buttons = vec![
            ("⏮", "Step Backward"),
            ("▶", "Play"),
            ("⏸", "Pause"),
            ("⏹", "Stop"),
            ("⏭", "Step Forward"),
        ];
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(4.0))
            .bg(theme.colors.surface_active)
            .rounded(px(24.0))
            .children(
                buttons.into_iter().map(move |(icon, _tooltip)| {
                    let theme = theme.clone();
                    
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(28.0))
                        .h(px(28.0))
                        .rounded(px(14.0))
                        .cursor_pointer()
                        .hover(|this| this.bg(theme.colors.surface_hover))
                        .child(
                            div()
                                .text_color(theme.colors.text)
                                .text_size(theme.typography.ml)
                                .child(icon.to_string())
                        )
                })
            )
    }

    /// Render the time display
    fn render_time_display(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let time_text = format!("{:.3} / {:.3} s", self.current_time, self.duration);
        
        div()
            .px(px(10.0))
            .py(px(4.0))
            .bg(theme.colors.canvas_background)
            .rounded(px(20.0))
            .child(
                div()
                    .font_family("monospace")
                    .text_size(theme.typography.ml)
                    .text_color(theme.colors.accent)
                    .child(time_text)
            )
    }

    /// Render the loop and speed controls
    fn render_loop_speed(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(12.0))
            .py(px(4.0))
            .bg(theme.colors.surface_active)
            .rounded(px(24.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("↻")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child("Loop")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("◷")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child(format!("{:.1}x", self.playback_speed))
                    )
            )
    }

    /// Render the status indicator (track count)
    fn render_status(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(12.0))
            .py(px(4.0))
            .bg(theme.colors.surface_active)
            .rounded(px(20.0))
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("◈")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child(format!("{} tracks", self.track_count))
            )
    }
}

impl Render for DirectorToolbar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(44.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(px(16.0))
            .gap(px(16.0))
            // Director title
            .child(self.render_title())
            // Timeline selector
            .child(self.render_timeline_selector())
            // Transport controls
            .child(self.render_transport_controls())
            // Time display
            .child(self.render_time_display())
            // Loop and speed
            .child(self.render_loop_speed())
            // Spacer
            .child(div().flex_1())
            // Status
            .child(self.render_status())
    }
}
