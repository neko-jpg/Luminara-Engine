//! Text Input Component (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct TextInputState {
    pub theme: Arc<Theme>,
    pub placeholder: String,
}

impl TextInputState {
    pub fn new(theme: Arc<Theme>, placeholder: impl Into<String>) -> Self {
        Self {
            theme,
            placeholder: placeholder.into(),
        }
    }

    pub fn build(&self, cx: &mut Context) {
        let theme = &self.theme;
        let bg = theme.colors.surface;
        let text_col = theme.colors.text_secondary;
        let border_col = theme.colors.border;
        let border_rad = theme.borders.xs;
        let font_sz = theme.typography.md;
        let pad = theme.spacing.sm;
        let placeholder = self.placeholder.clone();

        HStack::new(cx, move |cx| {
            Label::new(cx, &placeholder)
                .font_size(font_sz)
                .color(text_col);
        })
        .background_color(bg)
        .corner_radius(Pixels(border_rad))
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_left(Pixels(pad))
        .padding_right(Pixels(pad))
        .padding_top(Pixels(pad))
        .padding_bottom(Pixels(pad))
        .height(Pixels(28.0))
        .width(Stretch(1.0));
    }
}
