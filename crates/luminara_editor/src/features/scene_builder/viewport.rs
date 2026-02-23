//! Viewport Panel Component (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

pub struct GridBackground;

#[derive(Clone)]
pub struct Viewport3DState {
    pub theme: Arc<Theme>,
    pub is_playing: bool,
}

impl Viewport3DState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            is_playing: false,
        }
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    pub fn build(&mut self, cx: &mut Context) {
        let canvas_bg = self.theme.colors.canvas_background;

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
    }
}
