//! File Tree Component
//!
//! A VS Code-style file explorer panel with:
//! - Collapsible folder structure
//! - File icons based on extension
//! - Selection highlighting
//! - Expand/collapse controls

use gpui::{
    div, px, IntoElement, ParentElement, Styled, svg, InteractiveElement,
};
use std::sync::Arc;
use std::collections::HashSet;
use crate::ui::theme::Theme;

/// Type of file tree item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeItemType {
    /// Folder/directory
    Folder,
    /// File
    File,
}

/// A single item in the file tree
#[derive(Debug, Clone)]
pub struct FileTreeItem {
    /// Item name
    pub name: String,
    /// Item type
    pub item_type: FileTreeItemType,
    /// Full path
    pub path: String,
    /// Children (for folders)
    pub children: Vec<FileTreeItem>,
    /// Whether this folder is expanded
    pub is_expanded: bool,
}

impl FileTreeItem {
    /// Create a new folder
    pub fn folder(name: &str, path: &str, children: Vec<FileTreeItem>) -> Self {
        Self {
            name: name.to_string(),
            item_type: FileTreeItemType::Folder,
            path: path.to_string(),
            children,
            is_expanded: true,
        }
    }
    
    /// Create a new file
    pub fn file(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            item_type: FileTreeItemType::File,
            path: path.to_string(),
            children: Vec::new(),
            is_expanded: false,
        }
    }
    
    /// Get the appropriate icon based on file type
    pub fn icon(&self) -> &'static str {
        match self.item_type {
            FileTreeItemType::Folder => {
                if self.is_expanded {
                    "icons/folder-open.svg"
                } else {
                    "icons/folder.svg"
                }
            }
            FileTreeItemType::File => {
                match self.name.rsplit('.').next() {
                    Some("rs") => "icons/file-code.svg",
                    Some("sql") => "icons/database.svg",
                    Some("toml") => "icons/settings.svg",
                    Some("md") => "icons/file-text.svg",
                    _ => "icons/file.svg",
                }
            }
        }
    }
}

/// File Tree component
pub struct FileTree {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Root items
    root_items: Vec<FileTreeItem>,
    /// Currently selected item path
    selected_path: Option<String>,
    /// Expanded folder paths
    expanded_paths: HashSet<String>,
}

impl FileTree {
    /// Create a new File Tree with demo content
    pub fn new(theme: Arc<Theme>) -> Self {
        let root_items = vec![
            FileTreeItem::folder("scripts", "scripts", vec![
                FileTreeItem::folder("player", "scripts/player", vec![
                    FileTreeItem::file("player.rs", "scripts/player/player.rs"),
                    FileTreeItem::file("inventory.rs", "scripts/player/inventory.rs"),
                ]),
                FileTreeItem::folder("enemy", "scripts/enemy", vec![
                    FileTreeItem::file("enemy.rs", "scripts/enemy/enemy.rs"),
                    FileTreeItem::file("ai.rs", "scripts/enemy/ai.rs"),
                ]),
                FileTreeItem::file("main.rs", "scripts/main.rs"),
                FileTreeItem::file("lib.rs", "scripts/lib.rs"),
            ]),
            FileTreeItem::folder("queries", "queries", vec![
                FileTreeItem::file("entities.sql", "queries/entities.sql"),
                FileTreeItem::file("scenes.sql", "queries/scenes.sql"),
            ]),
            FileTreeItem::file("Cargo.toml", "Cargo.toml"),
            FileTreeItem::file("README.md", "README.md"),
        ];
        
        let mut expanded_paths = HashSet::new();
        expanded_paths.insert("scripts".to_string());
        expanded_paths.insert("scripts/player".to_string());
        
        Self {
            theme,
            root_items,
            selected_path: Some("scripts/player/player.rs".to_string()),
            expanded_paths,
        }
    }
    
    /// Toggle folder expansion
    pub fn toggle_folder(&mut self, path: &str) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_string());
        }
    }
    
    /// Select a file
    pub fn select_file(&mut self, path: &str) {
        self.selected_path = Some(path.to_string());
    }
    
    /// Check if a folder is expanded
    fn is_expanded(&self, path: &str) -> bool {
        self.expanded_paths.contains(path)
    }
    
    /// Check if an item is selected
    fn is_selected(&self, path: &str) -> bool {
        self.selected_path.as_ref().map(|p| p == path).unwrap_or(false)
    }
    
    /// Render the file tree panel
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let root_items = self.root_items.clone();
        
        div()
            .flex()
            .flex_col()
            .w(px(220.0))
            .h_full()
            .bg(theme.colors.surface)
            .border_r_1()
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
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Explorer")
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(theme.spacing.sm)
                            .child(
                                svg()
                                    .path("icons/plus.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.text_secondary)
                            )
                            .child(
                                svg()
                                    .path("icons/refresh.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.text_secondary)
                            )
                    )
            )
            // Section label
            .child(
                div()
                    .w_full()
                    .px(theme.spacing.md)
                    .py(theme.spacing.xs)
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.xs)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("SCRIPTS")
                    )
            )
            // Tree content
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .children(
                                root_items.iter().map(|item| {
                                    self.render_item(item, 0)
                                })
                            )
                    )
            )
    }
    
    /// Render a single tree item
    fn render_item(&self, item: &FileTreeItem, depth: usize) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_folder = item.item_type == FileTreeItemType::Folder;
        let is_expanded = self.is_expanded(&item.path);
        let is_selected = self.is_selected(&item.path);
        let indent = px((depth * 12) as f32);
        let icon = item.icon().to_string();
        let _item_path = item.path.clone();
        
        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(24.0))
                    .items_center()
                    .pl(px(8.0 + indent.0))
                    .pr(theme.spacing.sm)
                    .bg(if is_selected {
                        theme.colors.toolbar_active
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|this| {
                        if !is_selected {
                            this.bg(theme.colors.surface_hover)
                        } else {
                            this
                        }
                    })
                    .cursor_pointer()
                    // Expand/collapse arrow for folders
                    .child(
                        if is_folder {
                            div()
                                .w(px(16.0))
                                .h(px(16.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .mr(theme.spacing.xs)
                                .child(
                                    svg()
                                        .path(if is_expanded { "icons/chevron-down.svg" } else { "icons/chevron-right.svg" })
                                        .w(px(12.0))
                                        .h(px(12.0))
                                        .text_color(theme.colors.text_secondary)
                                )
                        } else {
                            div()
                                .w(px(16.0))
                                .mr(theme.spacing.xs)
                        }
                    )
                    // Item icon
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .mr(theme.spacing.xs)
                            .child(
                                svg()
                                    .path(icon)
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(if is_selected {
                                        theme.colors.text
                                    } else if is_folder {
                                        theme.colors.accent
                                    } else {
                                        theme.colors.text_secondary
                                    })
                            )
                    )
                    // Item name
                    .child(
                        div()
                            .text_color(if is_selected {
                                theme.colors.text
                            } else {
                                theme.colors.text
                            })
                            .text_size(theme.typography.sm)
                            .child(item.name.clone())
                    )
            )
            // Render children if folder is expanded
            .children(
                if is_folder && is_expanded {
                    Some(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .children(
                                item.children.iter().map(|child| {
                                    self.render_item(child, depth + 1)
                                })
                            )
                    )
                } else {
                    None
                }
            )
    }
}
