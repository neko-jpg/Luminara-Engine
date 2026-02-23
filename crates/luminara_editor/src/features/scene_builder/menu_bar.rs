//! Menu Bar (Vizia v0.3)
//!
//! Top menu bar: File | Edit | Assets | GameObject | Component | Window | AI | Help

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub label: String,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub items: Vec<MenuAction>,
}

impl MenuItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            items: Vec::new(),
        }
    }

    pub fn with_items(mut self, items: Vec<MenuAction>) -> Self {
        self.items = items;
        self
    }
}

#[derive(Clone)]
pub struct MenuBarState {
    pub theme: Arc<Theme>,
    pub menus: Vec<MenuItem>,
}

impl MenuBarState {
    pub fn new(theme: Arc<Theme>) -> Self {
        let menus = vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("Assets"),
            MenuItem::new("GameObject"),
            MenuItem::new("Component"),
            MenuItem::new("Window"),
            MenuItem::new("AI"),
            MenuItem::new("Help"),
        ];
        Self { theme, menus }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let font_sm = theme.typography.sm;
        let border_col = theme.colors.border;

        let menus = self.menus.clone();

        HStack::new(cx, move |cx| {
            for menu in &menus {
                let label = menu.label.clone();
                Label::new(cx, &label)
                    .font_size(font_sm)
                    .color(text_col)
                    .padding_left(Pixels(8.0))
                    .padding_right(Pixels(8.0));
            }
        })
        .height(Pixels(32.0))
        .width(Stretch(1.0))
        .background_color(panel_hdr)
        .border_width(Pixels(1.0))
        .border_color(border_col)
        .padding_top(Stretch(1.0))
        .padding_bottom(Stretch(1.0));
    }
}
