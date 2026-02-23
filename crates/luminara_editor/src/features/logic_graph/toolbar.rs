//! Logic Graph Toolbar (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Pan,
    Zoom,
    Connect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub struct StatusBarInfo {
    pub node_count: usize,
    pub connection_count: usize,
    pub zoom_level: f32,
}

#[derive(Clone)]
pub struct LogicGraphToolbarState {
    pub theme: Arc<Theme>,
    pub active_tool: Tool,
    pub simulation_state: SimulationState,
}

impl LogicGraphToolbarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tool: Tool::Select,
            simulation_state: SimulationState::Stopped,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let toolbar_bg = self.theme.colors.toolbar_bg;

        HStack::new(cx, |_cx| {})
            .height(Pixels(44.0))
            .width(Stretch(1.0))
            .background_color(toolbar_bg);
    }
}
