//! Menu Bar Component
//!
//! Top menu bar matching the HTML prototype:
//! File | Edit | Assets | GameObject | Component | Window | AI | Help

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext, InteractiveElement,
    MouseButton, MouseDownEvent, ClickEvent, AnyElement,
};
use gpui::prelude::FluentBuilder;
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::ui::components::Button;
use crate::services::engine_bridge::EngineHandle;
use crate::core::state::EditorStateManager;
use luminara_scene::Name;
use luminara_math::Transform;

/// Action emitted when a menu item is clicked
#[derive(Clone)]
pub struct MenuExecute {
    pub action: MenuActionType,
}

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
    engine_handle: Option<Arc<EngineHandle>>,
    state: Option<gpui::Model<EditorStateManager>>,
    items: Vec<MenuItem>,
    active_menu: Option<usize>,
    show_menu_index: Option<usize>,
}

impl gpui::EventEmitter<MenuExecute> for MenuBar {}

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
            engine_handle: None,
            state: None,
            items,
            active_menu: None,
            show_menu_index: None,
        }
    }

    /// Set engine handle for menu actions
    pub fn with_engine_handle(mut self, engine_handle: Arc<EngineHandle>) -> Self {
        self.engine_handle = Some(engine_handle);
        self
    }

    /// Set state model for menu actions
    pub fn with_state(mut self, state: gpui::Model<EditorStateManager>) -> Self {
        self.state = Some(state);
        self
    }

    /// Handle menu action
    fn handle_action(&self, action: &MenuActionType, cx: &mut ViewContext<Self>) {
        match action {
            MenuActionType::NewScene => {
                println!("New Scene - Creating empty scene");
                // Clear current scene and create new
                if let Some(engine) = &self.engine_handle {
                    let mut world = engine.world_mut();
                    // Despawn all entities
                    let entities: Vec<_> = world.entities().into_iter().collect();
                    for entity in entities {
                        world.despawn(entity);
                    }
                }
                cx.notify();
            }
            MenuActionType::OpenScene => {
                println!("Open Scene - Would open file dialog");
            }
            MenuActionType::SaveScene => {
                println!("Save Scene - Saving current scene");
                if let Some(engine) = &self.engine_handle {
                    let world = engine.world();
                    let scene = luminara_scene::Scene::from_world(&world);
                    if let Ok(json) = scene.to_json() {
                        println!("Scene JSON: {}", json.len());
                        // In real implementation, save to file
                    }
                }
            }
            MenuActionType::SaveAs => {
                println!("Save As - Would open save dialog");
            }
            MenuActionType::Exit => {
                println!("Exit - Closing application");
                std::process::exit(0);
            }
            MenuActionType::Undo => {
                println!("Undo");
                if let Some(state) = &self.state {
                    state.update(cx, |state, cx| state.undo(cx));
                }
            }
            MenuActionType::Redo => {
                println!("Redo");
                if let Some(state) = &self.state {
                    state.update(cx, |state, cx| state.redo(cx));
                }
            }
            MenuActionType::Cut => println!("Cut"),
            MenuActionType::Copy => println!("Copy"),
            MenuActionType::Paste => println!("Paste"),
            MenuActionType::Delete => {
                println!("Delete - Removing selected entities");
                if let (Some(engine), Some(state)) = (&self.engine_handle, &self.state) {
                    let selected_ids = state.read(cx).session.selected_entities.clone();
                    for id_str in selected_ids {
                        if let Some((id_part, gen_part)) = id_str.split_once(':') {
                            if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                let _ = engine.despawn_entity(luminara_core::Entity::from_raw(id, gen));
                            }
                        }
                    }
                    state.update(cx, |state, cx| {
                        state.select_entities(Vec::new(), cx);
                    });
                }
            }
            MenuActionType::SelectAll => {
                println!("Select All");
                if let (Some(engine), Some(state)) = (&self.engine_handle, &self.state) {
                    let world = engine.world();
                    let entities: Vec<String> = world.entities().into_iter()
                        .map(|e| format!("{}:{}", e.id(), e.generation()))
                        .collect();
                    drop(world);
                    state.update(cx, |state, cx| {
                        state.select_entities(entities, cx);
                    });
                }
            }
            MenuActionType::CreateEmpty => {
                println!("Create Empty GameObject");
                if let Some(engine) = &self.engine_handle {
                    let entity = engine.spawn_entity_with((
                        Transform::IDENTITY,
                        Name::new("GameObject"),
                    ));
                    if let Ok(entity) = entity {
                        if let Some(state) = &self.state {
                            state.update(cx, |state, cx| {
                                let entity_id = format!("{}:{}", entity.id(), entity.generation());
                                state.select_entities(vec![entity_id], cx);
                            });
                        }
                    }
                }
            }
            MenuActionType::CreateLight => {
                println!("Create Light");
                if let Some(engine) = &self.engine_handle {
                    let entity = engine.spawn_entity_with((
                        Transform::from_xyz(0.0, 5.0, 0.0),
                        Name::new("Light"),
                    ));
                    if let Ok(entity) = entity {
                        if let Some(state) = &self.state {
                            state.update(cx, |state, cx| {
                                let entity_id = format!("{}:{}", entity.id(), entity.generation());
                                state.select_entities(vec![entity_id], cx);
                            });
                        }
                    }
                }
            }
            MenuActionType::CreateCamera => {
                println!("Create Camera");
                if let Some(engine) = &self.engine_handle {
                    let entity = engine.spawn_entity_with((
                        Transform::from_xyz(0.0, 2.0, -10.0).looking_at(luminara_math::Vec3::ZERO, luminara_math::Vec3::Y),
                        Name::new("Camera"),
                    ));
                    if let Ok(entity) = entity {
                        if let Some(state) = &self.state {
                            state.update(cx, |state, cx| {
                                let entity_id = format!("{}:{}", entity.id(), entity.generation());
                                state.select_entities(vec![entity_id], cx);
                            });
                        }
                    }
                }
            }
            MenuActionType::ImportAsset => println!("Import Asset"),
            MenuActionType::ExportAsset => println!("Export Asset"),
            MenuActionType::Settings => println!("Settings"),
            MenuActionType::Help => println!("Help"),
            MenuActionType::About => {
                println!("About Luminara Editor");
            }
            MenuActionType::Custom(action) => println!("Custom action: {}", action),
        }
    }

    /// Toggle menu visibility
    fn toggle_menu(&mut self, index: usize) {
        if self.show_menu_index == Some(index) {
            self.show_menu_index = None;
        } else {
            self.show_menu_index = Some(index);
        }
    }

    /// Close menu
    fn close_menu(&mut self) {
        self.show_menu_index = None;
    }
}

impl IntoElement for MenuBar {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.render_element()
    }
}

impl MenuBar {
    fn render_element(&self) -> AnyElement {
        // Simple non-interactive version for now
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
            .into_any_element()
    }
}

impl Render for MenuBar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let items = self.items.clone();
        let show_menu_index = self.show_menu_index;

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
                items.into_iter().enumerate().map(|(index, item)| {
                    let theme = theme.clone();
                    let label = item.label.clone();
                    let is_active = show_menu_index == Some(index);
                    let items_clone = item.items.clone();

                    div()
                        .relative()
                        .child(
                            div()
                                .px(theme.spacing.md)
                                .py(theme.spacing.xs)
                                .rounded(theme.borders.xs)
                                .bg(if is_active { theme.colors.surface_hover } else { gpui::transparent_black() })
                                .hover(|this| this.bg(theme.colors.surface_hover))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                                    this.toggle_menu(index);
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .text_color(theme.colors.text)
                                        .text_size(theme.typography.md)
                                        .child(label.clone())
                                )
                        )
                        .when(is_active, |this| {
                            // Render dropdown menu
                            this.child(
                                div()
                                    .absolute()
                                    .top(px(28.0))
                                    .left(px(0.0))
                                    .min_w(px(200.0))
                                    .bg(theme.colors.surface)
                                    .border_1()
                                    .border_color(theme.colors.border)
                                    .rounded(theme.borders.sm)
                                    .shadow_md()
                                    .py(theme.spacing.xs)
                                    .children(items_clone.into_iter().map(|action| {
                                        let theme = theme.clone();
                                        let action_type = action.action.clone();
                                        let shortcut_text = action.shortcut.clone().unwrap_or_default();
                                        
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .px(theme.spacing.md)
                                            .py(theme.spacing.sm)
                                            .hover(|this| this.bg(theme.colors.surface_hover))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                                                this.handle_action(&action_type, cx);
                                                this.close_menu();
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .text_color(theme.colors.text)
                                                    .text_size(theme.typography.sm)
                                                    .child(action.label.clone())
                                            )
                                            .when(!shortcut_text.is_empty(), |this| {
                                                this.child(
                                                    div()
                                                        .text_color(theme.colors.text_secondary)
                                                        .text_size(theme.typography.xs)
                                                        .child(shortcut_text)
                                                )
                                            })
                                    }))
                            )
                        })
                })
            )
    }
}
