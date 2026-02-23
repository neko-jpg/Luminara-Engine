//! Backend & AI Box (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Lens, Clone, Data)]
pub struct BackendAIState {
    pub theme: Arc<Theme>,
}

impl BackendAIState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self { theme }
    }
}

impl BackendAIState {
    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;

        VStack::new(cx, |cx| {
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(theme.colors.background);
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    }
}
