use std::sync::Arc;
use gpui::{AppContext, Model, ModelContext, Task};
use luminara_db::LuminaraDatabase;
use crate::core::session::{EditorSession, WorkspaceType};

/// Global editor state manager backed by GPUI's reactive model.
pub struct EditorStateManager {
    pub session: EditorSession,
    pub db: Option<Arc<LuminaraDatabase>>,
    pub engine_handle: Option<Arc<crate::services::engine_bridge::EngineHandle>>,
    pub is_syncing: bool,
}

impl EditorStateManager {
    pub fn new(initial_session: EditorSession, db: Option<Arc<LuminaraDatabase>>) -> Self {
        Self {
            session: initial_session,
            db,
            engine_handle: None,
            is_syncing: false,
        }
    }

    pub fn set_database(&mut self, db: Arc<LuminaraDatabase>, cx: &mut ModelContext<Self>) {
        self.db = Some(db);
        cx.notify();
    }

    pub fn set_engine_handle(&mut self, handle: Arc<crate::services::engine_bridge::EngineHandle>, cx: &mut ModelContext<Self>) {
        self.engine_handle = Some(handle);
        cx.notify();
    }

    pub fn toggle_global_search(&mut self, cx: &mut ModelContext<Self>) {
        self.session.global_search_visible = !self.session.global_search_visible;
        cx.notify();
    }

    pub fn switch_workspace(&mut self, new_type: WorkspaceType, cx: &mut ModelContext<Self>) {
        self.session.active_workspace = new_type;
        cx.notify();
    }
    
    pub fn update_layout_config(&mut self, config: serde_json::Value, cx: &mut ModelContext<Self>) {
        self.session.layout_config = config;
        cx.notify();
    }

    pub fn select_entities(&mut self, entities: Vec<String>, cx: &mut ModelContext<Self>) {
        self.session.selected_entities = entities;
        cx.notify();
    }

    pub fn set_active_tool(&mut self, tool: String, cx: &mut ModelContext<Self>) {
        self.session.active_tool = tool;
        cx.notify();
    }

    pub fn set_active_bottom_tab(&mut self, tab: String, cx: &mut ModelContext<Self>) {
        self.session.active_bottom_tab = tab;
        cx.notify();
    }

    pub fn undo(&mut self, _cx: &mut ModelContext<Self>) {
        log::warn!("Undo not implemented in MVP");
    }

    pub fn redo(&mut self, _cx: &mut ModelContext<Self>) {
        log::warn!("Redo not implemented in MVP");
    }

    pub fn set_editor_mode(&mut self, mode: String, cx: &mut ModelContext<Self>) {
        self.session.editor_mode = mode;
        cx.notify();
    }
    
    pub fn spawn_entity(&mut self, name: &str, cx: &mut ModelContext<Self>) -> Option<luminara_core::Entity> {
        log::info!("Spawn entity '{}' (Command sent to Bevy)", name);
        if let Some(engine) = &self.engine_handle {
            // We can send a command to Bevy to spawn
            // But we can't return the Entity ID immediately because command is async via channel.
            // So we return None for now.
            // In a real implementation, we might generate ID here (if possible) or listen for event.
            let name_string = name.to_string();
            engine.execute_command(move |world| {
                world.spawn((
                    // Name::new(name_string) // Need Bevy Name
                    bevy::core::Name::new(name_string),
                    bevy::transform::components::Transform::default(),
                ));
            });
        }
        None
    }

    pub fn despawn_entity(&mut self, _entity: luminara_core::Entity, _cx: &mut ModelContext<Self>) {
        log::info!("Despawn entity (Stub)");
    }

    pub fn duplicate_entity(&mut self, _entity: luminara_core::Entity, _cx: &mut ModelContext<Self>) {
        log::info!("Duplicate entity (Stub)");
    }

    pub fn add_component(&mut self, _entity: luminara_core::Entity, _component: &str, _cx: &mut ModelContext<Self>) {
        log::info!("Add component (Stub)");
    }

    pub fn record_component_edit(&mut self, _id: String, _comp: String, _new: serde_json::Value, _old: serde_json::Value, _cx: &mut ModelContext<Self>) {
        // Stub
    }
    
    pub fn sync_to_db(&mut self, cx: &mut ModelContext<Self>, _action: &str, _payload: serde_json::Value, _inverse: serde_json::Value) {
        // Stub for now to avoid DB complexity
        cx.notify();
    }

    pub fn __sync_with_state(&mut self, cx: &mut ModelContext<crate::features::global_search::GlobalSearch>) {
        // Helper for GlobalSearch to access state?
        // Wait, EditorWindow called search.__sync_with_state(cx).
        // GlobalSearch has `__sync_with_state`.
        // This method is not in EditorStateManager.
    }
}
