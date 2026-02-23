//! Director Box (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct DirectorState {
    pub theme: Arc<Theme>,
    pub playing: bool,
    pub current_frame: u32,
    pub fps: f32,
}

impl DirectorState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            playing: false,
            current_frame: 0,
            fps: 30.0,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let bg = self.theme.colors.background;

        VStack::new(cx, |cx| {
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(bg);
    }
}
