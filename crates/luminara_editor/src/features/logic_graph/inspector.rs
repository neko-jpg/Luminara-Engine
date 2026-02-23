//! Node Inspector Panel (Vizia v0.3)

use super::GraphNode;
use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Properties,
    Conditions,
    Actions,
    Links,
}

#[derive(Clone)]
pub struct NodeInspectorState {
    pub theme: Arc<Theme>,
    pub selected_node: Option<GraphNode>,
    pub view_mode: ViewMode,
}

impl NodeInspectorState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            selected_node: None,
            view_mode: ViewMode::Properties,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let surface = theme.colors.surface;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let font_sm = theme.typography.sm;

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Node Inspector")
                    .font_size(font_sm)
                    .color(text_col)
                    .font_weight(FontWeight(700));
            })
            .height(Pixels(28.0))
            .width(Stretch(1.0))
            .background_color(panel_hdr)
            .padding_left(Pixels(8.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0));

            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(surface);
        })
        .width(Pixels(320.0))
        .height(Stretch(1.0))
        .background_color(surface);
    }
}
