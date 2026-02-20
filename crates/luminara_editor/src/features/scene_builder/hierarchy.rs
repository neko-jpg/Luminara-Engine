//! Hierarchy Panel Component
//!
//! Left panel showing scene hierarchy tree with filtering and entity management

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    WindowContext, View, MouseButton, MouseDownEvent, ClickEvent, prelude::*,
};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashSet;
use crate::ui::theme::Theme;
use crate::ui::components::{PanelHeader, Button, TextInput, ButtonVariant};
use crate::services::engine_bridge::EngineHandle;
use crate::services::component_binding::ComponentUpdateCommand;
use crate::core::commands::DuplicateEntityCommand;
use luminara_core::Entity;
use luminara_scene::{Parent, Children, Name};
use luminara_math::Transform;
use crate::features::scene_builder::SceneBuilderState;

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
    state: gpui::Model<SceneBuilderState>,
    expanded_entities: HashSet<Entity>,
    context_menu_entity: Option<Entity>,
    // For creating new entities
    next_entity_number: usize,
}

impl HierarchyPanel {
    /// Create a new hierarchy panel
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        state: gpui::Model<SceneBuilderState>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Duration;
        use crate::services::engine_bridge::Event;

        // Flag to tracking if hierarchy needs UI update
        let needs_update = Arc::new(AtomicBool::new(true)); // True initially to render first time
        let needs_update_ev = needs_update.clone();

        // Subscribe to engine events that affect hierarchy
        engine_handle.subscribe_events(move |event| {
            match event {
                Event::EntitySpawned { .. } |
                Event::EntityDespawned { .. } |
                Event::ComponentAdded { .. } |
                Event::ComponentRemoved { .. } => {
                    needs_update_ev.store(true, Ordering::Release);
                }
                _ => {}
            }
        });

        // Background task to poll the update flag at ~30Hz
        cx.spawn(|this, mut cx| async move {
            loop {
                cx.background_executor().timer(Duration::from_millis(32)).await;
                if needs_update.swap(false, Ordering::Acquire) {
                    let _ = this.update(&mut cx, |_, cx| cx.notify());
                }
            }
        }).detach();

        Self {
            theme,
            engine_handle,
            filter_text: String::new(),
            state,
            expanded_entities: HashSet::new(),
            context_menu_entity: None,
            next_entity_number: 1,
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

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
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
        self.state.update(cx, |state, cx| {
            if multi_select {
                if state.selected_entities.contains(&entity) {
                    state.selected_entities.remove(&entity);
                } else {
                    state.selected_entities.insert(entity);
                }
            } else {
                state.selected_entities.clear();
                state.selected_entities.insert(entity);
            }
            cx.notify();
        });
    }

    /// Create a new entity with default components
    pub fn create_new_entity(&mut self, cx: &mut ViewContext<Self>) {
        let entity_name = format!("GameObject {}", self.next_entity_number);
        self.next_entity_number += 1;

        // Spawn entity with components
        let world = self.engine_handle.world();
        let entity_count = world.entities().len();
        drop(world);

        // Create entity with transform and name
        let entity = self.engine_handle.spawn_entity_with((
            Transform::IDENTITY,
            Name::new(&entity_name),
        ));

        if let Ok(entity) = entity {
            // Select the new entity
            self.select_entity(entity, false, cx);
            
            // Expand to show the new entity
            self.expanded_entities.insert(entity);
        }

        cx.notify();
    }

    /// Delete selected entities
    pub fn delete_selected_entities(&mut self, cx: &mut ViewContext<Self>) {
        let selected: Vec<Entity> = self.state.read(cx).selected_entities.iter().copied().collect();
        
        for entity in selected {
            let _ = self.engine_handle.despawn_entity(entity);
        }

        // Clear selection
        self.state.update(cx, |state, cx| {
            state.selected_entities.clear();
            cx.notify();
        });

        cx.notify();
    }

    /// Duplicate an entity
    pub fn duplicate_entity(&mut self, entity: Entity, cx: &mut ViewContext<Self>) {
        // Use the engine bridge to execute duplicate command
        self.engine_handle.execute_command(Box::new(DuplicateEntityCommand::new(entity)));
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
        let world = self.engine_handle.world();
        if let Some(name) = world.get_component::<Name>(entity) {
            name.0.clone()
        } else {
            format!("Entity {:?}", entity)
        }
    }

    /// Check if entity matches filter
    fn entity_matches_filter(&self, entity: Entity) -> bool {
        if self.filter_text.is_empty() {
            return true;
        }
        let name = self.get_entity_name(entity);
        name.to_lowercase().contains(&self.filter_text.to_lowercase())
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

    /// Check if entity should be visible (matches filter or has visible children)
    fn is_entity_visible(&self, entity: Entity) -> bool {
        if self.filter_text.is_empty() {
            return true;
        }

        // Check if entity matches filter
        if self.entity_matches_filter(entity) {
            return true;
        }

        // Check if any children match filter
        let children = self.get_children(entity);
        children.iter().any(|&child| self.is_entity_visible_recursive(child))
    }

    fn is_entity_visible_recursive(&self, entity: Entity) -> bool {
        if self.entity_matches_filter(entity) {
            return true;
        }
        let children = self.get_children(entity);
        children.iter().any(|&child| self.is_entity_visible_recursive(child))
    }

    /// Render the toolbar (filter + buttons)
    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let filter_text = self.filter_text.clone();
        let has_filter = !filter_text.is_empty();

        div()
            .flex()
            .flex_row()
            .w_full()
            .gap(theme.spacing.xs)
            .mb(theme.spacing.md)
            .child(
                // Filter input with clear button
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        TextInput::new("hierarchy_filter")
                            .placeholder("Filter...")
                            .value(filter_text)
                            .on_change(cx.listener(|this, text: &str, _cx| {
                                this.filter_text = text.to_string();
                            }))
                    )
                    .when(has_filter, |this| {
                        this.child(
                            Button::new("clear_filter", "×")
                                .variant(ButtonVariant::Ghost)
                                .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                    this.clear_filter();
                                    cx.notify();
                                }))
                        )
                    })
            )
            .child(
                // Create entity button (Plus)
                Button::new("create_entity", "+")
                    .variant(ButtonVariant::Ghost)
                    .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                        this.create_new_entity(cx);
                    }))
            )
    }

    /// Render a single hierarchy item
    fn render_item(&self, entity: Entity, depth: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        // Check visibility based on filter
        if !self.is_entity_visible(entity) {
            return div().into_any_element();
        }
        
        // Read selection from global state
        let is_selected = self.state.read(cx).selected_entities.contains(&entity);

        let is_expanded = self.expanded_entities.contains(&entity);
        let has_children = self.has_children(entity);
        let entity_name = self.get_entity_name(entity);
        let entity_clone = entity;
        let is_context_menu_open = self.context_menu_entity == Some(entity);

        let indent = px((depth * 16) as f32);
        let engine_handle = self.engine_handle.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                // Entity row
                div()
                    .relative()
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
                        // Close context menu on left click
                        this.context_menu_entity = None;
                        cx.notify();
                    }))
                    .on_mouse_down(MouseButton::Right, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                        this.context_menu_entity = Some(entity_clone);
                        cx.notify();
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
                    // Context Menu
                    .when(is_context_menu_open, |this| {
                        let engine = engine_handle.clone();
                        let entity_target = entity_clone;
                        let entity_duplicate = entity_clone;
                        this.child(
                            div()
                                .absolute()
                                .top(px(28.0))
                                .left(px(40.0))
                                .bg(theme.colors.surface)
                                .border_1()
                                .border_color(theme.colors.border)
                                .rounded(theme.borders.sm)
                                .shadow_md()
                                .w(px(140.0))
                                .flex()
                                .flex_col()
                                .child(
                                    Button::new("duplicate_btn", "Duplicate")
                                        .variant(ButtonVariant::Ghost)
                                        .full_width(true)
                                        .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                            this.duplicate_entity(entity_duplicate, cx);
                                            this.context_menu_entity = None;
                                            cx.notify();
                                        }))
                                )
                                .child(
                                    Button::new("delete_btn", "Delete")
                                        .variant(ButtonVariant::Ghost)
                                        .full_width(true)
                                        .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                            let _ = engine.despawn_entity(entity_target);
                                            this.state.update(cx, |state, cx| {
                                                state.selected_entities.remove(&entity_target);
                                                cx.notify();
                                            });
                                            this.context_menu_entity = None;
                                            cx.notify();
                                        }))
                                )
                                .child(
                                    Button::new("rename_btn", "Rename")
                                        .variant(ButtonVariant::Ghost)
                                        .full_width(true)
                                        .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                            // TODO: Implement rename dialog
                                            this.context_menu_entity = None;
                                            cx.notify();
                                        }))
                                )
                        )
                    })
            )
            .children(
                // Render children if expanded
                if is_expanded && has_children {
                    let children: Vec<_> = self.get_children(entity)
                        .into_iter()
                        .filter(|&c| self.is_entity_visible(c))
                        .collect();
                    
                    if !children.is_empty() {
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
                } else {
                    None
                }
            )
            .into_any_element()
    }
}

impl Render for HierarchyPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let root_entities: Vec<_> = self.get_root_entities()
            .into_iter()
            .filter(|&e| self.is_entity_visible(e))
            .collect();

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
                                .child(
                                    div()
                                        .mt(theme.spacing.md)
                                        .flex()
                                        .justify_center()
                                        .child(
                                            Button::new("create_first_entity", "Create Entity")
                                                .variant(ButtonVariant::Primary)
                                                .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                                    this.create_new_entity(cx);
                                                }))
                                        )
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
