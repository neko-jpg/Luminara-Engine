//! Installed Extensions Panel (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct InstalledExtension {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct InstalledPanelState {
    pub theme: Arc<Theme>,
    pub extensions: Vec<InstalledExtension>,
}

impl InstalledPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            extensions: Vec::new(),
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
