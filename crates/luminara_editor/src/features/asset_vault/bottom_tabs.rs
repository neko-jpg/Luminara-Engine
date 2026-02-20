//! Asset Vault Bottom Tabs Component
//!
//! Bottom tab panel with three tabs:
//! - Import Log: Shows import status and results
//! - Missing Assets: Lists missing asset references
//! - Duplicates: Lists duplicate assets

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext, prelude::FluentBuilder,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Tab kinds for the bottom panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    /// Import log tab
    ImportLog,
    /// Missing assets tab
    MissingAssets,
    /// Duplicates tab
    Duplicates,
}

impl TabKind {
    /// Get display name for tab
    pub fn display_name(&self) -> &'static str {
        match self {
            TabKind::ImportLog => "Import Log",
            TabKind::MissingAssets => "Missing Assets",
            TabKind::Duplicates => "Duplicates",
        }
    }

    /// Get icon for tab
    pub fn icon(&self) -> &'static str {
        match self {
            TabKind::ImportLog => "📋",
            TabKind::MissingAssets => "⚠",
            TabKind::Duplicates => "📄",
        }
    }
}

/// Log entry with status
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Entry message
    pub message: String,
    /// Entry status
    pub status: LogStatus,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(message: &str, status: LogStatus) -> Self {
        Self {
            message: message.to_string(),
            status,
        }
    }
}

/// Log entry status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStatus {
    /// Success
    Success,
    /// Warning
    Warning,
    /// Error
    Error,
}

impl LogStatus {
    /// Get status icon
    pub fn icon(&self) -> &'static str {
        match self {
            LogStatus::Success => "✅",
            LogStatus::Warning => "⚠️",
            LogStatus::Error => "❌",
        }
    }

    /// Get status color
    pub fn color(&self, theme: &Theme) -> gpui::Hsla {
        match self {
            LogStatus::Success => theme.colors.success,
            LogStatus::Warning => theme.colors.warning,
            LogStatus::Error => theme.colors.error,
        }
    }
}

/// Missing asset entry
#[derive(Debug, Clone)]
pub struct MissingAsset {
    /// Asset name
    pub name: String,
    /// Referenced by
    pub referenced_by: String,
}

impl MissingAsset {
    /// Create a new missing asset entry
    pub fn new(name: &str, referenced_by: &str) -> Self {
        Self {
            name: name.to_string(),
            referenced_by: referenced_by.to_string(),
        }
    }
}

/// Duplicate asset entry
#[derive(Debug, Clone)]
pub struct DuplicateAsset {
    /// Asset name
    pub name: String,
    /// Number of copies
    pub count: u32,
    /// Total size in MB
    pub size_mb: f32,
}

impl DuplicateAsset {
    /// Create a new duplicate asset entry
    pub fn new(name: &str, count: u32, size_mb: f32) -> Self {
        Self {
            name: name.to_string(),
            count,
            size_mb,
        }
    }
}

/// Bottom Tabs component for Asset Vault
pub struct AssetVaultBottomTabs {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current active tab
    active_tab: TabKind,
    /// Import log entries
    import_log: Vec<LogEntry>,
    /// Missing assets
    missing_assets: Vec<MissingAsset>,
    /// Duplicate assets
    duplicates: Vec<DuplicateAsset>,
}

impl AssetVaultBottomTabs {
    /// Create a new bottom tabs panel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tab: TabKind::ImportLog,
            import_log: Vec::new(),
            missing_assets: Vec::new(),
            duplicates: Vec::new(),
        }
    }

    /// Create with sample data matching HTML prototype
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        let import_log = vec![
            LogEntry::new("Imported grass.png (1024x1024, compressed to WebP)", LogStatus::Success),
            LogEntry::new("Imported hero.glb (12k vertices, collider generated)", LogStatus::Success),
            LogEntry::new("bgm.ogg: minor clipping detected", LogStatus::Warning),
            LogEntry::new("enemy.glb: missing texture 'metal.png'", LogStatus::Error),
        ];

        let missing_assets = vec![
            MissingAsset::new("metal.png", "enemy.glb"),
            MissingAsset::new("old_font.ttf", "UI"),
        ];

        let duplicates = vec![
            DuplicateAsset::new("grass", 2, 2.1),
            DuplicateAsset::new("stone.png", 3, 3.4),
        ];

        Self {
            theme,
            active_tab: TabKind::ImportLog,
            import_log,
            missing_assets,
            duplicates,
        }
    }

    /// Set active tab
    pub fn set_active_tab(&mut self, tab: TabKind, cx: &mut ViewContext<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    /// Get active tab
    pub fn active_tab(&self) -> TabKind {
        self.active_tab
    }

    /// Add log entry
    pub fn add_log_entry(&mut self, entry: LogEntry, cx: &mut ViewContext<Self>) {
        self.import_log.push(entry);
        cx.notify();
    }

    /// Add missing asset
    pub fn add_missing_asset(&mut self, asset: MissingAsset, cx: &mut ViewContext<Self>) {
        self.missing_assets.push(asset);
        cx.notify();
    }

    /// Add duplicate
    pub fn add_duplicate(&mut self, duplicate: DuplicateAsset, cx: &mut ViewContext<Self>) {
        self.duplicates.push(duplicate);
        cx.notify();
    }

    /// Render a tab button
    fn render_tab_button(&self, tab: TabKind) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_active = self.active_tab == tab;

        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(16.0))
            .py(px(8.0))
            .text_size(theme.typography.sm)
            .border_b_2()
            .border_color(if is_active {
                theme.colors.accent
            } else {
                gpui::transparent_black()
            })
            .bg(if is_active {
                theme.colors.surface_active
            } else {
                gpui::transparent_black()
            })
            .text_color(if is_active {
                theme.colors.accent
            } else {
                theme.colors.text_secondary
            })
            .hover(|this| this.bg(theme.colors.surface_hover))
            .cursor_pointer()
            .child(tab.icon().to_string())
            .child(tab.display_name().to_string())
    }

    /// Render tab header
    fn render_tab_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(36.0))
            .px(px(8.0))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(self.render_tab_button(TabKind::ImportLog))
            .child(self.render_tab_button(TabKind::MissingAssets))
            .child(self.render_tab_button(TabKind::Duplicates))
    }

    /// Render import log content
    fn render_import_log(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let entries = self.import_log.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .overflow_hidden()
            .children(
                entries.into_iter().map(|entry| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .py(px(2.0))
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .font_family("monospace")
                        .text_size(px(11.0))
                        .child(entry.status.icon().to_string())
                        .child(
                            div()
                                .text_color(entry.status.color(&theme))
                                .child(entry.message)
                        )
                })
            )
    }

    /// Render missing assets content
    fn render_missing_assets(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let assets = self.missing_assets.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .overflow_hidden()
            .children(
                assets.into_iter().map(|asset| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .p(px(8.0))
                        .mb(px(4.0))
                        .bg(theme.colors.surface_active)
                        .rounded(px(4.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child("❓")
                                .child(
                                    div()
                                        .text_size(theme.typography.sm)
                                        .text_color(theme.colors.text)
                                        .child(format!("{} (referenced by {})", asset.name, asset.referenced_by))
                                )
                        )
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(theme.colors.accent)
                                .rounded(px(12.0))
                                .text_size(px(10.0))
                                .text_color(theme.colors.text)
                                .hover(|this| this.opacity(0.8))
                                .cursor_pointer()
                                .child("Locate")
                        )
                })
            )
    }

    /// Render duplicates content
    fn render_duplicates(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let duplicates = self.duplicates.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .overflow_hidden()
            .children(
                duplicates.into_iter().map(|dup| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .p(px(8.0))
                        .mb(px(4.0))
                        .bg(theme.colors.surface_active)
                        .rounded(px(4.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child("📄")
                                .child(
                                    div()
                                        .text_size(theme.typography.sm)
                                        .text_color(theme.colors.text)
                                        .child(format!("{} ({} copies, {:.1} MB)", dup.name, dup.count, dup.size_mb))
                                )
                        )
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(theme.colors.accent)
                                .rounded(px(12.0))
                                .text_size(px(10.0))
                                .text_color(theme.colors.text)
                                .hover(|this| this.opacity(0.8))
                                .cursor_pointer()
                                .child("Merge")
                        )
                })
            )
    }

    /// Render active tab content based on current tab
    fn render_active_content(&self) -> impl IntoElement + use<'_> {
        // Use when() pattern to conditionally render different content
        div()
            .flex()
            .flex_col()
            .size_full()
            .when(self.active_tab == TabKind::ImportLog, |this| {
                this.child(self.render_import_log())
            })
            .when(self.active_tab == TabKind::MissingAssets, |this| {
                this.child(self.render_missing_assets())
            })
            .when(self.active_tab == TabKind::Duplicates, |this| {
                this.child(self.render_duplicates())
            })
    }
}

impl Render for AssetVaultBottomTabs {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            // Tab header
            .child(self.render_tab_header())
            // Tab content
            .child(self.render_active_content())
    }
}
