//! Hierarchy Panel Component
//!
//! Left panel showing scene hierarchy tree with filtering

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent,
};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashSet;
use crate::ui::theme::Theme;
use crate::ui::components::{PanelHeader, Button, TextInput, ButtonVariant};
use crate::services::engine_bridge::EngineHandle;
use luminara_core::Entity;
use luminara_scene::{Parent, Children};

/// Hierarchy item representing an entity in the tree
#[derive(Debug, Clone)]
pub struct HierarchyItem {
    pub entity: Entity,
    pub name: String,
    pub icon: String,
    pub icon_color: gpui::Hsla,
    pub is_enabled: bool,
    pub depth: usize,
    pub has_children: bool,
}

/// Hierarchy panel component
pub struct HierarchyPanel {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    filter_text: String,
    selected_entities: Arc<RwLock<HashSet<Entity>>>,
    expanded_entities: HashSet<Entity>,
}

impl HierarchyPanel {
    /// Create a new hierarchy panel
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        selected_entities: Arc<RwLock<HashSet<Entity>>>,
    ) -> Self {
        Self {
            theme,
            engine_handle,
            filter_text: String::new(),
            selected_entities,
            expanded_entities: HashSet::new(),
        }
    }

    /// Set filter text
    pub fn set_filter(&mut self, text: String) {
        self.filter_text = text;
    }

    /// Get filter text
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Toggle entity expansion
    pub fn toggle_expansion(&mut self, entity: Entity) {
        if self.expanded_entities.contains(&entity) {
            self.expanded_entities.remove(&entity);
        } else {
            self.expanded_entities.insert(entity);
        }
    }

    /// Select an entity
    pub fn select_entity(&mut self, entity: Entity, multi_select: bool, cx: &mut ViewContext<Self>) {
        let mut selected = self.selected_entities.write();

        if multi_select {
            if selected.contains(&entity) {
                selected.remove(&entity);
            } else {
                selected.insert(entity);
            }
        } else {
            selected.clear();
            selected.insert(entity);
        }

        drop(selected);
        cx.notify();
    }

    /// Get root entities (entities without parent)
    fn get_root_entities(&self) -> Vec<Entity> {
        let world = self.engine_handle.world();
        let entities = world.entities();

        entities
            .into_iter()
            .filter(|&e| world.get_component::<Parent>(e).is_none())
            .collect()
    }

    /// Get entity name
    fn get_entity_name(&self, entity: Entity) -> String {
        format!("Entity {:?}", entity)
    }

    /// Check if entity has children
    fn has_children(&self, entity: Entity) -> bool {
        let world = self.engine_handle.world();
        world.get_component::<Children>(entity)
            .map(|c| !c.0.is_empty())
            .unwrap_or(false)
    }

    /// Get children of an entity
    fn get_children(&self, entity: Entity) -> Vec<Entity> {
        let world = self.engine_handle.world();
        world.get_component::<Children>(entity)
            .map(|c| c.0.clone())
            .unwrap_or_default()
    }

    /// Render the toolbar (filter + buttons)
    fn render_toolbar(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .w_full()
            .gap(theme.spacing.xs)
            .mb(theme.spacing.md)
            .child(TextInput::new("filter").placeholder("Filter..."))
            .child(
                // DB button
                Button::new("db_btn", "◉")
                    .variant(ButtonVariant::Ghost)
            )
            .child(
                // Plus button
                Button::new("plus_btn", "+")
                    .variant(ButtonVariant::Ghost)
            )
    }

    /// Render a single hierarchy item
    fn render_item(&self, entity: Entity, depth: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let selected = self.selected_entities.read();
        let is_selected = selected.contains(&entity);
        drop(selected);

        let is_expanded = self.expanded_entities.contains(&entity);
        let has_children = self.has_children(entity);
        let entity_name = self.get_entity_name(entity);
        let entity_clone = entity;

        let indent = px((depth * 16) as f32);

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                // Entity row
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(28.0))
                    .items_center()
                    .pl(indent)
                    .pr(theme.spacing.sm)
                    .bg(if is_selected {
                        theme.colors.accent
                    } else {
                        theme.colors.surface
                    })
                    .hover(|this| {
                        if !is_selected {
                            this.bg(theme.colors.surface_hover)
                        } else {
                            this
                        }
                    })
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &MouseDownEvent, cx| {
                        let multi_select = event.modifiers.shift;
                        this.select_entity(entity_clone, multi_select, cx);
                    }))
                    .child(
                        // Expand/collapse arrow
                        if has_children {
                            let entity_toggle = entity;
                            div()
                                .w(px(16.0))
                                .h(px(16.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .mr(theme.spacing.xs)
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                                    this.toggle_expansion(entity_toggle);
                                    cx.stop_propagation();
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .text_color(theme.colors.text_secondary)
                                        .text_size(theme.typography.xs)
                                        .child(if is_expanded { "▼" } else { "▶" })
                                )
                        } else {
                            div()
                                .w(px(16.0))
                                .mr(theme.spacing.xs)
                        }
                    )
                    .child(
                        // Entity icon
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .mr(theme.spacing.xs)
                            .child(
                                div()
                                    .text_color(if is_selected { theme.colors.background } else { theme.colors.accent })
                                    .text_size(theme.typography.sm)
                                    .child("◆")
                            )
                    )
                    .child(
                        // Entity name
                        div()
                            .text_color(if is_selected { theme.colors.background } else { theme.colors.text })
                            .text_size(theme.typography.sm)
                            .child(entity_name)
                    )
                    .child(div().flex_1())
                    .child(
                        // Enabled checkbox
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_color(if is_selected { theme.colors.background } else { theme.colors.success })
                                    .text_size(theme.typography.sm)
                                    .child("◉")
                            )
                    )
            )
            .children(
                // Render children if expanded
                if is_expanded && has_children {
                    let children = self.get_children(entity);
                    Some(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .children(
                                children.into_iter().map(|child| {
                                    self.render_item(child, depth + 1, cx)
                                })
                            )
                    )
                } else {
                    None
                }
            )
    }
}

impl Render for HierarchyPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme = self.theme.clone();
        let root_entities = self.get_root_entities();

        div()
            .flex()
            .flex_col()
            .w(px(260.0))
            .h_full()
            .bg(theme.colors.surface)
            .border_r_1()
            .border_color(theme.colors.border)
            .child(PanelHeader::new("Scene Hierarchy"))
            .child(
                // Panel content
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .p(theme.spacing.sm)
                    .child(self.render_toolbar(cx))
                    .child(
                        if root_entities.is_empty() {
                            div()
                                .p(theme.spacing.sm)
                                .child(
                                    div()
                                        .text_color(theme.colors.text_secondary)
                                        .text_size(theme.typography.sm)
                                        .child("No entities in scene")
                                )
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .children(
                                    root_entities.into_iter().map(|entity| {
                                        self.render_item(entity, 0, cx)
                                    })
                                )
                        }
                    )
            )
    }
}
