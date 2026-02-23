//! Global Search (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPrefix {
    File,
    Command,
    Symbol,
    Settings,
    Debug,
    Extension,
    Git,
    Default,
}

impl SearchPrefix {
    pub fn parse(query: &str) -> (Self, &str) {
        if let Some(rest) = query.strip_prefix('>') {
            (SearchPrefix::Command, rest.trim())
        } else if let Some(rest) = query.strip_prefix('#') {
            (SearchPrefix::Symbol, rest.trim())
        } else if let Some(rest) = query.strip_prefix('@') {
            (SearchPrefix::Settings, rest.trim())
        } else if let Some(rest) = query.strip_prefix('!') {
            (SearchPrefix::Debug, rest.trim())
        } else if let Some(rest) = query.strip_prefix("ext:") {
            (SearchPrefix::Extension, rest.trim())
        } else if let Some(rest) = query.strip_prefix("git:") {
            (SearchPrefix::Git, rest.trim())
        } else {
            (SearchPrefix::File, query)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub label: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Default)]
pub struct GroupedResults {
    pub files: Vec<SearchResult>,
    pub commands: Vec<SearchResult>,
    pub symbols: Vec<SearchResult>,
}

#[derive(Clone)]
pub struct GlobalSearchState {
    pub theme: Arc<Theme>,
    pub query: String,
    pub results: GroupedResults,
    pub visible: bool,
}

impl GlobalSearchState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            query: String::new(),
            results: GroupedResults::default(),
            visible: false,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;
        let text_col = self.theme.colors.text;
        let text_muted = self.theme.colors.text_secondary;
        let font_md = self.theme.typography.md;
        let border_col = self.theme.colors.border;

        VStack::new(cx, |cx| {
            // Search input area
            HStack::new(cx, |cx| {
                Label::new(cx, "🔍")
                    .font_size(font_md)
                    .color(text_muted);
                Label::new(cx, "Search files, commands (>), symbols (#)...")
                    .font_size(font_md)
                    .color(text_muted);
            })
            .height(Pixels(36.0))
            .width(Stretch(1.0))
            .padding_left(Pixels(12.0))
            .padding_right(Pixels(12.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0))
            .background_color(surface)
            .border_width(Pixels(1.0))
            .border_color(border_col)
            .corner_radius(Pixels(6.0));

            // Results area
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(300.0))
                .background_color(surface);
        })
        .width(Pixels(600.0))
        .background_color(surface)
        .corner_radius(Pixels(8.0))
        .border_width(Pixels(1.0))
        .border_color(border_col);
    }
}
