//! Editor application root (Vizia version)
//!
//! The EditorApp is the root Vizia application that manages the editor lifecycle,
//! initializes the Vizia runtime with GPU acceleration, and integrates with Luminara Engine.

use crate::rendering::RenderingServer;
use crate::services::engine_bridge::EngineHandle;
use parking_lot::RwLock;
use std::sync::Arc;
use vizia::prelude::*;

pub struct EditorApp {
    engine: Arc<EngineHandle>,
    rendering_server: Option<Arc<RwLock<RenderingServer>>>,
}

impl EditorApp {
    pub fn new(engine: Arc<EngineHandle>) -> Self {
        Self {
            engine,
            rendering_server: None,
        }
    }

    pub fn engine(&self) -> &Arc<EngineHandle> {
        &self.engine
    }

    pub fn init_rendering_server(&mut self, gpu_context: &luminara_render::GpuContext) {
        if self.rendering_server.is_none() {
            let server = RenderingServer::new(gpu_context);
            self.rendering_server = Some(Arc::new(RwLock::new(server)));
            log::info!("RenderingServer initialized for editor");
        }
    }

    pub fn rendering_server(&self) -> Option<Arc<RwLock<RenderingServer>>> {
        self.rendering_server.clone()
    }

    pub fn run(self) {
        log::info!("Starting Vizia application");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_app_creation() {
        let engine = Arc::new(EngineHandle::mock());
        let app = EditorApp::new(engine.clone());
        assert!(Arc::ptr_eq(&app.engine, &engine));
    }
}
