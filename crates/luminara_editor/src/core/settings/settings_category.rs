//! Settings Category Component

use crate::ui::theme::Theme;
use gpui::{
    div, px, IntoElement, ParentElement, RenderOnce, Styled,
};
use std::sync::Arc;

/// A category item in the settings sidebar
#[derive(Debug, Clone)]
pub struct SettingsCategoryItem {
    name: String,
    active: bool,
    theme: Arc<Theme>,
}

impl SettingsCategoryItem {
    pub fn new(name: impl Into<String>, active: bool, theme: Arc<Theme>) -> Self {
        Self {
            name: name.into(),
            active,
            theme,
        }
    }
}

impl IntoElement for SettingsCategoryItem {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .px(px(16.0))
            .py(px(10.0))
            .rounded(px(8.0))
            .bg(if self.active { self.theme.colors.toolbar_active } else { self.theme.colors.background })
            .text_color(if self.active { self.theme.colors.text } else { self.theme.colors.text_secondary })
            .text_size(px(13.0))
            .child(self.name)
    }
}
