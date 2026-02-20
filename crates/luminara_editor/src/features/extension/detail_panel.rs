//! Extension Detail Panel Component
//!
//! Displays detailed information about the selected extension:
//! - Mini tabs: Details, Settings, Logic Nodes, Uninstall
//! - Manifest information table
//! - Action buttons (Export Manifest, Reload)

use gpui::{
    div, px, svg, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;
use super::ExtensionItem;

/// Current detail tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    /// Extension details/manifest
    Details,
    /// Extension settings
    Settings,
    /// Logic nodes provided
    LogicNodes,
    /// Uninstall confirmation
    Uninstall,
}

/// The Extension Detail Panel component
pub struct DetailPanel {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Currently displayed extension
    extension: Option<ExtensionItem>,
    /// Current active tab
    current_tab: DetailTab,
}

impl DetailPanel {
    /// Create a new Extension Detail Panel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            extension: None,
            current_tab: DetailTab::Details,
        }
    }

    /// Set the extension to display
    pub fn set_extension(&mut self, extension: ExtensionItem) {
        self.extension = Some(extension);
    }

    /// Clear the displayed extension
    pub fn clear_extension(&mut self) {
        self.extension = None;
    }

    /// Set the current tab
    pub fn set_tab(&mut self, tab: DetailTab) {
        self.current_tab = tab;
    }

    /// Get the current tab
    pub fn current_tab(&self) -> DetailTab {
        self.current_tab
    }

    /// Render the panel header
    fn render_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let title = self.extension.as_ref()
            .map(|e| format!("{} — Details", e.name()))
            .unwrap_or_else(|| "Extension Details".to_string());
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(36.0))
            .bg(theme.colors.panel_header)
            .px(theme.spacing.md)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        svg()
                            .path("icons/palette.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.md)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(title)
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.md)
                    .child(
                        svg()
                            .path("icons/settings.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        svg()
                            .path("icons/dots-vertical.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
            )
    }

    /// Render the mini tab bar
    fn render_mini_tabs(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let tabs = vec![
            (DetailTab::Details, "Details"),
            (DetailTab::Settings, "Settings"),
            (DetailTab::LogicNodes, "Logic Nodes"),
        ];
        let theme_for_uninstall = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .border_b_1()
            .border_color(theme.colors.border)
            .mb(theme.spacing.md)
            .gap(px(2.0))
            .children(
                tabs.into_iter().map(move |(tab, label)| {
                    let is_active = self.current_tab == tab;
                    let label = label.to_string();
                    
                    div()
                        .px(theme.spacing.md)
                        .py(px(4.0))
                        .rounded_t(px(16.0))
                        .bg(if is_active { theme.colors.toolbar_active } else { theme.colors.panel_header })
                        .text_color(if is_active { theme.colors.text } else { theme.colors.text_secondary })
                        .text_size(theme.typography.sm)
                        .cursor_pointer()
                        .hover(|this| {
                            if !is_active {
                                this.bg(theme.colors.surface_hover)
                            } else {
                                this
                            }
                        })
                        .child(label)
                })
            )
            // Uninstall tab (special styling)
            .child(
                div()
                    .px(theme_for_uninstall.spacing.md)
                    .py(px(4.0))
                    .rounded_t(px(16.0))
                    .bg(if self.current_tab == DetailTab::Uninstall { theme_for_uninstall.colors.error } else { theme_for_uninstall.colors.panel_header })
                    .text_color(if self.current_tab == DetailTab::Uninstall { theme_for_uninstall.colors.text } else { theme_for_uninstall.colors.error })
                    .text_size(theme_for_uninstall.typography.sm)
                    .cursor_pointer()
                    .hover(|this| this.opacity(0.8))
                    .child("Uninstall")
            )
    }

    /// Render the manifest table row
    fn render_manifest_row(&self, key: &str, value: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let key = key.to_string();
        let value = value.to_string();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .py(px(6.0))
            .px(px(4.0))
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .w(px(100.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child(key)
            )
            .child(
                div()
                    .flex_1()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.sm)
                    .child(value)
            )
    }

    /// Render the Details tab content
    fn render_details_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        if let Some(ref ext) = self.extension {
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap(px(4.0))
                // Manifest table
                .child(self.render_manifest_row("Name", ext.id()))
                .child(self.render_manifest_row("Version", ext.version()))
                .child(self.render_manifest_row("Author", ext.author()))
                .child(self.render_manifest_row("Description", ext.description()))
                .child(self.render_manifest_row("Icon", &format!("{}", ext.icon())))
                .child(self.render_manifest_row("Min Luminara", "0.1.0"))
                .child(self.render_manifest_row("Contributes", "boxes: shader-editor\nwidgets: ShaderGraphCanvas, ShaderPreview\ncomponents: CustomShader\nasset_importers: .shadergraph\ncommands: shader.compile\nlogic_nodes: ShaderSwitch"))
                .child(self.render_manifest_row("Dependencies", "luminara-core >=0.1.0"))
                // Action buttons
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(theme.spacing.md)
                        .mt(theme.spacing.lg)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .px(theme.spacing.md)
                                .py(px(6.0))
                                .rounded(theme.borders.sm)
                                .bg(rgb_to_hsla(0x4a6a9a))
                                .cursor_pointer()
                                .hover(|this| this.opacity(0.8))
                                .child(
                                    svg()
                                        .path("icons/file-export.svg")
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .text_color(theme.colors.text)
                                )
                                .child(
                                    div()
                                        .text_color(theme.colors.text)
                                        .text_size(theme.typography.md)
                                        .child("Export Manifest")
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .px(theme.spacing.md)
                                .py(px(6.0))
                                .rounded(theme.borders.sm)
                                .bg(theme.colors.toolbar_active)
                                .cursor_pointer()
                                .hover(|this| this.opacity(0.8))
                                .child(
                                    svg()
                                        .path("icons/refresh.svg")
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .text_color(theme.colors.text)
                                )
                                .child(
                                    div()
                                        .text_color(theme.colors.text)
                                        .text_size(theme.typography.md)
                                        .child("Reload")
                                )
                        )
                )
        } else {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .flex_1()
                .text_color(theme.colors.text_secondary)
                .text_size(theme.typography.md)
                .child("Select an extension to view details")
        }
    }

    /// Render the Settings tab content
    fn render_settings_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .text_color(theme.colors.text_secondary)
            .text_size(theme.typography.md)
            .child(format!(
                "{} settings will appear here.",
                self.extension.as_ref().map(|e| e.name()).unwrap_or("Extension")
            ))
    }

    /// Render the Logic Nodes tab content
    fn render_logic_nodes_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .gap(theme.spacing.md)
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(theme.typography.md)
                    .child("Custom logic nodes provided by this extension:")
            )
            .child(
                div()
                    .ml(theme.spacing.lg)
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("• ShaderSwitch (Logic Graph)")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("• TextureSample")
                    )
            )
    }

    /// Render the Uninstall tab content
    fn render_uninstall_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .gap(theme.spacing.md)
            .child(
                div()
                    .text_color(theme.colors.error)
                    .text_size(theme.typography.md)
                    .child("Are you sure? This action cannot be undone.")
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px(theme.spacing.md)
                    .py(px(8.0))
                    .rounded(theme.borders.sm)
                    .bg(rgb_to_hsla(0x8a3a3a))
                    .cursor_pointer()
                    .hover(|this| this.opacity(0.8))
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.md)
                            .child("Uninstall Extension")
                    )
            )
    }

    /// Render the current tab content
    fn render_tab_content(&self) -> impl IntoElement {
        match self.current_tab {
            DetailTab::Details => self.render_details_tab().into_any_element(),
            DetailTab::Settings => self.render_settings_tab().into_any_element(),
            DetailTab::LogicNodes => self.render_logic_nodes_tab().into_any_element(),
            DetailTab::Uninstall => self.render_uninstall_tab().into_any_element(),
        }
    }
}

impl Render for DetailPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_t(px(4.0))
            // Header
            .child(self.render_header())
            // Content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p(theme.spacing.md)
                    .overflow_hidden()
                    // Mini tabs
                    .child(self.render_mini_tabs())
                    // Tab content
                    .child(self.render_tab_content())
            )
    }
}

/// Helper to convert RGB to Hsla
fn rgb_to_hsla(rgb: u32) -> gpui::Hsla {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    
    let s = if max == min {
        0.0
    } else {
        let d = max - min;
        if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) }
    };
    
    let h = if max == min {
        0.0
    } else if max == r {
        ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    
    gpui::Hsla { h, s, l, a: 1.0 }
}
