//! Bottom Tab Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicGraphBottomTabKind {
    DbQuery,
    AiAssistant,
    NodePalette,
    Variables,
}

#[derive(Clone)]
pub struct LogicGraphBottomTabPanelState {
    pub theme: Arc<Theme>,
    pub active_tab: LogicGraphBottomTabKind,
}

impl LogicGraphBottomTabPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tab: LogicGraphBottomTabKind::NodePalette,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;

        VStack::new(cx, |_cx| {})
            .height(Pixels(200.0))
            .width(Stretch(1.0))
            .background_color(surface);
    }
}
