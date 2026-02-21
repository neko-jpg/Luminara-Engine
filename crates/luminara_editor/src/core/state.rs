use std::sync::Arc;
use gpui::{AppContext, Model, ModelContext, Task};
use luminara_db::LuminaraDatabase;
use crate::core::session::{EditorSession, WorkspaceType};

/// Global editor state manager backed by GPUI's reactive model.
///
/// This serves as the integration point between the Local-First database layer
/// and the GPUI frontend. It stores the optimistic UI state and handles background
/// synchronization with the database.
pub struct EditorStateManager {
    pub session: EditorSession,
    pub db: Option<Arc<LuminaraDatabase>>,
    pub engine_handle: Option<Arc<crate::services::engine_bridge::EngineHandle>>,
    /// True if there is an active database sync task running
    pub is_syncing: bool,
}

impl EditorStateManager {
    /// Initialize a new state manager.
    ///
    /// The database is optional to allow the UI to start immediately with default
    /// state while the database initializes in the background.
    pub fn new(initial_session: EditorSession, db: Option<Arc<LuminaraDatabase>>) -> Self {
        Self {
            session: initial_session,
            db,
            engine_handle: None,
            is_syncing: false,
        }
    }

    /// Provide the database post-initialization.
    pub fn set_database(&mut self, db: Arc<LuminaraDatabase>, cx: &mut ModelContext<Self>) {
        self.db = Some(db);
        cx.notify();
    }

    /// Set the engine handle post-initialization.
    pub fn set_engine_handle(&mut self, handle: Arc<crate::services::engine_bridge::EngineHandle>, cx: &mut ModelContext<Self>) {
        self.engine_handle = Some(handle);
        cx.notify();
    }

    /// Set the entire session from the database (e.g., on first load).
    pub fn set_session(&mut self, session: EditorSession, cx: &mut ModelContext<Self>) {
        self.session = session;
        cx.notify();
    }

    // --- Action Methods (Optimistic Updates) ---

    /// Toggle global search visibility.
    pub fn toggle_global_search(&mut self, cx: &mut ModelContext<Self>) {
        let old_val = self.session.global_search_visible;
        
        // Optimistic UI update
        self.session.global_search_visible = !self.session.global_search_visible;
        log::info!("Global Search toggled: {}", self.session.global_search_visible);
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "TOGGLE_SEARCH", 
            serde_json::json!({ "visible": self.session.global_search_visible }),
            serde_json::json!({ "visible": old_val })
        );
    }

    /// Switch the active workspace.
    pub fn switch_workspace(&mut self, new_type: WorkspaceType, cx: &mut ModelContext<Self>) {
        if self.session.active_workspace == new_type {
            return;
        }

        let old_type = self.session.active_workspace;
        
        // Optimistic UI update
        self.session.active_workspace = new_type;
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "SWITCH_WORKSPACE", 
            serde_json::json!({ "from": old_type as usize, "to": new_type as usize }),
            serde_json::json!({ "from": new_type as usize, "to": old_type as usize })
        );
    }
    
    /// Update the layout configuration.
    /// This should be called by the View after debouncing resize/drag events.
    pub fn update_layout_config(&mut self, config: serde_json::Value, cx: &mut ModelContext<Self>) {
        let old_config = self.session.layout_config.clone();
        
        // Optimistic UI update
        self.session.layout_config = config.clone();
        cx.notify();

        // Background sync
        self.sync_to_db(cx, "UPDATE_LAYOUT", config, old_config);
    }

    /// Update entity selection.
    pub fn select_entities(&mut self, entities: Vec<String>, cx: &mut ModelContext<Self>) {
        let old_selection = self.session.selected_entities.clone();
        
        // Optimistic UI update
        self.session.selected_entities = entities.clone();
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "SELECT_ENTITIES", 
            serde_json::json!({ "entities": entities }), 
            serde_json::json!({ "entities": old_selection })
        );
    }

    /// Set the active tool mode.
    pub fn set_active_tool(&mut self, tool: String, cx: &mut ModelContext<Self>) {
        if self.session.active_tool == tool {
            return;
        }

        let old_tool = self.session.active_tool.clone();
        
        // Optimistic UI update
        self.session.active_tool = tool.clone();
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "SET_TOOL", 
            serde_json::json!({ "tool": tool }), 
            serde_json::json!({ "tool": old_tool })
        );
    }

    /// Set the active bottom tab.
    pub fn set_active_bottom_tab(&mut self, tab: String, cx: &mut ModelContext<Self>) {
        if self.session.active_bottom_tab == tab {
            return;
        }

        let old_tab = self.session.active_bottom_tab.clone();
        
        // Optimistic UI update
        self.session.active_bottom_tab = tab.clone();
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "SET_BOTTOM_TAB", 
            serde_json::json!({ "tab": tab }), 
            serde_json::json!({ "tab": old_tab })
        );
    }

    /// Record a component edit in the database for Undo/Redo and persist component state.
    pub fn record_component_edit(
        &mut self,
        entity_id: String,
        component: String,
        new_value: serde_json::Value,
        old_value: serde_json::Value,
        cx: &mut ModelContext<Self>
    ) {
        self.sync_to_db(
            cx,
            "UPDATE_COMPONENT",
            serde_json::json!({
                "entity": entity_id,
                "component": component,
                "value": new_value
            }),
            serde_json::json!({
                "entity": entity_id,
                "component": component,
                "value": old_value
            })
        );

        // Map component name to type_id for reflection (rough mapping for now)
        let type_id = match component.as_str() {
            "Transform" => "luminara_math::Transform",
            "Name" => "luminara_scene::Name",
            _ => "unknown",
        }.to_string();

        if let Some(db) = self.db.clone() {
            let entity_id_clone = entity_id.clone();
            let component_clone = component.clone();
            let value_clone = new_value.clone();
            cx.spawn(|_, _| async move {
                let comp_id = luminara_db::RecordId::from(("component", format!("{}_{}", entity_id_clone, component_clone.to_lowercase())));
                let mut comp_record = luminara_db::schema::ComponentRecord::new(
                    component_clone,
                    type_id,
                    value_clone,
                    luminara_db::RecordId::from(("entity", entity_id_clone.as_str()))
                );
                comp_record.id = Some(comp_id);
                let _ = db.store_component(comp_record).await;
            }).detach();
        }
    }

    /// Add a new component to an entity
    pub fn add_component(&mut self, entity: luminara_core::Entity, component_type: &str, cx: &mut ModelContext<Self>) {
        let Some(engine) = self.engine_handle.clone() else { return; };
        
        let entity_id_str = format!("{}:{}", entity.id(), entity.generation());
        let comp_type_str = component_type.to_string();

        // Ensure we record undo command
        self.sync_to_db(
            cx,
            "ADD_COMPONENT",
            serde_json::json!({
                "entity": entity_id_str,
                "component": comp_type_str
            }),
            serde_json::json!({
                "entity": entity_id_str,
                "component": comp_type_str
            })
        );

        let initial_value = match component_type {
            "Transform" => {
                let val = luminara_math::Transform::IDENTITY;
                let _ = engine.update_component(entity, val);
                serde_json::to_value(&val).unwrap_or_default()
            }
            "Name" => {
                let val = luminara_scene::Name::new("New Entity");
                let _ = engine.update_component(entity, val.clone());
                serde_json::to_value(&val).unwrap_or_default()
            }
            _ => serde_json::json!({})
        };

        if let Some(db) = self.db.clone() {
            let type_id = match component_type {
                "Transform" => "luminara_math::Transform",
                "Name" => "luminara_scene::Name",
                _ => "unknown",
            }.to_string();

            let entity_id_clone = entity_id_str.clone();
            let component_clone = comp_type_str.clone();
            cx.spawn(|_, _| async move {
                let comp_id = luminara_db::RecordId::from(("component", format!("{}_{}", entity_id_clone, component_clone.to_lowercase())));
                let mut comp_record = luminara_db::schema::ComponentRecord::new(
                    component_clone,
                    type_id,
                    initial_value,
                    luminara_db::RecordId::from(("entity", entity_id_clone.as_str()))
                );
                comp_record.id = Some(comp_id);
                let _ = db.store_component(comp_record).await;
            }).detach();
        }
    }

    /// Set the editor mode (Edit, Play, Pause).
    pub fn set_editor_mode(&mut self, mode: String, cx: &mut ModelContext<Self>) {
        if self.session.editor_mode == mode {
            return;
        }

        let old_mode = self.session.editor_mode.clone();
        
        // Optimistic UI update
        self.session.editor_mode = mode.clone();
        cx.notify();

        // Background sync
        self.sync_to_db(
            cx, 
            "SET_EDITOR_MODE", 
            serde_json::json!({ "mode": mode }), 
            serde_json::json!({ "mode": old_mode })
        );
    }

    // --- Entity CRUD & Hierarchy Operations ---
    
    /// Spawn a new entity and sync to the backend
    pub fn spawn_entity(&mut self, name: &str, cx: &mut ModelContext<Self>) -> Option<luminara_core::Entity> {
        let Some(engine) = self.engine_handle.clone() else { return None; };

        // 1. Spawn in ECS
        let entity_res = engine.spawn_entity_with((
            luminara_math::Transform::IDENTITY,
            luminara_scene::Name::new(name),
        ));

        let Ok(entity) = entity_res else { return None; };

        // 2. Persist to DB
        let entity_id_str = format!("{}:{}", entity.id(), entity.generation());
        let name_clone = name.to_string();

        if let Some(db) = self.db.clone() {
            let entity_id_str_clone = entity_id_str.clone();
            cx.spawn(|this, mut cx| async move {
                let mut entity_record = luminara_db::schema::EntityRecord::new(Some(name_clone.clone()));
                entity_record.id = Some(luminara_db::RecordId::from(("entity", entity_id_str_clone.as_str())));
                
                let _ = db.store_entity(entity_record.clone()).await;
                
                let mut transform_record = luminara_db::schema::ComponentRecord::new(
                    "Transform",
                    "luminara_math::Transform",
                    serde_json::json!(luminara_math::Transform::IDENTITY),
                    entity_record.id.clone().unwrap()
                );
                transform_record.id = Some(luminara_db::RecordId::from(("component", format!("{}_transform", entity_id_str_clone))));
                let _ = db.store_component(transform_record).await;
                
                let mut name_record = luminara_db::schema::ComponentRecord::new(
                    "Name",
                    "luminara_scene::Name",
                    serde_json::json!(luminara_scene::Name::new(&name_clone)),
                    entity_record.id.clone().unwrap()
                );
                name_record.id = Some(luminara_db::RecordId::from(("component", format!("{}_name", entity_id_str_clone))));
                let _ = db.store_component(name_record).await;

                // UI Command for Undo
                let _ = this.update(&mut cx, |this_model, cx| {
                    this_model.sync_to_db(
                        cx,
                        "SPAWN_ENTITY",
                        serde_json::json!({ "entity": entity_id_str_clone, "name": name_clone }),
                        serde_json::json!({ "entity": entity_id_str_clone })
                    );
                });
            }).detach();
        }

        Some(entity)
    }

    /// Despawn an entity and sync to the backend
    pub fn despawn_entity(&mut self, entity: luminara_core::Entity, cx: &mut ModelContext<Self>) {
        let Some(engine) = self.engine_handle.clone() else { return; };
        
        let entity_id_str = format!("{}:{}", entity.id(), entity.generation());
        
        // Despawn from ECS
        let _ = engine.despawn_entity(entity);

        // Remove from selection if present
        let mut selected = self.session.selected_entities.clone();
        if let Some(pos) = selected.iter().position(|id| id == &entity_id_str) {
            selected.remove(pos);
            self.select_entities(selected, cx);
        }

        // Persist to DB
        if let Some(db) = self.db.clone() {
            let entity_id_str_clone = entity_id_str.clone();
            cx.spawn(|this, mut cx| async move {
                let id = luminara_db::RecordId::from(("entity", entity_id_str_clone.as_str()));
                // Load it to get components for undo history
                if let Ok((_ent_rec, comp_recs)) = db.load_entity_with_components(&id).await {
                    let _ = db.delete_entity(&id).await;
                    for comp in comp_recs {
                        if let Some(comp_id) = comp.id {
                            let _ = db.delete_component(&comp_id).await;
                        }
                    }
                }
                
                let _ = this.update(&mut cx, |this_model, cx| {
                    this_model.sync_to_db(
                        cx,
                        "DESPAWN_ENTITY",
                        serde_json::json!({ "entity": entity_id_str_clone }),
                        serde_json::json!({})
                    );
                });
            }).detach();
        }
    }

    /// Duplicate an entity and sync to the backend
    pub fn duplicate_entity(&mut self, entity: luminara_core::Entity, cx: &mut ModelContext<Self>) {
        let Some(engine) = self.engine_handle.clone() else { return; };
        
        engine.execute_command(Box::new(crate::core::commands::DuplicateEntityCommand::new(entity)));
        
        let entity_id_str = format!("{}:{}", entity.id(), entity.generation());
        self.sync_to_db(
            cx,
            "DUPLICATE_ENTITY",
            serde_json::json!({ "source_entity": entity_id_str }),
            serde_json::json!({}) // Inverse would need to despawn the new entity id, which we don't know here
        );
    }

    pub fn rename_entity(&mut self, entity: luminara_core::Entity, new_name: String, cx: &mut ModelContext<Self>) {
        let Some(engine) = self.engine_handle.clone() else { return; };
        
        let old_name = engine.world().get_component::<luminara_scene::Name>(entity).map(|n| n.0.clone()).unwrap_or_default();
        if old_name == new_name { return; }

        let _ = engine.update_component(entity, luminara_scene::Name::new(&new_name));
        
        // Sync to DB
        let entity_id_str = format!("{}:{}", entity.id(), entity.generation());
        self.sync_to_db(
            cx,
            "RENAME_ENTITY",
            serde_json::json!({ "entity": entity_id_str, "name": new_name }),
            serde_json::json!({ "entity": entity_id_str, "name": old_name })
        );

        if let Some(db) = self.db.clone() {
            let entity_id_str_clone = entity_id_str.clone();
            let new_name_clone = new_name.clone();
            cx.spawn(|_, _| async move {
                let comp_id = luminara_db::RecordId::from(("component", format!("{}_name", entity_id_str_clone)));
                let mut name_record = luminara_db::schema::ComponentRecord::new(
                    "Name",
                    "luminara_scene::Name",
                    serde_json::json!(luminara_scene::Name::new(&new_name_clone)),
                    luminara_db::RecordId::from(("entity", entity_id_str_clone.as_str()))
                );
                name_record.id = Some(comp_id);
                let _ = db.store_component(name_record).await;
            }).detach();
        }
    }

    /// Undo the last UI action
    pub fn undo(&mut self, cx: &mut ModelContext<Self>) {
        let Some(db) = self.db.clone() else { return };
        let session_id = luminara_db::RecordId::from(("editor_session", self.session.name.clone()));
        
        self.is_syncing = true;
        cx.notify();

        cx.spawn(|this, mut cx| async move {
            // 1. Fetch latest command that is NOT undone
            if let Ok(mut cmds) = db.load_ui_commands(&session_id, 1, false).await {
                if let Some(cmd) = cmds.pop() {
                    let action = cmd.action.clone();
                    let inverse = cmd.inverse_payload.clone();
                    let cmd_id = cmd.id.clone();
                    
                    // 2. Apply inverse payload to UI state
                    let _ = this.update(&mut cx, |this_model, cx| {
                        match action.as_str() {
                            "TOGGLE_SEARCH" => {
                                if let Some(v) = inverse.get("visible").and_then(|v| v.as_bool()) {
                                    this_model.session.global_search_visible = v;
                                }
                            }
                            "SWITCH_WORKSPACE" => {
                                if let Some(t) = inverse.get("to").and_then(|t| t.as_u64()) {
                                    this_model.session.active_workspace = match t {
                                        0 => WorkspaceType::SceneBuilder,
                                        1 => WorkspaceType::LogicGraph,
                                        _ => WorkspaceType::SceneBuilder,
                                    };
                                }
                            }
                            "UPDATE_LAYOUT" => {
                                this_model.session.layout_config = inverse;
                            }
                            "SELECT_ENTITIES" => {
                                if let Some(entities) = inverse.get("entities").and_then(|v| v.as_array()) {
                                    this_model.session.selected_entities = entities.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect();
                                }
                            }
                            "SET_TOOL" => {
                                if let Some(tool) = inverse.get("tool").and_then(|v| v.as_str()) {
                                    this_model.session.active_tool = tool.to_string();
                                }
                            }
                            "SET_BOTTOM_TAB" => {
                                if let Some(tab) = inverse.get("tab").and_then(|v| v.as_str()) {
                                    this_model.session.active_bottom_tab = tab.to_string();
                                }
                            }
                            "SET_EDITOR_MODE" => {
                                if let Some(mode) = inverse.get("mode").and_then(|v| v.as_str()) {
                                    this_model.session.editor_mode = mode.to_string();
                                }
                            }
                            "UPDATE_COMPONENT" | "RENAME_ENTITY" => {
                                let entity_str = inverse.get("entity").and_then(|v| v.as_str());
                                let component = inverse.get("component").and_then(|v| v.as_str());
                                let value = inverse.get("value");
                                
                                if let (Some(entity_str), Some(component), Some(value), Some(engine)) = (entity_str, component, value, &this_model.engine_handle) {
                                    if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                        if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                            let entity = luminara_core::Entity::from_raw(id, gen);
                                            match component {
                                                "Transform" => {
                                                    if let Ok(transform) = serde_json::from_value::<luminara_math::Transform>(value.clone()) {
                                                        let _ = engine.update_component(entity, transform);
                                                    }
                                                }
                                                "Name" => {
                                                    if let Some(name) = value.as_str() {
                                                        let _ = engine.update_component(entity, luminara_scene::Name::new(name));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            "ADD_COMPONENT" => {
                                let entity_str = inverse.get("entity").and_then(|v| v.as_str());
                                let component = inverse.get("component").and_then(|v| v.as_str());
                                
                                if let (Some(entity_str), Some(component), Some(engine)) = (entity_str, component, &this_model.engine_handle) {
                                    if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                        if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                            let entity = luminara_core::Entity::from_raw(id, gen);
                                            match component {
                                                "Transform" => {
                                                    let _ = engine.remove_component::<luminara_math::Transform>(entity);
                                                }
                                                "Name" => {
                                                    let _ = engine.remove_component::<luminara_scene::Name>(entity);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            "DESPAWN_ENTITY" => {
                                // Undo despawn = Spawn it back. Requires full recreation which isn't easy here, skip for now.
                                log::warn!("Undo for despawn not fully implemented.");
                            }
                            "SPAWN_ENTITY" => {
                                let entity_str = inverse.get("entity").and_then(|v| v.as_str());
                                if let (Some(entity_str), Some(engine)) = (entity_str, &this_model.engine_handle) {
                                    if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                        if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                            let entity = luminara_core::Entity::from_raw(id, gen);
                                            let _ = engine.despawn_entity(entity);
                                        }
                                    }
                                }
                            }
                            "DUPLICATE_ENTITY" => {
                                // We don't have the duplicate ID easily, but we could find it.
                                log::warn!("Undo duplicate not fully implemented.");
                            }
                            _ => {}
                        }
                        cx.notify();
                    });
                    
                    // 3. Mark the command as undone in DB history
                    if let Some(id) = cmd_id {
                        let _ = db.execute_query(&format!("UPDATE {} SET is_undone = true", id)).await;
                    }
                    
                    // 4. Update the session record with the reverted state
                    let session_record = this.update(&mut cx, |this_model, _cx| {
                        luminara_db::schema::EditorSessionRecord {
                            id: None,
                            name: this_model.session.name.clone(),
                            active_workspace: this_model.session.active_workspace as usize,
                            global_search_visible: this_model.session.global_search_visible,
                            layout_config: this_model.session.layout_config.clone(),
                            selected_entities: this_model.session.selected_entities.clone(),
                            active_tool: this_model.session.active_tool.clone(),
                            active_bottom_tab: this_model.session.active_bottom_tab.clone(),
                            editor_mode: this_model.session.editor_mode.clone(),
                            last_updated: chrono::Utc::now().timestamp(),
                        }
                    }).ok();

                    if let Some(session_record) = session_record {
                        let _ = db.store_session(session_record).await;
                    }
                }
            }
            
            let _ = this.update(&mut cx, |this_model, cx| {
                this_model.is_syncing = false;
                cx.notify();
            });
        }).detach();
    }
    

    /// Redo the last undone action
    pub fn redo(&mut self, cx: &mut ModelContext<Self>) {
        let Some(db) = self.db.clone() else { return };
        let session_id = luminara_db::RecordId::from(("editor_session", self.session.name.clone()));
        
        self.is_syncing = true;
        cx.notify();

        cx.spawn(|this, mut cx| async move {
            let query = format!("SELECT * FROM ui_command WHERE session_id = {} AND is_undone = true ORDER BY timestamp ASC LIMIT 1", session_id);
            if let Ok(mut result) = db.execute_query(&query).await {
                if let Ok(mut cmds) = result.take::<Vec<luminara_db::schema::UiCommandRecord>>(0) {
                    if let Some(cmd) = cmds.pop() {
                        let action = cmd.action.clone();
                        let payload = cmd.payload.clone();
                        let cmd_id = cmd.id.clone();
                        
                        // Apply normal payload to UI state
                        let _ = this.update(&mut cx, |this_model, cx| {
                            match action.as_str() {
                                "TOGGLE_SEARCH" => {
                                    if let Some(v) = payload.get("visible").and_then(|v| v.as_bool()) {
                                        this_model.session.global_search_visible = v;
                                    }
                                }
                                "SWITCH_WORKSPACE" => {
                                    if let Some(t) = payload.get("to").and_then(|t| t.as_u64()) {
                                        this_model.session.active_workspace = match t {
                                            0 => WorkspaceType::SceneBuilder,
                                            1 => WorkspaceType::LogicGraph,
                                            _ => WorkspaceType::SceneBuilder,
                                        };
                                    }
                                }
                                "SELECT_ENTITIES" => {
                                    if let Some(entities) = payload.get("entities").and_then(|v| v.as_array()) {
                                        this_model.session.selected_entities = entities.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect();
                                    }
                                }
                                "SET_TOOL" => {
                                    if let Some(tool) = payload.get("tool").and_then(|v| v.as_str()) {
                                        this_model.session.active_tool = tool.to_string();
                                    }
                                }
                                "SET_BOTTOM_TAB" => {
                                    if let Some(tab) = payload.get("tab").and_then(|v| v.as_str()) {
                                        this_model.session.active_bottom_tab = tab.to_string();
                                    }
                                }
                                "SET_EDITOR_MODE" => {
                                    if let Some(mode) = payload.get("mode").and_then(|v| v.as_str()) {
                                        this_model.session.editor_mode = mode.to_string();
                                    }
                                }
                                "UPDATE_COMPONENT" | "RENAME_ENTITY" => {
                                    let entity_str = payload.get("entity").and_then(|v| v.as_str());
                                    let component = payload.get("component").and_then(|v| v.as_str());
                                    let value = payload.get("value");
                                    
                                    if let (Some(entity_str), Some(component), Some(value), Some(engine)) = (entity_str, component, value, &this_model.engine_handle) {
                                        if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                            if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                                let entity = luminara_core::Entity::from_raw(id, gen);
                                                match component {
                                                    "Transform" => {
                                                        if let Ok(transform) = serde_json::from_value::<luminara_math::Transform>(value.clone()) {
                                                            let _ = engine.update_component(entity, transform);
                                                        }
                                                    }
                                                    "Name" => {
                                                        if let Some(name) = value.as_str() {
                                                            let _ = engine.update_component(entity, luminara_scene::Name::new(name));
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                "ADD_COMPONENT" => {
                                    let entity_str = payload.get("entity").and_then(|v| v.as_str());
                                    let component = payload.get("component").and_then(|v| v.as_str());
                                    
                                    if let (Some(entity_str), Some(component), Some(engine)) = (entity_str, component, &this_model.engine_handle) {
                                        if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                            if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                                let entity = luminara_core::Entity::from_raw(id, gen);
                                                match component {
                                                    "Transform" => {
                                                        let _ = engine.update_component(entity, luminara_math::Transform::IDENTITY);
                                                    }
                                                    "Name" => {
                                                        let _ = engine.update_component(entity, luminara_scene::Name::new("New Entity"));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                "DESPAWN_ENTITY" => {
                                    let entity_str = payload.get("entity").and_then(|v| v.as_str());
                                    if let (Some(entity_str), Some(engine)) = (entity_str, &this_model.engine_handle) {
                                        if let Some((id_part, gen_part)) = entity_str.split_once(':') {
                                            if let (Ok(id), Ok(gen)) = (id_part.parse::<u32>(), gen_part.parse::<u32>()) {
                                                let entity = luminara_core::Entity::from_raw(id, gen);
                                                let _ = engine.despawn_entity(entity);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                            cx.notify();
                        });
                        
                        // Mark the command as valid (not undone) in DB history
                        if let Some(id) = cmd_id {
                            let _ = db.execute_query(&format!("UPDATE ui_command SET is_undone = false WHERE id = {}", id)).await;
                        }
                        
                        let session_record = this.update(&mut cx, |this_model, _cx| {
                            luminara_db::schema::EditorSessionRecord {
                                id: None,
                                name: this_model.session.name.clone(),
                                active_workspace: this_model.session.active_workspace as usize,
                                global_search_visible: this_model.session.global_search_visible,
                                layout_config: this_model.session.layout_config.clone(),
                                selected_entities: this_model.session.selected_entities.clone(),
                                active_tool: this_model.session.active_tool.clone(),
                                active_bottom_tab: this_model.session.active_bottom_tab.clone(),
                                editor_mode: this_model.session.editor_mode.clone(),
                                last_updated: chrono::Utc::now().timestamp(),
                            }
                        }).ok();

                        if let Some(session_record) = session_record {
                            let _ = db.store_session(session_record).await;
                        }
                    }
                }
            }
            
            let _ = this.update(&mut cx, |this_model, cx| {
                this_model.is_syncing = false;
                cx.notify();
            });
        }).detach();
    }

    // --- Database Synchronization ---

    /// Sync the current session state to the database in the background.
    pub fn sync_to_db(&mut self, cx: &mut ModelContext<Self>, action: &str, payload: serde_json::Value, inverse_payload: serde_json::Value) {
        let Some(db) = self.db.clone() else {
            log::warn!("Attempted to sync state to DB, but DB is not initialized.");
            return;
        };

        let session_record = luminara_db::schema::EditorSessionRecord {
            id: None, // Will update existing if we query it first, or let DB handle it. For now, create new or update 'default'
            name: self.session.name.clone(),
            active_workspace: self.session.active_workspace as usize,
            global_search_visible: self.session.global_search_visible,
            layout_config: self.session.layout_config.clone(),
            selected_entities: self.session.selected_entities.clone(),
            active_tool: self.session.active_tool.clone(),
            active_bottom_tab: self.session.active_bottom_tab.clone(),
            editor_mode: self.session.editor_mode.clone(),
            last_updated: chrono::Utc::now().timestamp(),
        };
        
        let command_session_id: luminara_db::RecordId = luminara_db::RecordId::from(("editor_session", self.session.name.clone()));

        let command_record = luminara_db::schema::UiCommandRecord::new(
            command_session_id.clone(),
            action,
            payload,
            inverse_payload,
            chrono::Utc::now().timestamp(),
        );

        self.is_syncing = true;
        cx.notify();

        cx.spawn(|this, mut cx| async move {
            // 1. Store the session state
            let session_res = db.store_session(session_record).await;
            if let Err(e) = session_res {
                log::error!("Failed to sync EditorSession to DB: {}", e);
            }

            // Clear any undone commands since we are adding a new action
            let _ = db.delete_undone_commands(&command_session_id).await;

            // 2. Store the command for Undo/Redo
            let cmd_res = db.store_ui_command(command_record).await;
            if let Err(e) = cmd_res {
                log::error!("Failed to store UiCommand to DB: {}", e);
            }

            // 3. Mark sync as complete
            let _ = this.update(&mut cx, |this_model, cx| {
                this_model.is_syncing = false;
                cx.notify();
            });
        }).detach();
    }
}
