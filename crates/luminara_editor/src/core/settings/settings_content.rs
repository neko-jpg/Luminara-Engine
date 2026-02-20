//! Settings Content Components

use crate::ui::theme::Theme;
use gpui::{
    div, px, IntoElement, ParentElement, Styled,
};
use std::sync::Arc;

/// A text input component
#[derive(Debug, Clone)]
pub struct TextInput {
    value: String,
    theme: Arc<Theme>,
}

impl TextInput {
    pub fn new(value: impl Into<String>, theme: Arc<Theme>) -> Self {
        Self {
            value: value.into(),
            theme,
        }
    }
}

impl IntoElement for TextInput {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .bg(self.theme.colors.surface)
            .border_1()
            .border_color(self.theme.colors.border)
            .rounded(px(6.0))
            .px(px(12.0))
            .py(px(8.0))
            .min_w(px(200.0))
            .text_size(px(13.0))
            .text_color(self.theme.colors.text)
            .child(self.value)
    }
}
