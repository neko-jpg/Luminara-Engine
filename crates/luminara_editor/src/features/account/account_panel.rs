//! Account Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone)]
pub struct AccountPanelState {
    pub theme: Arc<Theme>,
    pub user: Option<UserProfile>,
    pub signed_in: bool,
}

impl AccountPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            user: None,
            signed_in: false,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;
        let text_col = self.theme.colors.text;
        let text_muted = self.theme.colors.text_secondary;
        let font_md = self.theme.typography.md;
        let font_sm = self.theme.typography.sm;
        let border_col = self.theme.colors.border;

        VStack::new(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "Account")
                    .font_size(font_md)
                    .color(text_col)
                    .font_weight(FontWeight(700));
            })
            .height(Pixels(36.0))
            .width(Stretch(1.0))
            .padding_left(Pixels(12.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0));

            // Content
            VStack::new(cx, |cx| {
                Label::new(cx, "Sign in to sync settings")
                    .font_size(font_sm)
                    .color(text_muted);
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .padding_left(Pixels(12.0))
            .padding_top(Pixels(12.0));
        })
        .width(Pixels(280.0))
        .height(Pixels(200.0))
        .background_color(surface)
        .corner_radius(Pixels(6.0))
        .border_width(Pixels(1.0))
        .border_color(border_col);
    }
}
