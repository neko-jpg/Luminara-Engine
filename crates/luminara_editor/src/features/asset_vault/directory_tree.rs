//! Directory Tree Component
//!
//! A tree view for navigating the asset directory structure with:
//! - Expandable/collapsible folders
//! - File and folder icons
//! - Selection highlighting
//! - Filter/search within tree

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext, prelude::FluentBuilder,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Type of tree item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeItemType {
    /// Folder that can contain other items
    Folder,
    /// File with specific type
    File,
}

/// A single item in the directory tree
#[derive(Debug, Clone)]
pub struct TreeItem {
    /// Display name
    pub name: String,
    /// Item type
    pub item_type: TreeItemType,
    /// Indentation level
    pub level: u32,
    /// Whether folder is expanded
    pub is_expanded: bool,
    /// Icon character/symbol
    pub icon: String,
}

impl TreeItem {
    /// Create a new tree item
    pub fn new(name: &str, item_type: TreeItemType, level: u32) -> Self {
        let icon = match item_type {
            TreeItemType::Folder => "📁",
            TreeItemType::File => "📄",
        }.to_string();

        Self {
            name: name.to_string(),
            item_type,
            level,
            is_expanded: true,
            icon,
        }
    }

    /// Create a folder item
    pub fn folder(name: &str, level: u32) -> Self {
        Self::new(name, TreeItemType::Folder, level)
    }

    /// Create a file item
    pub fn file(name: &str, level: u32) -> Self {
        Self::new(name, TreeItemType::File, level)
    }

    /// Set custom icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    /// Set expanded state
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }
}

/// Directory Tree component
pub struct DirectoryTree {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Tree items
    items: Vec<TreeItem>,
    /// Currently selected item index
    selected_index: Option<usize>,
    /// Filter text
    filter_text: String,
}

impl DirectoryTree {
    /// Create a new empty directory tree
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            items: Vec::new(),
            selected_index: None,
            filter_text: String::new(),
        }
    }

    /// Create with sample data matching HTML prototype
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        let items = vec![
            TreeItem::folder("assets/", 0).with_icon("📂"),
            TreeItem::folder("models/", 1),
            TreeItem::file("hero.glb", 2).with_icon("🧊"),
            TreeItem::file("enemy.glb", 2).with_icon("🧊"),
            TreeItem::folder("textures/", 1),
            TreeItem::file("grass.png", 2).with_icon("🖼"),
            TreeItem::file("stone.png", 2).with_icon("🖼"),
            TreeItem::folder("audio/", 1),
            TreeItem::file("bgm.ogg", 2).with_icon("🎵"),
            TreeItem::file("sfx_jump.wav", 2).with_icon("🎵"),
            TreeItem::folder("scripts/", 1),
            TreeItem::file("player.rs", 2).with_icon("📝"),
            TreeItem::folder("prefabs/", 1),
            TreeItem::file("player_prefab", 2).with_icon("🎲"),
        ];

        Self {
            theme,
            items,
            selected_index: Some(2), // hero.glb selected by default
            filter_text: String::new(),
        }
    }

    /// Set tree items
    pub fn set_items(&mut self, items: Vec<TreeItem>) {
        self.items = items;
    }

    /// Get tree items
    pub fn items(&self) -> &[TreeItem] {
        &self.items
    }

    /// Select an item
    pub fn select_item(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        if index < self.items.len() {
            self.selected_index = Some(index);
            cx.notify();
        }
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&TreeItem> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }

    /// Get selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Toggle folder expansion
    pub fn toggle_folder(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        if let Some(item) = self.items.get_mut(index) {
            if item.item_type == TreeItemType::Folder {
                item.is_expanded = !item.is_expanded;
                cx.notify();
            }
        }
    }

    /// Set filter text
    pub fn set_filter_text(&mut self, text: String, cx: &mut ViewContext<Self>) {
        self.filter_text = text;
        cx.notify();
    }

    /// Render a single tree item
    fn render_item(&self, index: usize, item: &TreeItem) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_selected = self.selected_index == Some(index);
        let level = item.level;
        let indent = level as f32 * 20.0;

        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(28.0))
            .px(px(8.0))
            .pl(px(8.0 + indent))
            .gap(px(4.0))
            .rounded(px(4.0))
            .when(is_selected, |this| {
                this.bg(theme.colors.accent)
            })
            .when(!is_selected, |this| {
                this.hover(|this| this.bg(theme.colors.surface_hover))
            })
            .cursor_pointer()
            .child(
                div()
                    .w(px(20.0))
                    .child(item.icon.clone())
            )
            .child(
                div()
                    .flex_1()
                    .text_size(theme.typography.sm)
                    .text_color(if is_selected {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .child(item.name.clone())
            )
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
                    .child("🌳")
                    .child("Directory Tree")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .child("⋮")
            )
    }

    /// Render filter input
    fn render_filter(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .w_full()
            .p(px(8.0))
            .child(
                div()
                    .w_full()
                    .bg(theme.colors.surface_active)
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(px(4.0))
                    .px(px(8.0))
                    .py(px(5.0))
                    .text_size(theme.typography.sm)
                    .text_color(theme.colors.text_secondary)
                    .child("Filter tree...")
            )
    }
}

impl Render for DirectoryTree {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let items: Vec<_> = self.items.clone();

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
            // Filter input
            .child(self.render_filter())
            // Tree items
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p(px(4.0))
                    .children(
                        items.into_iter().enumerate().map(|(index, item)| {
                            self.render_item(index, &item)
                        })
                    )
            )
    }
}
