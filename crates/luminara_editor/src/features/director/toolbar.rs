//! Director Toolbar (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct DirectorToolbarState {
    pub theme: Arc<Theme>,
    pub playing: bool,
}

impl DirectorToolbarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            playing: false,
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
