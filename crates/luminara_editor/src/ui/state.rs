//! Global Editor State

use vizia::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelType {
    GlobalSearch,
    SceneBuilder,
    LogicGraph,
    Director,
    BackendAI,
    AssetVault,
    Extensions,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DragItem {
    Panel(PanelType),
}

#[derive(Lens)]
pub struct EditorState {
    pub active_panel: PanelType,
    pub drag_item: Option<DragItem>,
    pub activity_bar_items: Vec<ActivityItem>,
    pub dummy_text: String, // For textboxes
}

#[derive(Clone, Data, Debug, PartialEq)]
pub enum ActivityItem {
    Single(PanelType),
    Folder(String, Vec<PanelType>), // Name, Items
}

pub enum EditorEvent {
    SetPanel(PanelType),
    DragStart(DragItem),
    DragEnd,
    DropOnItem(usize, PanelType),
    DropOnFolder(usize, PanelType),
    MoveItem(usize, usize),
    UpdateText(String),
}

impl Model for EditorState {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            EditorEvent::SetPanel(panel) => {
                self.active_panel = *panel;
                println!("Switched to panel: {:?}", panel);
            }
            EditorEvent::DragStart(item) => {
                self.drag_item = Some(item.clone());
                println!("Drag started: {:?}", item);
            }
            EditorEvent::DragEnd => {
                self.drag_item = None;
                println!("Drag ended");
            }
            EditorEvent::DropOnItem(target_idx, dropped_panel) => {
                 // Logic to create a folder (simplified)
                 if let Some(ActivityItem::Single(target_panel)) = self.activity_bar_items.get(*target_idx).cloned() {
                     let new_folder = ActivityItem::Folder("New Group".to_string(), vec![target_panel, *dropped_panel]);
                     self.activity_bar_items[*target_idx] = new_folder;
                     println!("Created folder with {:?} and {:?}", target_panel, dropped_panel);
                }
            }
            EditorEvent::DropOnFolder(folder_idx, dropped_panel) => {
                if let Some(ActivityItem::Folder(_, items)) = self.activity_bar_items.get_mut(*folder_idx) {
                    items.push(*dropped_panel);
                    println!("Added {:?} to folder {}", dropped_panel, folder_idx);
                }
            }
            EditorEvent::MoveItem(from, to) => {
                 println!("Moved item from {} to {}", from, to);
            }
            EditorEvent::UpdateText(text) => {
                self.dummy_text = text.clone();
            }
        });
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_panel: PanelType::SceneBuilder,
            drag_item: None,
            activity_bar_items: vec![
                ActivityItem::Single(PanelType::GlobalSearch),
                ActivityItem::Single(PanelType::SceneBuilder),
                ActivityItem::Single(PanelType::LogicGraph),
                ActivityItem::Single(PanelType::Director),
                ActivityItem::Single(PanelType::BackendAI),
                ActivityItem::Single(PanelType::AssetVault),
                ActivityItem::Single(PanelType::Extensions),
            ],
            dummy_text: String::new(),
        }
    }
}
