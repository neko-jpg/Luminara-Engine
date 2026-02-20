use gpui::{
    div, px, rgb, IntoElement, InteractiveElement, ParentElement, Styled, WindowContext,
    StatefulInteractiveElement, ClickEvent, ElementId, AnyElement
};
use std::sync::Arc;
use crate::ui::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub struct Button {
    id: ElementId,
    label: String,
    icon: Option<&'static str>,
    variant: ButtonVariant,
    on_click: Option<Arc<dyn Fn(&ClickEvent, &mut WindowContext) + Send + Sync>>,
    disabled: bool,
    full_width: bool,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::Secondary,
            on_click: None,
            disabled: false,
            full_width: false,
        }
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self, full: bool) -> Self {
        self.full_width = full;
        self
    }
}

impl IntoElement for Button {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = Theme::default_dark(); // Fallback to fetching from context if available, but static for now.
        // Actually, later we can fetch theme from context if provided as global.
        
        let (bg, bg_hover, text_color, border) = match self.variant {
            ButtonVariant::Primary => (
                theme.colors.accent,
                theme.colors.accent_hover,
                theme.colors.background,
                theme.colors.accent,
            ),
            ButtonVariant::Secondary => (
                theme.colors.surface,
                theme.colors.surface_hover,
                theme.colors.text,
                theme.colors.border,
            ),
            ButtonVariant::Ghost => (
                gpui::transparent_black(),
                theme.colors.surface_hover,
                theme.colors.text,
                gpui::transparent_black(),
            ),
            ButtonVariant::Danger => (
                theme.colors.error,
                rgb(0xff6666).into(),
                theme.colors.background,
                theme.colors.error,
            ),
        };

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(theme.spacing.xs)
            .px(theme.spacing.md)
            .py(px(6.0))
            .rounded(theme.borders.md)
            .bg(if self.disabled { theme.colors.surface_hover } else { bg })
            .border_1()
            .border_color(if self.disabled { theme.colors.border } else { border })
            .text_color(if self.disabled { theme.colors.text_secondary } else { text_color })
            .text_size(theme.typography.sm);

        if self.full_width {
            el = el.w_full();
        }

        if !self.disabled {
            el = el.hover(|style| style.bg(bg_hover).cursor_pointer());
            
            if let Some(on_click) = self.on_click {
                el = el.on_click(move |e, cx| on_click(e, cx));
            }
        }

        if let Some(icon) = self.icon {
            el = el.child(
                div().child(icon).flex_none()
            );
        }

        el.child(self.label).into_any_element()
    }
}
