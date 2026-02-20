//! Editor application root
//!
//! The EditorApp is the root GPUI application that manages the editor lifecycle,
//! initializes the GPUI runtime with GPU acceleration, and integrates with Luminara Engine.

use crate::services::engine_bridge::EngineHandle;
use crate::core::window::EditorWindow;
use gpui::WindowHandle;
use std::sync::Arc;

/// The root GPUI application for the Luminara Editor
pub struct EditorApp {
    /// Handle to the Luminara Engine
    engine: Arc<EngineHandle>,
    /// Handle to the main editor window
    window: Option<WindowHandle<EditorWindow>>,
}

impl EditorApp {
    /// Create a new EditorApp with the given engine handle
    ///
    /// # Arguments
    /// * `engine` - Arc-wrapped handle to the Luminara Engine
    ///
    /// # Returns
    /// A new EditorApp instance
    pub fn new(engine: Arc<EngineHandle>) -> Self {
        Self {
            engine,
            window: None,
        }
    }

    /// Get a reference to the engine handle
    pub fn engine(&self) -> &Arc<EngineHandle> {
        &self.engine
    }

    /// Get a reference to the main window handle
    pub fn window(&self) -> Option<&WindowHandle<EditorWindow>> {
        self.window.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_app_creation() {
        // Create a mock engine handle
        let engine = Arc::new(EngineHandle::mock());
        
        // Create the editor app
        let app = EditorApp::new(engine.clone());
        
        // Verify the engine handle is stored correctly
        assert!(Arc::ptr_eq(&app.engine, &engine));
        assert!(app.window.is_none());
    }
}
