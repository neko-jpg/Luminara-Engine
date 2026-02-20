//! Inspector Panel Component
//!
//! Right panel showing selected entity properties

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent,
};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashSet;
use crate::ui::components::{Button, PanelHeader};
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use luminara_core::Entity;
use luminara_math::{Transform, Vec3, Quat};

/// Transform property editor
pub struct TransformEditor {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for TransformEditor {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl TransformEditor {
    /// Create from Transform component
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            position: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

/// Inspector panel component
pub struct InspectorPanel {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    selected_entities: Arc<RwLock<HashSet<Entity>>>,
    entity_name: String,
    is_active: bool,
    tags: Vec<String>,
}

impl InspectorPanel {
    /// Create a new inspector panel
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        selected_entities: Arc<RwLock<HashSet<Entity>>>,
    ) -> Self {
        Self {
            theme,
            engine_handle,
            selected_entities,
            entity_name: "Player".to_string(),
            is_active: true,
            tags: vec!["Player".to_string(), "Character".to_string()],
        }
    }

    /// Get selected entity
    fn get_selected_entity(&self) -> Option<Entity> {
        let selected = self.selected_entities.read();
        selected.iter().next().copied()
    }

    /// Get entity transform if available
    fn get_transform(&self, entity: Entity) -> Option<TransformEditor> {
        let world = self.engine_handle.world();
        world.get_component::<Transform>(entity)
            .map(|t| TransformEditor::from_transform(t))
    }

    /// Toggle active state
    pub fn toggle_active(&mut self) {
        self.is_active = !self.is_active;
    }

    /// Render vector3 input (Position, Rotation, Scale)
    fn render_vector3_input(&self, label: &str, values: [f32; 3], _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();

        div()
            .flex()
            .flex_row()
            .w_full()
            .items_center()
            .gap(theme.spacing.md)
            .py(theme.spacing.sm)
            .child(
                div()
                    .w(px(80.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child(label)
                    )
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .child(
                        // X input
                        div()
                            .flex_1()
                            .h(px(28.0))
                            .px(theme.spacing.sm)
                            .bg(theme.colors.background)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child(format!("{:.1}", values[0]))
                            )
                    )
                    .child(div().text_color(theme.colors.text_secondary).child(","))
                    .child(
                        // Y input
                        div()
                            .flex_1()
                            .h(px(28.0))
                            .px(theme.spacing.sm)
                            .bg(theme.colors.background)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child(format!("{:.1}", values[1]))
                            )
                    )
                    .child(div().text_color(theme.colors.text_secondary).child(","))
                    .child(
                        // Z input
                        div()
                            .flex_1()
                            .h(px(28.0))
                            .px(theme.spacing.sm)
                            .bg(theme.colors.background)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child(format!("{:.1}", values[2]))
                            )
                    )
            )
    }

    /// Render component header
    fn render_component_header(&self, title: &str, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let title = title.to_string();

        PanelHeader::new(title)
            .right_element(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.md)
                    .cursor_pointer()
                    .hover(|this| this.text_color(theme.colors.text))
                    .child("⋮⋮")
            )
    }

    /// Render Transform component
    fn render_transform_component(&self, editor: &TransformEditor, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .border_b_1()
            .border_color(theme.colors.border)
            .child(self.render_component_header("Transform", cx))
            .child(
                div()
                    .p(theme.spacing.md)
                    .child(self.render_vector3_input("Position", [editor.position.x, editor.position.y, editor.position.z], cx))
                    .child(self.render_vector3_input("Rotation", [editor.rotation.x, editor.rotation.y, editor.rotation.z], cx))
                    .child(self.render_vector3_input("Scale", [editor.scale.x, editor.scale.y, editor.scale.z], cx))
            )
    }

    /// Render no selection state
    fn render_no_selection(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .p(theme.spacing.lg)
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("No entity selected")
            )
            .child(
                div()
                    .mt(theme.spacing.sm)
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child("Select an entity in the hierarchy or viewport")
            )
    }

    /// Render entity inspector
    fn render_entity_inspector(&self, entity: Entity, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let transform_editor = self.get_transform(entity).unwrap_or_default();
        let is_active = self.is_active;
        let entity_name = self.entity_name.clone();
        let tags = self.tags.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                // Entity header
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .p(theme.spacing.md)
                    .bg(theme.colors.background)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .items_center()
                            .gap(theme.spacing.md)
                            .child(
                                // Active checkbox
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(theme.spacing.xs)
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _event: &MouseDownEvent, cx| {
                                        this.toggle_active();
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .w(px(16.0))
                                            .h(px(16.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .text_color(if is_active { theme.colors.success } else { theme.colors.text_secondary })
                                                    .text_size(theme.typography.md)
                                                    .child(if is_active { "☑" } else { "☐" })
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.colors.text)
                                            .text_size(theme.typography.sm)
                                            .child("Active")
                                    )
                            )
                            .child(
                                // Entity name input
                                div()
                                    .flex_1()
                                    .h(px(28.0))
                                    .px(theme.spacing.sm)
                                    .bg(theme.colors.background)
                                    .border_1()
                                    .border_color(theme.colors.border)
                                    .rounded(theme.borders.xs)
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_color(theme.colors.text)
                                            .text_size(theme.typography.sm)
                                            .child(entity_name)
                                    )
                            )
                    )
                    .child(
                        // Tags
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .mt(theme.spacing.sm)
                            .gap(theme.spacing.xs)
                            .children(tags.iter().map(|tag| {
                                let tag = tag.clone();
                                div()
                                    .px(theme.spacing.sm)
                                    .py(theme.spacing.xs)
                                    .bg(theme.colors.accent.opacity(0.2))
                                    .border_1()
                                    .border_color(theme.colors.accent)
                                    .rounded(theme.borders.xs)
                                    .child(
                                        div()
                                            .text_color(theme.colors.accent)
                                            .text_size(theme.typography.xs)
                                            .child(tag)
                                    )
                            }))
                    )
            )
            .child(self.render_transform_component(&transform_editor, cx))
            .child(
                // Add Component button
                div()
                    .mt(theme.spacing.md)
                    .p(theme.spacing.md)
                    .child(
                        Button::new("add_component", "Add Component")
                            .icon("+")
                            .full_width(true)
                    )
            )
    }
}

impl Render for InspectorPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .w(px(320.0))
            .h_full()
            .bg(theme.colors.surface)
            .border_l_1()
            .border_color(theme.colors.border)
            .child(
                // Panel header
                PanelHeader::new("Inspector")
                    .right_element(
                        div()
                            .flex()
                            .items_center()
                            .child(
                                // DB Sync toggle
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(theme.spacing.xs)
                                    .mr(theme.spacing.md)
                                    .cursor_pointer()
                                    .hover(|this| this.bg(theme.colors.surface_hover))
                                    .child(
                                        div()
                                            .text_color(theme.colors.accent)
                                            .text_size(theme.typography.xs)
                                            .child("↻")
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.colors.text_secondary)
                                            .text_size(theme.typography.xs)
                                            .child("DB Sync")
                                    )
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.md)
                                    .cursor_pointer()
                                    .hover(|this| this.text_color(theme.colors.text))
                                    .child("⋮")
                            )
                    )
            )
            .child(
                // Panel content
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child({
                        if let Some(entity) = self.get_selected_entity() {
                            div().child(self.render_entity_inspector(entity, cx))
                        } else {
                            div().child(self.render_no_selection())
                        }
                    })
            )
    }
}
