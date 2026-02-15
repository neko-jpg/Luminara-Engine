use luminara_core::*;
use tokio::sync::{mpsc, oneshot};
use crate::{connection::DbConnection, stores::asset_store::AssetStore, stores::scene_store::SceneStore, stores::undo_store::UndoStore, models::scene::*, models::asset_meta::*, models::undo_meta::*, error::DbError};
use std::sync::Mutex;

pub enum DbCommand {
    SaveScene {
        snapshot: SceneSnapshot,
        callback: oneshot::Sender<Result<(), DbError>>,
    },
    LoadScene {
        scene_name: String,
        callback: oneshot::Sender<Result<SceneSnapshot, DbError>>,
    },
    RawQuery {
        surql: String,
        callback: oneshot::Sender<Result<serde_json::Value, DbError>>,
    },
    RegisterAsset {
        meta: AssetMeta,
        callback: oneshot::Sender<Result<(), DbError>>,
    },
    RecordUndo {
        entry: UndoEntry,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct DbCommandSender {
    pub tx: mpsc::UnboundedSender<DbCommand>,
}

impl Resource for DbCommandSender {}

impl DbCommandSender {
    pub fn save_scene(&self, snapshot: SceneSnapshot) -> oneshot::Receiver<Result<(), DbError>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(DbCommand::SaveScene { snapshot, callback: tx });
        rx
    }

    pub fn load_scene(&self, scene_name: String) -> oneshot::Receiver<Result<SceneSnapshot, DbError>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(DbCommand::LoadScene { scene_name, callback: tx });
        rx
    }

    pub fn query(&self, surql: String) -> oneshot::Receiver<Result<serde_json::Value, DbError>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(DbCommand::RawQuery { surql, callback: tx });
        rx
    }

    pub fn record_undo(&self, entry: UndoEntry) {
        let _ = self.tx.send(DbCommand::RecordUndo { entry });
    }
}

#[derive(Default)]
pub struct DbResults {
    pub pending_scene_loads: Mutex<Vec<(String, oneshot::Receiver<Result<SceneSnapshot, DbError>>)>>,
}

impl Resource for DbResults {}

pub async fn db_worker(
    conn: DbConnection,
    mut rx: mpsc::UnboundedReceiver<DbCommand>,
) {
    tracing::info!("DB worker started");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            DbCommand::SaveScene { snapshot, callback } => {
                let store = SceneStore::new(&conn);
                let result = store.save_scene(&snapshot).await;
                let _ = callback.send(result);
            }
            DbCommand::LoadScene { scene_name, callback } => {
                let store = SceneStore::new(&conn);
                let result = store.load_scene(&scene_name).await;
                let _ = callback.send(result);
            }
            DbCommand::RawQuery { surql, callback } => {
                let result = conn.query(&surql).await.and_then(|mut resp| {
                    let val: surrealdb::Value = resp.take(0)
                        .map_err(|e| DbError::QueryError(e.to_string()))?;
                    let json_val = serde_json::to_value(val)?;
                    Ok(json_val)
                });
                let _ = callback.send(result);
            }
            DbCommand::RegisterAsset { meta, callback } => {
                let store = AssetStore::new(&conn);
                let result = store.register(&meta).await.map(|_| ());
                let _ = callback.send(result);
            }
            DbCommand::RecordUndo { entry } => {
                let store = UndoStore::new(&conn, 1000);
                if let Err(e) = store.push(&entry).await {
                    tracing::error!("Failed to record undo: {}", e);
                }
            }
            DbCommand::Shutdown => {
                tracing::info!("DB worker shutting down");
                conn.shutdown().await.ok();
                break;
            }
        }
    }
}
