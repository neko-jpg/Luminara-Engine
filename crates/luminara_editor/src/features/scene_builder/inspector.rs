//! Inspector Panel (Vizia v0.3)
//!
//! Right panel showing selected entity properties.

use crate::ui::theme::Theme;
use luminara_core::Entity;
use luminara_math::Vec3;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct TransformEditor {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for TransformEditor {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Clone)]
pub struct InspectorPanelState {
    pub theme: Arc<Theme>,
    pub selected_entity: Option<Entity>,
    pub entity_name: String,
    pub transform: TransformEditor,
}

impl InspectorPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            selected_entity: None,
            entity_name: String::new(),
            transform: TransformEditor::default(),
        }
    }

    pub fn set_entity(&mut self, entity: Entity, name: &str) {
        self.selected_entity = Some(entity);
        self.entity_name = name.to_string();
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let surface = theme.colors.surface;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let text_sec = theme.colors.text_secondary;
        let font_sm = theme.typography.sm;
        let font_xs = theme.typography.xs;
        let border_col = theme.colors.border;

        let entity_name = self.entity_name.clone();
        let transform = self.transform.clone();

        VStack::new(cx, move |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "Inspector")
                    .font_size(font_sm)
                    .color(text_col)
                    .font_weight(FontWeight(700));
            })
            .height(Pixels(28.0))
            .width(Stretch(1.0))
            .background_color(panel_hdr)
            .padding_left(Pixels(8.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0))
            .border_width(Pixels(1.0))
            .border_color(border_col);

            // Entity name
            HStack::new(cx, |cx| {
                Label::new(cx, "Name:")
                    .font_size(font_xs)
                    .color(text_sec);
                Label::new(cx, &entity_name)
                    .font_size(font_xs)
                    .color(text_col);
            })
            .padding_left(Pixels(8.0))
            .padding_top(Pixels(6.0))
            .gap(Pixels(4.0))
            .width(Stretch(1.0));

            // Transform section
            Label::new(cx, "Transform")
                .font_size(font_sm)
                .color(text_col)
                .font_weight(FontWeight(700))
                .padding_left(Pixels(8.0))
                .padding_top(Pixels(8.0));

            // Position
            let pos = transform.position;
            let pos_text = format!("X: {:.2}  Y: {:.2}  Z: {:.2}", pos.x, pos.y, pos.z);
            HStack::new(cx, move |cx| {
                Label::new(cx, "Position")
                    .font_size(font_xs)
                    .color(text_sec)
                    .width(Pixels(60.0));
                Label::new(cx, &pos_text)
                    .font_size(font_xs)
                    .color(text_col);
            })
            .padding_left(Pixels(12.0))
            .padding_top(Pixels(4.0))
            .gap(Pixels(4.0))
            .width(Stretch(1.0));

            // Rotation
            let rot = transform.rotation;
            let rot_text = format!("X: {:.2}  Y: {:.2}  Z: {:.2}", rot.x, rot.y, rot.z);
            HStack::new(cx, move |cx| {
                Label::new(cx, "Rotation")
                    .font_size(font_xs)
                    .color(text_sec)
                    .width(Pixels(60.0));
                Label::new(cx, &rot_text)
                    .font_size(font_xs)
                    .color(text_col);
            })
            .padding_left(Pixels(12.0))
            .padding_top(Pixels(4.0))
            .gap(Pixels(4.0))
            .width(Stretch(1.0));

            // Scale
            let scl = transform.scale;
            let scl_text = format!("X: {:.2}  Y: {:.2}  Z: {:.2}", scl.x, scl.y, scl.z);
            HStack::new(cx, move |cx| {
                Label::new(cx, "Scale")
                    .font_size(font_xs)
                    .color(text_sec)
                    .width(Pixels(60.0));
                Label::new(cx, &scl_text)
                    .font_size(font_xs)
                    .color(text_col);
            })
            .padding_left(Pixels(12.0))
            .padding_top(Pixels(4.0))
            .gap(Pixels(4.0))
            .width(Stretch(1.0));

            // Spacer
            Element::new(cx).height(Stretch(1.0));
        })
        .width(Pixels(320.0))
        .height(Stretch(1.0))
        .background_color(surface);
    }
}
