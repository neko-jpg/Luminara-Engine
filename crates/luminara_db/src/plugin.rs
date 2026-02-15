use luminara_core::*;
use luminara_core::system::FunctionMarker;
use crate::{connection::*, config::DbConfig, sync::{commands::*, ComponentRegistry, Persistent, SaveExclude, DbDirty}};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Default)]
pub struct LuminaraDbPlugin {
    pub config: DbConfig,
}

impl Plugin for LuminaraDbPlugin {
    fn name(&self) -> &str {
        "LuminaraDbPlugin"
    }

    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        app
            .insert_resource(config)
            .insert_resource(ComponentRegistry::default())
            .register_component::<Persistent>()
            .register_component::<SaveExclude>()
            .register_component::<DbDirty>()
            .add_startup_system::<ExclusiveMarker>(db_init_system)
            .add_system::<(FunctionMarker, Res<DbResults>)>(CoreStage::PostUpdate, db_command_processor);
    }
}

pub fn db_init_system(
    world: &mut World,
) {
    let config = if let Some(cfg) = world.get_resource::<DbConfig>() {
        cfg.clone()
    } else {
        tracing::error!("DbConfig resource not found in db_init_system");
        return;
    };

    let (tx, rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async move {
            match DbConnection::connect(config).await {
                Ok(conn) => {
                    db_worker(conn, rx).await;
                }
                Err(e) => {
                    tracing::error!("Failed to connect to DB: {}", e);
                }
            }
        });
    });

    world.insert_resource(DbCommandSender { tx });
    world.insert_resource(DbResults::default());
}

pub fn db_command_processor(
    db_results: Res<DbResults>,
) {
    if let Ok(mut pending) = db_results.pending_scene_loads.lock() {
        pending.retain_mut(|(name, rx)| {
            match rx.try_recv() {
                Ok(Ok(_snapshot)) => {
                    tracing::info!("Scene '{}' loaded from DB", name);
                    // In a real implementation, we would emit an event or trigger restoration.
                    false
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to load scene '{}': {}", name, e);
                    false
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    true
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    tracing::error!("DB worker channel closed");
                    false
                }
            }
        });
    }
}
