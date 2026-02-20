//! Engine integration layer
//!
//! The EngineHandle provides a bridge between the GPUI UI and Luminara Engine,
//! exposing safe interfaces for ECS, Asset System, Database, and Render Pipeline access.

use luminara_core::World;
use luminara_asset::AssetServer;
use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use std::collections::VecDeque;

// Temporary mock Database until luminara_db compilation issues are resolved
pub struct Database;

impl Database {
    pub fn memory() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Database)
    }
}

// Temporary mock RenderPipeline
pub struct RenderPipeline;

impl RenderPipeline {
    pub fn mock() -> Self {
        RenderPipeline
    }
}

/// Command queue for editor operations
///
/// Commands are queued and executed in order to ensure consistency
/// and enable undo/redo functionality.
pub struct CommandQueue {
    commands: VecDeque<Box<dyn EditorCommand>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    pub fn push(&mut self, command: Box<dyn EditorCommand>) {
        self.commands.push_back(command);
    }

    pub fn pop(&mut self) -> Option<Box<dyn EditorCommand>> {
        self.commands.pop_front()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for editor commands
pub trait EditorCommand: Send + Sync {
    fn execute(&mut self, world: &mut World);
    fn name(&self) -> &str;
}

/// Event bus for communication between UI and engine
///
/// Events are published by the engine and consumed by the UI
/// to update the display in response to engine state changes.
pub struct EventBus {
    listeners: Vec<Box<dyn Fn(&Event) + Send + Sync>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(handler));
    }

    pub fn publish(&self, event: &Event) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Events that can be published by the engine
#[derive(Debug, Clone)]
pub enum Event {
    EntitySpawned { entity: luminara_core::Entity },
    EntityDespawned { entity: luminara_core::Entity },
    ComponentAdded { entity: luminara_core::Entity, component_name: String },
    ComponentRemoved { entity: luminara_core::Entity, component_name: String },
    AssetLoaded { asset_path: String },
    AssetFailed { asset_path: String, error: String },
}

/// Bridge between GPUI UI and Luminara Engine
///
/// This struct provides thread-safe access to all major engine subsystems,
/// allowing the UI to query and modify game state safely.
pub struct EngineHandle {
    /// ECS World for entity and component management
    world: Arc<RwLock<World>>,
    /// Asset loading and management
    asset_server: Arc<AssetServer>,
    /// Database for persistence and queries
    database: Arc<Database>,
    /// Render pipeline for 3D viewport
    render_pipeline: Arc<RwLock<RenderPipeline>>,
    /// Command queue for editor operations
    command_queue: Arc<Mutex<CommandQueue>>,
    /// Event bus for UI-engine communication
    event_bus: Arc<Mutex<EventBus>>,
}

impl EngineHandle {
    /// Create a new EngineHandle
    ///
    /// # Arguments
    /// * `world` - Arc-wrapped RwLock to the ECS World
    /// * `asset_server` - Arc-wrapped AssetServer
    /// * `database` - Arc-wrapped Database
    /// * `render_pipeline` - Arc-wrapped RwLock to the RenderPipeline
    ///
    /// # Requirements
    /// - Requirement 12.1: ECS Integration
    /// - Requirement 12.2: Asset System Integration
    /// - Requirement 12.3: Database Integration
    /// - Requirement 12.4: Render Pipeline Integration
    pub fn new(
        world: Arc<RwLock<World>>,
        asset_server: Arc<AssetServer>,
        database: Arc<Database>,
        render_pipeline: Arc<RwLock<RenderPipeline>>,
    ) -> Self {
        Self {
            world,
            asset_server,
            database,
            render_pipeline,
            command_queue: Arc::new(Mutex::new(CommandQueue::new())),
            event_bus: Arc::new(Mutex::new(EventBus::new())),
        }
    }

    /// Get read access to the ECS World
    ///
    /// # Requirements
    /// - Requirement 12.1.1: Query entities from ECS
    pub fn world(&self) -> parking_lot::RwLockReadGuard<'_, World> {
        self.world.read()
    }

    /// Get write access to the ECS World
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update components in ECS
    /// - Requirement 12.1.3: Spawn entities in ECS
    pub fn world_mut(&self) -> parking_lot::RwLockWriteGuard<'_, World> {
        self.world.write()
    }

    /// Get a reference to the AssetServer
    ///
    /// # Requirements
    /// - Requirement 12.2.1: Use AssetServer for asset loading
    pub fn asset_server(&self) -> &Arc<AssetServer> {
        &self.asset_server
    }

    /// Get a reference to the Database
    ///
    /// # Requirements
    /// - Requirement 12.3.1: Use SurrealDB for data persistence
    /// - Requirement 12.3.2: Serialize scenes to database
    pub fn database(&self) -> &Arc<Database> {
        &self.database
    }

    /// Get read access to the RenderPipeline
    ///
    /// # Requirements
    /// - Requirement 12.4: Render Pipeline Integration
    pub fn render_pipeline(&self) -> parking_lot::RwLockReadGuard<'_, RenderPipeline> {
        self.render_pipeline.read()
    }

    /// Get write access to the RenderPipeline
    pub fn render_pipeline_mut(&self) -> parking_lot::RwLockWriteGuard<'_, RenderPipeline> {
        self.render_pipeline.write()
    }

    /// Get a reference to the command queue
    ///
    /// # Requirements
    /// - Requirement 12.1: ECS Integration - command queue for editor operations
    pub fn command_queue(&self) -> &Arc<Mutex<CommandQueue>> {
        &self.command_queue
    }

    /// Get a reference to the event bus
    ///
    /// # Requirements
    /// - Requirement 12.1: ECS Integration - event bus for UI-engine communication
    pub fn event_bus(&self) -> &Arc<Mutex<EventBus>> {
        &self.event_bus
    }

    /// Execute a command through the command queue
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update components in ECS
    pub fn execute_command(&self, mut command: Box<dyn EditorCommand>) {
        let mut world = self.world_mut();
        command.execute(&mut world);
        
        // Optionally queue for undo/redo
        self.command_queue.lock().push(command);
    }

    /// Subscribe to engine events
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update UI when ECS changes
    pub fn subscribe_events<F>(&self, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.event_bus.lock().subscribe(handler);
    }

    /// Publish an event to all subscribers
    pub fn publish_event(&self, event: Event) {
        self.event_bus.lock().publish(&event);
    }

    // ===== ECS Integration Methods =====

    /// Query entity data by ID
    ///
    /// Returns entity information if it exists in the World.
    ///
    /// # Requirements
    /// - Requirement 12.1.1: Query entities from ECS
    pub fn query_entity(&self, entity: luminara_core::Entity) -> Option<EntityData> {
        
        use luminara_math::Transform;
        use luminara_scene::{Parent, Children};
        
        let world = self.world();
        let entities = world.entities();
        
        // Check if entity exists
        if !entities.contains(&entity) {
            return None;
        }

        let mut components = Vec::new();
        
        // Query Transform component
        if let Some(transform) = world.get_component::<Transform>(entity) {
            if let Ok(json) = serde_json::to_value(transform) {
                components.push(ComponentData {
                    type_name: "luminara_math::Transform".to_string(),
                    data: json,
                });
            }
        }
        
        // Query Parent component
        if let Some(parent) = world.get_component::<Parent>(entity) {
            let parent_json = serde_json::json!({
                "parent": format!("{:?}", parent.0)
            });
            components.push(ComponentData {
                type_name: "luminara_scene::Parent".to_string(),
                data: parent_json,
            });
        }
        
        // Query Children component
        if let Some(children) = world.get_component::<Children>(entity) {
            let children_json = serde_json::json!({
                "count": children.0.len(),
                "children": children.0.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>()
            });
            components.push(ComponentData {
                type_name: "luminara_scene::Children".to_string(),
                data: children_json,
            });
        }

        Some(EntityData {
            entity,
            components,
        })
    }

    /// Update a component on an entity
    ///
    /// This method updates an existing component or adds it if it doesn't exist.
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update components in ECS
    pub fn update_component<C: luminara_core::Component>(&self, entity: luminara_core::Entity, component: C) -> Result<(), String> {
        let mut world = self.world_mut();
        
        // Check if entity exists
        let entities = world.entities();
        if !entities.contains(&entity) {
            return Err(format!("Entity {:?} does not exist", entity));
        }

        // Add the component using add_component
        world.add_component(entity, component)
            .map_err(|e| format!("Failed to update component: {}", e))?;

        // Publish event
        drop(world);
        self.publish_event(Event::ComponentAdded {
            entity,
            component_name: std::any::type_name::<C>().to_string(),
        });

        Ok(())
    }

    /// Spawn a new entity
    ///
    /// # Requirements
    /// - Requirement 12.1.3: Spawn entities in ECS
    pub fn spawn_entity(&self) -> luminara_core::Entity {
        let mut world = self.world_mut();
        let entity = world.spawn();
        
        // Publish event
        drop(world);
        self.publish_event(Event::EntitySpawned { entity });
        
        entity
    }

    /// Spawn an entity with a bundle of components
    ///
    /// # Requirements
    /// - Requirement 12.1.3: Spawn entities in ECS
    pub fn spawn_entity_with<B>(&self, bundle: B) -> Result<luminara_core::Entity, String>
    where
        B: luminara_core::Bundle,
    {
        let mut world = self.world_mut();
        let entity = world.spawn_bundle(bundle)
            .map_err(|e| format!("Failed to spawn entity: {}", e))?;
        
        // Publish event
        drop(world);
        self.publish_event(Event::EntitySpawned { entity });
        
        Ok(entity)
    }

    /// Despawn an entity
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update ECS state
    pub fn despawn_entity(&self, entity: luminara_core::Entity) -> Result<(), String> {
        let mut world = self.world_mut();
        
        let entities = world.entities();
        if !entities.contains(&entity) {
            return Err(format!("Entity {:?} does not exist", entity));
        }

        let success = world.despawn(entity);
        if !success {
            return Err(format!("Failed to despawn entity {:?}", entity));
        }
        
        // Publish event
        drop(world);
        self.publish_event(Event::EntityDespawned { entity });
        
        Ok(())
    }

    /// Remove a component from an entity
    ///
    /// # Requirements
    /// - Requirement 12.1.2: Update components in ECS
    pub fn remove_component<C: luminara_core::Component>(&self, entity: luminara_core::Entity) -> Result<(), String> {
        let mut world = self.world_mut();
        
        let entities = world.entities();
        if !entities.contains(&entity) {
            return Err(format!("Entity {:?} does not exist", entity));
        }

        world.remove_component::<C>(entity)
            .map_err(|e| format!("Failed to remove component: {}", e))?;
        
        // Publish event
        drop(world);
        self.publish_event(Event::ComponentRemoved {
            entity,
            component_name: std::any::type_name::<C>().to_string(),
        });
        
        Ok(())
    }

    // ===== Asset System Integration Methods =====

    /// Load an asset asynchronously
    ///
    /// Returns a handle to the asset that can be used to retrieve it once loaded.
    ///
    /// # Requirements
    /// - Requirement 12.2.1: Use AssetServer for asset loading
    /// - Requirement 12.2.7: Async loading without blocking UI
    pub fn load_asset<T: luminara_asset::Asset>(&self, path: &str) -> luminara_asset::Handle<T> {
        let handle = self.asset_server.load(path);
        
        // Publish event when loading starts
        self.publish_event(Event::AssetLoaded {
            asset_path: path.to_string(),
        });
        
        handle
    }

    /// Load an asset with priority
    ///
    /// # Requirements
    /// - Requirement 12.2.1: Use AssetServer for asset loading
    pub fn load_asset_with_priority<T: luminara_asset::Asset>(
        &self,
        path: &str,
        priority: luminara_asset::LoadPriority,
    ) -> luminara_asset::Handle<T> {
        let handle = self.asset_server.load_with_priority(path, priority);
        
        self.publish_event(Event::AssetLoaded {
            asset_path: path.to_string(),
        });
        
        handle
    }

    /// Get an asset if it's loaded
    ///
    /// Returns None if the asset is not yet loaded or doesn't exist.
    ///
    /// # Requirements
    /// - Requirement 12.2.1: Use AssetServer for asset loading
    pub fn get_asset<T: luminara_asset::Asset>(&self, handle: &luminara_asset::Handle<T>) -> Option<Arc<T>> {
        self.asset_server.get(handle)
    }

    /// Check the load state of an asset
    ///
    /// # Requirements
    /// - Requirement 12.2.3: Display asset loading progress
    pub fn asset_load_state(&self, id: luminara_asset::AssetId) -> luminara_asset::LoadState {
        self.asset_server.load_state(id)
    }

    /// Get overall asset loading progress
    ///
    /// # Requirements
    /// - Requirement 12.2.3: Display asset loading progress
    pub fn asset_load_progress(&self) -> luminara_asset::LoadProgress {
        self.asset_server.load_progress()
    }

    /// Add an asset directly to the asset server
    ///
    /// This is useful for runtime-generated assets.
    ///
    /// # Requirements
    /// - Requirement 12.2.1: Use AssetServer for asset loading
    pub fn add_asset<T: luminara_asset::Asset>(&self, asset: T) -> luminara_asset::Handle<T> {
        self.asset_server.add(asset)
    }

    // ===== Database Integration Methods =====

    /// Query data from the database
    ///
    /// This is a placeholder implementation until luminara_db is fully integrated.
    ///
    /// # Requirements
    /// - Requirement 12.3.1: Use SurrealDB for data persistence
    /// - Requirement 12.3.3: Support real-time queries
    pub fn query_database(&self, _query: &str) -> Result<Vec<serde_json::Value>, String> {
        // Placeholder implementation
        // In a real implementation, this would query the SurrealDB database
        Ok(Vec::new())
    }

    /// Save data to the database
    ///
    /// This is a placeholder implementation until luminara_db is fully integrated.
    ///
    /// # Requirements
    /// - Requirement 12.3.2: Serialize scenes to database
    pub fn save_to_database(&self, _table: &str, _data: serde_json::Value) -> Result<(), String> {
        // Placeholder implementation
        // In a real implementation, this would save to SurrealDB
        Ok(())
    }

    /// Delete data from the database
    ///
    /// # Requirements
    /// - Requirement 12.3.1: Use SurrealDB for data persistence
    pub fn delete_from_database(&self, _table: &str, _id: &str) -> Result<(), String> {
        // Placeholder implementation
        Ok(())
    }

    /// Update data in the database with optimistic UI updates
    ///
    /// # Requirements
    /// - Requirement 12.3.5: Implement optimistic UI updates with DB sync
    pub fn update_database_optimistic(&self, _table: &str, _id: &str, _data: serde_json::Value) -> Result<(), String> {
        // Placeholder implementation
        // In a real implementation, this would:
        // 1. Update the UI immediately (optimistic)
        // 2. Queue the database update
        // 3. Rollback if the update fails
        Ok(())
    }
}

/// Entity data returned by queries
#[derive(Debug, Clone)]
pub struct EntityData {
    pub entity: luminara_core::Entity,
    pub components: Vec<ComponentData>,
}

/// Component data for serialization
#[derive(Debug, Clone)]
pub struct ComponentData {
    pub type_name: String,
    pub data: serde_json::Value,
}

impl EngineHandle {
    /// Create a mock EngineHandle for testing
    ///
    /// This creates a minimal EngineHandle with mock subsystems for testing purposes.
    /// The mock handle provides access to all subsystems but with minimal functionality.
    pub fn mock() -> Self {
        use luminara_core::App;
        use std::path::PathBuf;
        
        // Create a minimal app for testing
        let app = App::new();
        let world = Arc::new(RwLock::new(app.world));
        
        // Create mock subsystems
        let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));
        let database = Arc::new(Database::memory().expect("Failed to create memory database"));
        let render_pipeline = Arc::new(RwLock::new(RenderPipeline::mock()));
        
        Self::new(world, asset_server, database, render_pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_handle_creation() {
        let handle = EngineHandle::mock();
        
        // Verify we can access all subsystems
        let _world = handle.world();
        let _asset_server = handle.asset_server();
        let _database = handle.database();
    }

    #[test]
    fn test_engine_handle_world_access() {
        let handle = EngineHandle::mock();
        
        // Test read access
        {
            let world = handle.world();
            // World should be accessible - just verify we can get entities
            let _entities = world.entities();
        }
        
        // Test write access
        {
            let mut world = handle.world_mut();
            // Should be able to spawn entities
            let _entity = world.spawn();
        }
    }
}
