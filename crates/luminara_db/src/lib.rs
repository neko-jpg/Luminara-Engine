//! Luminara Database - SurrealDB embedded mode integration
//!
//! This crate provides graph-based asset management and persistent undo/redo
//! using SurrealDB's embedded mode. It enables complex graph queries on assets,
//! entity relationships, and operation history.
//!
//! # Features
//!
//! - Embedded SurrealDB (no external server required)
//! - Graph-based entity and asset relationships using Record Links
//! - Complex queries with SurrealQL
//! - Persistent undo/redo operation timeline
//! - WASM support with IndexedDB backend
//!
//! # WASM Support
//!
//! When compiled for `wasm32` target with the `wasm` feature, the database
//! uses IndexedDB as the storage backend. See the [WASM documentation](docs/wasm_indexeddb_support.md)
//! for details.
//!
//! ```toml
//! [dependencies]
//! luminara_db = { version = "0.1", features = ["wasm"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use luminara_db::{LuminaraDatabase, EntityRecord};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize embedded database
//! let db = LuminaraDatabase::new("./data/luminara.db").await?;
//!
//! // Store an entity
//! let entity_id = db.store_entity(EntityRecord {
//!     id: None,
//!     name: Some("Player".to_string()),
//!     tags: vec!["player".to_string()],
//!     components: vec![],
//!     parent: None,
//!     children: vec![],
//! }).await?;
//!
//! // Query entities
//! let entities = db.query_entities("SELECT * FROM entity WHERE 'player' IN tags").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # WASM Example
//!
//! ```no_run
//! # #[cfg(target_arch = "wasm32")]
//! # use luminara_db::{LuminaraDatabase, EntityRecord};
//! # #[cfg(target_arch = "wasm32")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize with IndexedDB backend (WASM only)
//! let db = LuminaraDatabase::new_indexeddb("my_game_db").await?;
//!
//! // Use the same API as native backends
//! let entity = EntityRecord::new(Some("Player".to_string()));
//! let entity_id = db.store_entity(entity).await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod schema;
pub mod database;
pub mod query;
pub mod sync;
pub mod timeline;
pub mod migration;

pub use error::{DbError, DbResult};
pub use schema::{EntityRecord, ComponentRecord, AssetRecord, OperationRecord};
pub use database::{LuminaraDatabase, EntityHierarchy, EntityWithRelationships};
pub use query::QueryBuilder;
pub use sync::{WorldSync, SyncStatistics, SyncResult};
pub use timeline::{OperationTimeline, BranchInfo, TimelineStatistics};
pub use migration::{RonMigrationTool, MigrationStatistics};

// Re-export RecordId from surrealdb
pub use surrealdb::RecordId;
