//! Extension Manager Box Component
//!
//! The main container for the Extension Manager interface with:
//! - Menu bar at the top (File, Edit, Extensions, Marketplace, Development, Help)
//! - Toolbar with view controls and search
//! - Main area: Installed List (left) + Extension Details (center) + Marketplace/Dev (right)
//! - Bottom tab panel: Extension Console, API Docs, Change Log

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, View, ViewContext, VisualContext as _,
};
use std::sync::Arc;

use crate::ui::theme::Theme;
use crate::ui::layouts::{WorkspaceLayout, MenuBar};
use crate::services::engine_bridge::EngineHandle;

use super::{
    ExtensionToolbar, ToolbarTab,
    InstalledPanel, ExtensionItem,
    DetailPanel,
    MarketplacePanel,
    ExtensionBottomTabs,
};

/// The Extension Manager Box component
///
/// Layout structure matching HTML prototype:
/// ```
/// ┌─────────────────────────────────────────────────────────────┐
/// │ Menu Bar (File, Edit, Extensions, Marketplace, Dev, Help)   │
/// ├─────────────────────────────────────────────────────────────┤
/// │ Toolbar (Check Updates, Install, New, Installed, etc.)     │
/// ├─────────────────────┬───────────────────┬───────────────────┤
/// │                     │                   │                   │
/// │ Installed List      │ Extension Details │ Marketplace/Dev   │
/// │ (280px fixed)       │ (flexible)        │ (300px fixed)     │
/// │                     │                   │                   │
/// ├─────────────────────┴───────────────────┴───────────────────┤
/// │ Bottom Tab Panel (Console / API Docs / Change Log)          │
/// └─────────────────────────────────────────────────────────────┘
/// ```
pub struct ExtensionBox {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Engine handle
    _engine_handle: Arc<EngineHandle>,
    /// Toolbar component
    toolbar: View<ExtensionToolbar>,
    /// Installed extensions panel
    installed_panel: View<InstalledPanel>,
    /// Extension details panel
    detail_panel: View<DetailPanel>,
    /// Marketplace and development panel
    marketplace_panel: View<MarketplacePanel>,
    /// Bottom tab panel component
    bottom_tabs: View<ExtensionBottomTabs>,
    /// Currently selected extension
    selected_extension: Option<ExtensionItem>,
    /// Current toolbar tab
    current_tab: ToolbarTab,
    /// Filter text for extensions
    filter_text: String,
}

impl ExtensionBox {
    /// Create a new Extension Manager Box
    pub fn new(engine_handle: Arc<EngineHandle>, theme: Arc<Theme>, cx: &mut ViewContext<Self>) -> Self {
        let toolbar = cx.new_view(|_cx| ExtensionToolbar::new(theme.clone()));
        let installed_panel = cx.new_view(|_cx| InstalledPanel::with_sample_data(theme.clone()));
        let detail_panel = cx.new_view(|_cx| DetailPanel::new(theme.clone()));
        let marketplace_panel = cx.new_view(|_cx| MarketplacePanel::with_sample_data(theme.clone()));
        let bottom_tabs = cx.new_view(|_cx| ExtensionBottomTabs::new(theme.clone()));

        // Select the first extension by default
        let selected_extension = Some(ExtensionItem::new(
            "shader-editor",
            "Shader Editor",
            "v1.0.0",
            "username",
            "palette"
        ).with_description("Node-based shader editor"));

        Self {
            theme,
            _engine_handle: engine_handle,
            toolbar,
            installed_panel,
            detail_panel,
            marketplace_panel,
            bottom_tabs,
            selected_extension,
            current_tab: ToolbarTab::Installed,
            filter_text: String::new(),
        }
    }

    /// Create with sample data for testing
    pub fn with_sample_data(self, cx: &mut ViewContext<Self>) -> Self {
        // Set default selected extension
        if let Some(ref ext) = self.selected_extension {
            self.detail_panel.update(cx, |panel, _cx| {
                panel.set_extension(ext.clone());
            });
            self.installed_panel.update(cx, |panel, _cx| {
                panel.set_selected(Some(ext.id().to_string()));
            });
        }
        self
    }

    /// Render the menu bar (kept for potential future use)
    #[allow(dead_code)]
    fn render_menu_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let menu_items = vec!["File", "Edit", "Extensions", "Marketplace", "Development", "Help"];
        
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
            // Left: Installed Extensions List (280px)
            .child(
                div()
                    .w(px(280.0))
                    .h_full()
                    .child(self.installed_panel.clone())
            )
            // Center: Extension Details (flexible)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.detail_panel.clone())
            )
            // Right: Marketplace & Development (300px)
            .child(
                div()
                    .w(px(300.0))
                    .h_full()
                    .child(self.marketplace_panel.clone())
            )
    }

    /// Set the selected extension
    pub fn select_extension(&mut self, extension: Option<ExtensionItem>, cx: &mut ViewContext<Self>) {
        self.selected_extension = extension.clone();
        
        // Update detail panel
        self.detail_panel.update(cx, |panel, _cx| {
            if let Some(ref ext) = extension {
                panel.set_extension(ext.clone());
            } else {
                panel.clear_extension();
            }
        });
        
        // Update installed panel selection
        self.installed_panel.update(cx, |panel, _cx| {
            panel.set_selected(extension.as_ref().map(|e| e.id().to_string()));
        });
        
        cx.notify();
    }

    /// Get the currently selected extension
    pub fn selected_extension(&self) -> Option<&ExtensionItem> {
        self.selected_extension.as_ref()
    }

    /// Set the current toolbar tab
    pub fn set_tab(&mut self, tab: ToolbarTab, cx: &mut ViewContext<Self>) {
        self.current_tab = tab;
        self.toolbar.update(cx, |toolbar, _cx| {
            toolbar.set_tab(tab);
        });
        cx.notify();
    }

    /// Set filter text for extensions
    pub fn set_filter_text(&mut self, text: String, cx: &mut ViewContext<Self>) {
        self.filter_text = text.clone();
        self.installed_panel.update(cx, |panel, _cx| {
            panel.set_filter(text);
        });
        cx.notify();
    }
}

impl Render for ExtensionBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        // Use the unified WorkspaceLayout for consistent layout structure
        // Note: ExtensionBox uses custom panel widths (280px left, 300px right)
        WorkspaceLayout::new(theme.clone())
            .menu_bar(
                MenuBar::new(theme.clone())
                    .items(vec!["File", "Edit", "Extensions", "Marketplace", "Development", "Help"])
            )
            .toolbar(self.toolbar.clone())
            .left_panel(
                div()
                    .w(px(280.0))
                    .h_full()
                    .child(self.installed_panel.clone())
            )
            .center_panel(self.detail_panel.clone())
            .right_panel(
                div()
                    .w(px(300.0))
                    .h_full()
                    .child(self.marketplace_panel.clone())
            )
            .bottom_panel(self.bottom_tabs.clone())
    }
}
