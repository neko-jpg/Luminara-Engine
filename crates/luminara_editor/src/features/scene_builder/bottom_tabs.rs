//! Bottom Tab Panel (Vizia v0.3)
//!
//! Bottom panel with Console, Assets, DB Query, and AI Assistant tabs.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Console,
    Assets,
    DBQuery,
    AIAssistant,
}

impl BottomTab {
    pub fn label(&self) -> &'static str {
        match self {
            BottomTab::Console => "Console",
            BottomTab::Assets => "Asset Browser",
            BottomTab::DBQuery => "DB Query",
            BottomTab::AIAssistant => "AI Assistant",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BottomTab::Console => "🖥",
            BottomTab::Assets => "📦",
            BottomTab::DBQuery => "🗄",
            BottomTab::AIAssistant => "🤖",
        }
    }
}

#[derive(Clone)]
pub struct BottomTabPanelState {
    pub theme: Arc<Theme>,
    pub active_tab: BottomTab,
}

impl BottomTabPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tab: BottomTab::Console,
        }
    }

    pub fn set_tab(&mut self, tab: BottomTab) {
        self.active_tab = tab;
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let surface = theme.colors.surface;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let text_sec = theme.colors.text_secondary;
        let font_sm = theme.typography.sm;
        let border_col = theme.colors.border;

        let tabs = [
            BottomTab::Console,
            BottomTab::Assets,
            BottomTab::DBQuery,
            BottomTab::AIAssistant,
        ];
        let active = self.active_tab;

        VStack::new(cx, move |cx| {
            // Tab bar
            HStack::new(cx, move |cx| {
                for tab in &tabs {
                    let label = tab.label().to_string();
                    let icon = tab.icon().to_string();
                    let is_active = *tab == active;
                    let col = if is_active { text_col } else { text_sec };

                    HStack::new(cx, move |cx| {
                        Label::new(cx, &icon).font_size(font_sm).color(col);
                        Label::new(cx, &label).font_size(font_sm).color(col);
                    })
                    .padding_left(Pixels(8.0))
                    .padding_right(Pixels(8.0))
                    .padding_top(Pixels(4.0))
                    .padding_bottom(Pixels(4.0))
                    .gap(Pixels(4.0));
                }
            })
            .height(Pixels(28.0))
            .width(Stretch(1.0))
            .background_color(panel_hdr)
            .border_width(Pixels(1.0))
            .border_color(border_col);

            // Content area
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(surface);
        })
        .height(Pixels(200.0))
        .width(Stretch(1.0))
        .background_color(surface);
    }
}
