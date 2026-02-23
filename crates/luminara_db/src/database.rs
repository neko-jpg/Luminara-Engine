//! Core database implementation with CRUD operations

use crate::error::{DbError, DbResult};
use crate::schema::{AssetRecord, ComponentRecord, EditorSessionRecord, EntityRecord, OperationRecord, UiCommandRecord};
use surrealdb::{RecordId, Surreal};

#[cfg(feature = "memory")]
use surrealdb::engine::local::{Db, Mem};

#[cfg(target_arch = "wasm32")]
use surrealdb::engine::local::IndxDb;

/// Main database interface for Luminara Engine
///
/// Provides embedded SurrealDB with CRUD operations for entities, components,
/// assets, and operations. Supports graph queries via SurrealQL.
#[derive(Clone)]
pub struct LuminaraDatabase {
    /// Embedded SurrealDB instance
    #[cfg(feature = "memory")]
    db: Surreal<Db>,
    #[cfg(not(feature = "memory"))]
    db: Surreal<surrealdb::engine::local::Db>, 
}

impl LuminaraDatabase {
    /// Initialize a new embedded database with in-memory backend
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = LuminaraDatabase::new_memory().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "memory")]
    pub async fn new_memory() -> DbResult<Self> {
        // Create database with in-memory backend
        let db: Surreal<Db> = Surreal::new::<Mem>(()).await?;

        // Use namespace and database
        db.use_ns("luminara").use_db("engine").await?;

        // Initialize schema
        Self::init_schema(&db).await?;

        Ok(Self { db })
    }

    /// Initialize a new embedded database with IndexedDB backend (WASM only)
    ///
    /// This method is only available when compiling for WASM target.
    /// It uses the browser's IndexedDB as the storage backend.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(target_arch = "wasm32")]
    /// # use luminara_db::LuminaraDatabase;
    /// # #[cfg(target_arch = "wasm32")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = LuminaraDatabase::new_indexeddb("luminara_db").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub async fn new_indexeddb(db_name: &str) -> DbResult<Self> {
        // Create database with IndexedDB backend
        let db = Surreal::new::<IndxDb>(db_name).await?;

        // Use namespace and database
        db.use_ns("luminara").use_db("engine").await?;

        // Initialize schema
        Self::init_schema(&db).await?;

        Ok(Self { db })
    }

    /// Initialize database schema
    async fn init_schema(db: &Surreal<Db>) -> DbResult<()> {
        // Define entity table
        db.query("DEFINE TABLE entity SCHEMALESS;").await?;

        // Define component table
        db.query("DEFINE TABLE component SCHEMALESS;").await?;

        // Define asset table
        db.query("DEFINE TABLE asset SCHEMALESS;").await?;

        // Define operation table
        db.query("DEFINE TABLE operation SCHEMALESS;").await?;

        // Define UI command table for editor undo/redo
        db.query("DEFINE TABLE ui_command SCHEMALESS;").await?;

        // Define editor session table
        db.query("DEFINE TABLE editor_session SCHEMALESS;").await?;

        Ok(())
    }

    // ==================== Entity Operations ====================

    /// Store an entity to the database
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::{LuminaraDatabase, EntityRecord};
    /// # async fn example(db: &LuminaraDatabase) -> Result<(), Box<dyn std::error::Error>> {
    /// let entity = EntityRecord::new(Some("Player".to_string()))
    ///     .with_tag("player");
    ///
    /// let entity_id = db.store_entity(entity).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_entity(&self, entity: EntityRecord) -> DbResult<RecordId> {
        let result: Option<EntityRecord> = self.db.create("entity").content(entity).await?;

        result
            .and_then(|e| e.id)
            .ok_or_else(|| DbError::Other("Failed to create entity".to_string()))
    }

    /// Load an entity from the database
    pub async fn load_entity(&self, id: &RecordId) -> DbResult<EntityRecord> {
        let entity: Option<EntityRecord> = self.db.select(id.clone()).await?;

        entity.ok_or_else(|| DbError::EntityNotFound(id.to_string()))
    }

    /// Load an entity with all its components
    ///
    /// Uses FETCH to load related component records in a single query.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, entity_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// let (entity, components) = db.load_entity_with_components(entity_id).await?;
    /// println!("Entity {} has {} components", entity.name.unwrap_or_default(), components.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_entity_with_components(
        &self,
        id: &RecordId,
    ) -> DbResult<(EntityRecord, Vec<ComponentRecord>)> {
        // Load entity
        let entity = self.load_entity(id).await?;

        // Load components by querying component table
        let query = format!("SELECT * FROM component WHERE entity = {}", id);
        let mut result = self.db.query(&query).await?;
        let components: Vec<ComponentRecord> = result.take(0)?;

        Ok((entity, components))
    }

    /// Load an entity with its full hierarchy (parent and children)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, entity_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// let hierarchy = db.load_entity_hierarchy(entity_id).await?;
    /// println!("Entity has {} children", hierarchy.children.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_entity_hierarchy(&self, id: &RecordId) -> DbResult<EntityHierarchy> {
        // Load the entity
        let entity = self.load_entity(id).await?;

        // Load parent if exists
        let parent = if let Some(parent_id) = &entity.parent {
            Some(Box::new(self.load_entity(parent_id).await?))
        } else {
            None
        };

        // Load children
        let mut children = Vec::new();
        for child_id in &entity.children {
            if let Ok(child) = self.load_entity(child_id).await {
                children.push(child);
            }
        }

        Ok(EntityHierarchy {
            entity,
            parent,
            children,
        })
    }

    /// Load an entity with all relationships (components, parent, children)
    ///
    /// This is the most comprehensive load operation, fetching all related data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, entity_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// let full_entity = db.load_entity_with_relationships(entity_id).await?;
    /// println!("Loaded entity with {} components and {} children",
    ///     full_entity.components.len(),
    ///     full_entity.hierarchy.children.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_entity_with_relationships(
        &self,
        id: &RecordId,
    ) -> DbResult<EntityWithRelationships> {
        // Load entity with components
        let (entity, components) = self.load_entity_with_components(id).await?;

        // Load parent if exists
        let parent = if let Some(parent_id) = &entity.parent {
            Some(Box::new(self.load_entity(parent_id).await?))
        } else {
            None
        };

        // Load children
        let mut children = Vec::new();
        for child_id in &entity.children {
            if let Ok(child) = self.load_entity(child_id).await {
                children.push(child);
            }
        }

        Ok(EntityWithRelationships {
            entity: entity.clone(),
            components,
            hierarchy: EntityHierarchy {
                entity,
                parent,
                children,
            },
        })
    }

    /// Find all entities with a specific component type
    ///
    /// Uses graph traversal to find entities that have components of the specified type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # async fn example(db: &LuminaraDatabase) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all entities with Transform component
    /// let entities = db.find_entities_with_component("Transform").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_entities_with_component(
        &self,
        component_type: &str,
    ) -> DbResult<Vec<EntityRecord>> {
        // First, find all components of the specified type
        let query = format!(
            "SELECT * FROM component WHERE type_name = '{}'",
            component_type
        );
        let mut result = self.db.query(&query).await?;
        let components: Vec<ComponentRecord> = result.take(0)?;

        // Then, load the entities for those components
        let mut entities = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for component in components {
            if seen_ids.insert(component.entity.clone()) {
                if let Ok(entity) = self.load_entity(&component.entity).await {
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    /// Find all descendants of an entity (recursive children)
    ///
    /// Uses graph traversal to find all entities in the hierarchy below this entity.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, entity_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// let descendants = db.find_entity_descendants(entity_id).await?;
    /// println!("Found {} descendants", descendants.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_entity_descendants(&self, id: &RecordId) -> DbResult<Vec<EntityRecord>> {
        self.find_entity_descendants_recursive(id).await
    }

    /// Internal recursive helper for finding descendants
    fn find_entity_descendants_recursive<'a>(
        &'a self,
        id: &'a RecordId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DbResult<Vec<EntityRecord>>> + Send + 'a>>
    {
        Box::pin(async move {
            // Load entity and recursively collect all descendants
            let entity = self.load_entity(id).await?;
            let mut descendants = Vec::new();

            // Collect immediate children
            for child_id in &entity.children {
                if let Ok(child) = self.load_entity(child_id).await {
                    descendants.push(child.clone());
                    // Recursively get descendants of this child
                    if let Ok(child_descendants) =
                        self.find_entity_descendants_recursive(child_id).await
                    {
                        descendants.extend(child_descendants);
                    }
                }
            }

            Ok(descendants)
        })
    }

    /// Find all ancestors of an entity (recursive parents)
    ///
    /// Uses graph traversal to find all entities in the hierarchy above this entity.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, entity_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// let ancestors = db.find_entity_ancestors(entity_id).await?;
    /// println!("Found {} ancestors", ancestors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_entity_ancestors(&self, id: &RecordId) -> DbResult<Vec<EntityRecord>> {
        self.find_entity_ancestors_recursive(id).await
    }

    /// Internal recursive helper for finding ancestors
    fn find_entity_ancestors_recursive<'a>(
        &'a self,
        id: &'a RecordId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DbResult<Vec<EntityRecord>>> + Send + 'a>>
    {
        Box::pin(async move {
            // Load entity and recursively collect all ancestors
            let entity = self.load_entity(id).await?;
            let mut ancestors = Vec::new();

            // Collect parent and its ancestors
            if let Some(parent_id) = &entity.parent {
                if let Ok(parent) = self.load_entity(parent_id).await {
                    ancestors.push(parent.clone());
                    // Recursively get ancestors of this parent
                    if let Ok(parent_ancestors) =
                        self.find_entity_ancestors_recursive(parent_id).await
                    {
                        ancestors.extend(parent_ancestors);
                    }
                }
            }

            Ok(ancestors)
        })
    }

    /// Update an entity in the database
    pub async fn update_entity(&self, id: &RecordId, entity: EntityRecord) -> DbResult<()> {
        let _: Option<EntityRecord> = self.db.update(id.clone()).content(entity).await?;
        Ok(())
    }

    /// Delete an entity from the database
    pub async fn delete_entity(&self, id: &RecordId) -> DbResult<()> {
        let _: Option<EntityRecord> = self.db.delete(id.clone()).await?;
        Ok(())
    }

    /// Query entities with SurrealQL
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # async fn example(db: &LuminaraDatabase) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all entities with "player" tag
    /// let entities = db.query_entities("SELECT * FROM entity WHERE 'player' IN tags").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_entities(&self, query: &str) -> DbResult<Vec<EntityRecord>> {
        let mut result = self.db.query(query).await?;
        let entities: Vec<EntityRecord> = result.take(0)?;
        Ok(entities)
    }

    // ==================== Component Operations ====================

    /// Store a component to the database
    pub async fn store_component(&self, component: ComponentRecord) -> DbResult<RecordId> {
        let result: Option<ComponentRecord> =
            self.db.create("component").content(component).await?;

        result
            .and_then(|c| c.id)
            .ok_or_else(|| DbError::Other("Failed to create component".to_string()))
    }

    /// Load a component from the database
    pub async fn load_component(&self, id: &RecordId) -> DbResult<ComponentRecord> {
        let component: Option<ComponentRecord> = self.db.select(id.clone()).await?;

        component.ok_or_else(|| DbError::ComponentNotFound(id.to_string()))
    }

    /// Update a component in the database
    pub async fn update_component(
        &self,
        id: &RecordId,
        component: ComponentRecord,
    ) -> DbResult<()> {
        let _: Option<ComponentRecord> = self.db.update(id.clone()).content(component).await?;
        Ok(())
    }

    /// Delete a component from the database
    pub async fn delete_component(&self, id: &RecordId) -> DbResult<()> {
        let _: Option<ComponentRecord> = self.db.delete(id.clone()).await?;
        Ok(())
    }

    /// Query components with SurrealQL
    pub async fn query_components(&self, query: &str) -> DbResult<Vec<ComponentRecord>> {
        let mut result = self.db.query(query).await?;
        let components: Vec<ComponentRecord> = result.take(0)?;
        Ok(components)
    }

    // ==================== Asset Operations ====================

    /// Store an asset to the database
    pub async fn store_asset(&self, asset: AssetRecord) -> DbResult<RecordId> {
        let result: Option<AssetRecord> = self.db.create("asset").content(asset).await?;

        result
            .and_then(|a| a.id)
            .ok_or_else(|| DbError::Other("Failed to create asset".to_string()))
    }

    /// Load an asset from the database
    pub async fn load_asset(&self, id: &RecordId) -> DbResult<AssetRecord> {
        let asset: Option<AssetRecord> = self.db.select(id.clone()).await?;

        asset.ok_or_else(|| DbError::AssetNotFound(id.to_string()))
    }

    /// Load an asset by path
    pub async fn load_asset_by_path(&self, path: &str) -> DbResult<AssetRecord> {
        let query = format!("SELECT * FROM asset WHERE path = '{}'", path);
        let mut result = self.db.query(&query).await?;
        let assets: Vec<AssetRecord> = result.take(0)?;

        assets
            .into_iter()
            .next()
            .ok_or_else(|| DbError::AssetNotFound(path.to_string()))
    }

    /// Update an asset in the database
    pub async fn update_asset(&self, id: &RecordId, asset: AssetRecord) -> DbResult<()> {
        let _: Option<AssetRecord> = self.db.update(id.clone()).content(asset).await?;
        Ok(())
    }

    /// Delete an asset from the database
    pub async fn delete_asset(&self, id: &RecordId) -> DbResult<()> {
        let _: Option<AssetRecord> = self.db.delete(id.clone()).await?;
        Ok(())
    }

    /// Find direct asset dependencies (non-transitive)
    ///
    /// Returns only the immediate dependencies of the specified asset.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, asset_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find direct dependencies
    /// let dependencies = db.find_asset_dependencies(asset_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_asset_dependencies(&self, asset_id: &RecordId) -> DbResult<Vec<AssetRecord>> {
        // Load the asset
        let asset = self.load_asset(asset_id).await?;

        // Load all direct dependencies
        let mut dependencies = Vec::new();
        for dep_id in &asset.dependencies {
            if let Ok(dep) = self.load_asset(dep_id).await {
                dependencies.push(dep);
            }
        }

        Ok(dependencies)
    }

    /// Find transitive asset dependencies (recursive)
    ///
    /// Returns all dependencies of the specified asset, including dependencies
    /// of dependencies (transitive closure).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, asset_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all transitive dependencies
    /// let all_deps = db.find_asset_dependencies_transitive(asset_id).await?;
    /// println!("Asset has {} total dependencies", all_deps.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_asset_dependencies_transitive(
        &self,
        asset_id: &RecordId,
    ) -> DbResult<Vec<AssetRecord>> {
        let mut all_dependencies = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut to_visit = vec![asset_id.clone()];

        while let Some(current_id) = to_visit.pop() {
            // Skip if already visited
            if !visited.insert(current_id.clone()) {
                continue;
            }

            // Load asset and its dependencies
            if let Ok(asset) = self.load_asset(&current_id).await {
                // Add dependencies to visit queue
                for dep_id in &asset.dependencies {
                    if !visited.contains(dep_id) {
                        to_visit.push(dep_id.clone());

                        // Load and add to results
                        if let Ok(dep) = self.load_asset(dep_id).await {
                            all_dependencies.push(dep);
                        }
                    }
                }
            }
        }

        Ok(all_dependencies)
    }

    /// Find assets that depend on the specified asset (reverse dependencies)
    ///
    /// Returns all assets that have the specified asset as a dependency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, asset_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find what depends on this texture
    /// let dependents = db.find_asset_dependents(asset_id).await?;
    /// println!("{} assets depend on this texture", dependents.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_asset_dependents(&self, asset_id: &RecordId) -> DbResult<Vec<AssetRecord>> {
        // Query all assets that have this asset in their dependencies
        let query = format!("SELECT * FROM asset WHERE {} IN dependencies", asset_id);
        let mut result = self.db.query(&query).await?;
        let dependents: Vec<AssetRecord> = result.take(0)?;
        Ok(dependents)
    }

    /// Find all assets of a specific type used in a scene
    ///
    /// This performs a complex graph query to find all assets of a given type
    /// that are used by entities in the specified scene.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, scene_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all textures used in the scene
    /// let textures = db.find_assets_in_scene(scene_id, "Texture").await?;
    /// println!("Scene uses {} textures", textures.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_assets_in_scene(
        &self,
        scene_id: &RecordId,
        asset_type: &str,
    ) -> DbResult<Vec<AssetRecord>> {
        // Find all entities in the scene (descendants)
        let entities = self.find_entity_descendants(scene_id).await?;

        // Collect all unique assets of the specified type
        let mut assets = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // For each entity, find components that reference assets
        for entity in entities {
            if let Some(entity_id) = &entity.id {
                // Load components
                let query = format!("SELECT * FROM component WHERE entity = {}", entity_id);
                let mut result = self.db.query(&query).await?;
                let components: Vec<ComponentRecord> = result.take(0)?;

                // Extract asset references from component data
                for component in components {
                    if let Some(asset_refs) = self.extract_asset_references(&component.data) {
                        for asset_ref in asset_refs {
                            if seen_ids.insert(asset_ref.clone()) {
                                if let Ok(asset) = self.load_asset(&asset_ref).await {
                                    if asset.asset_type == asset_type {
                                        assets.push(asset);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(assets)
    }

    /// Find all textures used by materials in a scene
    ///
    /// This is a specialized query that finds materials in a scene and then
    /// finds all textures those materials depend on.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # use surrealdb::RecordId;
    /// # async fn example(db: &LuminaraDatabase, scene_id: &RecordId) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all textures used by materials in this scene
    /// let textures = db.find_textures_used_by_materials_in_scene(scene_id).await?;
    /// println!("Scene materials use {} textures", textures.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_textures_used_by_materials_in_scene(
        &self,
        scene_id: &RecordId,
    ) -> DbResult<Vec<AssetRecord>> {
        // Find all materials in the scene
        let materials = self.find_assets_in_scene(scene_id, "Material").await?;

        // For each material, find texture dependencies
        let mut textures = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for material in materials {
            if let Some(material_id) = &material.id {
                // Find dependencies of this material
                let deps = self.find_asset_dependencies(material_id).await?;

                // Filter for textures
                for dep in deps {
                    if dep.asset_type == "Texture" {
                        if let Some(dep_id) = &dep.id {
                            if seen_ids.insert(dep_id.clone()) {
                                textures.push(dep);
                            }
                        }
                    }
                }
            }
        }

        Ok(textures)
    }

    /// Query assets with SurrealQL
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::LuminaraDatabase;
    /// # async fn example(db: &LuminaraDatabase) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all texture assets
    /// let textures = db.query_assets("SELECT * FROM asset WHERE asset_type = 'Texture'").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_assets(&self, query: &str) -> DbResult<Vec<AssetRecord>> {
        let mut result = self.db.query(query).await?;
        let assets: Vec<AssetRecord> = result.take(0)?;
        Ok(assets)
    }

    /// Extract asset references from component data
    ///
    /// This helper function looks for RecordId references in component data
    /// that point to assets.
    fn extract_asset_references(&self, data: &serde_json::Value) -> Option<Vec<RecordId>> {
        let mut refs = Vec::new();

        // Recursively search for asset references in the JSON data
        self.extract_asset_references_recursive(data, &mut refs);

        if refs.is_empty() {
            None
        } else {
            Some(refs)
        }
    }

    /// Recursive helper for extracting asset references
    fn extract_asset_references_recursive(
        &self,
        value: &serde_json::Value,
        refs: &mut Vec<RecordId>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                // Check if this looks like an asset reference
                if let Some(id_val) = map.get("asset_id") {
                    if let Ok(record_id) = serde_json::from_value::<RecordId>(id_val.clone()) {
                        refs.push(record_id);
                    }
                }

                // Recursively check all values
                for val in map.values() {
                    self.extract_asset_references_recursive(val, refs);
                }
            }
            serde_json::Value::Array(arr) => {
                for val in arr {
                    self.extract_asset_references_recursive(val, refs);
                }
            }
            _ => {}
        }
    }

    // ==================== Operation Operations ====================

    /// Store an operation to the timeline
    pub async fn store_operation(&self, operation: OperationRecord) -> DbResult<RecordId> {
        let result: Option<OperationRecord> =
            self.db.create("operation").content(operation).await?;

        result
            .and_then(|o| o.id)
            .ok_or_else(|| DbError::Other("Failed to create operation".to_string()))
    }

    /// Load an operation from the database
    pub async fn load_operation(&self, id: &RecordId) -> DbResult<OperationRecord> {
        let operation: Option<OperationRecord> = self.db.select(id.clone()).await?;

        operation.ok_or_else(|| DbError::OperationNotFound(id.to_string()))
    }

    /// Load operation history (most recent first)
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of operations to retrieve
    /// * `branch` - Optional branch name filter
    pub async fn load_operation_history(
        &self,
        limit: usize,
        branch: Option<&str>,
    ) -> DbResult<Vec<OperationRecord>> {
        let query = if let Some(branch_name) = branch {
            format!(
                "SELECT * FROM operation WHERE branch = '{}' ORDER BY timestamp DESC, id DESC LIMIT {}",
                branch_name, limit
            )
        } else {
            format!(
                "SELECT * FROM operation ORDER BY timestamp DESC, id DESC LIMIT {}",
                limit
            )
        };

        let mut result = self.db.query(&query).await?;
        let operations: Vec<OperationRecord> = result.take(0)?;
        Ok(operations)
    }

    /// Delete an operation from the database
    pub async fn delete_operation(&self, id: &RecordId) -> DbResult<()> {
        let _: Option<OperationRecord> = self.db.delete(id.clone()).await?;
        Ok(())
    }

    // ==================== UI Command Operations ====================

    /// Store a UI command for undo/redo
    pub async fn store_ui_command(&self, command: UiCommandRecord) -> DbResult<RecordId> {
        let result: Option<UiCommandRecord> = self.db.create("ui_command").content(command).await?;

        result
            .and_then(|c| c.id)
            .ok_or_else(|| DbError::Other("Failed to create UI command".to_string()))
    }

    /// Load UI commands for a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to load commands for
    /// * `limit` - Maximum number of commands to retrieve
    /// * `undone` - If true, load undone commands; if false, load non-undone commands
    pub async fn load_ui_commands(
        &self,
        session_id: &str,
        limit: usize,
        undone: bool,
    ) -> DbResult<Vec<UiCommandRecord>> {
        let query = format!(
            "SELECT * FROM ui_command WHERE session_id = '{}' AND is_undone = {} ORDER BY timestamp DESC LIMIT {}",
            session_id, undone, limit
        );
        let mut result = self.db.query(&query).await?;
        let commands: Vec<UiCommandRecord> = result.take(0)?;
        Ok(commands)
    }

    /// Delete undone commands after a new command is executed (to maintain linear undo history)
    pub async fn delete_undone_commands(&self, session_id: &str) -> DbResult<()> {
        let query = format!(
            "DELETE FROM ui_command WHERE session_id = '{}' AND is_undone = true",
            session_id
        );
        self.db.query(&query).await?;
        Ok(())
    }

    // ==================== Editor Session Operations ====================

    /// Store an editor session
    pub async fn store_session(&self, session: EditorSessionRecord) -> DbResult<RecordId> {
        // Check if session with this name already exists
        let query = format!("SELECT * FROM editor_session WHERE name = '{}'", session.name);
        let mut result = self.db.query(&query).await?;
        let existing: Vec<EditorSessionRecord> = result.take(0)?;

        if let Some(existing_session) = existing.into_iter().next() {
            // Update existing session
            if let Some(id) = existing_session.id {
                let _: Option<EditorSessionRecord> = self.db.update(&id).content(session).await?;
                return Ok(id);
            }
        }

        // Create new session
        let result: Option<EditorSessionRecord> = self.db.create("editor_session").content(session).await?;
        result
            .and_then(|s| s.id)
            .ok_or_else(|| DbError::Other("Failed to create editor session".to_string()))
    }

    /// Load an editor session by name
    pub async fn load_session(&self, name: &str) -> DbResult<EditorSessionRecord> {
        let query = format!("SELECT * FROM editor_session WHERE name = '{}'", name);
        let mut result = self.db.query(&query).await?;
        let sessions: Vec<EditorSessionRecord> = result.take(0)?;

        sessions
            .into_iter()
            .next()
            .ok_or_else(|| DbError::Other(format!("Session '{}' not found", name)))
    }

    // ==================== Utility Operations ====================

    /// Execute a raw SurrealQL query
    pub async fn execute_query(&self, query: &str) -> DbResult<surrealdb::Response> {
        Ok(self.db.query(query).await?)
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> DbResult<DatabaseStatistics> {
        let entity_count: Vec<CountResult> = self
            .db
            .query("SELECT count() as count FROM entity GROUP ALL")
            .await?
            .take(0)?;

        let component_count: Vec<CountResult> = self
            .db
            .query("SELECT count() as count FROM component GROUP ALL")
            .await?
            .take(0)?;

        let asset_count: Vec<CountResult> = self
            .db
            .query("SELECT count() as count FROM asset GROUP ALL")
            .await?
            .take(0)?;

        let operation_count: Vec<CountResult> = self
            .db
            .query("SELECT count() as count FROM operation GROUP ALL")
            .await?
            .take(0)?;

        Ok(DatabaseStatistics {
            entity_count: entity_count.first().map(|r| r.count).unwrap_or(0),
            component_count: component_count.first().map(|r| r.count).unwrap_or(0),
            asset_count: asset_count.first().map(|r| r.count).unwrap_or(0),
            operation_count: operation_count.first().map(|r| r.count).unwrap_or(0),
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub entity_count: i64,
    pub component_count: i64,
    pub asset_count: i64,
    pub operation_count: i64,
}

#[derive(Debug, serde::Deserialize)]
struct CountResult {
    count: i64,
}

/// Entity hierarchy with parent and children
#[derive(Debug, Clone)]
pub struct EntityHierarchy {
    /// The entity itself
    pub entity: EntityRecord,
    /// Parent entity (if exists)
    pub parent: Option<Box<EntityRecord>>,
    /// Child entities
    pub children: Vec<EntityRecord>,
}

/// Entity with all relationships loaded
#[derive(Debug, Clone)]
pub struct EntityWithRelationships {
    /// The entity itself
    pub entity: EntityRecord,
    /// All components attached to this entity
    pub components: Vec<ComponentRecord>,
    /// Hierarchy information (parent and children)
    pub hierarchy: EntityHierarchy,
}
