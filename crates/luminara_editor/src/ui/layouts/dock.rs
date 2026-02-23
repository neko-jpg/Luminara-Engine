//! Dock Layout (Vizia v0.3)
//!
//! Provides a docking system for panel arrangement.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPosition {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

#[derive(Clone)]
pub struct DockState {
    pub theme: Arc<Theme>,
}

impl DockState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self { theme }
    }

    pub fn build(&self, cx: &mut Context) {
        let bg = self.theme.colors.background;

        HStack::new(cx, |cx| {
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(bg);
    }
}
