//! File Tree (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum FileTreeItemType {
    File,
    Folder,
}

#[derive(Debug, Clone, Data)]
pub struct FileTreeItem {
    pub name: String,
    pub path: String,
    pub item_type: FileTreeItemType,
    pub children: Vec<FileTreeItem>,
    pub is_expanded: bool,
}

#[derive(Lens, Clone, Data)]
pub struct FileTreeState {
    pub theme: Arc<Theme>,
    pub root_items: Vec<FileTreeItem>,
    pub selected_path: Option<String>,
}

impl FileTreeState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            root_items: Vec::new(),
            selected_path: None,
        }
    }
}
