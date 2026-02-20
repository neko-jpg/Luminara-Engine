//! Asset Vault Toolbar Component
//!
//! Toolbar with:
//! - View mode toggle (Grid/List)
//! - Sort options (Name, Size, Date)
//! - Filter/search input
//! - Status bar (asset count, total size)

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext, prelude::FluentBuilder,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// View mode for asset display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Grid view with thumbnails
    Grid,
    /// List view with details
    List,
}

/// Sort mode for assets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Sort by name
    Name,
    /// Sort by size
    Size,
    /// Sort by date
    Date,
}

/// Status information for the toolbar
#[derive(Debug, Clone)]
pub struct AssetStatus {
    /// Number of assets
    pub asset_count: u32,
    /// Total size in MB
    pub total_size_mb: f32,
}

impl AssetStatus {
    /// Create new status
    pub fn new(count: u32, size_mb: f32) -> Self {
        Self {
            asset_count: count,
            total_size_mb: size_mb,
        }
    }

    /// Format size for display
    pub fn formatted_size(&self) -> String {
        if self.total_size_mb >= 1024.0 {
            format!("{:.1} GB", self.total_size_mb / 1024.0)
        } else {
            format!("{:.1} MB", self.total_size_mb)
        }
    }
}

/// Toolbar component for Asset Vault
pub struct AssetVaultToolbar {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current view mode
    view_mode: ViewMode,
    /// Current sort mode
    sort_mode: SortMode,
    /// Filter text
    filter_text: String,
    /// Asset status
    status: AssetStatus,
}

impl AssetVaultToolbar {
    /// Create a new toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            view_mode: ViewMode::Grid,
            sort_mode: SortMode::Name,
            filter_text: String::new(),
            status: AssetStatus::new(124, 12.4),
        }
    }

    /// Create with sample data
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            view_mode: ViewMode::Grid,
            sort_mode: SortMode::Name,
            filter_text: "hero".to_string(),
            status: AssetStatus::new(124, 12.4),
        }
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Get current view mode
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set sort mode
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }

    /// Get current sort mode
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Set filter text
    pub fn set_filter_text(&mut self, text: String) {
        self.filter_text = text;
    }

    /// Get filter text
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Set status
    pub fn set_status(&mut self, status: AssetStatus) {
        self.status = status;
    }

    /// Render view mode buttons
    fn render_view_buttons(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let view_mode = self.view_mode;

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .bg(theme.colors.surface_active)
            .rounded(px(6.0))
            .px(px(4.0))
            // Grid button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .when(view_mode == ViewMode::Grid, |this: gpui::Div| {
                        this.bg(theme.colors.toolbar_active)
                    })
                    .text_color(if view_mode == ViewMode::Grid {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .child("⊞")
                    .child("Grid")
            )
            // List button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .when(view_mode == ViewMode::List, |this: gpui::Div| {
                        this.bg(theme.colors.toolbar_active)
                    })
                    .text_color(if view_mode == ViewMode::List {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .child("☰")
                    .child("List")
            )
    }

    /// Render sort buttons
    fn render_sort_buttons(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let sort_mode = self.sort_mode;

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .bg(theme.colors.surface_active)
            .rounded(px(6.0))
            .px(px(4.0))
            // Name button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .when(sort_mode == SortMode::Name, |this: gpui::Div| {
                        this.bg(theme.colors.toolbar_active)
                    })
                    .text_color(if sort_mode == SortMode::Name {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .child("A-Z")
                    .child("Name")
            )
            // Size button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .when(sort_mode == SortMode::Size, |this: gpui::Div| {
                        this.bg(theme.colors.toolbar_active)
                    })
                    .text_color(if sort_mode == SortMode::Size {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .child("⇵")
                    .child("Size")
            )
            // Date button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .when(sort_mode == SortMode::Date, |this: gpui::Div| {
                        this.bg(theme.colors.toolbar_active)
                    })
                    .text_color(if sort_mode == SortMode::Date {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .child("📅")
                    .child("Date")
            )
    }

    /// Render filter input
    fn render_filter_input(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let filter_text = self.filter_text.clone();

        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .flex_1()
            .max_w(px(300.0))
            .bg(theme.colors.surface_active)
            .rounded(px(6.0))
            .px(px(8.0))
            .py(px(4.0))
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .child("🔍")
            )
            .child(
                div()
                    .flex_1()
                    .child(filter_text)
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.md)
            )
    }

    /// Render status bar
    fn render_status_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let status = self.status.clone();

        div()
            .flex()
            .items_center()
            .gap(px(16.0))
            .bg(theme.colors.surface_active)
            .rounded(px(20.0))
            .px(px(12.0))
            .py(px(4.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("📦")
                    .child(format!("{} assets", status.asset_count))
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("💾")
                    .child(status.formatted_size())
            )
    }
}

impl Render for AssetVaultToolbar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(44.0))
            .px(px(12.0))
            .gap(px(12.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            // View mode buttons
            .child(self.render_view_buttons())
            // Sort buttons
            .child(self.render_sort_buttons())
            // Filter input
            .child(self.render_filter_input())
            // Spacer
            .child(div().flex_1())
            // Status bar
            .child(self.render_status_bar())
    }
}
