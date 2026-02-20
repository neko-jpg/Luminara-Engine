//! Viewport Panel Component
//!
//! Center panel with 3D viewport, grid background, and viewport toolbar

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent,
};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use crate::core::viewport::{ViewportElement, Camera, SharedRenderTarget, GizmoMode};
use crate::features::scene_builder::SceneBuilderState;
use crate::features::scene_builder::toolbar::ToolMode;

/// Grid background pattern for viewport
pub struct GridBackground;

impl GridBackground {
    /// Render grid dots pattern
    pub fn render(theme: Arc<Theme>) -> impl IntoElement {
        div()
            .absolute()
            .inset(px(0.0))
            .size_full()
            .child(
                div()
                    .size_full()
                    .child(
                        div()
                            .size_full()
                            .child(
                                // Grid dots using radial gradient simulation
                                div()
                                    .size_full()
                                    .bg(theme.colors.background)
                            )
                    )
            )
    }
}

/// 3D Viewport component
pub struct Viewport3D {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    viewport_element: ViewportElement,
    state: gpui::Model<SceneBuilderState>,
}

impl Viewport3D {
    /// Create a new 3D viewport
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        state: gpui::Model<SceneBuilderState>,
    ) -> Self {
        // Create shared render target
        let render_target = Arc::new(RwLock::new(SharedRenderTarget::new((800, 600))));

        // Create camera
        let camera = Arc::new(RwLock::new(Camera::new()));

        // Create viewport element
        let viewport_element = ViewportElement::new(
            render_target.clone(),
            camera.clone(),
            GizmoMode::Translate,
            theme.clone(),
        )
        .with_engine_handle(engine_handle.clone());

        Self {
            theme,
            engine_handle,
            viewport_element,
            state,
        }
    }

    /// Render viewport toolbar
    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let active_tool = self.state.read(cx).active_tool;

        div()
            .absolute()
            .top(px(12.0))
            .left(px(12.0))
            .flex()
            .flex_row()
            .gap(theme.spacing.xs)
            .px(theme.spacing.md)
            .py(theme.spacing.sm)
            .bg(theme.colors.surface.opacity(0.9))
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.borders.rounded)
            .children([
                (ToolMode::Move, "W"),
                (ToolMode::Rotate, "E"),
                (ToolMode::Scale, "R"),
            ].into_iter().map(move |(tool, _shortcut)| {
                let theme = theme.clone();
                let is_active = active_tool == tool;
                let tool_clone = tool;
                let state_clone = self.state.clone();

                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .px(theme.spacing.md)
                    .py(theme.spacing.xs)
                    .rounded(theme.borders.xs)
                    .bg(if is_active { theme.colors.toolbar_active } else { gpui::transparent_black() })
                    .hover(|this| {
                        if !is_active {
                            this.bg(theme.colors.surface_hover)
                        } else {
                            this
                        }
                    })
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _event: &MouseDownEvent, cx| {
                        state_clone.update(cx, |state, cx| {
                            state.active_tool = tool_clone;
                            cx.notify();
                        });
                    }))
                    .child(
                        div()
                            .text_color(if is_active { theme.colors.text } else { theme.colors.text })
                            .text_size(theme.typography.sm)
                            .child(tool.icon())
                    )
                    .child(
                        div()
                            .text_color(if is_active { theme.colors.text } else { theme.colors.text })
                            .text_size(theme.typography.sm)
                            .child(tool.label())
                    )
            }))
    }

    /// Get selected entity name for display
    fn get_selection_text(&self, cx: &ViewContext<Self>) -> String {
        let selected = &self.state.read(cx).selected_entities;
        if selected.is_empty() {
            "No selection".to_string()
        } else if selected.len() == 1 {
            format!("{:?} selected", selected.iter().next().unwrap())
        } else {
            format!("{} entities selected", selected.len())
        }
    }

    /// Render selection indicator
    fn render_selection_indicator(&self, cx: &ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let text = self.get_selection_text(cx);

        div()
            .absolute()
            .bottom(px(12.0))
            .left(px(50.0))
            .right(px(50.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.sm)
                    .px(theme.spacing.lg)
                    .py(theme.spacing.sm)
                    .bg(theme.colors.background.opacity(0.8))
                    .rounded(theme.borders.rounded)
                    .border_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .text_color(theme.colors.accent)
                            .text_size(theme.typography.lg)
                            .child("◆")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child(text)
                    )
            )
    }
}

impl Render for Viewport3D {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        let state = self.state.read(cx);
        let active_tool = state.active_tool;
        let selected_entities = state.selected_entities.clone();
        
        let mut element = self.viewport_element.clone()
            .with_selected_entities(selected_entities);
        element.gizmo_mode = match active_tool {
            ToolMode::Move => GizmoMode::Translate,
            ToolMode::Rotate => GizmoMode::Rotate,
            ToolMode::Scale => GizmoMode::Scale,
            ToolMode::Select => GizmoMode::None,
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme.colors.background)
            .child(
                // Panel header
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .items_center()
                    .px(theme.spacing.md)
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.md)
                            .child("Viewport [3D]")
                    )
                    .child(div().flex_1())
                    .child(
                        // Header icons
                        div()
                            .flex()
                            .items_center()
                            .gap(theme.spacing.md)
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.md)
                                    .cursor_pointer()
                                    .hover(|this| this.text_color(theme.colors.text))
                                    .child("⊡")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.md)
                                    .cursor_pointer()
                                    .hover(|this| this.text_color(theme.colors.text))
                                    .child("▶")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.md)
                                    .cursor_pointer()
                                    .hover(|this| this.text_color(theme.colors.text))
                                    .child("⚙")
                            )
                    )
            )
            .child(
                // Viewport area
                div()
                    .flex_1()
                    .relative()
                    .child(GridBackground::render(theme.clone()))
                    .child(self.render_toolbar(cx))
                    .child(self.render_selection_indicator(cx))
                    .child(
                        // 3D viewport element placeholder
                        div()
                            .absolute()
                            .inset(px(0.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(element)
                    )
            )
    }
}
