//! Director Viewport Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct DirectorViewportPanelState {
    pub theme: Arc<Theme>,
}

impl DirectorViewportPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self { theme }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let canvas_bg = self.theme.colors.canvas_background;

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
    }
}
