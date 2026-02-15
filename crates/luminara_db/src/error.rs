use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Scene not found: {0}")]
    SceneNotFound(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Schema migration failed: {0}")]
    MigrationFailed(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Channel send error")]
    ChannelSend,

    #[error("Channel receive error")]
    ChannelRecv,
}
