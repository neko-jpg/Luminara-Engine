//! Viewport Panel Component
//!
//! Center panel with 3D viewport, grid background, and viewport toolbar

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent,
};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashSet;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::EngineHandle;
use crate::core::viewport::{ViewportElement, Camera, SharedRenderTarget, GizmoMode};
use luminara_core::Entity;

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

/// Viewport tool mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportTool {
    Move,
    Rotate,
    Scale,
}

impl ViewportTool {
    pub fn icon(&self) -> &'static str {
        match self {
            ViewportTool::Move => "↔",
            ViewportTool::Rotate => "↻",
            ViewportTool::Scale => "⤢",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewportTool::Move => "Move",
            ViewportTool::Rotate => "Rotate",
            ViewportTool::Scale => "Scale",
        }
    }
}

/// 3D Viewport component
pub struct Viewport3D {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    viewport_element: ViewportElement,
    active_tool: ViewportTool,
    selected_entities: Arc<RwLock<HashSet<Entity>>>,
}

impl Viewport3D {
    /// Create a new 3D viewport
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        selected_entities: Arc<RwLock<HashSet<Entity>>>,
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
        .with_engine_handle(engine_handle.clone())
        .with_selected_entities(selected_entities.clone());

        Self {
            theme,
            engine_handle,
            viewport_element,
            active_tool: ViewportTool::Move,
            selected_entities,
        }
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, tool: ViewportTool) {
        self.active_tool = tool;
        // Update viewport element gizmo mode
        let render_target = self.viewport_element.render_target.clone();
        let camera = self.viewport_element.camera.clone();
        let gizmo_mode = match tool {
            ViewportTool::Move => GizmoMode::Translate,
            ViewportTool::Rotate => GizmoMode::Rotate,
            ViewportTool::Scale => GizmoMode::Scale,
        };
        self.viewport_element = ViewportElement::new(
            render_target,
            camera,
            gizmo_mode,
            self.theme.clone(),
        )
        .with_engine_handle(self.engine_handle.clone())
        .with_selected_entities(self.selected_entities.clone());
    }

    /// Render viewport toolbar
    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

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
                (ViewportTool::Move, "W"),
                (ViewportTool::Rotate, "E"),
                (ViewportTool::Scale, "R"),
            ].into_iter().map(move |(tool, _shortcut)| {
                let theme = theme.clone();
                let is_active = self.active_tool == tool;
                let tool_clone = tool;

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
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                        this.set_active_tool(tool_clone);
                        cx.notify();
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
    fn get_selection_text(&self) -> String {
        let selected = self.selected_entities.read();
        if selected.is_empty() {
            "No selection".to_string()
        } else if selected.len() == 1 {
            format!("{:?} selected", selected.iter().next().unwrap())
        } else {
            format!("{} entities selected", selected.len())
        }
    }

    /// Render selection indicator
    fn render_selection_indicator(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let text = self.get_selection_text();

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
        let theme = self.theme.clone();

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
                    .child(self.render_selection_indicator())
                    .child(
                        // 3D viewport element placeholder
                        div()
                            .absolute()
                            .inset(px(0.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.viewport_element.clone())
                    )
            )
    }
}
