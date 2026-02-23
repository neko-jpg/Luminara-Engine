//! Resizable Panel (Vizia v0.3)
//!
//! A panel component with min/max size constraints.

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct ResizablePanelState {
    pub theme: Arc<Theme>,
    pub orientation: Orientation,
    pub size: f32,
    pub min_size: f32,
    pub max_size: f32,
}

impl ResizablePanelState {
    pub fn new(theme: Arc<Theme>, orientation: Orientation) -> Self {
        Self {
            theme,
            orientation,
            size: 250.0,
            min_size: 150.0,
            max_size: 600.0,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn min_size(mut self, min: f32) -> Self {
        self.min_size = min;
        self
    }

    pub fn max_size(mut self, max: f32) -> Self {
        self.max_size = max;
        self
    }

    pub fn build(&self, cx: &mut Context) {
        let border_col = self.theme.colors.border;

        match self.orientation {
            Orientation::Horizontal => {
                Element::new(cx)
                    .width(Pixels(self.size))
                    .min_width(Pixels(self.min_size))
                    .max_width(Pixels(self.max_size))
                    .height(Stretch(1.0))
                    .border_width(Pixels(1.0))
                    .border_color(border_col);
            }
            Orientation::Vertical => {
                Element::new(cx)
                    .height(Pixels(self.size))
                    .min_height(Pixels(self.min_size))
                    .max_height(Pixels(self.max_size))
                    .width(Stretch(1.0))
                    .border_width(Pixels(1.0))
                    .border_color(border_col);
            }
        }
    }
}
