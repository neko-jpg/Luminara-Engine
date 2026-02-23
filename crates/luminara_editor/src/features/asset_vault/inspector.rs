//! Asset Inspector (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct AssetInspectorState {
    pub theme: Arc<Theme>,
    pub selected_asset: Option<String>,
}

impl AssetInspectorState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            selected_asset: None,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;
        let panel_hdr = self.theme.colors.panel_header;
        let text_col = self.theme.colors.text;
        let font_sm = self.theme.typography.sm;

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Asset Inspector")
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
