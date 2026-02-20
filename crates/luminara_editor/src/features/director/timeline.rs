//! Director Timeline Component
//!
//! Displays the timeline panel with:
//! - Time ruler with markers
//! - Track list with headers
//! - Keyframe visualization
//! - Playhead position indicator
//! - Footer controls

use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Track kind (what type of property this track controls)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Transform,
    Camera,
    Events,
}

impl TrackKind {
    /// Get the icon for this track kind
    pub fn icon(&self) -> &'static str {
        match self {
            TrackKind::Transform => "👤",
            TrackKind::Camera => "📷",
            TrackKind::Events => "🚩",
        }
    }
}

/// Keyframe kind (normal or event)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeKind {
    Normal,
    Event,
}

/// A single keyframe
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Time position in seconds
    pub time: f32,
    /// Kind of keyframe
    pub kind: KeyframeKind,
    /// Optional label for events
    pub label: Option<String>,
}

impl Keyframe {
    /// Create a new normal keyframe
    pub fn new(time: f32) -> Self {
        Self {
            time,
            kind: KeyframeKind::Normal,
            label: None,
        }
    }

    /// Create a new event keyframe
    pub fn event(time: f32, label: impl Into<String>) -> Self {
        Self {
            time,
            kind: KeyframeKind::Event,
            label: Some(label.into()),
        }
    }
}

/// A timeline track
#[derive(Debug, Clone)]
pub struct Track {
    /// Track name
    pub name: String,
    /// Track kind
    pub kind: TrackKind,
    /// Keyframes on this track
    pub keyframes: Vec<Keyframe>,
    /// Whether this track is expanded
    pub is_expanded: bool,
    /// Indentation level (0 = root, 1 = child, etc.)
    pub indent: usize,
}

impl Track {
    /// Create a new track
    pub fn new(name: impl Into<String>, kind: TrackKind) -> Self {
        Self {
            name: name.into(),
            kind,
            keyframes: Vec::new(),
            is_expanded: true,
            indent: 0,
        }
    }

    /// Create a child track (indented)
    pub fn child(name: impl Into<String>, kind: TrackKind) -> Self {
        Self {
            name: name.into(),
            kind,
            keyframes: Vec::new(),
            is_expanded: true,
            indent: 1,
        }
    }

    /// Add a keyframe
    pub fn with_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self
    }

    /// Expand/collapse
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }
}

/// The Timeline component
pub struct Timeline {
    /// Theme for styling
    theme: Arc<Theme>,
    /// List of tracks
    tracks: Vec<Track>,
    /// Current playhead position (in seconds)
    playhead_time: f32,
    /// Timeline duration (in seconds)
    duration: f32,
    /// Pixels per second for time-to-pixel conversion
    pixels_per_second: f32,
    /// Track header width
    header_width: f32,
}

impl Timeline {
    /// Create a new Timeline
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            tracks: Vec::new(),
            playhead_time: 1.5,
            duration: 5.0,
            pixels_per_second: 40.0,
            header_width: 140.0,
        }
    }

    /// Add sample tracks matching the HTML prototype
    pub fn add_sample_tracks(&mut self) {
        // Player track
        self.tracks.push(
            Track::new("Player", TrackKind::Transform)
                .with_keyframe(Keyframe::new(0.2))
                .with_keyframe(Keyframe::new(1.0))
                .with_keyframe(Keyframe::new(2.5))
                .with_keyframe(Keyframe::new(3.8))
        );

        // position.x child track
        self.tracks.push(
            Track::child("position.x", TrackKind::Transform)
                .with_keyframe(Keyframe::new(0.2))
                .with_keyframe(Keyframe::new(1.0))
        );

        // position.y child track
        self.tracks.push(
            Track::child("position.y", TrackKind::Transform)
                .with_keyframe(Keyframe::new(1.0))
                .with_keyframe(Keyframe::new(2.5))
        );

        // Camera track
        self.tracks.push(
            Track::new("Camera", TrackKind::Camera)
                .with_keyframe(Keyframe::new(0.5))
                .with_keyframe(Keyframe::new(3.0))
        );

        // Events track
        self.tracks.push(
            Track::new("Events", TrackKind::Events)
                .with_keyframe(Keyframe::event(0.8, "sfx"))
                .with_keyframe(Keyframe::event(2.2, "dialogue"))
                .with_keyframe(Keyframe::event(4.0, "fade"))
        );
    }

    /// Get pixel position from time
    fn time_to_pixels(&self, time: f32) -> f32 {
        self.header_width + time * self.pixels_per_second
    }

    /// Render the time ruler
    fn render_ruler(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme_post = self.theme.clone();
        let seconds = (self.duration as usize) + 1;
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(24.0))
            .border_b_1()
            .border_color(theme.colors.border)
            .pb(px(4.0))
            .mb(px(4.0))
            // Track header placeholder
            .child(
                div()
                    .w(px(self.header_width))
                    .h_full()
                    .flex()
                    .items_center()
                    .px(px(8.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.xs)
                            .child("Track Headers")
                    )
            )
            // Ruler markers
            .child(
                div()
                    .flex_1()
                    .relative()
                    .flex()
                    .flex_row()
                    .children(
                        (0..seconds).map(move |s| {
                            let theme = theme.clone();
                            div()
                                .flex_1()
                                .h_full()
                                .border_l_1()
                                .border_color(theme.colors.border)
                                .flex()
                                .justify_center()
                                .child(
                                    div()
                                        .text_color(theme.colors.text_secondary)
                                        .text_size(theme.typography.xs)
                                        .child(format!("{}s", s))
                                )
                        })
                    )
                    // Playhead indicator on ruler
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .w(px(2.0))
                            .bg(theme_post.colors.error)
                            .left(px(self.time_to_pixels(self.playhead_time)))
                    )
            )
    }

    /// Render a single track row
    fn render_track_row(&self, track: &Track) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme_post = self.theme.clone();
        let header_width = self.header_width - (track.indent as f32 * 20.0);
        let indent = px((track.indent * 16) as f32);
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(32.0))
            .border_b_1()
            .border_color(theme.colors.border)
            // Track header
            .child(
                div()
                    .w(px(header_width))
                    .ml(indent)
                    .h_full()
                    .bg(theme.colors.panel_header)
                    .border_r_1()
                    .border_color(theme.colors.border)
                    .flex()
                    .flex_row()
                    .items_center()
                    .pl(px(8.0))
                    .gap(px(6.0))
                    // Expand/collapse arrow
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.xs)
                            .child(if track.is_expanded { "▼" } else { "▶" })
                    )
                    // Track icon
                    .child(
                        div()
                            .text_color(theme.colors.accent)
                            .text_size(theme.typography.sm)
                            .child(track.kind.icon())
                    )
                    // Track name
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(track.name.clone())
                    )
            )
            // Keyframe area
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .relative()
                    .bg(theme.colors.surface)
                    // Grid lines
                    .child(
                        div()
                            .absolute()
                            .inset(px(0.0))
                            // Simulated vertical grid lines
                    )
                    // Keyframes
                    .children(
                        track.keyframes.iter().map(move |kf| {
                            let theme = theme.clone();
                            let left = self.time_to_pixels(kf.time);
                            
                            match kf.kind {
                                KeyframeKind::Normal => {
                                    // Normal keyframe (circle)
                                    div()
                                        .absolute()
                                        .top(px(8.0))
                                        .left(px(left - 4.0)) // Center the 8px dot
                                        .w(px(8.0))
                                        .h(px(8.0))
                                        .bg(theme.colors.accent)
                                        .rounded(px(4.0))
                                        .cursor_pointer()
                                }
                                KeyframeKind::Event => {
                                    // Event keyframe (square)
                                    div()
                                        .absolute()
                                        .top(px(6.0))
                                        .left(px(left - 5.0)) // Center the 10px square
                                        .w(px(10.0))
                                        .h(px(10.0))
                                        .bg(theme.colors.warning)
                                        .rounded(px(2.0))
                                        .cursor_pointer()
                                }
                            }
                        })
                    )
                    // Playhead line
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .w(px(2.0))
                            .bg(theme_post.colors.error)
                            .left(px(self.time_to_pixels(self.playhead_time)))
                    )
            )
    }

    /// Render the track list
    fn render_track_list(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .children(
                self.tracks.iter().map(move |track| {
                    self.render_track_row(track)
                })
            )
            // Add Track button
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.panel_header)
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .text_color(theme.colors.accent)
                            .text_size(theme.typography.sm)
                            .child("+")
                            .child("Add Track")
                    )
            )
    }

    /// Render the timeline footer
    fn render_footer(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(40.0))
            .bg(theme.colors.panel_header)
            .border_t_1()
            .border_color(theme.colors.border)
            .mt(px(4.0))
            .px(px(12.0))
            .items_center()
            .gap(px(16.0))
            // Footer controls
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .cursor_pointer()
                            .hover(|this| this.text_color(theme.colors.text))
                            .child("⏮")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .cursor_pointer()
                            .hover(|this| this.text_color(theme.colors.accent))
                            .child("▶")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .cursor_pointer()
                            .hover(|this| this.text_color(theme.colors.text))
                            .child("⏭")
                    )
            )
            // Time display
            .child(
                div()
                    .font_family("monospace")
                    .text_color(theme.colors.accent)
                    .text_size(theme.typography.ml)
                    .child(format!("{:.3} / {:.3} s", self.playhead_time, self.duration))
            )
            // Loop indicator
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("↻")
                    .child("Loop")
            )
            // Speed indicator
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("◷")
                    .child("1.0x")
            )
            // Spacer
            .child(div().flex_1())
            // Resize handle
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .cursor_pointer()
                    .child("☰")
            )
    }
}

impl Render for Timeline {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            .min_h(px(240.0))
            .max_h(px(300.0))
            .p(px(8.0))
            // Ruler
            .child(self.render_ruler())
            // Track list
            .child(self.render_track_list())
            // Footer
            .child(self.render_footer())
    }
}
