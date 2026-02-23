//! Editor state manager (Vizia version)

use crate::core::session::{EditorSession, WorkspaceType};
use luminara_db::LuminaraDatabase;
use std::sync::Arc;

#[derive(Clone)]
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

    pub fn set_database(&mut self, db: Arc<LuminaraDatabase>) {
        self.db = Some(db);
    }

    pub fn set_engine_handle(&mut self, handle: Arc<crate::services::engine_bridge::EngineHandle>) {
        self.engine_handle = Some(handle);
    }

    pub fn set_session(&mut self, session: EditorSession) {
        self.session = session;
    }

    pub fn workspace_type(&self) -> WorkspaceType {
        self.session.active_workspace
    }

    pub fn set_workspace_type(&mut self, workspace_type: WorkspaceType) {
        self.session.active_workspace = workspace_type;
    }

    pub fn database(&self) -> Option<&Arc<LuminaraDatabase>> {
        self.db.as_ref()
    }

    pub fn engine(&self) -> Option<&Arc<crate::services::engine_bridge::EngineHandle>> {
        self.engine_handle.as_ref()
    }
}

impl Default for EditorStateManager {
    fn default() -> Self {
        Self::new(EditorSession::default(), None)
    }
}
