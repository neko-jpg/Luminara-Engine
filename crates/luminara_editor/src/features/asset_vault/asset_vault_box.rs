//! Asset Vault Box (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct AssetVaultState {
    pub theme: Arc<Theme>,
    pub search_query: String,
}

impl AssetVaultState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            search_query: String::new(),
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let bg = self.theme.colors.background;

        VStack::new(cx, |cx| {
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(bg);
    }
}
