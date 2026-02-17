//! Database schema definitions for entities, components, assets, and operations

use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

/// Entity record stored in the database
///
/// Represents a game entity with its metadata and relationships.
/// Uses Record Links to represent parent-child hierarchies and component associations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    /// Unique identifier (optional for creation, assigned by database)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Optional entity name
    pub name: Option<String>,

    /// Tags for categorization and querying
    #[serde(default)]
    pub tags: Vec<String>,

    /// Links to component records
    #[serde(default)]
    pub components: Vec<RecordId>,

    /// Link to parent entity (for hierarchy)
    pub parent: Option<RecordId>,

    /// Links to child entities
    #[serde(default)]
    pub children: Vec<RecordId>,
}

/// Component record stored in the database
///
/// Represents a component attached to an entity.
/// The data field contains the serialized component data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRecord {
    /// Unique identifier (optional for creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Component type name (e.g., "Transform", "Mesh", "Material")
    pub type_name: String,

    /// Type ID for reflection
    pub type_id: String,

    /// Serialized component data
    pub data: serde_json::Value,

    /// Link to the entity this component belongs to
    pub entity: RecordId,
}

/// Asset record stored in the database
///
/// Represents an asset with its metadata and dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    /// Unique identifier (optional for creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Asset file path
    pub path: String,

    /// Asset type (e.g., "Texture", "Mesh", "Material", "Scene")
    pub asset_type: String,

    /// Content hash for change detection
    pub hash: String,

    /// Links to assets this asset depends on
    #[serde(default)]
    pub dependencies: Vec<RecordId>,

    /// Metadata (format-specific information)
    pub metadata: AssetMetadata,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// File size in bytes
    pub size_bytes: u64,

    /// Last modified timestamp (Unix timestamp)
    pub modified_timestamp: i64,

    /// Format-specific metadata
    #[serde(default)]
    pub custom: serde_json::Value,
}

/// Operation record for undo/redo timeline
///
/// Stores operations with their inverse commands for persistent undo/redo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Unique identifier (optional for creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Operation type (e.g., "SpawnEntity", "ModifyComponent")
    pub operation_type: String,

    /// Human-readable description
    pub description: String,

    /// Forward commands (serialized)
    pub commands: Vec<serde_json::Value>,

    /// Inverse commands for undo (serialized)
    pub inverse_commands: Vec<serde_json::Value>,

    /// Links to affected entities
    #[serde(default)]
    pub affected_entities: Vec<RecordId>,

    /// Timestamp (Unix timestamp)
    pub timestamp: i64,

    /// Parent operation (for operation grouping)
    pub parent: Option<RecordId>,

    /// Branch name (for Git-like workflow)
    pub branch: Option<String>,

    /// AI intent that generated this operation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
}

impl EntityRecord {
    /// Create a new entity record
    pub fn new(name: Option<String>) -> Self {
        Self {
            id: None,
            name,
            tags: Vec::new(),
            components: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a tag to the entity
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the entity
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }
}

impl ComponentRecord {
    /// Create a new component record
    pub fn new(
        type_name: impl Into<String>,
        type_id: impl Into<String>,
        data: serde_json::Value,
        entity: RecordId,
    ) -> Self {
        Self {
            id: None,
            type_name: type_name.into(),
            type_id: type_id.into(),
            data,
            entity,
        }
    }
}

impl AssetRecord {
    /// Create a new asset record
    pub fn new(
        path: impl Into<String>,
        asset_type: impl Into<String>,
        hash: impl Into<String>,
        metadata: AssetMetadata,
    ) -> Self {
        Self {
            id: None,
            path: path.into(),
            asset_type: asset_type.into(),
            hash: hash.into(),
            dependencies: Vec::new(),
            metadata,
        }
    }

    /// Add a dependency to the asset
    pub fn with_dependency(mut self, dependency: RecordId) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

impl OperationRecord {
    /// Create a new operation record
    pub fn new(
        operation_type: impl Into<String>,
        description: impl Into<String>,
        commands: Vec<serde_json::Value>,
        inverse_commands: Vec<serde_json::Value>,
        timestamp: i64,
    ) -> Self {
        Self {
            id: None,
            operation_type: operation_type.into(),
            description: description.into(),
            commands,
            inverse_commands,
            affected_entities: Vec::new(),
            timestamp,
            parent: None,
            branch: None,
            intent: None,
        }
    }

    /// Add an affected entity
    pub fn with_affected_entity(mut self, entity: RecordId) -> Self {
        self.affected_entities.push(entity);
        self
    }

    /// Set the branch name
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Set the AI intent
    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }
}
