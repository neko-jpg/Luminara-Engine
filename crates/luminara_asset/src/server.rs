use crate::{Asset, AssetId, AssetLoadError, AssetLoader, Handle, HandleAllocator};
use luminara_core::shared_types::Resource;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
}

struct AssetEntry {
    asset: Arc<dyn Any + Send + Sync>,
    generation: u32,
}

pub struct AssetServer {
    asset_dir: PathBuf,
    handle_allocator: HandleAllocator,
    loaders: RwLock<HashMap<String, Arc<dyn ErasedAssetLoader>>>,
    load_states: RwLock<HashMap<AssetId, LoadState>>,
    assets: RwLock<HashMap<AssetId, AssetEntry>>,

    // Async loading
    load_request_tx: Sender<LoadRequest>,
    load_result_rx: Receiver<LoadResult>,
}

struct LoadRequest {
    path: PathBuf,
    id: AssetId,
    extension: String,
    loader: Arc<dyn ErasedAssetLoader>,
}

struct LoadResult {
    id: AssetId,
    result: Result<Arc<dyn Any + Send + Sync>, AssetLoadError>,
}

impl AssetServer {
    pub fn new(asset_dir: impl Into<PathBuf>) -> Self {
        let (load_request_tx, load_request_rx) = unbounded::<LoadRequest>();
        let (load_result_tx, load_result_rx) = unbounded::<LoadResult>();

        // Start a background loader thread
        std::thread::spawn(move || {
            for req in load_request_rx {
                let bytes = match std::fs::read(&req.path) {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = load_result_tx.send(LoadResult {
                            id: req.id,
                            result: Err(e.into()),
                        });
                        continue;
                    }
                };

                let result = req.loader.load(&bytes, &req.path);
                let _ = load_result_tx.send(LoadResult {
                    id: req.id,
                    result,
                });
            }
        });

        Self {
            asset_dir: asset_dir.into(),
            handle_allocator: HandleAllocator::new(),
            loaders: RwLock::new(HashMap::new()),
            load_states: RwLock::new(HashMap::new()),
            assets: RwLock::new(HashMap::new()),
            load_request_tx,
            load_result_rx,
        }
    }

    pub fn asset_dir(&self) -> &Path {
        &self.asset_dir
    }

    /// Get a reference to the handle allocator
    pub fn handle_allocator(&self) -> &HandleAllocator {
        &self.handle_allocator
    }

    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        // Path validation to prevent path traversal
        let path_obj = Path::new(path);
        if path.contains("..") || path_obj.is_absolute() {
            log::error!("Invalid asset path: {}", path);
            return Handle::new(self.handle_allocator.id_for_path(path), 0);
        }

        let id = self.handle_allocator.id_for_path(path);

        {
            let states = self.load_states.read().unwrap();
            let assets = self.assets.read().unwrap();

            if let Some(entry) = assets.get(&id) {
                // If loaded, return handle with current generation
                return Handle::new(id, entry.generation);
            }

            if let Some(state) = states.get(&id) {
                if *state == LoadState::Loading {
                    // Already loading, return generic handle (generation 0 or predict?)
                    // We assume generation 0 for new loads
                    return Handle::new(id, 0);
                }
            }
        }

        // Start loading
        self.load_states
            .write()
            .unwrap()
            .insert(id, LoadState::Loading);

        let full_path = self.asset_dir.join(path);

        // Find loader
        let extension = path_obj
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let loader = {
            let loaders = self.loaders.read().unwrap();
            loaders.get(&extension).cloned()
        };

        if let Some(loader) = loader {
            // Send to async loader
            let _ = self.load_request_tx.send(LoadRequest {
                path: full_path,
                id,
                extension,
                loader,
            });
        } else {
             self.load_states
                .write()
                .unwrap()
                .insert(id, LoadState::Failed(format!("No loader for extension {}", extension)));
        }

        Handle::new(id, 0)
    }

    // Process async results. Should be called every frame.
    pub fn update(&self) {
        while let Ok(result) = self.load_result_rx.try_recv() {
            match result.result {
                Ok(asset_arc) => {
                    let mut assets = self.assets.write().unwrap();
                    let current_gen = if let Some(entry) = assets.get(&result.id) {
                        entry.generation + 1
                    } else {
                        0
                    };

                    assets.insert(result.id, AssetEntry {
                        asset: asset_arc,
                        generation: current_gen,
                    });

                    self.load_states
                        .write()
                        .unwrap()
                        .insert(result.id, LoadState::Loaded);

                    log::info!("Loaded asset: {:?}", result.id);
                }
                Err(e) => {
                    log::error!("Failed to load asset {:?}: {}", result.id, e);
                    self.load_states
                        .write()
                        .unwrap()
                        .insert(result.id, LoadState::Failed(e.to_string()));
                }
            }
        }
    }

    fn load_internal_erased(
        &self,
        path: &Path,
    ) -> Result<Arc<dyn Any + Send + Sync>, AssetLoadError> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AssetLoadError::UnsupportedFormat("No extension".to_string()))?;

        let loader = {
            let loaders = self.loaders.read().unwrap();
            loaders.get(extension).cloned().ok_or_else(|| {
                AssetLoadError::UnsupportedFormat(format!("No loader for extension {}", extension))
            })?
        };

        let bytes = std::fs::read(path)?;
        loader.load(&bytes, path)
    }

    pub fn reload(&self, path: &Path) {
        // Assume path is absolute or relative to current dir,
        // we need to find its relative path to asset_dir to get the same AssetId.
        if let Ok(rel_path) = path.strip_prefix(&self.asset_dir) {
            if let Some(rel_path_str) = rel_path.to_str() {
                let id = self.handle_allocator.id_for_path(rel_path_str);

                match self.load_internal_erased(path) {
                    Ok(asset_arc) => {
                        let mut assets = self.assets.write().unwrap();
                        let current_gen = if let Some(entry) = assets.get(&id) {
                            entry.generation + 1
                        } else {
                            0
                        };

                        assets.insert(id, AssetEntry {
                            asset: asset_arc,
                            generation: current_gen,
                        });

                        self.load_states
                            .write()
                            .unwrap()
                            .insert(id, LoadState::Loaded);
                        log::info!("Reloaded asset: {:?}", rel_path);
                    }
                    Err(e) => {
                        log::error!("Failed to reload asset {:?}: {}", rel_path, e);
                        self.load_states
                            .write()
                            .unwrap()
                            .insert(id, LoadState::Failed(e.to_string()));
                    }
                }
            }
        }
    }

    pub fn load_state(&self, id: AssetId) -> LoadState {
        self.load_states
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .unwrap_or(LoadState::NotLoaded)
    }

    pub fn register_loader<L: AssetLoader>(&mut self, loader: L) {
        let erased = Arc::new(LoaderWrapper { loader });
        let mut loaders = self.loaders.write().unwrap();
        for ext in erased.extensions() {
            loaders.insert(ext.to_string(), erased.clone());
        }
    }

    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        let assets = self.assets.read().unwrap();
        assets.get(&handle.id()).and_then(|entry| {
            // Should we check generation?
            // If handle generation < entry generation, it's an old handle but we have newer data. Return newer data.
            // If handle generation > entry generation, we have old data (impossible unless handle comes from future).
            // If mismatch, we can log warning or return None if we strictly enforce versioning.
            // For now, return the asset.
            entry.asset.clone()
                .downcast::<T>()
                .map_err(|_| "Downcast failed")
                .ok()
        })
    }

    pub fn add<T: Asset>(&self, asset: T) -> Handle<T> {
        let id = AssetId::new();
        let mut assets = self.assets.write().unwrap();

        assets.insert(id, AssetEntry {
            asset: Arc::new(asset),
            generation: 0,
        });

        self.load_states
            .write()
            .unwrap()
            .insert(id, LoadState::Loaded);

        Handle::new(id, 0)
    }
}

impl Resource for AssetServer {}

trait ErasedAssetLoader: Send + Sync {
    fn extensions(&self) -> &[&str];
    fn load(&self, bytes: &[u8], path: &Path)
        -> Result<Arc<dyn Any + Send + Sync>, AssetLoadError>;
}

struct LoaderWrapper<L: AssetLoader> {
    loader: L,
}

impl<L: AssetLoader> ErasedAssetLoader for LoaderWrapper<L> {
    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }
    fn load(
        &self,
        bytes: &[u8],
        path: &Path,
    ) -> Result<Arc<dyn Any + Send + Sync>, AssetLoadError> {
        let asset = self.loader.load(bytes, path)?;
        Ok(Arc::new(asset))
    }
}
