//! Extension Manager Toolbar Component
//!
//! Provides toolbar controls for the Extension Manager:
//! - Check Updates, Install, New buttons (left group)
//! - Installed, Marketplace, Develop tabs (center group)
//! - Status bar showing active extensions, updates, DB status, API status (right)

use gpui::{
    div, px, svg, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Current toolbar tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarTab {
    /// Show installed extensions
    Installed,
    /// Show marketplace
    Marketplace,
    /// Show development tools
    Develop,
}

/// The Extension Manager Toolbar component
pub struct ExtensionToolbar {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current active tab
    current_tab: ToolbarTab,
    /// Filter text
    filter_text: String,
}

impl ExtensionToolbar {
    /// Create a new Extension Toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            current_tab: ToolbarTab::Installed,
            filter_text: String::new(),
        }
    }

    /// Set the current tab
    pub fn set_tab(&mut self, tab: ToolbarTab) {
        self.current_tab = tab;
    }

    /// Get the current tab
    pub fn current_tab(&self) -> ToolbarTab {
        self.current_tab
    }

    /// Set filter text
    pub fn set_filter_text(&mut self, text: String) {
        self.filter_text = text;
    }

    /// Render a toolbar button with icon and label
    fn render_button(&self, icon: &str, label: &str, is_active: bool) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        let icon = icon.to_string();
        
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(theme.spacing.md)
            .py(px(5.0))
            .rounded(theme.borders.sm)
            .bg(if is_active { theme.colors.toolbar_active } else { theme.colors.surface })
            .hover(|this| this.bg(theme.colors.surface_hover))
            .cursor_pointer()
            .child(
                svg()
                    .path(format!("icons/{}.svg", icon))
                    .w(px(14.0))
                    .h(px(14.0))
                    .text_color(theme.colors.text)
            )
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.md)
                    .child(label)
            )
    }

    /// Render a separator between toolbar groups
    fn render_separator(&self) -> impl IntoElement {
        div()
            .w(px(1.0))
            .h(px(24.0))
            .bg(self.theme.colors.border)
    }

    /// Render the left button group (Check Updates, Install, New)
    fn render_left_group(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .bg(theme.colors.surface)
            .rounded(theme.borders.sm)
            .p(px(2.0))
            .child(self.render_button("refresh", "Check Updates", false))
            .child(self.render_button("download", "Install", false))
            .child(self.render_button("plus-circle", "New", false))
    }

    /// Render the center tab group (Installed, Marketplace, Develop)
    fn render_center_group(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .bg(theme.colors.surface)
            .rounded(theme.borders.sm)
            .p(px(2.0))
            .child(self.render_button("list", "Installed", self.current_tab == ToolbarTab::Installed))
            .child(self.render_button("store", "Marketplace", self.current_tab == ToolbarTab::Marketplace))
            .child(self.render_button("code", "Develop", self.current_tab == ToolbarTab::Develop))
    }

    /// Render the status bar (active extensions, updates, DB status, API status)
    fn render_status_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .items_center()
            .gap(theme.spacing.lg)
            .ml_auto()
            .bg(theme.colors.surface)
            .rounded(px(20.0))
            .px(theme.spacing.md)
            .py(px(4.0))
            .child(self.render_status_item("puzzle", "12 active"))
            .child(self.render_status_item("download", "4 updates"))
            .child(self.render_status_item("database", "DB: extensions"))
            .child(self.render_status_item("robot", "API ready"))
    }

    /// Render a single status bar item with icon and text
    fn render_status_item(&self, icon: &str, text: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let text = text.to_string();
        
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                svg()
                    .path(format!("icons/{}.svg", icon))
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(theme.colors.text_secondary)
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child(text)
            )
    }
}

impl Render for ExtensionToolbar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(44.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.lg)
            // Left: Check Updates, Install, New buttons
            .child(self.render_left_group())
            // Separator
            .child(self.render_separator())
            // Center: Installed, Marketplace, Develop tabs
            .child(self.render_center_group())
            // Right: Status bar
            .child(self.render_status_bar())
    }
}
