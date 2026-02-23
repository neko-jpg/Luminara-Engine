//! Logic Graph Box (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Connect,
    Pan,
    Zoom,
}

#[derive(Clone)]
pub struct LogicGraphState {
    pub theme: Arc<Theme>,
    pub tool_mode: ToolMode,
    pub show_grid: bool,
}

impl LogicGraphState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            tool_mode: ToolMode::Select,
            show_grid: true,
        }
    }

    pub fn set_tool(&mut self, tool: ToolMode) {
        self.tool_mode = tool;
    }

    pub fn build(&mut self, cx: &mut Context) {
        let canvas_bg = self.theme.colors.canvas_background;
        let bg = self.theme.colors.background;

        VStack::new(cx, |cx| {
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(canvas_bg);
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(bg);
    }
}
