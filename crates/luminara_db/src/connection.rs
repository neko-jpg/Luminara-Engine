use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;
use crate::config::{DbConfig, DbBackend};
use crate::error::DbError;
use crate::schema::SchemaManager;

#[cfg(feature = "memory")]
use surrealdb::engine::local::Mem;

#[cfg(feature = "rocksdb")]
use surrealdb::engine::local::RocksDb;

#[cfg(target_arch = "wasm32")]
use surrealdb::engine::local::IndxDb;

pub struct DbConnection {
    db: Surreal<Db>,
    config: DbConfig,
}

impl DbConnection {
    pub async fn connect(config: DbConfig) -> Result<Self, DbError> {
        let db = match config.backend {
            DbBackend::SurrealKV => {
                let db = Surreal::new::<SurrealKv>(config.data_path.clone()).await?;
                db
            }
            DbBackend::RocksDb => {
                #[cfg(feature = "rocksdb")]
                {
                    let db = Surreal::new::<RocksDb>(config.data_path.clone()).await?;
                    db
                }
                #[cfg(not(feature = "rocksdb"))]
                {
                    return Err(DbError::ConnectionError("RocksDb backend not enabled".into()));
                }
            }
            DbBackend::Memory => {
                #[cfg(feature = "memory")]
                {
                    let db = Surreal::new::<Mem>(()).await?;
                    db
                }
                #[cfg(not(feature = "memory"))]
                {
                    return Err(DbError::ConnectionError("Memory backend not enabled".into()));
                }
            }
            #[cfg(target_arch = "wasm32")]
            DbBackend::IndexedDb => {
                let db = Surreal::new::<IndxDb>("luminara_db").await?;
                db
            }
        };

        db.use_ns(&config.namespace).await?;
        db.use_db("project").await?;

        let connection = Self { db, config };

        if connection.config.auto_migrate {
             let schema_manager = SchemaManager::new(&connection);
             schema_manager.migrate().await?;
        }

        tracing::info!("SurrealDB embedded mode connected: {:?}", connection.config.data_path);
        Ok(connection)
    }

    pub async fn use_database(&self, db_name: &str) -> Result<(), DbError> {
        self.db.use_db(db_name).await?;
        Ok(())
    }

    pub fn inner(&self) -> &Surreal<Db> {
        &self.db
    }

    pub async fn query(&self, surql: &str) -> Result<surrealdb::Response, DbError> {
        let response = self.db.query(surql).await?;
        Ok(response)
    }

    pub async fn shutdown(&self) -> Result<(), DbError> {
        tracing::info!("Shutting down SurrealDB...");
        Ok(())
    }
}
