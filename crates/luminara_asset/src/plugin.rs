use crate::{asset_hot_reload_system, AssetServer, HotReloadWatcher};
use luminara_core::shared_types::{App, AppInterface, CoreStage, Plugin, ResMut};
use luminara_core::system::FunctionMarker;
use std::path::PathBuf;

pub struct AssetPlugin {
    pub asset_dir: PathBuf,
}

impl Default for AssetPlugin {
    fn default() -> Self {
        Self {
            asset_dir: PathBuf::from("assets"),
        }
    }
}

impl Plugin for AssetPlugin {
    fn name(&self) -> &str {
        "AssetPlugin"
    }

    fn build(&self, app: &mut App) {
        let server = AssetServer::new(&self.asset_dir);

        // Setup hot reload watcher if possible
        if let Ok(watcher) = HotReloadWatcher::new(self.asset_dir.clone()) {
            app.insert_resource(watcher);
        }

        app.insert_resource(server).add_system::<(
            FunctionMarker,
            ResMut<'static, AssetServer>,
            ResMut<'static, HotReloadWatcher>,
        )>(CoreStage::PreUpdate, asset_hot_reload_system);
    }
}
