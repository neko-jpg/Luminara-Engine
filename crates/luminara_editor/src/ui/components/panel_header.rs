//! Panel Header Component (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct PanelHeaderState {
    pub theme: Arc<Theme>,
    pub title: String,
}

impl PanelHeaderState {
    pub fn new(theme: Arc<Theme>, title: impl Into<String>) -> Self {
        Self {
            theme,
            title: title.into(),
        }
    }

    pub fn build(&self, cx: &mut Context) {
        let theme = &self.theme;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let border_col = theme.colors.border;
        let font_sm = theme.typography.sm;
        let title = self.title.clone();

        HStack::new(cx, move |cx| {
            Label::new(cx, &title)
                .font_size(font_sm)
                .color(text_col)
                .font_weight(FontWeight(700));
        })
        .height(Pixels(28.0))
        .width(Stretch(1.0))
        .background_color(panel_hdr)
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_left(Pixels(8.0))
        .padding_top(Stretch(1.0))
        .padding_bottom(Stretch(1.0));
    }
}
