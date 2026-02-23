//! Main editor window (Vizia version)
//!
//! The EditorWindow is the root view that contains all UI elements including
//! the Activity Bar, active Box, and overlays.

use crate::services::engine_bridge::EngineHandle;
use crate::ui::theme::ColorPalette;
use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

pub struct ToggleGlobalSearch;
pub struct Undo;
pub struct Redo;

pub struct EditorWindowState {
    pub colors: ColorPalette,
}

impl EditorWindowState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            colors: theme.colors.clone(),
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let bg = self.colors.background;
        let surface = self.colors.surface;

        HStack::new(cx, |cx| {
            Element::new(cx)
                .width(Pixels(52.0))
                .height(Stretch(1.0))
                .background_color(surface);

            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(bg);
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    }
}
