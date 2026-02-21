//! Viewport Panel Component
//!
//! Center panel with 3D viewport displaying Bevy output

use gpui::{
    div, px, size, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent, MouseUpEvent, Point, Pixels, Bounds,
    img, VisualContext, ImageSource, canvas,
};
use std::sync::Arc;
use std::collections::HashSet;
use parking_lot::RwLock;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::{EngineHandle, PreviewBillboard};
use crate::core::viewport::{ViewportElement, Camera, SharedRenderTarget, GizmoMode};
use crate::core::state::EditorStateManager;
use crate::features::scene_builder::toolbar::ToolMode;
use std::time::Duration;

pub struct GridBackground;

pub struct Viewport3D {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    viewport_element: ViewportElement,
    state: gpui::Model<EditorStateManager>,
    // Legacy field, kept for compatibility if needed or can be removed
    last_billboards: Vec<PreviewBillboard>,
    drag_start_pos: Option<Point<Pixels>>,
    vp_bounds: Arc<RwLock<Bounds<Pixels>>>,
    current_image: Option<ImageSource>,
}

impl Viewport3D {
    /// Create a new 3D viewport
    pub fn new(
        theme: Arc<Theme>,
        engine_handle: Arc<EngineHandle>,
        state: gpui::Model<EditorStateManager>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        cx.observe(&state, |_this: &mut Viewport3D, _model, cx| {
            cx.notify();
        }).detach();

        // Spawn update loop for rendering (60fps)
        cx.spawn(|this, mut cx| async move {
            loop {
                // Wait for ~16ms
                cx.background_executor().timer(Duration::from_secs_f64(1.0/60.0)).await;

                // Update and Request Repaint
                let _ = this.update(&mut cx, |_this, cx| {
                    cx.notify();
                });
            }
        }).detach();

        // Create shared render target (Legacy WGPU integration, kept for ViewportElement compatibility)
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
            last_billboards: Vec::new(),
            drag_start_pos: None,
            vp_bounds: Arc::new(RwLock::new(Bounds::default())),
            current_image: None,
        }
    }

    /// Render viewport toolbar
    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let active_tool = self.state.read(cx).session.active_tool.clone();

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
                let is_active = active_tool == tool.label();
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
                            state.set_active_tool(tool_clone.label().to_string(), cx);
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
}

impl Render for Viewport3D {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let state = self.state.read(cx);
        let active_tool = &state.session.active_tool;
        
        let selected_entities: HashSet<luminara_core::Entity> = state.session.selected_entities.iter()
            .filter_map(|id_str| {
                // Bevy Entity parsing (index:generation)
                if let Some((id_part, gen_part)) = id_str.split_once(':') {
                    if let (Ok(id), Ok(gen)) = (id_part.parse::<u64>(), gen_part.parse::<u32>()) {
                         return Some(luminara_core::Entity::from_raw(id as u32));
                    }
                }
                None
            })
            .collect();
            
        // Check for new frame from Bevy
        if let Some(data) = self.engine_handle.poll_image_event() {
             // FIXME: Image creation is disabled for MVP stability
             // self.current_image = Some(ImageSource::from_bytes(data, 800, 600));
        }

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
                    .child(div().text_color(theme.colors.text).child("Viewport [Bevy Backend]"))
            )
            .child(
                // Viewport area
                div()
                    .flex_1()
                    .relative()
                    .child(
                         // Render Bevy Image (via canvas for MVP)
                         if let Some(frame) = self.engine_handle.get_last_frame() {
                             canvas(
                                 |_cx, _bounds| {},
                                 move |bounds, _, cx| {
                                     // Draw something to prove frame update
                                     let r = frame.get(0).unwrap_or(&0);
                                     let g = frame.get(1).unwrap_or(&0);
                                     let b = frame.get(2).unwrap_or(&0);

                                     cx.paint_quad(gpui::fill(
                                         bounds,
                                         gpui::Rgba { r: *r as f32 / 255.0, g: *g as f32 / 255.0, b: *b as f32 / 255.0, a: 1.0 }
                                     ));
                                 }
                             )
                             .size_full()
                             .into_any_element()
                         } else {
                             div().child("Waiting for Bevy...").into_any_element()
                         }
                    )
                    .child(self.render_toolbar(cx))
            )
    }
}
