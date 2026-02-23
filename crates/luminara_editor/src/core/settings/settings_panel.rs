//! Settings Panel (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Editor,
    Shortcuts,
    AiAssistant,
    Appearance,
    Extensions,
}

#[derive(Lens, Clone, Data)]
pub struct SettingsPanelState {
    pub theme: Arc<Theme>,
    pub visible: bool,
    pub active_category: SettingsCategory,
}

impl SettingsPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            visible: false,
            active_category: SettingsCategory::General,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl SettingsPanelState {
    pub fn build(&mut self, cx: &mut Context) {
        if !self.visible {
            return;
        }

        let theme = &self.theme;

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(Color::rgba(0, 0, 0, 0.9))
            .child(
                Element::new(cx)
                    .width(800.0)
                    .height(600.0)
                    .background_color(theme.colors.surface)
                    .border_radius(theme.borders.lg),
            );
    }
}
