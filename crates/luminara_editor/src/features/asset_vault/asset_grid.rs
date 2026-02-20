//! Asset Grid Component
//!
//! A grid view for displaying assets with:
//! - Thumbnail icons
//! - Asset names
//! - Selection highlighting
//! - Support for both Grid and List view modes

use gpui::{
    div, px, uniform_list, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
    prelude::*, UniformListScrollHandle, SharedString,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Type of asset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// 3D Model
    Model,
    /// Texture/Image
    Texture,
    /// Audio file
    Audio,
    /// Script/Code file
    Script,
    /// Prefab
    Prefab,
    /// Font
    Font,
    /// Unknown type
    Unknown,
}

impl AssetType {
    /// Get icon for asset type
    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Model => "🧊",
            AssetType::Texture => "🖼",
            AssetType::Audio => "🎵",
            AssetType::Script => "📝",
            AssetType::Prefab => "🎲",
            AssetType::Font => "🔤",
            AssetType::Unknown => "📄",
        }
    }

    /// Get display name for asset type
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetType::Model => "3D Model",
            AssetType::Texture => "Texture",
            AssetType::Audio => "Audio",
            AssetType::Script => "Script",
            AssetType::Prefab => "Prefab",
            AssetType::Font => "Font",
            AssetType::Unknown => "Unknown",
        }
    }
}

/// A single asset item
#[derive(Debug, Clone)]
pub struct AssetItem {
    /// Asset name
    name: String,
    /// Asset type
    asset_type: AssetType,
    /// File size
    size: Option<String>,
    /// File path
    path: Option<String>,
    /// Whether selected
    is_selected: bool,
}

impl AssetItem {
    /// Create a new asset item
    pub fn new(name: &str, asset_type: AssetType) -> Self {
        Self {
            name: name.to_string(),
            asset_type,
            size: None,
            path: None,
            is_selected: false,
        }
    }

    /// Set file size
    pub fn with_size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Set file path
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Set selected state
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get asset type
    pub fn asset_type(&self) -> AssetType {
        self.asset_type
    }

    /// Get size
    pub fn size(&self) -> Option<&str> {
        self.size.as_deref()
    }

    /// Get path
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Check if selected
    pub fn is_selected(&self) -> bool {
        self.is_selected
    }
}

/// Asset Grid component
pub struct AssetGrid {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Asset items
    items: Vec<AssetItem>,
    /// Currently selected item name
    selected_name: Option<String>,
}

impl AssetGrid {
    /// Create a new empty asset grid
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            items: Vec::new(),
            selected_name: None,
        }
    }

    /// Create with sample data matching HTML prototype
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        let items = vec![
            AssetItem::new("hero.glb", AssetType::Model)
                .with_selected(true),
            AssetItem::new("enemy.glb", AssetType::Model),
            AssetItem::new("grass.png", AssetType::Texture),
            AssetItem::new("stone.png", AssetType::Texture),
            AssetItem::new("bgm.ogg", AssetType::Audio),
            AssetItem::new("sfx_jump.wav", AssetType::Audio),
            AssetItem::new("player.rs", AssetType::Script),
            AssetItem::new("player_prefab", AssetType::Prefab),
            AssetItem::new("npc.glb", AssetType::Model),
            AssetItem::new("arial.ttf", AssetType::Font),
        ];

        Self {
            theme,
            items,
            selected_name: Some("hero.glb".to_string()),
        }
    }

    /// Set asset items
    pub fn set_items(&mut self, items: Vec<AssetItem>) {
        self.items = items;
    }

    /// Get asset items
    pub fn items(&self) -> &[AssetItem] {
        &self.items
    }

    /// Set selected asset by name
    pub fn set_selected(&mut self, name: Option<String>) {
        self.selected_name = name;
    }

    /// Get selected asset
    pub fn selected(&self) -> Option<&AssetItem> {
        self.selected_name.as_ref()
            .and_then(|name| self.items.iter().find(|item| item.name() == name))
    }

    /// Render the panel header
    fn render_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(32.0))
            .px(px(12.0))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(theme.typography.sm)
                    .text_color(theme.colors.text_secondary)
                    .child("⊞")
                    .child("Asset Grid & Preview")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .child("⚙")
            )
    }

    /// Render a single asset item
    fn render_asset_item(&self, item: &AssetItem, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_selected = self.selected_name.as_ref()
            .map(|name| name == item.name())
            .unwrap_or(false);

        let item_name = item.name().to_string();
        let view = cx.view().clone();

        div()
            .id(SharedString::from(item_name.clone()))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w(px(90.0))
            .h(px(90.0))
            .bg(theme.colors.surface_active)
            .border_1()
            .border_color(if is_selected {
                theme.colors.accent
            } else {
                theme.colors.border
            })
            .when(is_selected, |this| {
                this.border_2()
            })
            .rounded(px(8.0))
            .gap(px(4.0))
            .hover(|this| this.bg(theme.colors.surface_hover))
            .cursor_pointer()
            .on_click(move |_e, cx| {
                let name = item_name.clone();
                view.update(cx, |this, cx| {
                    this.set_selected(Some(name));
                    cx.notify();
                });
            })
            .child(
                div()
                    .text_size(px(32.0))
                    .child(item.asset_type.icon().to_string())
            )
            .child(
                div()
                    .text_size(theme.typography.xs)
                    .text_color(if is_selected {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .child(item.name.clone())
            )
    }

    /// Render the asset grid
    fn render_grid(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let items = self.items.clone();
        let cols = 4; // Number of columns in grid
        let rows = (items.len() + cols - 1) / cols;
        let view = cx.view().clone();

        div()
            .flex_1()
            .bg(theme.colors.surface_active)
            .rounded(px(6.0))
            .h(px(300.0)) // Fixed height for virtual list container
            .child(
                uniform_list(
                    view,
                    "asset-grid",
                    rows,
                    move |this, range, cx| {
                        let theme = this.theme.clone();
                        range.map(|row_idx| {
                             div()
                                .flex()
                                .gap(px(8.0))
                                .p(px(4.0))
                                .children((0..cols).map(|c| {
                                    let item_idx = row_idx * cols + c;
                                    if item_idx < this.items.len() {
                                        this.render_asset_item(&this.items[item_idx], cx).into_any_element()
                                    } else {
                                        // Spacer to maintain alignment
                                        div().w(px(90.0)).into_any_element()
                                    }
                                }))
                                .into_any_element()
                        }).collect()
                    }
                )
                .track_scroll(UniformListScrollHandle::new())
            )
    }

    /// Render the preview area
    fn render_preview(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let selected = self.selected().cloned();

        div()
            .flex_1()
            .bg(theme.colors.background)
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.colors.border)
            .p(px(12.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            // Preview header
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(theme.typography.md)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text)
                    .child("👁")
                    .child(format!("Preview: {}", 
                        selected.as_ref().map(|s| s.name().to_string()).unwrap_or_else(|| "None".to_string())))
            )
            // Preview content
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(theme.colors.surface_active)
                    .rounded(px(4.0))
                    .child(
                        if let Some(ref item) = selected {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .w(px(80.0))
                                        .h(px(80.0))
                                        .bg(theme.colors.surface)
                                        .rounded(px(4.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(48.0))
                                        .child(item.asset_type.icon().to_string())
                                )
                                .child(
                                    div()
                                        .text_size(theme.typography.sm)
                                        .text_color(theme.colors.text_secondary)
                                        .child(format!("{} ({})", 
                                            item.asset_type.display_name(),
                                            item.size().unwrap_or("Unknown size")))
                                )
                        } else {
                            div()
                                .text_size(theme.typography.sm)
                                .text_color(theme.colors.text_secondary)
                                .child("Select an asset to preview")
                        }
                    )
            )
            // Preview controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .text_color(theme.colors.text_secondary)
                    .child("↺")
                    .child("↻")
                    .child("🔍+")
                    .child("⛶")
            )
    }
}

impl Render for AssetGrid {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.background)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_t(px(4.0))
            // Header
            .child(self.render_header())
            // Content: Grid + Preview
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .p(px(8.0))
                    .gap(px(12.0))
                    // Asset grid
                    .child(self.render_grid(cx))
                    // Preview area
                    .child(self.render_preview())
            )
    }
}
