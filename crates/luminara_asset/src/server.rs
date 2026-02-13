use crate::{Asset, AssetId, AssetLoadError, AssetLoader, Handle, HandleAllocator};
use luminara_core::shared_types::Resource;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
}

pub struct AssetServer {
    asset_dir: PathBuf,
    handle_allocator: HandleAllocator,
    loaders: RwLock<HashMap<String, Arc<dyn ErasedAssetLoader>>>,
    load_states: RwLock<HashMap<AssetId, LoadState>>,
    assets: RwLock<HashMap<AssetId, Arc<dyn Any + Send + Sync>>>,
}

impl AssetServer {
    pub fn new(asset_dir: impl Into<PathBuf>) -> Self {
        Self {
            asset_dir: asset_dir.into(),
            handle_allocator: HandleAllocator::new(),
            loaders: RwLock::new(HashMap::new()),
            load_states: RwLock::new(HashMap::new()),
            assets: RwLock::new(HashMap::new()),
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
            return Handle::new(self.handle_allocator.id_for_path(path));
        }

        let id = self.handle_allocator.id_for_path(path);

        {
            let states = self.load_states.read().unwrap();
            if let Some(state) = states.get(&id) {
                if *state == LoadState::Loaded {
                    return Handle::new(id);
                }
            }
        }

        self.load_states
            .write()
            .unwrap()
            .insert(id, LoadState::Loading);

        let full_path = self.asset_dir.join(path);
        match self.load_internal_erased(&full_path) {
            Ok(asset_arc) => {
                self.assets.write().unwrap().insert(id, asset_arc);
                self.load_states
                    .write()
                    .unwrap()
                    .insert(id, LoadState::Loaded);
            }
            Err(e) => {
                self.load_states
                    .write()
                    .unwrap()
                    .insert(id, LoadState::Failed(e.to_string()));
            }
        }

        Handle::new(id)
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
                        self.assets.write().unwrap().insert(id, asset_arc);
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
        assets.get(&handle.id()).and_then(|arc| {
            arc.clone()
                .downcast::<T>()
                .map_err(|_| "Downcast failed")
                .ok()
        })
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
