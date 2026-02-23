//! Dropdown Component (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct DropdownState {
    pub theme: Arc<Theme>,
    pub options: Vec<String>,
    pub selected_index: Option<usize>,
}

impl DropdownState {
    pub fn new(theme: Arc<Theme>, options: Vec<String>) -> Self {
        Self {
            theme,
            options,
            selected_index: None,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    pub fn build(&self, cx: &mut Context) {
        let theme = &self.theme;
        let selected_text = self
            .selected_index
            .and_then(|i| <[String]>::get(&self.options, i))
            .cloned()
            .unwrap_or_else(|| "Select...".to_string());

        let text_col = theme.colors.text;
        let font_sz = theme.typography.md;
        let bg = theme.colors.surface;
        let border_col = theme.colors.border;
        let border_rad = theme.borders.xs;
        let pad_h = theme.spacing.md;
        let pad_v = theme.spacing.sm;

        HStack::new(cx, move |cx| {
            Label::new(cx, &selected_text)
                .font_size(font_sz)
                .color(text_col);
        })
        .background_color(bg)
        .corner_radius(Pixels(border_rad))
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_left(Pixels(pad_h))
        .padding_right(Pixels(pad_h))
        .padding_top(Pixels(pad_v))
        .padding_bottom(Pixels(pad_v))
        .height(Pixels(28.0));
    }
}
