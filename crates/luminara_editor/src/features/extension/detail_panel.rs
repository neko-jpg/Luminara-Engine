//! Extension Detail Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct DetailPanelState {
    pub theme: Arc<Theme>,
    pub extension_name: Option<String>,
}

impl DetailPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            extension_name: None,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;
        let text_col = self.theme.colors.text;
        let font_md = self.theme.typography.md;
        let name = self.extension_name.clone().unwrap_or_default();

        VStack::new(cx, |cx| {
            Label::new(cx, &name)
                .font_size(font_md)
                .color(text_col);
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(surface);
    }
}
