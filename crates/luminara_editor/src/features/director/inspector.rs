//! Director Keyframe Inspector Component
//!
//! Displays and edits keyframe properties:
//! - Time value
//! - Property type (position.x, position.y, rotation, etc.)
//! - Value input
//! - Easing function selector
//! - Bezier curve preview
//! - Previous/Next navigation
//! - Track settings

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Easing function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bezier,
}

impl EasingFunction {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            EasingFunction::Linear => "Linear",
            EasingFunction::EaseIn => "EaseIn",
            EasingFunction::EaseOut => "EaseOut",
            EasingFunction::EaseInOut => "EaseInOut",
            EasingFunction::Bezier => "Bezier",
        }
    }
}

/// Property types for keyframes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    PositionX,
    PositionY,
    PositionZ,
    Rotation,
    Scale,
}

impl PropertyType {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            PropertyType::PositionX => "position.x",
            PropertyType::PositionY => "position.y",
            PropertyType::PositionZ => "position.z",
            PropertyType::Rotation => "rotation",
            PropertyType::Scale => "scale",
        }
    }
}

/// The Keyframe Inspector component
pub struct KeyframeInspector {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current keyframe time
    keyframe_time: f32,
    /// Selected property type
    property_type: PropertyType,
    /// Current value
    value: String,
    /// Selected easing function
    easing: EasingFunction,
    /// Track target name
    track_target: String,
    /// Track color
    track_color: gpui::Hsla,
}

impl KeyframeInspector {
    /// Create a new Keyframe Inspector
    pub fn new(theme: Arc<Theme>) -> Self {
        let theme_clone = theme.clone();
        let accent = theme_clone.colors.accent;
        Self {
            theme,
            keyframe_time: 1.5,
            property_type: PropertyType::PositionX,
            value: "5.23".to_string(),
            easing: EasingFunction::EaseInOut,
            track_target: "Player.Transform".to_string(),
            track_color: accent,
        }
    }

    /// Render a detail row with label and value
    fn render_detail_row(
        &self,
        label: &str,
        content: impl IntoElement,
    ) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .mb(px(12.0))
            .gap(px(8.0))
            .child(
                div()
                    .w(px(70.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child(label)
            )
            .child(
                div()
                    .flex_1()
                    .child(content)
            )
    }

    /// Render a text input field
    fn render_input(&self, value: &str) -> impl IntoElement {
        crate::ui::components::TextInput::new("input_field")
            .value(value)
    }

    /// Render a dropdown selector
    fn render_dropdown(&self, _options: Vec<&str>, selected: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let selected = selected.to_string();
        
        div()
            .flex_1()
            .px(px(8.0))
            .py(px(6.0))
            .bg(theme.colors.surface_active)
            .border_1()
            .border_color(theme.colors.border)
            .rounded(px(6.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.sm)
                    .child(selected)
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("▼")
            )
    }

    /// Render the keyframe detail section
    fn render_keyframe_detail(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .p(theme.spacing.md)
            .bg(theme.colors.surface)
            .rounded(px(8.0))
            .border_1()
            .border_color(theme.colors.border)
            // Time row
            .child(self.render_detail_row(
                "Time",
                self.render_input(&format!("{:.3} s", self.keyframe_time))
            ))
            // Property row
            .child(self.render_detail_row(
                "Property",
                self.render_dropdown(
                    vec!["position.x", "position.y", "position.z", "rotation", "scale"],
                    self.property_type.label()
                )
            ))
            // Value row
            .child(self.render_detail_row(
                "Value",
                self.render_input(&self.value)
            ))
            // Easing row
            .child(self.render_detail_row(
                "Easing",
                self.render_dropdown(
                    vec!["Linear", "EaseIn", "EaseOut", "EaseInOut", "Bezier"],
                    self.easing.label()
                )
            ))
            // Bezier preview
            .child(
                div()
                    .mt(px(12.0))
                    .p(theme.spacing.md)
                    .bg(theme.colors.canvas_background)
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(theme.colors.border)
                    .items_center().justify_center()
                    .child(
                        div()
                            .text_color(theme.colors.accent)
                            .text_size(theme.typography.xs)
                            .child("╭────╮\n╭╯    ╰╮\n╯      ╰──╮\nBezier handle")
                    )
            )
            // Prev/Next navigation
            .child(
                div()
                    .mt(px(16.0))
                    .text_size(theme.typography.sm)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .child("← Prev")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .child("Next →")
                    )
            )
            // Select in Scene Builder button
            .child(
                div()
                    .mt(px(8.0))
                    .w_full()
                    .child(
                        crate::ui::components::Button::new("select_in_scene", "Select in Scene Builder")
                            .icon("◈")
                            .variant(crate::ui::components::ButtonVariant::Secondary)
                            .full_width(true)
                    )
            )
    }

    /// Render the track settings section
    fn render_track_settings(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .mt(px(16.0))
            // Section header
            .child(
                div()
                    .mb(px(8.0))
                    .text_size(theme.typography.sm)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .child("☰")
                    )
                    .child("Track Settings")
            )
            // Target row
            .child(self.render_detail_row(
                "Target",
                div()
                    .flex_1()
                    .px(px(8.0))
                    .py(px(6.0))
                    .child(
                        div()
                            .font_family("monospace")
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child(self.track_target.clone())
                    )
            ))
            // Color row
            .child(self.render_detail_row(
                "Color",
                div()
                    .w(px(20.0))
                    .h(px(20.0))
                    .bg(self.track_color)
                    .rounded(px(4.0))
            ))
    }
}

impl Render for KeyframeInspector {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_l_1()
            .border_color(theme.colors.border)
            // Panel header
            .child(
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
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("ℹ")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.md)
                                    .child("Inspector (Keyframe)")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(4.0))
                                    .text_size(theme.typography.xs)
                                    .text_color(theme.colors.text_secondary)
                                    .child("↻ DB Sync")
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("⋮")
                            )
                    )
            )
            // Panel content
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(theme.spacing.md)
                    .overflow_hidden()
                    .child(self.render_keyframe_detail())
                    .child(self.render_track_settings())
            )
    }
}
