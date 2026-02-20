//! Main Toolbar Component
//!
//! Toolbar with transform tools and status bar matching HTML prototype

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Tool mode for transform tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Move,
    Rotate,
    Scale,
    Select,
}

impl ToolMode {
    pub fn icon(&self) -> &'static str {
        match self {
            ToolMode::Move => "↔",
            ToolMode::Rotate => "↻",
            ToolMode::Scale => "⤢",
            ToolMode::Select => "◉",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolMode::Move => "Move",
            ToolMode::Rotate => "Rotate",
            ToolMode::Scale => "Scale",
            ToolMode::Select => "Select",
        }
    }
}

/// Main toolbar component
pub struct MainToolbar {
    theme: Arc<Theme>,
    active_tool: ToolMode,
    is_playing: bool,
    fps: u32,
    entity_count: u32,
    database_name: String,
    ai_status: String,
}

impl MainToolbar {
    /// Create a new toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tool: ToolMode::Move,
            is_playing: false,
            fps: 120,
            entity_count: 32,
            database_name: "scenes".to_string(),
            ai_status: "ready".to_string(),
        }
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, tool: ToolMode) {
        self.active_tool = tool;
    }

    /// Toggle play mode
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Update FPS
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps;
    }

    /// Update entity count
    pub fn set_entity_count(&mut self, count: u32) {
        self.entity_count = count;
    }

    /// Render play button
    fn render_play_button(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_playing = self.is_playing;
        let view = cx.view().clone();

        crate::ui::components::Button::new("play_button", if is_playing { "Pause" } else { "Play" })
            .icon(if is_playing { "⏸" } else { "▶" })
            .variant(if is_playing { crate::ui::components::ButtonVariant::Primary } else { crate::ui::components::ButtonVariant::Secondary })
            .on_click(move |_e, cx| {
                view.update(cx, |this, cx| {
                    this.toggle_play();
                    cx.notify();
                });
            })
    }

    /// Render tool button
    fn render_tool_button(&self, tool: ToolMode, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_active = self.active_tool == tool;
        let tool_clone = tool;
        let view = cx.view().clone();

        crate::ui::components::Button::new(
            "tool_button",
            tool.label(),
        )
        .icon(tool.icon())
        .variant(if is_active {
            crate::ui::components::ButtonVariant::Primary
        } else {
            crate::ui::components::ButtonVariant::Ghost
        })
        .on_click(move |_e, cx| {
            view.update(cx, |this, cx| {
                this.set_active_tool(tool_clone);
                cx.notify();
            });
        })
    }

    /// Render separator
    fn render_separator(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        div()
            .w(px(1.0))
            .h(px(24.0))
            .bg(theme.colors.border)
    }

    /// Render status bar
    fn render_status_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .ml_auto()
            .flex()
            .items_center()
            .gap(theme.spacing.lg)
            .px(theme.spacing.md)
            .py(theme.spacing.xs)
            .rounded(theme.borders.rounded)
            .bg(theme.colors.background)
            .child(
                // FPS indicator
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(theme.colors.success)
                            .text_size(theme.typography.sm)
                            .child("◉")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child(format!("{} FPS", self.fps))
                    )
            )
            .child(
                // Entity count
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(theme.colors.accent)
                            .text_size(theme.typography.sm)
                            .child("◆")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child(format!("{} Entities", self.entity_count))
                    )
            )
            .child(
                // Database
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(theme.colors.warning)
                            .text_size(theme.typography.sm)
                            .child("◉")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child(format!("DB: {}", self.database_name))
                    )
            )
            .child(
                // AI status
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(theme.colors.port_true)
                            .text_size(theme.typography.sm)
                            .child("◉")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child(format!("AI {}", self.ai_status))
                    )
            )
    }
}

impl Render for MainToolbar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(40.0))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.sm)
            // Play/Pause/Stop buttons group
            .child(self.render_play_button(cx))
            .child(
                crate::ui::components::Button::new("stop_button", "Stop")
                    .icon("⏹")
                    .variant(crate::ui::components::ButtonVariant::Secondary)
                    .on_click(|_e, _cx| {
                        // Dummy click handler for stop button mockup
                        // Note: cx.notify() requires EntityId in this GPUI version
                    })
            )
            // Separator
            .child(self.render_separator())
            // Transform tools group
            .child(self.render_tool_button(ToolMode::Move, cx))
            .child(self.render_tool_button(ToolMode::Rotate, cx))
            .child(self.render_tool_button(ToolMode::Scale, cx))
            .child(self.render_tool_button(ToolMode::Select, cx))
            // Spacer and status bar
            .child(self.render_status_bar())
    }
}
