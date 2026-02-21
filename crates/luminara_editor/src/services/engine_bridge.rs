//! Engine integration layer
//!
//! The EngineHandle provides a bridge between the GPUI UI and Bevy Engine,
//! exposing safe interfaces for ECS via commands, Asset System, Database, and Render Pipeline access.

use crate::services::bevy_bridge::BevyBridge;
use parking_lot::RwLock;
use std::sync::Arc;
use luminara_asset::AssetServer;
pub use luminara_db::LuminaraDatabase as Database;

// Re-export Bevy types needed for commands
pub use bevy::prelude::World;

#[derive(Debug, Clone)]
pub struct PreviewBillboard {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub depth: f32,
    pub selected: bool,
}

pub trait EditorCommand: Send + Sync {
    fn execute(&mut self, world: &mut World);
}

pub struct EngineHandle {
    bridge: Arc<BevyBridge>,
    asset_server: Arc<AssetServer>,
    database: Arc<Database>,
    last_frame: Arc<RwLock<Option<Vec<u8>>>>,
}

impl EngineHandle {
    pub fn new(
        bridge: Arc<BevyBridge>,
        asset_server: Arc<AssetServer>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            bridge,
            asset_server,
            database,
            last_frame: Arc::new(RwLock::new(None)),
        }
    }

    /// Execute a command on the Bevy world
    pub fn execute_command<F>(&self, command: F)
    where
        F: FnOnce(&mut World) + Send + Sync + 'static,
    {
        let _ = self.bridge.command_sender.send(Box::new(command));
    }

    /// Poll for new rendered frame from Bevy
    pub fn poll_image_event(&self) -> Option<Vec<u8>> {
        // Drain channel to get the latest frame
        let mut latest = None;
        while let Ok(frame) = self.bridge.image_receiver.try_recv() {
            latest = Some(frame);
        }
        
        if let Some(frame) = latest {
            *self.last_frame.write() = Some(frame.clone());
            Some(frame)
        } else {
            None
        }
    }

    /// Get the last received frame
    pub fn get_last_frame(&self) -> Option<Vec<u8>> {
        self.last_frame.read().clone()
    }

    pub fn asset_server(&self) -> &Arc<AssetServer> {
        &self.asset_server
    }

    pub fn database(&self) -> &Arc<Database> {
        &self.database
    }

    // Mock for testing
    pub fn mock() -> Self {
        // Create a dummy bridge
        let (bridge, _) = BevyBridge::new();
        // AssetServer and Database mocks
        let asset_server = Arc::new(AssetServer::new(std::path::PathBuf::from("assets")));
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let database = Arc::new(rt.block_on(Database::new_memory()).expect("Failed to create database"));
        
        Self::new(Arc::new(bridge), asset_server, database)
    }

    // Legacy stubs for compatibility (returning empty/dummy values)

    pub fn project_scene_billboards(
        &self,
        _camera_pos: luminara_math::Vec3,
        _camera_target: luminara_math::Vec3,
        _camera_up: luminara_math::Vec3,
        _camera_fov_deg: f32,
        _viewport_width: f32,
        _viewport_height: f32,
        _selected_entities: &std::collections::HashSet<String>,
    ) -> Vec<PreviewBillboard> {
        Vec::new()
    }
}
