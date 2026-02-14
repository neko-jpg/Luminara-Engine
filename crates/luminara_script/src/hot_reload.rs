use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, Config, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashMap;
use crate::{ScriptId, ScriptError};
use std::time::{Instant, Duration};

pub struct HotReloadSystem {
    watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    path_map: HashMap<PathBuf, ScriptId>,
    // Debounce tracking
    last_event: HashMap<PathBuf, Instant>,
}

impl HotReloadSystem {
    pub fn new() -> Result<Self, ScriptError> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| ScriptError::Runtime(format!("Failed to create watcher: {}", e)))?;

        Ok(Self {
            watcher,
            rx,
            path_map: HashMap::new(),
            last_event: HashMap::new(),
        })
    }

    pub fn watch(&mut self, path: &Path, script_id: ScriptId) -> Result<(), ScriptError> {
        self.watcher.watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| ScriptError::Runtime(format!("Failed to watch path {:?}: {}", path, e)))?;

        self.path_map.insert(path.to_path_buf(), script_id);
        Ok(())
    }

    pub fn process_events(&mut self) -> Vec<ScriptId> {
        let mut reloads = Vec::new();

        // Non-blocking iterator
        while let Ok(res) = self.rx.try_recv() {
            match res {
                Ok(event) => {
                    // Check if modification
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if let Some(&id) = self.path_map.get(&path) {
                                // Debounce: ignore if within 100ms of last event
                                let now = Instant::now();
                                if let Some(&last) = self.last_event.get(&path) {
                                    if now.duration_since(last) < Duration::from_millis(100) {
                                        continue;
                                    }
                                }
                                self.last_event.insert(path.clone(), now);

                                if !reloads.contains(&id) {
                                    reloads.push(id);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {:?}", e);
                }
            }
        }

        reloads
    }
}
