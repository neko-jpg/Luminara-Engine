use crate::AssetServer;
use crossbeam_channel::Receiver;
use luminara_core::shared_types::{ResMut, Resource};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<Event>,
    pending_reloads: HashMap<PathBuf, Instant>,
    debounce_duration: Duration,
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
            pending_reloads: HashMap::new(),
            debounce_duration: Duration::from_millis(50), // 50ms debounce
        })
    }

    pub fn receiver(&self) -> &Receiver<Event> {
        &self.rx
    }

    /// Poll events with debouncing to avoid redundant reloads
    /// Returns paths that are ready to be reloaded
    pub fn poll_events(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        let mut ready_paths = Vec::new();

        // Collect new events
        while let Ok(event) = self.rx.try_recv() {
            if event.kind.is_modify() {
                for path in event.paths {
                    self.pending_reloads.insert(path, now);
                }
            }
        }

        // Process debounced events
        self.pending_reloads.retain(|path, timestamp| {
            if now.duration_since(*timestamp) >= self.debounce_duration {
                ready_paths.push(path.clone());
                false // Remove from pending
            } else {
                true // Keep pending
            }
        });

        ready_paths
    }
}

impl Resource for HotReloadWatcher {}

pub fn asset_hot_reload_system(server: ResMut<AssetServer>, mut watcher: ResMut<HotReloadWatcher>) {
    // Use debounced polling instead of direct event processing
    let ready_paths = watcher.poll_events();
    
    for path in ready_paths {
        server.reload(&path);
    }
}
