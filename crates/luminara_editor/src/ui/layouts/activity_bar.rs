//! Activity Bar Layout (Vizia v0.3)
//!
//! VS Code-style vertical icon bar on the left edge of the editor window.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

pub const ACTIVITY_BAR_WIDTH: f32 = 52.0;

#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub id: String,
    pub icon: String,
    pub title: String,
    pub badge: Option<u32>,
}

impl ActivityItem {
    pub fn new(id: &str, icon: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            icon: icon.to_string(),
            title: title.to_string(),
            badge: None,
        }
    }

    pub fn with_badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }
}

#[derive(Clone)]
pub struct ActivityBarState {
    pub theme: Arc<Theme>,
    pub items: Vec<ActivityItem>,
    pub active_index: usize,
    pub bottom_items: Vec<ActivityItem>,
}

impl ActivityBarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            items: Vec::new(),
            active_index: 0,
            bottom_items: Vec::new(),
        }
    }

    pub fn with_items(mut self, items: Vec<ActivityItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_bottom_items(mut self, items: Vec<ActivityItem>) -> Self {
        self.bottom_items = items;
        self
    }

    pub fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }

    pub fn build(&self, cx: &mut Context) {
        let theme = &self.theme;
        let bg = theme.colors.background;
        let border_col = theme.colors.border;
        let text_col = theme.colors.text_secondary;
        let _accent = theme.colors.accent;
        let font_xl = theme.typography.xl;

        let items = self.items.clone();
        let bottom_items = self.bottom_items.clone();
        let active_index = self.active_index;

        VStack::new(cx, move |cx| {
            // Top items
            for (i, item) in items.iter().enumerate() {
                let icon = item.icon.clone();
                let is_active = i == active_index;
                let item_text = if is_active {
                    theme.colors.text
                } else {
                    text_col
                };

                VStack::new(cx, move |cx| {
                    Label::new(cx, &icon)
                        .font_size(font_xl)
                        .color(item_text)
                        .text_align(TextAlign::Center);
                })
                .width(Pixels(ACTIVITY_BAR_WIDTH))
                .height(Pixels(48.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0));
            }

            // Spacer
            Element::new(cx).height(Stretch(1.0));

            // Bottom items
            for item in bottom_items.iter() {
                let icon = item.icon.clone();

                VStack::new(cx, move |cx| {
                    Label::new(cx, &icon)
                        .font_size(font_xl)
                        .color(text_col)
                        .text_align(TextAlign::Center);
                })
                .width(Pixels(ACTIVITY_BAR_WIDTH))
                .height(Pixels(48.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0));
            }
        })
        .width(Pixels(ACTIVITY_BAR_WIDTH))
        .height(Stretch(1.0))
        .background_color(bg)
        .border_width(Pixels(1.0))
        .border_color(border_col);
    }
}
