//! Director Box Component
//!
//! The Director is a timeline-based animation and cutscene editor matching the HTML prototype:
//! - Menu bar at the top (File, Edit, Assets, GameObject, Component, Window, AI, Help)
//! - Director toolbar with transport controls and timeline selector
//! - Main area: Viewport preview (left) + Keyframe Inspector (right, 300px)
//! - Timeline panel at the bottom with tracks and keyframes

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext as _, InteractiveElement,
};
use std::sync::Arc;

use crate::ui::theme::Theme;
use crate::ui::layouts::{WorkspaceLayout, MenuBar};
use crate::services::engine_bridge::EngineHandle;

use super::toolbar::DirectorToolbar;
use super::viewport_panel::DirectorViewportPanel;
use super::inspector::KeyframeInspector;
use super::timeline::Timeline;

/// The Director Box component
///
/// Layout structure matching HTML prototype:
/// ```
/// ┌─────────────────────────────────────────────────────────────┐
/// │ Menu Bar (File, Edit, Assets, GameObject, Component, ...)  │
/// ├─────────────────────────────────────────────────────────────┤
/// │ Director Toolbar (Transport, Timeline selector, Status)    │
/// ├─────────────────────────────────────┬───────────────────────┤
/// │                                     │                       │
/// │   Viewport Preview                  │  Keyframe Inspector   │
/// │   (Camera path preview)             │  (300px fixed)        │
/// │                                     │                       │
/// ├─────────────────────────────────────┴───────────────────────┤
/// │ Timeline Panel (Tracks, Keyframes, Ruler)                  │
/// └─────────────────────────────────────────────────────────────┘
/// ```
pub struct DirectorBox {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Engine handle
    _engine_handle: Arc<EngineHandle>,
    /// Toolbar component
    toolbar: View<DirectorToolbar>,
    /// Viewport panel
    viewport_panel: View<DirectorViewportPanel>,
    /// Inspector panel
    inspector: View<KeyframeInspector>,
    /// Timeline panel
    timeline: View<Timeline>,
    /// Current playback time (in seconds)
    current_time: f32,
    /// Total timeline duration (in seconds)
    duration: f32,
    /// Whether playback is active
    is_playing: bool,
}

impl DirectorBox {
    /// Create a new Director Box
    pub fn new(engine_handle: Arc<EngineHandle>, theme: Arc<Theme>, cx: &mut ViewContext<Self>) -> Self {
        // Create child views
        let toolbar = cx.new_view(|_cx| DirectorToolbar::new(theme.clone()));
        let viewport_panel = cx.new_view(|_cx| DirectorViewportPanel::new(theme.clone()));
        let inspector = cx.new_view(|_cx| KeyframeInspector::new(theme.clone()));
        
        // Create timeline with sample tracks
        let timeline = cx.new_view(|_cx| {
            let mut timeline = Timeline::new(theme.clone());
            timeline.add_sample_tracks();
            timeline
        });

        Self {
            theme,
            _engine_handle: engine_handle,
            toolbar,
            viewport_panel,
            inspector,
            timeline,
            current_time: 1.5, // Start at 1.5s as per HTML
            duration: 5.0,     // 5 seconds total
            is_playing: false,
        }
    }

    /// Render the menu bar (kept for potential future use)
    #[allow(dead_code)]
    fn render_menu_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let menu_items = vec!["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"];
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(32.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.lg)
            .children(
                menu_items.into_iter().map(move |label| {
                    let theme = theme.clone();
                    let label = label.to_string();
                    
                    div()
                        .px(theme.spacing.md)
                        .py(theme.spacing.xs)
                        .rounded(theme.borders.xs)
                        .hover(|this| this.bg(theme.colors.surface_hover))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_color(theme.colors.text)
                                .text_size(theme.typography.md)
                                .child(label)
                        )
                })
            )
    }

    /// Render the main content area (kept for potential future use)
    #[allow(dead_code)]
    fn render_main_area(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .flex_1()
            .w_full()
            .gap(px(4.0))
            .bg(theme.colors.background)
            // Left: Viewport preview
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.viewport_panel.clone())
            )
            // Right: Inspector (300px fixed)
            .child(
                div()
                    .w(px(300.0))
                    .h_full()
                    .child(self.inspector.clone())
            )
    }

    /// Get current playback time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Get total duration
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Check if playback is active
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Set playback state
    pub fn set_playing(&mut self, playing: bool, cx: &mut ViewContext<Self>) {
        self.is_playing = playing;
        cx.notify();
    }

    /// Seek to specific time
    pub fn seek_to(&mut self, time: f32, cx: &mut ViewContext<Self>) {
        self.current_time = time.clamp(0.0, self.duration);
        cx.notify();
    }
}

impl Render for DirectorBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        // Use the unified WorkspaceLayout for consistent layout structure
        // Note: Director uses a 280px bottom panel for the timeline (taller than standard 200px)
        WorkspaceLayout::new(theme.clone())
            .menu_bar(
                MenuBar::new(theme.clone())
                    .items(vec!["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"])
            )
            .toolbar(self.toolbar.clone())
            .center_panel(self.viewport_panel.clone())
            .right_panel(
                div()
                    .w(px(300.0))
                    .h_full()
                    .child(self.inspector.clone())
            )
            .bottom_panel(
                div()
                    .h(px(280.0))
                    .w_full()
                    .child(self.timeline.clone())
            )
    }
}
