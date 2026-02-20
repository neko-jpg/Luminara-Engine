//! Asset Vault Box Component
//!
//! The main container for the Asset Vault interface with:
//! - Menu bar at the top (File, Edit, Assets, View, Help)
//! - Toolbar with view controls and search
//! - Main area: Directory Tree (left) + Asset Grid & Preview (center) + Inspector (right)
//! - Bottom tab panel: Import Log, Missing Assets, Duplicates

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, View, ViewContext, VisualContext as _,
};
use std::sync::Arc;

use crate::ui::theme::Theme;
use crate::ui::layouts::{WorkspaceLayout, MenuBar};
use crate::services::engine_bridge::EngineHandle;

use super::{
    AssetVaultToolbar, ViewMode, SortMode,
    DirectoryTree,
    AssetGrid, AssetItem, AssetType,
    AssetInspector, AssetMetadata,
    AssetVaultBottomTabs,
};

/// The Asset Vault Box component
///
/// Layout structure matching HTML prototype:
/// ```
/// ┌─────────────────────────────────────────────────────────────┐
/// │ Menu Bar (File, Edit, Assets, View, Help)                   │
/// ├─────────────────────────────────────────────────────────────┤
/// │ Toolbar (Grid/List, Sort, Filter, Search, Status)          │
/// ├─────────────────────┬───────────────────┬───────────────────┤
/// │                     │                   │                   │
/// │ Directory Tree      │ Asset Grid +      │ Asset Inspector   │
/// │ (260px fixed)       │ Preview           │ (320px fixed)     │
/// │                     │ (flexible)        │                   │
/// ├─────────────────────┴───────────────────┴───────────────────┤
/// │ Bottom Tab Panel (Import Log / Missing / Duplicates)        │
/// └─────────────────────────────────────────────────────────────┘
/// ```
pub struct AssetVaultBox {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Engine handle
    _engine_handle: Arc<EngineHandle>,
    /// Toolbar component
    toolbar: View<AssetVaultToolbar>,
    /// Directory tree component
    directory_tree: View<DirectoryTree>,
    /// Asset grid component
    asset_grid: View<AssetGrid>,
    /// Asset inspector component
    inspector: View<AssetInspector>,
    /// Bottom tab panel component
    bottom_tabs: View<AssetVaultBottomTabs>,
    /// Currently selected asset
    selected_asset: Option<AssetItem>,
    /// Current view mode
    view_mode: ViewMode,
    /// Current sort mode
    sort_mode: SortMode,
    /// Filter text
    filter_text: String,
}

impl AssetVaultBox {
    /// Create a new Asset Vault Box
    pub fn new(engine_handle: Arc<EngineHandle>, theme: Arc<Theme>, cx: &mut ViewContext<Self>) -> Self {
        let toolbar = cx.new_view(|_cx| AssetVaultToolbar::new(theme.clone()));
        let directory_tree = cx.new_view(|_cx| DirectoryTree::with_sample_data(theme.clone()));
        let asset_grid = cx.new_view(|_cx| AssetGrid::with_sample_data(theme.clone()));
        let inspector = cx.new_view(|_cx| AssetInspector::new(theme.clone()));
        let bottom_tabs = cx.new_view(|_cx| AssetVaultBottomTabs::new(theme.clone()));

        Self {
            theme,
            _engine_handle: engine_handle,
            toolbar,
            directory_tree,
            asset_grid,
            inspector,
            bottom_tabs,
            selected_asset: None,
            view_mode: ViewMode::Grid,
            sort_mode: SortMode::Name,
            filter_text: String::new(),
        }
    }

    /// Create with sample data for testing
    pub fn with_sample_data(mut self, cx: &mut ViewContext<Self>) -> Self {
        self.selected_asset = Some(AssetItem::new("hero.glb", AssetType::Model)
            .with_size("2.4 MB")
            .with_path("assets/models/hero.glb"));
        
        // Update inspector with sample metadata
        self.inspector.update(cx, |inspector, _cx| {
            inspector.set_metadata(AssetMetadata::sample_hero_glb());
        });
        
        self
    }

    /// Render the menu bar (kept for potential future use)
    #[allow(dead_code)]
    fn render_menu_bar(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let menu_items = vec!["File", "Edit", "Assets", "View", "Help"];
        
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
            // Left: Directory Tree (260px)
            .child(
                div()
                    .w(px(260.0))
                    .h_full()
                    .child(self.directory_tree.clone())
            )
            // Center: Asset Grid + Preview (flexible)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.asset_grid.clone())
            )
            // Right: Inspector (320px)
            .child(
                div()
                    .w(px(320.0))
                    .h_full()
                    .child(self.inspector.clone())
            )
    }

    /// Set the selected asset
    pub fn select_asset(&mut self, asset: Option<AssetItem>, cx: &mut ViewContext<Self>) {
        self.selected_asset = asset.clone();
        
        // Update inspector
        self.inspector.update(cx, |inspector, _cx| {
            if let Some(ref asset) = asset {
                inspector.set_metadata(AssetMetadata::from_asset(asset));
            } else {
                inspector.clear_metadata();
            }
        });
        
        // Update asset grid selection
        self.asset_grid.update(cx, |grid, _cx| {
            grid.set_selected(asset.as_ref().map(|a| a.name().to_string()));
        });
        
        cx.notify();
    }

    /// Get the currently selected asset
    pub fn selected_asset(&self) -> Option<&AssetItem> {
        self.selected_asset.as_ref()
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode, cx: &mut ViewContext<Self>) {
        self.view_mode = mode;
        self.toolbar.update(cx, |toolbar, _cx| {
            toolbar.set_view_mode(mode);
        });
        cx.notify();
    }

    /// Set sort mode
    pub fn set_sort_mode(&mut self, mode: SortMode, cx: &mut ViewContext<Self>) {
        self.sort_mode = mode;
        self.toolbar.update(cx, |toolbar, _cx| {
            toolbar.set_sort_mode(mode);
        });
        cx.notify();
    }

    /// Set filter text
    pub fn set_filter_text(&mut self, text: String, cx: &mut ViewContext<Self>) {
        self.filter_text = text.clone();
        self.toolbar.update(cx, |toolbar, _cx| {
            toolbar.set_filter_text(text);
        });
        cx.notify();
    }
}

impl Render for AssetVaultBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        // Use the unified WorkspaceLayout for consistent layout structure
        WorkspaceLayout::new(theme.clone())
            .menu_bar(
                MenuBar::new(theme.clone())
                    .items(vec!["File", "Edit", "Assets", "View", "Help"])
            )
            .toolbar(self.toolbar.clone())
            .left_panel(self.directory_tree.clone())
            .center_panel(self.asset_grid.clone())
            .right_panel(self.inspector.clone())
            .bottom_panel(self.bottom_tabs.clone())
    }
}
