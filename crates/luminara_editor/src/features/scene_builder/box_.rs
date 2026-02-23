//! Scene Builder Box (Vizia v0.3)

use crate::core::state::EditorStateManager;
use crate::services::engine_bridge::EngineHandle;
use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Clone)]
pub struct SceneBuilderState {
    pub theme: Arc<Theme>,
    pub engine_handle: Arc<EngineHandle>,
    pub editor_state: EditorStateManager,
    pub selected_entity: Option<u64>,
}

impl SceneBuilderState {
    pub fn new(
        engine_handle: Arc<EngineHandle>,
        theme: Arc<Theme>,
        editor_state: EditorStateManager,
    ) -> Self {
        Self {
            theme,
            engine_handle,
            editor_state,
            selected_entity: None,
        }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let surface = theme.colors.surface;
        let canvas_bg = theme.colors.canvas_background;
        let bg = theme.colors.background;

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Element::new(cx)
                    .width(Pixels(260.0))
                    .height(Stretch(1.0))
                    .background_color(surface);
            });

            Element::new(cx)
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(canvas_bg);

            Element::new(cx)
                .width(Pixels(320.0))
                .height(Stretch(1.0))
                .background_color(surface);
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(bg);
    }
}
