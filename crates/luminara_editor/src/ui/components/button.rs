//! Button Component (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Clone)]
pub struct ButtonState {
    pub theme: Arc<Theme>,
    pub label: String,
    pub variant: ButtonVariant,
    pub disabled: bool,
}

impl ButtonState {
    pub fn new(theme: Arc<Theme>, label: impl Into<String>) -> Self {
        Self {
            theme,
            label: label.into(),
            variant: ButtonVariant::Secondary,
            disabled: false,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn build(&self, cx: &mut Context) {
        let theme = &self.theme;
        let (bg, text_col) = match self.variant {
            ButtonVariant::Primary => (theme.colors.accent, theme.colors.background),
            ButtonVariant::Secondary => (theme.colors.surface, theme.colors.text),
            ButtonVariant::Ghost => (Color::rgba(0, 0, 0, 0), theme.colors.text),
            ButtonVariant::Danger => (theme.colors.error, theme.colors.background),
        };
        let border_col = theme.colors.border;
        let border_rad = theme.borders.xs;
        let pad_h = theme.spacing.md;
        let pad_v = theme.spacing.sm;
        let font_sz = theme.typography.sm;
        let label = self.label.clone();

        Button::new(cx, move |cx| {
            Label::new(cx, &label)
                .font_size(font_sz)
                .color(text_col)
        })
        .background_color(bg)
        .corner_radius(Pixels(border_rad))
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_left(Pixels(pad_h))
        .padding_right(Pixels(pad_h))
        .padding_top(Pixels(pad_v))
        .padding_bottom(Pixels(pad_v))
        .disabled(self.disabled);
    }
}
