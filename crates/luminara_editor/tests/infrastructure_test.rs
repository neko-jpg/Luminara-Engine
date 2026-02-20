//! Infrastructure tests for GPUI Editor
//!
//! These tests verify the basic infrastructure setup without requiring
//! a full GPUI runtime.

use luminara_editor::{EditorApp, EngineHandle};
use std::sync::Arc;

#[test]
fn test_engine_handle_mock_creation() {
    // Verify we can create a mock engine handle for testing
    let engine = EngineHandle::mock();
    
    // Verify all subsystems are accessible
    let _world = engine.world();
    let _asset_server = engine.asset_server();
    let _database = engine.database();
}

#[test]
fn test_editor_app_creation() {
    // Create a mock engine handle
    let engine = Arc::new(EngineHandle::mock());
    
    // Create the editor app
    let app = EditorApp::new(engine.clone());
    
    // Verify the engine handle is stored correctly
    assert!(Arc::ptr_eq(app.engine(), &engine));
    assert!(app.window().is_none());
}

#[test]
fn test_engine_handle_world_read_write() {
    let engine = EngineHandle::mock();
    
    // Test read access
    {
        let world = engine.world();
        // World should be accessible
        let _entities = world.entities();
    }
    
    // Test write access
    {
        let mut world = engine.world_mut();
        // Should be able to spawn entities
        let _entity = world.spawn();
    }
    
    // Verify the entity was created
    {
        let world = engine.world();
        assert!(world.entities().len() > 0);
    }
}

#[test]
fn test_engine_handle_arc_sharing() {
    let engine = Arc::new(EngineHandle::mock());
    let engine_clone = engine.clone();
    
    // Verify Arc sharing works correctly
    assert!(Arc::ptr_eq(&engine, &engine_clone));
    assert_eq!(Arc::strong_count(&engine), 2);
}
