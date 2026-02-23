//! Marketplace Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct MarketplaceEntry {
    pub name: String,
    pub author: String,
    pub downloads: u64,
    pub description: String,
}

#[derive(Clone)]
pub struct MarketplacePanelState {
    pub theme: Arc<Theme>,
    pub entries: Vec<MarketplaceEntry>,
    pub search_query: String,
}

impl MarketplacePanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            entries: Vec::new(),
            search_query: String::new(),
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;

        VStack::new(cx, |_cx| {})
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(surface);
    }
}
