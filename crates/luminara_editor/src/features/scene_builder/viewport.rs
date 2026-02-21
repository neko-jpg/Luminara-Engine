//! Viewport Panel Component
//!
//! Center panel with 3D viewport, grid background, and viewport toolbar

use gpui::{
    canvas, div, px, size, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent, MouseUpEvent, Point, Pixels, Bounds,
};
use std::sync::Arc;
use std::collections::HashSet;
use parking_lot::RwLock;
use crate::ui::theme::Theme;
use crate::services::engine_bridge::{EngineHandle, PreviewBillboard};
use crate::core::viewport::{ViewportElement, Camera, SharedRenderTarget, GizmoMode};
use crate::core::state::EditorStateManager;
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
                    .bg(theme.colors.background)
                    .child(
                        div()
                            .absolute()
                            .inset(px(0.0))
                            .bg(theme.colors.surface.opacity(0.35))
                    )
            )
    }
}

pub struct Viewport3D {
    theme: Arc<Theme>,
    engine_handle: Arc<EngineHandle>,
    viewport_element: ViewportElement,
    state: gpui::Model<EditorStateManager>,
    last_billboards: Vec<PreviewBillboard>,
    drag_start_pos: Option<Point<Pixels>>,
    vp_bounds: Arc<RwLock<Bounds<Pixels>>>,
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
            last_billboards: Vec::new(),
            drag_start_pos: None,
            vp_bounds: Arc::new(RwLock::new(Bounds::default())),
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

    /// Get selected entity name for display
    fn get_selection_text(&self, cx: &ViewContext<Self>) -> String {
        let selected = &self.state.read(cx).session.selected_entities;
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

    fn render_empty_hint(&self, cx: &ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let has_selection = !self.state.read(cx).session.selected_entities.is_empty();

        div()
            .absolute()
            .right(px(12.0))
            .top(px(12.0))
            .px(theme.spacing.md)
            .py(theme.spacing.sm)
            .bg(theme.colors.surface.opacity(0.9))
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.borders.sm)
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child(if has_selection {
                        "LMB: Orbit  |  MMB: Pan  |  RMB: Zoom"
                    } else {
                        "Select an entity in Hierarchy to inspect it here"
                    })
            )
    }

    fn render_scene_preview(
        &self,
        entities: Vec<PreviewBillboard>,
        theme: Arc<Theme>,
    ) -> impl IntoElement {
        let bg = theme.colors.background;
        let grid_major = theme.colors.border.opacity(0.5);
        let grid_minor = theme.colors.border.opacity(0.2);
        let text = theme.colors.text_secondary;
        let accent = theme.colors.accent;
        let accent_hover = theme.colors.accent_hover;
        let normal = theme.colors.text;
        let camera_pos = self.viewport_element.camera.read().position;
        let camera_target = self.viewport_element.camera.read().target;

        let vp_bounds_clone = self.vp_bounds.clone();
        canvas(
            move |_, _| {},
            move |bounds, _, cx| {
                *vp_bounds_clone.write() = bounds;
                cx.paint_quad(gpui::fill(bounds, bg));

                let width = bounds.size.width.0;
                let height = bounds.size.height.0;
                if width <= 1.0 || height <= 1.0 {
                    return;
                }

                let center_x = width / 2.0;
                let center_y = height / 2.0;

                // Perspective grid (major lines every 100px, minor every 20px)
                let major_spacing = 100.0;
                let minor_spacing = 20.0;
                
                // Draw minor grid lines
                let mut x = center_x % minor_spacing;
                while x <= width {
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(x), px(0.0)),
                            size(px(1.0), px(height)),
                        ),
                        grid_minor,
                    ));
                    x += minor_spacing;
                }
                let mut y = center_y % minor_spacing;
                while y <= height {
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(0.0), px(y)),
                            size(px(width), px(1.0)),
                        ),
                        grid_minor,
                    ));
                    y += minor_spacing;
                }

                // Draw major grid lines (world origin)
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(center_x), px(0.0)),
                        size(px(2.0), px(height)),
                    ),
                    grid_major,
                ));
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(0.0), px(center_y)),
                        size(px(width), px(2.0)),
                    ),
                    grid_major,
                ));

                // Draw 3D axis indicator at bottom-left
                let axis_x = 40.0;
                let axis_y = height - 50.0;
                let axis_len = 30.0;
                
                // X axis (red) - using hex color
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(axis_x), px(axis_y)),
                        size(px(axis_len), px(3.0)),
                    ),
                    gpui::rgb(0xe85d5d), // Red
                ));
                // X label
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(axis_x + axis_len + 4.0), px(axis_y - 6.0)),
                        size(px(12.0), px(14.0)),
                    ),
                    gpui::rgb(0xe85d5d),
                ));
                
                // Y axis (green) - pointing up in screen space
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(axis_x), px(axis_y - axis_len)),
                        size(px(3.0), px(axis_len)),
                    ),
                    gpui::rgb(0x5de875), // Green
                ));
                // Y label
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(axis_x - 4.0), px(axis_y - axis_len - 16.0)),
                        size(px(12.0), px(14.0)),
                    ),
                    gpui::rgb(0x5de875),
                ));
                
                // Z axis (blue) - diagonal for 3D effect
                let z_end_x = axis_x + axis_len * 0.7;
                let z_end_y = axis_y + axis_len * 0.7;
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(axis_x), px(axis_y)),
                        size(px(3.0), px(3.0)),
                    ),
                    gpui::rgb(0x5d95e8), // Blue
                ));
                // Z label
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(z_end_x + 4.0), px(z_end_y - 6.0)),
                        size(px(12.0), px(14.0)),
                    ),
                    gpui::rgb(0x5d95e8),
                ));

                // Sort entities by depth (back to front for proper occlusion)
                let mut sorted_entities = entities.clone();
                sorted_entities.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

                for entity in &sorted_entities {
                    let sx = entity.x;
                    let sy = entity.y;
                    let radius = entity.radius;
                    let id_mix = entity.id.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
                    let alpha_boost = 0.75 + ((id_mix as f32 / 255.0) * 0.2);
                    
                    // Depth-based alpha fade
                    let depth_alpha = (1.0 - (entity.depth / 50.0).min(1.0) * 0.5).max(0.4);

                    // Selection glow effect
                    if entity.selected {
                        // Outer glow
                        cx.paint_quad(gpui::fill(
                            gpui::Bounds::new(
                                gpui::Point::new(px(sx - radius - 4.0), px(sy - radius - 4.0)),
                                size(px(radius * 2.0 + 8.0), px(radius * 2.0 + 8.0)),
                            ),
                            accent.opacity(0.3),
                        ));
                        // Inner highlight
                        cx.paint_quad(gpui::fill(
                            gpui::Bounds::new(
                                gpui::Point::new(px(sx - radius - 2.0), px(sy - radius - 2.0)),
                                size(px(radius * 2.0 + 4.0), px(radius * 2.0 + 4.0)),
                            ),
                            accent.opacity(0.6),
                        ));
                    }

                    // Entity representation (cube-like with depth shading)
                    let entity_color = if entity.selected {
                        accent
                    } else {
                        normal.opacity(alpha_boost * depth_alpha)
                    };
                    
                    // Main entity quad
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(sx - radius), px(sy - radius)),
                            size(px(radius * 2.0), px(radius * 2.0)),
                        ),
                        entity_color,
                    ));
                    
                    // Top face (3D effect)
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(sx - radius + 3.0), px(sy - radius - 4.0)),
                            size(px(radius * 2.0 - 3.0), px(4.0)),
                        ),
                        if entity.selected { accent_hover } else { normal.opacity(0.6) },
                    ));
                    
                    // Right face (3D effect)
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(sx + radius), px(sy - radius + 3.0)),
                            size(px(4.0), px(radius * 2.0 - 3.0)),
                        ),
                        if entity.selected { accent_hover.opacity(0.7) } else { normal.opacity(0.4) },
                    ));

                    // Entity label background
                    let label_w = (entity.name.len() as f32 * 6.0 + 8.0).clamp(30.0, 140.0);
                    let label_h = 16.0;
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(sx - label_w / 2.0), px(sy + radius + 6.0)),
                            size(px(label_w), px(label_h)),
                        ),
                        bg.opacity(0.8),
                    ));
                    cx.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::Point::new(px(sx - label_w / 2.0), px(sy + radius + 6.0)),
                            size(px(label_w), px(1.0)),
                        ),
                        if entity.selected { accent } else { text.opacity(0.3) },
                    ));
                }
                
                // Camera info overlay at top-right
                let info_x = width - 140.0;
                let info_y = 10.0;
                cx.paint_quad(gpui::fill(
                    gpui::Bounds::new(
                        gpui::Point::new(px(info_x), px(info_y)),
                        size(px(130.0), px(50.0)),
                    ),
                    bg.opacity(0.7),
                ));
            },
        )
        .absolute()
        .inset(px(0.0))
        .w_full()
        .h_full()
    }
}

impl Render for Viewport3D {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        let state = self.state.read(cx);
        let active_tool = &state.session.active_tool;
        
        let selected_entities: HashSet<luminara_core::Entity> = state.session.selected_entities.iter()
            .filter_map(|id_str| {
                if let Some((id_part, gen_part)) = id_str.split_once(':') {
                    if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                        return Some(luminara_core::Entity::from_raw(id, gen));
                    }
                }
                None
            })
            .collect();

        let camera_snapshot = self.viewport_element.camera.read().clone();
        let selected_ids: HashSet<String> = state.session.selected_entities.iter().cloned().collect();
        let preview_size = self.viewport_element.render_target.read().size();
        let world = self.engine_handle.world();
        let preview_entities = self
            .engine_handle
            .render_pipeline_mut()
            .project_scene_billboards(
                &world,
                camera_snapshot.position,
                camera_snapshot.target,
                camera_snapshot.up,
                camera_snapshot.fov,
                preview_size.0 as f32,
                preview_size.1 as f32,
                &selected_ids,
            );
        drop(world);
        
        let mut element = self.viewport_element.clone()
            .with_selected_entities(selected_entities);
            
        element.gizmo_mode = match active_tool.as_str() {
            "Move" => GizmoMode::Translate,
            "Rotate" => GizmoMode::Rotate,
            "Scale" => GizmoMode::Scale,
            _ => GizmoMode::None,
        };

        self.last_billboards = preview_entities.clone();

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
                    .child(
                        // 3D viewport element placeholder
                        div()
                            .absolute()
                            .inset(px(0.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _cx| {
                                this.drag_start_pos = Some(event.position);
                            }))
                            .on_mouse_up(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, cx| {
                                if let Some(start_pos) = this.drag_start_pos.take() {
                                    let dist = (event.position.x - start_pos.x).0.abs() + (event.position.y - start_pos.y).0.abs();
                                    if dist < 5.0 {
                                        let bounds = *this.vp_bounds.read();
                                        let local_x = (event.position.x - bounds.origin.x).0;
                                        let local_y = (event.position.y - bounds.origin.y).0;
                                        
                                        let mut sorted = this.last_billboards.clone();
                                        sorted.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));
                                        
                                        let mut clicked_id = None;
                                        for bb in sorted {
                                            let dx = bb.x - local_x;
                                            let dy = bb.y - local_y;
                                            let pick_radius = bb.radius.max(20.0);
                                            if dx * dx + dy * dy <= pick_radius * pick_radius {
                                                clicked_id = Some(bb.id.clone());
                                                break;
                                            }
                                        }
                                        
                                        this.state.update(cx, |state, cx| {
                                            if let Some(id) = clicked_id {
                                                let mut new_selection = state.session.selected_entities.clone();
                                                if event.modifiers.shift {
                                                    if new_selection.contains(&id) {
                                                        new_selection.retain(|x| x != &id);
                                                    } else {
                                                        new_selection.push(id.clone());
                                                    }
                                                } else {
                                                    new_selection = vec![id.clone()];
                                                }
                                                state.select_entities(new_selection, cx);
                                            } else if !event.modifiers.shift {
                                                state.select_entities(vec![], cx);
                                            }
                                        });
                                        cx.notify();
                                    }
                                }
                            }))
                            .child(GridBackground::render(theme.clone()))
                            .child(self.render_scene_preview(preview_entities, theme.clone()))
                            .child(element)
                    )
                    .child(self.render_toolbar(cx))
                    .child(self.render_empty_hint(cx))
                    .child(self.render_selection_indicator(cx))
            )
    }
}
