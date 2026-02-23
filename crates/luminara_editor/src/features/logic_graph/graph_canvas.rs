//! Graph Canvas Component (Vizia v0.3)

use super::{GraphNode, LogicGraph};
use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct GraphCanvas {
    pub graph: LogicGraph,
    pub theme: Arc<Theme>,
}

impl GraphCanvas {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            graph: LogicGraph::default(),
            theme,
        }
    }
}

#[derive(Clone)]
pub struct GraphCanvasPanelState {
    pub theme: Arc<Theme>,
    pub nodes: Vec<GraphNode>,
    pub zoom: f32,
    pub pan: [f32; 2],
}

impl GraphCanvasPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            nodes: Vec::new(),
            zoom: 1.0,
            pan: [0.0, 0.0],
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let canvas_bg = self.theme.colors.canvas_background;

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
    }
}
