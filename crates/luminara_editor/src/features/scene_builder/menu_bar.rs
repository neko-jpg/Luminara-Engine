//! Menu Bar Component
//!
//! Top menu bar matching the HTML prototype:
//! File | Edit | Assets | GameObject | Component | Window | AI | Help

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext, InteractiveElement,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub items: Vec<MenuAction>,
}

/// Menu action (submenu item)
#[derive(Debug, Clone)]
pub struct MenuAction {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: MenuActionType,
}

/// Types of menu actions
#[derive(Debug, Clone)]
pub enum MenuActionType {
    NewScene,
    OpenScene,
    SaveScene,
    SaveAs,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    CreateEmpty,
    CreateLight,
    CreateCamera,
    ImportAsset,
    ExportAsset,
    Settings,
    Help,
    About,
    Custom(String),
}

/// Menu Bar component
pub struct MenuBar {
    theme: Arc<Theme>,
    items: Vec<MenuItem>,
    #[allow(dead_code)]
    active_menu: Option<usize>,
}

impl MenuBar {
    /// Create a new menu bar with default items
    pub fn new(theme: Arc<Theme>) -> Self {
        let items = vec![
            // File menu
            MenuItem {
                label: "File".to_string(),
                items: vec![
                    MenuAction { label: "New Scene".to_string(), shortcut: Some("Ctrl+N".to_string()), action: MenuActionType::NewScene },
                    MenuAction { label: "Open Scene...".to_string(), shortcut: Some("Ctrl+O".to_string()), action: MenuActionType::OpenScene },
                    MenuAction { label: "Save".to_string(), shortcut: Some("Ctrl+S".to_string()), action: MenuActionType::SaveScene },
                    MenuAction { label: "Save As...".to_string(), shortcut: Some("Ctrl+Shift+S".to_string()), action: MenuActionType::SaveAs },
                    MenuAction { label: "Exit".to_string(), shortcut: Some("Alt+F4".to_string()), action: MenuActionType::Exit },
                ],
            },
            // Edit menu
            MenuItem {
                label: "Edit".to_string(),
                items: vec![
                    MenuAction { label: "Undo".to_string(), shortcut: Some("Ctrl+Z".to_string()), action: MenuActionType::Undo },
                    MenuAction { label: "Redo".to_string(), shortcut: Some("Ctrl+Y".to_string()), action: MenuActionType::Redo },
                    MenuAction { label: "Cut".to_string(), shortcut: Some("Ctrl+X".to_string()), action: MenuActionType::Cut },
                    MenuAction { label: "Copy".to_string(), shortcut: Some("Ctrl+C".to_string()), action: MenuActionType::Copy },
                    MenuAction { label: "Paste".to_string(), shortcut: Some("Ctrl+V".to_string()), action: MenuActionType::Paste },
                    MenuAction { label: "Delete".to_string(), shortcut: Some("Del".to_string()), action: MenuActionType::Delete },
                    MenuAction { label: "Select All".to_string(), shortcut: Some("Ctrl+A".to_string()), action: MenuActionType::SelectAll },
                ],
            },
            // Assets menu
            MenuItem {
                label: "Assets".to_string(),
                items: vec![
                    MenuAction { label: "Import...".to_string(), shortcut: None, action: MenuActionType::ImportAsset },
                    MenuAction { label: "Export...".to_string(), shortcut: None, action: MenuActionType::ExportAsset },
                ],
            },
            // GameObject menu
            MenuItem {
                label: "GameObject".to_string(),
                items: vec![
                    MenuAction { label: "Create Empty".to_string(), shortcut: Some("Ctrl+Shift+N".to_string()), action: MenuActionType::CreateEmpty },
                    MenuAction { label: "Light".to_string(), shortcut: None, action: MenuActionType::CreateLight },
                    MenuAction { label: "Camera".to_string(), shortcut: None, action: MenuActionType::CreateCamera },
                ],
            },
            // Component menu
            MenuItem {
                label: "Component".to_string(),
                items: vec![
                    MenuAction { label: "Add Component".to_string(), shortcut: None, action: MenuActionType::Custom("add_component".to_string()) },
                ],
            },
            // Window menu
            MenuItem {
                label: "Window".to_string(),
                items: vec![
                    MenuAction { label: "Settings".to_string(), shortcut: None, action: MenuActionType::Settings },
                ],
            },
            // AI menu
            MenuItem {
                label: "AI".to_string(),
                items: vec![
                    MenuAction { label: "AI Assistant".to_string(), shortcut: None, action: MenuActionType::Custom("ai_assistant".to_string()) },
                ],
            },
            // Help menu
            MenuItem {
                label: "Help".to_string(),
                items: vec![
                    MenuAction { label: "Documentation".to_string(), shortcut: Some("F1".to_string()), action: MenuActionType::Help },
                    MenuAction { label: "About".to_string(), shortcut: None, action: MenuActionType::About },
                ],
            },
        ];

        Self {
            theme,
            items,
            active_menu: None,
        }
    }

    /// Handle menu action
    #[allow(dead_code)]
    fn handle_action(&self, action: &MenuActionType) {
        match action {
            MenuActionType::NewScene => println!("New Scene"),
            MenuActionType::OpenScene => println!("Open Scene"),
            MenuActionType::SaveScene => println!("Save Scene"),
            MenuActionType::SaveAs => println!("Save As"),
            MenuActionType::Exit => println!("Exit"),
            MenuActionType::Undo => println!("Undo"),
            MenuActionType::Redo => println!("Redo"),
            MenuActionType::Cut => println!("Cut"),
            MenuActionType::Copy => println!("Copy"),
            MenuActionType::Paste => println!("Paste"),
            MenuActionType::Delete => println!("Delete"),
            MenuActionType::SelectAll => println!("Select All"),
            MenuActionType::CreateEmpty => println!("Create Empty"),
            MenuActionType::CreateLight => println!("Create Light"),
            MenuActionType::CreateCamera => println!("Create Camera"),
            MenuActionType::ImportAsset => println!("Import Asset"),
            MenuActionType::ExportAsset => println!("Export Asset"),
            MenuActionType::Settings => println!("Settings"),
            MenuActionType::Help => println!("Help"),
            MenuActionType::About => println!("About"),
            MenuActionType::Custom(action) => println!("Custom action: {}", action),
        }
    }
}

impl Render for MenuBar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let items = self.items.clone();

        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(32.0))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.md)
            .children(
                items.into_iter().map(move |item| {
                    let theme = theme.clone();
                    let label = item.label.clone();

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
}
