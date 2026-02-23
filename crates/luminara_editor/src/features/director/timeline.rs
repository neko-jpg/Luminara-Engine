//! Timeline Component (Vizia v0.3)

use crate::ui::theme::Theme;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyframeKind {
    Position,
    Rotation,
    Scale,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub frame: u32,
    pub kind: KeyframeKind,
    pub value: f64,
}

#[derive(Clone)]
pub struct TimelineState {
    pub theme: Arc<Theme>,
    pub keyframes: Vec<Keyframe>,
    pub current_frame: u32,
    pub total_frames: u32,
    pub playing: bool,
}

impl TimelineState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            keyframes: Vec::new(),
            current_frame: 0,
            total_frames: 300,
            playing: false,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;
        let text_col = self.theme.colors.text;
        let font_sm = self.theme.typography.sm;

        VStack::new(cx, |cx| {
            Label::new(cx, "Timeline")
                .font_size(font_sm)
                .color(text_col);
        })
        .height(Pixels(200.0))
        .width(Stretch(1.0))
        .background_color(surface);
    }
}
