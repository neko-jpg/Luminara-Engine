//! Asset Grid (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub name: String,
    pub kind: String,
    pub path: String,
}

#[derive(Clone)]
pub struct AssetGridState {
    pub theme: Arc<Theme>,
    pub assets: Vec<AssetEntry>,
    pub selected_index: Option<usize>,
}

impl AssetGridState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            assets: Vec::new(),
            selected_index: None,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(surface);
    }
}
