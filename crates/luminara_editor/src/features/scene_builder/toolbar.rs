//! Main Toolbar (Vizia v0.3)
//!
//! Toolbar with transform tools and status bar.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

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

#[derive(Clone)]
pub struct MainToolbarState {
    pub theme: Arc<Theme>,
    pub active_tool: ToolMode,
    pub is_playing: bool,
    pub fps: f32,
}

impl MainToolbarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tool: ToolMode::Select,
            is_playing: false,
            fps: 0.0,
        }
    }

    pub fn set_tool(&mut self, tool: ToolMode) {
        self.active_tool = tool;
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = self.theme.clone();
        let toolbar_bg = theme.colors.toolbar_bg;
        let text_col = theme.colors.text;
        let text_sec = theme.colors.text_secondary;
        let font_sm = theme.typography.sm;
        let border_col = theme.colors.border;
        let is_playing = self.is_playing;

        let tools = [ToolMode::Select, ToolMode::Move, ToolMode::Rotate, ToolMode::Scale];
        let active = self.active_tool;

        HStack::new(cx, move |cx| {
            // Tool buttons
            for tool in &tools {
                let icon = tool.icon().to_string();
                let is_active = *tool == active;
                let col = if is_active { text_col } else { text_sec };
                let btn_bg = if is_active {
                    theme.colors.toolbar_active
                } else {
                    toolbar_bg
                };

                VStack::new(cx, move |cx| {
                    Label::new(cx, &icon)
                        .font_size(font_sm)
                        .color(col)
                        .text_align(TextAlign::Center);
                })
                .width(Pixels(36.0))
                .height(Pixels(36.0))
                .background_color(btn_bg)
                .corner_radius(Pixels(4.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0));
            }

            // Spacer
            Element::new(cx).width(Stretch(1.0));

            // Play button
            let play_text = if is_playing { "⏸" } else { "▶" };
            Label::new(cx, play_text)
                .font_size(font_sm)
                .color(text_col)
                .padding_right(Pixels(12.0));
        })
        .height(Pixels(44.0))
        .width(Stretch(1.0))
        .background_color(toolbar_bg)
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_left(Pixels(8.0))
        .padding_top(Stretch(1.0))
        .padding_bottom(Stretch(1.0))
        .gap(Pixels(4.0));
    }
}
