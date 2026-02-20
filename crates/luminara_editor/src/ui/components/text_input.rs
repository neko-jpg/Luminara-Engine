use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Styled, WindowContext,
    ElementId, FocusHandle, AnyElement
};
use std::sync::Arc;
use crate::ui::theme::Theme;

pub struct TextInput {
    id: ElementId,
    placeholder: String,
    value: String,
    icon: Option<&'static str>,
    focus_handle: Option<FocusHandle>,
    on_change: Option<Arc<dyn Fn(&str, &mut WindowContext) + Send + Sync>>,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            placeholder: String::new(),
            value: String::new(),
            icon: None,
            focus_handle: None,
            on_change: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn value(mut self, text: impl Into<String>) -> Self {
        self.value = text.into();
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut WindowContext) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl IntoElement for TextInput {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = Theme::default_dark();

        let base_container = div()
            .id(self.id)
            .flex()
            .items_center()
            .w_full()
            .h(px(32.0))
            .px(theme.spacing.sm)
            .gap(theme.spacing.sm)
            .rounded(theme.borders.md)
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .hover(|style| style.border_color(theme.colors.accent));

        let display_text = if self.value.is_empty() {
            div()
                .text_color(theme.colors.text_secondary)
                .child(self.placeholder)
        } else {
            div()
                .text_color(theme.colors.text)
                .child(self.value)
        };

        let content = base_container
            .children(
                self.icon.map(|icon| {
                    div()
                        .text_color(theme.colors.text_secondary)
                        .child(icon)
                })
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_size(theme.typography.sm)
                    .child(display_text)
            );

        if let Some(handle) = self.focus_handle {
            content.track_focus(&handle).into_any_element()
        } else {
            content.into_any_element()
        }
    }
}
