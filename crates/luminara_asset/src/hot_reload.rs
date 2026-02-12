use crate::AssetServer;
use crossbeam_channel::Receiver;
use luminara_core::shared_types::{ResMut, Resource};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;

pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<Event>,
}

impl HotReloadWatcher {
    pub fn new(path: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    pub fn receiver(&self) -> &Receiver<Event> {
        &self.rx
    }
}

impl Resource for HotReloadWatcher {}

pub fn asset_hot_reload_system(server: ResMut<AssetServer>, watcher: ResMut<HotReloadWatcher>) {
    while let Ok(event) = watcher.receiver().try_recv() {
        if event.kind.is_modify() {
            for path in event.paths {
                server.reload(&path);
            }
        }
    }
}
