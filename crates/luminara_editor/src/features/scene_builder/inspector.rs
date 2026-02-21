//! Inspector Panel Component
//!
//! Right panel showing selected entity properties with editable fields

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent, ClickEvent, prelude::*,
};
use std::sync::Arc;
use std::collections::HashSet;
use crate::ui::components::{PanelHeader, Dropdown, TextInput, Button, ButtonVariant};
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use luminara_core::Entity;
use luminara_math::{Transform, Vec3, Quat};
use luminara_scene::{Name, Tag};
use crate::core::state::EditorStateManager;

/// Transform property editor
#[derive(Debug, Clone)]
pub struct TransformEditor {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles for easier editing
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

impl TransformEditor {
    /// Create from Transform component
    pub fn from_transform(transform: &Transform) -> Self {
        // Convert quaternion to Euler angles for editing
        let (yaw, pitch, roll) = transform.rotation.to_euler(luminara_math::EulerRot::YXZ);
        Self {
            position: transform.translation,
            rotation: Vec3::new(
                pitch.to_degrees(),
                yaw.to_degrees(),
                roll.to_degrees()
            ),
            scale: transform.scale,
        }
    }

    /// Convert back to Transform
    pub fn to_transform(&self) -> Transform {
        Transform {
            translation: self.position,
            rotation: Quat::from_euler(
                luminara_math::EulerRot::YXZ,
                self.rotation.y.to_radians(),
                self.rotation.x.to_radians(),
                self.rotation.z.to_radians()
            ),
            scale: self.scale,
        }
    }
}

/// Inspector panel component
pub struct InspectorPanel {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    state: gpui::Model<EditorStateManager>,
    // For tracking which component is being added
    show_add_component_menu: bool,
    // Editing state
    editing_name: String,
    is_editing_name: bool,
    // Pending transform updates
    pending_position: Option<Vec3>,
    pending_rotation: Option<Vec3>,
    pending_scale: Option<Vec3>,
}

impl InspectorPanel {
    /// Create a new inspector panel
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        state: gpui::Model<EditorStateManager>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        cx.observe(&state, |_this: &mut InspectorPanel, _model, cx| {
            cx.notify();
        }).detach();

        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Duration;
        use crate::services::engine_bridge::Event;

        // Flag to tracking if inspector needs UI update
        let needs_update = Arc::new(AtomicBool::new(true)); // True initially
        let needs_update_ev = needs_update.clone();

        // Subscribe to engine events that affect inspector (component changes on selected entities)
        engine_handle.subscribe_events(move |event| {
            match event {
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

        let state_clone = state.clone();
        Self {
            theme,
            engine_handle,
            state: state_clone,
            show_add_component_menu: false,
            editing_name: String::new(),
            is_editing_name: false,
            pending_position: None,
            pending_rotation: None,
            pending_scale: None,
        }
    }

    /// Get selected entity
    fn get_selected_entity(&self, cx: &ViewContext<Self>) -> Option<Entity> {
        let state = self.state.read(cx);
        // Take the first selected entity ID string and parse it back to Entity
        state.session.selected_entities.first().and_then(|id_str| {
            if let Some((id_part, gen_part)) = id_str.split_once(':') {
                if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                    return Some(Entity::from_raw(id, gen));
                }
            }
            None
        })
    }

    /// Get entity transform if available
    fn get_transform(&self, entity: Entity) -> Option<TransformEditor> {
        let world = self.engine_handle.world();
        world.get_component::<Transform>(entity)
            .map(|t| TransformEditor::from_transform(t))
    }

    /// Update entity transform
    fn update_transform(&mut self, entity: Entity, editor: &TransformEditor, cx: &mut ViewContext<Self>) {
        let transform = editor.to_transform();
        
        // Store old value for undo
        let world = self.engine_handle.world();
        let old_transform = world.get_component::<Transform>(entity).copied();
        drop(world);
 
        // Record in global state manager (Local-First SSOT)
        if let Some(old) = old_transform {
            let state_clone = self.state.clone();
            let entity_id = format!("{}:{}", entity.id(), entity.generation());
            state_clone.update(cx, |state, cx| {
                state.record_component_edit(
                    entity_id,
                    "Transform".to_string(),
                    serde_json::to_value(&transform).unwrap_or_default(),
                    serde_json::to_value(&old).unwrap_or_default(),
                    cx
                );
            });
        }

        // Apply update
        let _ = self.engine_handle.update_component(entity, transform);
        cx.notify();
    }

    /// Apply pending transform updates
    fn apply_pending_updates(&mut self, entity: Entity, cx: &mut ViewContext<Self>) {
        if let Some(mut editor) = self.get_transform(entity) {
            if let Some(pos) = self.pending_position {
                editor.position = pos;
                self.pending_position = None;
            }
            if let Some(rot) = self.pending_rotation {
                editor.rotation = rot;
                self.pending_rotation = None;
            }
            if let Some(scale) = self.pending_scale {
                editor.scale = scale;
                self.pending_scale = None;
            }
            self.update_transform(entity, &editor, cx);
        }
    }

    /// Update entity name
    fn update_entity_name(&mut self, entity: Entity, name: String, cx: &mut ViewContext<Self>) {
        let world = self.engine_handle.world();
        let old_name = world.get_component::<Name>(entity).map(|n| n.0.clone()).unwrap_or_default();
        drop(world);

        let entity_id = format!("{}:{}", entity.id(), entity.generation());
        let name_clone = name.clone();
        
        self.state.update(cx, |state, cx| {
            state.record_component_edit(
                entity_id,
                "Name".to_string(),
                serde_json::json!(&name_clone),
                serde_json::json!(&old_name),
                cx
            );
        });

        let _ = self.engine_handle.update_component(entity, Name::new(&name));
        self.is_editing_name = false;
        cx.notify();
    }

    /// Add a component to entity
    fn add_component_to_entity(&mut self, entity: Entity, component_type: &str, cx: &mut ViewContext<Self>) {
        self.state.update(cx, |state, cx| {
            state.add_component(entity, component_type, cx);
        });
        self.show_add_component_menu = false;
        cx.notify();
    }


    /// Render vector3 input (Position, Rotation, Scale)
    fn render_vector3_input(
        &self, 
        label: &str, 
        values: Vec3, 
        on_change: impl Fn(f32, usize) + 'static,
        _cx: &mut ViewContext<Self>
    ) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        let on_change = Arc::new(on_change);

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
                    .child(self.render_float_input("x", values.x, theme.colors.error, {
                        let on_change = on_change.clone();
                        move |val| on_change(val, 0)
                    }))
                    .child(self.render_float_input("y", values.y, theme.colors.success, {
                        let on_change = on_change.clone();
                        move |val| on_change(val, 1)
                    }))
                    .child(self.render_float_input("z", values.z, theme.colors.accent, {
                        let on_change = on_change.clone();
                        move |val| on_change(val, 2)
                    }))
            )
    }

    /// Render a single float input field
    fn render_float_input(
        &self,
        label: &str,
        value: f32,
        color: gpui::Hsla,
        on_change: impl Fn(f32) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        let value_str = format!("{:.2}", value);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .child(
                div()
                    .text_color(color)
                    .text_size(theme.typography.xs)
                    .child(label)
            )
            .child(
                div()
                    .w(px(60.0))
                    .h(px(24.0))
                    .px(theme.spacing.xs)
                    .bg(theme.colors.background)
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(theme.borders.xs)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_text()
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child(value_str)
                    )
            )
    }

    /// Render component header
    fn render_component_header(&self, title: &str, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
    fn render_transform_component(&mut self, entity: Entity, editor: &TransformEditor, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let position = editor.position;
        let rotation = editor.rotation;
        let scale = editor.scale;

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
                    .child(self.render_vector3_input("Position", position, {
                        move |val, idx| {
                            // Queue position update - would need proper state management
                            println!("Position {}: {}", idx, val);
                        }
                    }, cx))
                    .child(self.render_vector3_input("Rotation", rotation, {
                        move |val, idx| {
                            println!("Rotation {}: {}", idx, val);
                        }
                    }, cx))
                    .child(self.render_vector3_input("Scale", scale, {
                        move |val, idx| {
                            println!("Scale {}: {}", idx, val);
                        }
                    }, cx))
                    .child(
                        Button::new("apply_transform", "Apply Changes")
                            .variant(ButtonVariant::Primary)
                            .full_width(true)
                            .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                this.apply_pending_updates(entity, cx);
                            }))
                    )
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
    fn render_entity_inspector(&mut self, entity: Entity, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        // Fetch actual data from ECS
        let world = self.engine_handle.world();
        
        let entity_name = if let Some(name_comp) = world.get_component::<Name>(entity) {
            name_comp.0.clone()
        } else {
            format!("Entity {:?}", entity)
        };
        
        let tags: Vec<String> = if let Some(tag_comp) = world.get_component::<Tag>(entity) {
            tag_comp.0.iter().cloned().collect()
        } else {
            Vec::new()
        };
        
        // Drop the read lock before rendering components that might need it
        drop(world);

        let transform_editor = self.get_transform(entity).unwrap_or_default();

        // Update editing name if not currently editing
        if !self.is_editing_name {
            self.editing_name = entity_name.clone();
        }

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
                            .justify_between()
                            .mb(theme.spacing.sm)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(theme.spacing.md)
                                    .child(
                                        div()
                                            .text_color(theme.colors.text_secondary)
                                            .text_size(theme.typography.sm)
                                            .child("Layer")
                                    )
                                    .child(
                                        div()
                                            .w(px(140.0))
                                            .px(theme.spacing.sm)
                                            .py(theme.spacing.xs)
                                            .bg(theme.colors.surface)
                                            .border_1()
                                            .border_color(theme.colors.border)
                                            .rounded(theme.borders.xs)
                                            .text_color(theme.colors.text)
                                            .text_size(theme.typography.sm)
                                            .child("Default")
                                    )
                            )
                    )
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
                                        // Toggle active state - would need an Active component
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
                                                    .text_color(theme.colors.success)
                                                    .text_size(theme.typography.md)
                                                    .child("☑")
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
                                    .child(
                                        TextInput::new("entity_name_input")
                                            .value(self.editing_name.clone())
                                            .on_change(cx.listener(|this, text: &str, _cx| {
                                                this.editing_name = text.to_string();
                                                this.is_editing_name = true;
                                            }))
                                    )
                            )
                            .child(
                                // Save name button
                                Button::new("save_name", "✓")
                                    .variant(ButtonVariant::Ghost)
                                    .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                        let new_name = this.editing_name.clone();
                                        this.update_entity_name(entity, new_name, cx);
                                    }))
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
            .child(self.render_transform_component(entity, &transform_editor, cx))
            .child(
                // Add Component section
                div()
                    .mt(theme.spacing.md)
                    .p(theme.spacing.md)
                    .child(
                        if self.show_add_component_menu {
                            div()
                                .flex()
                                .flex_col()
                                .gap(theme.spacing.xs)
                                .child(
                                    div()
                                        .text_color(theme.colors.text_secondary)
                                        .text_size(theme.typography.sm)
                                        .child("Add Component:")
                                )
                                .child(
                                    Button::new("add_transform", "Transform")
                                        .variant(ButtonVariant::Secondary)
                                        .full_width(true)
                                        .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                            this.add_component_to_entity(entity, "Transform", cx);
                                        }))
                                )
                                .child(
                                    Button::new("add_name", "Name")
                                        .variant(ButtonVariant::Secondary)
                                        .full_width(true)
                                        .on_click(cx.listener(move |this, _event: &ClickEvent, cx| {
                                            this.add_component_to_entity(entity, "Name", cx);
                                        }))
                                )
                                .child(
                                    Button::new("cancel_add", "Cancel")
                                        .variant(ButtonVariant::Ghost)
                                        .full_width(true)
                                        .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                            this.show_add_component_menu = false;
                                            cx.notify();
                                        }))
                                )
                        } else {
                            div().child(
                                Button::new("add_component", "Add Component")
                                    .icon("+")
                                    .full_width(true)
                                    .variant(ButtonVariant::Secondary)
                                    .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                        this.show_add_component_menu = true;
                                        cx.notify();
                                    }))
                            )
                        }
                    )
            )
            .child(
                // Undo/Redo buttons
                div()
                    .mt(theme.spacing.md)
                    .p(theme.spacing.md)
                    .flex()
                    .flex_row()
                    .gap(theme.spacing.sm)
                    .child(
                        Button::new("undo_btn", "Undo")
                            .variant(ButtonVariant::Ghost)
                            .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                this.state.update(cx, |state, cx| {
                                    state.undo(cx);
                                });
                            }))
                    )
                    .child(
                        Button::new("redo_btn", "Redo")
                            .variant(ButtonVariant::Ghost)
                            .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                this.state.update(cx, |state, cx| {
                                    state.redo(cx);
                                });
                            }))
                    )
            )
    }
}

impl Render for InspectorPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
                            .child({
                                let is_syncing = self.state.read(cx).is_syncing;
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(theme.spacing.xs)
                                    .mr(theme.spacing.md)
                                    .child(
                                        div()
                                            .text_color(if is_syncing { theme.colors.accent } else { theme.colors.success })
                                            .text_size(theme.typography.xs)
                                            .child(if is_syncing { "↻" } else { "✓" })
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.colors.text_secondary)
                                            .text_size(theme.typography.xs)
                                            .child(if is_syncing { "Syncing..." } else { "Saved" })
                                    )
                            })
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
                        if let Some(entity) = self.get_selected_entity(cx) {
                            div().child(self.render_entity_inspector(entity, cx))
                        } else {
                            div().child(self.render_no_selection())
                        }
                    })
            )
    }
}
