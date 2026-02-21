//! Main Toolbar Component
//!
//! Toolbar with transform tools and status bar matching HTML prototype

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    ClickEvent,
};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::ui::components::Button;
use crate::core::state::EditorStateManager;

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

    pub fn shortcut(&self) -> &'static str {
        match self {
            ToolMode::Move => "W",
            ToolMode::Rotate => "E",
            ToolMode::Scale => "R",
            ToolMode::Select => "Q",
        }
    }
}

/// Main toolbar component
pub struct MainToolbar {
    theme: Arc<Theme>,
    state: gpui::Model<EditorStateManager>,
    fps: u32,
    entity_count: u32,
    database_name: String,
    ai_status: String,
}

impl MainToolbar {
    /// Create a new toolbar
    pub fn new(theme: Arc<Theme>, state: gpui::Model<EditorStateManager>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe(&state, |_this: &mut MainToolbar, _model, cx| {
            cx.notify();
        }).detach();

        Self {
            theme,
            state,
            fps: 120,
            entity_count: 32,
            database_name: "scenes".to_string(),
            ai_status: "ready".to_string(),
        }
    }

    /// Update FPS
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps;
    }

    /// Update entity count
    pub fn set_entity_count(&mut self, count: u32) {
        self.entity_count = count;
    }

    /// Enter play mode
    fn enter_play_mode(&self, cx: &mut ViewContext<Self>) {
        self.state.update(cx, |state, cx| {
            state.set_editor_mode("Play".to_string(), cx);
        });
    }

    /// Enter pause mode
    fn enter_pause_mode(&self, cx: &mut ViewContext<Self>) {
        self.state.update(cx, |state, cx| {
            state.set_editor_mode("Pause".to_string(), cx);
        });
    }

    /// Stop play mode (return to edit)
    fn stop_play_mode(&self, cx: &mut ViewContext<Self>) {
        self.state.update(cx, |state, cx| {
            state.set_editor_mode("Edit".to_string(), cx);
        });
    }

    /// Render play button
    fn render_play_button(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_playing = self.state.read(cx).session.editor_mode == "Play";
        let is_paused = self.state.read(cx).session.editor_mode == "Pause";
        let state_clone = self.state.clone();

        let (icon, label, variant) = if is_playing {
            ("⏸", "Pause", crate::ui::components::ButtonVariant::Primary)
        } else if is_paused {
            ("▶", "Resume", crate::ui::components::ButtonVariant::Primary)
        } else {
            ("▶", "Play", crate::ui::components::ButtonVariant::Secondary)
        };

        crate::ui::components::Button::new("play_button", label)
            .icon(icon)
            .variant(variant)
            .on_click(move |_e, cx| {
                state_clone.update(cx, |state, cx| {
                    let next_mode = match state.session.editor_mode.as_str() {
                        "Play" => "Pause",
                        "Pause" => "Play",
                        "Edit" => "Play",
                        _ => "Play",
                    };
                    state.set_editor_mode(next_mode.to_string(), cx);
                });
            })
    }

    /// Render stop button
    fn render_stop_button(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_editing = self.state.read(cx).session.editor_mode == "Edit";
        let state_clone = self.state.clone();

        crate::ui::components::Button::new("stop_button", "Stop")
            .icon("⏹")
            .variant(crate::ui::components::ButtonVariant::Secondary)
            .disabled(is_editing)
            .on_click(move |_e, cx| {
                state_clone.update(cx, |state, cx| {
                    state.set_editor_mode("Edit".to_string(), cx);
                });
            })
    }

    /// Render tool button
    fn render_tool_button(&self, tool: ToolMode, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_active = self.state.read(cx).session.active_tool == tool.label();
        let tool_clone = tool;
        let state_clone = self.state.clone();

        crate::ui::components::Button::new(
            "tool_button",
            "",
        )
        .icon(tool.icon())
        .variant(if is_active {
            crate::ui::components::ButtonVariant::Primary
        } else {
            crate::ui::components::ButtonVariant::Secondary
        })
        .on_click(move |_e, cx| {
            state_clone.update(cx, |state, cx| {
                state.set_active_tool(tool_clone.label().to_string(), cx);
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
    fn render_status_bar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let editor_mode = self.state.read(cx).session.editor_mode.clone();

        let mode_color = match editor_mode.as_str() {
            "Edit" => theme.colors.success,
            "Play" => theme.colors.accent,
            "Pause" => theme.colors.warning,
            _ => theme.colors.success,
        };

        let mode_text = editor_mode.to_uppercase();

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
                // Mode indicator
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(mode_color)
                            .text_size(theme.typography.sm)
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(mode_text)
                    )
            )
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
            .child(self.render_stop_button(cx))
            // Separator
            .child(self.render_separator())
            // Transform tools group
            .child(self.render_tool_button(ToolMode::Move, cx))
            .child(self.render_tool_button(ToolMode::Rotate, cx))
            .child(self.render_tool_button(ToolMode::Scale, cx))
            .child(self.render_tool_button(ToolMode::Select, cx))
            // Spacer and status bar
            .child(self.render_status_bar(cx))
    }
}
