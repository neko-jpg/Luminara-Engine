use gpui::{
    div, px, IntoElement, ParentElement, Styled,
    FontWeight, AnyElement
};
use crate::ui::theme::Theme;

/// A generic panel header component used across different editor panels
pub struct PanelHeader {
    title: String,
    icon: Option<&'static str>,
    right_element: Option<gpui::AnyElement>,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            right_element: None,
        }
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn right_element(mut self, el: impl IntoElement) -> Self {
        self.right_element = Some(el.into_any_element());
        self
    }
}

impl IntoElement for PanelHeader {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = Theme::default_dark();

        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(32.0))
            .px(theme.spacing.sm)
            .bg(theme.colors.panel_header)
            .border_b_1()
            .border_color(theme.colors.border);

        let mut left_side = div()
            .flex()
            .items_center()
            .gap(theme.spacing.xs);

        if let Some(icon) = self.icon {
            left_side = left_side.child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child(icon)
            );
        }

        left_side = left_side.child(
            div()
                .text_color(theme.colors.text)
                .text_size(theme.typography.sm)
                .font_weight(FontWeight::BOLD)
                .child(self.title)
        );

        header = header.child(left_side);

        if let Some(right) = self.right_element {
            header = header.child(right);
        }

        header.into_any_element()
    }
}
