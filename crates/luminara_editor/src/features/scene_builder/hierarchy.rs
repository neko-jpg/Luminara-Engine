//! Hierarchy Panel (Vizia v0.3)
//!
//! Left panel showing scene hierarchy tree.

use crate::ui::theme::Theme;
use luminara_core::Entity;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct HierarchyItem {
    pub entity: Entity,
    pub name: String,
    pub icon: String,
    pub is_enabled: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub has_children: bool,
}

impl HierarchyItem {
    pub fn new(entity: Entity, name: &str) -> Self {
        Self {
            entity,
            name: name.to_string(),
            icon: "📦".to_string(),
            is_enabled: true,
            depth: 0,
            is_expanded: false,
            has_children: false,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_children(mut self, has: bool) -> Self {
        self.has_children = has;
        self
    }
}

#[derive(Clone)]
pub struct HierarchyPanelState {
    pub theme: Arc<Theme>,
    pub items: Vec<HierarchyItem>,
    pub selected_entity: Option<Entity>,
    pub filter_text: String,
}

impl HierarchyPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            items: Vec::new(),
            selected_entity: None,
            filter_text: String::new(),
        }
    }

    pub fn set_items(&mut self, items: Vec<HierarchyItem>) {
        self.items = items;
    }

    pub fn select(&mut self, entity: Entity) {
        self.selected_entity = Some(entity);
    }

    pub fn build(&mut self, cx: &mut Context) {
        let theme = &self.theme;
        let surface = theme.colors.surface;
        let panel_hdr = theme.colors.panel_header;
        let text_col = theme.colors.text;
        let text_sec = theme.colors.text_secondary;
        let font_sm = theme.typography.sm;
        let border_col = theme.colors.border;

        let items = self.items.clone();
        let selected = self.selected_entity;

        VStack::new(cx, move |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "Hierarchy")
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

            // Item list
            VStack::new(cx, move |cx| {
                for item in &items {
                    let name = item.name.clone();
                    let icon = item.icon.clone();
                    let depth = item.depth;
                    let is_selected = selected.map_or(false, |e| e == item.entity);
                    let col = if is_selected { text_col } else { text_sec };
                    let item_bg = if is_selected {
                        theme.colors.surface_active
                    } else {
                        surface
                    };

                    HStack::new(cx, move |cx| {
                        Label::new(cx, &icon).font_size(font_sm).color(col);
                        Label::new(cx, &name).font_size(font_sm).color(col);
                    })
                    .padding_left(Pixels(8.0 + (depth as f32) * 16.0))
                    .padding_top(Pixels(2.0))
                    .padding_bottom(Pixels(2.0))
                    .gap(Pixels(4.0))
                    .width(Stretch(1.0))
                    .height(Pixels(24.0))
                    .background_color(item_bg);
                }
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0));
        })
        .width(Pixels(260.0))
        .height(Stretch(1.0))
        .background_color(surface);
    }
}
