//! Error types for the database

use thiserror::Error;

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;

/// Database error types
#[derive(Debug, Error)]
pub enum DbError {
    /// SurrealDB error
    #[error("Database error: {0}")]
    Surreal(#[from] surrealdb::Error),

    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Asset not found
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    /// Operation not found
    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Other errors
    #[error("{0}")]
    Other(String),
}
