use crate::{Asset, AssetId, AssetLoadError, AssetLoader, Handle, HandleAllocator, PlaceholderRegistry};
use crossbeam_channel::{unbounded, Receiver, Sender};
use luminara_core::shared_types::Resource;
use std::any::{Any, TypeId};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

/// Priority level for asset loading
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    /// Lowest priority - background assets
    Low = 0,
    /// Normal priority - most assets
    Normal = 1,
    /// High priority - immediately visible assets
    High = 2,
    /// Critical priority - essential for gameplay
    Critical = 3,
}

impl Default for LoadPriority {
    fn default() -> Self {
        LoadPriority::Normal
    }
}

/// Configuration for retry logic with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        
        let delay_ms = (self.initial_delay.as_millis() as f32) 
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        let delay = Duration::from_millis(delay_ms as u64);
        
        delay.min(self.max_delay)
    }
}

/// Progress tracking for asset loading
#[derive(Debug, Clone)]
pub struct LoadProgress {
    /// Total number of assets to load
    pub total: usize,
    /// Number of assets loaded successfully
    pub loaded: usize,
    /// Number of assets currently loading
    pub loading: usize,
    /// Number of assets that failed to load
    pub failed: usize,
}

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
    loaders: Arc<RwLock<HashMap<String, Arc<dyn ErasedAssetLoader>>>>,
    load_states: Arc<RwLock<HashMap<AssetId, LoadState>>>,
    assets: Arc<RwLock<HashMap<AssetId, AssetEntry>>>,
    fallbacks: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    placeholders: Arc<PlaceholderRegistry>,

    // Async loading with tokio runtime
    load_request_tx: Sender<LoadRequest>,
    load_result_rx: Receiver<LoadResult>,
    runtime: Arc<Runtime>,
    
    // Thread pool configuration
    thread_count: usize,
    
    // Sequence counter for stable ordering
    sequence_counter: Arc<RwLock<u64>>,
    
    // Retry configuration
    retry_config: RetryConfig,
}

struct LoadRequest {
    path: PathBuf,
    id: AssetId,
    expected_type: TypeId,
    _extension: String,
    loader: Arc<dyn ErasedAssetLoader>,
    priority: LoadPriority,
    sequence: u64, // For stable ordering when priorities are equal
    retry_attempt: u32, // Current retry attempt (0 = first attempt)
}

// Implement ordering for priority queue (higher priority first)
impl Ord for LoadRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher priority first)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // If priorities are equal, use sequence number (lower sequence first = FIFO)
                other.sequence.cmp(&self.sequence)
            }
            other => other,
        }
    }
}

impl PartialOrd for LoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for LoadRequest {}

struct LoadResult {
    id: AssetId,
    expected_type: TypeId,
    result: Result<Arc<dyn Any + Send + Sync>, AssetLoadError>,
}

impl AssetServer {
    pub fn new(asset_dir: impl Into<PathBuf>) -> Self {
        Self::with_config(asset_dir, num_cpus::get().min(8), RetryConfig::default())
    }

    pub fn with_thread_count(asset_dir: impl Into<PathBuf>, thread_count: usize) -> Self {
        Self::with_config(asset_dir, thread_count, RetryConfig::default())
    }
    
    pub fn with_config(
        asset_dir: impl Into<PathBuf>,
        thread_count: usize,
        retry_config: RetryConfig,
    ) -> Self {
        let (load_request_tx, load_request_rx) = unbounded::<LoadRequest>();
        let (load_result_tx, load_result_rx) = unbounded::<LoadResult>();

        // Create tokio runtime for async I/O operations
        let thread_count = thread_count.max(1);
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_count)
            .thread_name("asset-loader")
            .enable_time()
            .build()
            .expect("Failed to create tokio runtime for asset loading");

        let runtime = Arc::new(runtime);

        // Clone retry config for the loader thread
        let retry_config_clone = retry_config.clone();
        let load_request_tx_clone = load_request_tx.clone();

        // Spawn a dedicated thread to manage priority queue and dispatch to async runtime
        let runtime_handle = runtime.handle().clone();
        std::thread::spawn(move || {
            let mut priority_queue = BinaryHeap::new();
            
            loop {
                // Try to receive new requests (non-blocking)
                while let Ok(req) = load_request_rx.try_recv() {
                    priority_queue.push(req);
                }
                
                // Dispatch requests from priority queue to tokio runtime
                if let Some(req) = priority_queue.pop() {
                    let result_tx = load_result_tx.clone();
                    let retry_config = retry_config_clone.clone();
                    let request_tx = load_request_tx_clone.clone();
                    
                    // Spawn each load operation as a tokio task
                    runtime_handle.spawn(async move {
                        // Apply retry delay if this is a retry attempt
                        if req.retry_attempt > 0 {
                            let delay = retry_config.delay_for_attempt(req.retry_attempt);
                            log::debug!(
                                "Retrying asset load (attempt {}/{}): {:?} after {:?}",
                                req.retry_attempt + 1,
                                retry_config.max_retries + 1,
                                req.path,
                                delay
                            );
                            tokio::time::sleep(delay).await;
                        }
                        
                        // Clone path for error messages
                        let path_for_error = req.path.clone();
                        
                        // Use tokio::fs for non-blocking I/O
                        let bytes = match tokio::fs::read(&req.path).await {
                            Ok(b) => b,
                            Err(e) => {
                                // Check if we should retry
                                if req.retry_attempt < retry_config.max_retries && is_transient_error(&e) {
                                    log::warn!(
                                        "Transient error loading asset {:?}: {}. Retrying...",
                                        path_for_error,
                                        e
                                    );
                                    
                                    // Re-queue with incremented retry count
                                    let retry_req = LoadRequest {
                                        retry_attempt: req.retry_attempt + 1,
                                        ..req
                                    };
                                    let _ = request_tx.send(retry_req);
                                    return;
                                }
                                
                                log::error!(
                                    "Failed to load asset {:?} after {} attempts: {}",
                                    path_for_error,
                                    req.retry_attempt + 1,
                                    e
                                );
                                
                                let _ = result_tx.send(LoadResult {
                                    id: req.id,
                                    expected_type: req.expected_type,
                                    result: Err(e.into()),
                                });
                                return;
                            }
                        };

                        // Clone what we need before moving into spawn_blocking
                        let loader = req.loader.clone();
                        let path = req.path.clone();
                        let path_for_error2 = req.path.clone();
                        let id = req.id;
                        let expected_type = req.expected_type;
                        let retry_attempt = req.retry_attempt;

                        // Asset parsing happens in background thread pool
                        let result = tokio::task::spawn_blocking(move || {
                            loader.load(&bytes, &path)
                        })
                        .await;

                        let load_result = match result {
                            Ok(Ok(asset)) => Ok(asset),
                            Ok(Err(e)) => {
                                // Check if we should retry on parse errors
                                if retry_attempt < retry_config.max_retries && is_transient_parse_error(&e) {
                                    log::warn!(
                                        "Transient parse error for asset {:?}: {}. Retrying...",
                                        path_for_error2,
                                        e
                                    );
                                    
                                    // Re-queue with incremented retry count
                                    let retry_req = LoadRequest {
                                        retry_attempt: retry_attempt + 1,
                                        path: path_for_error2,
                                        id,
                                        expected_type,
                                        _extension: req._extension,
                                        loader: req.loader,
                                        priority: req.priority,
                                        sequence: req.sequence,
                                    };
                                    let _ = request_tx.send(retry_req);
                                    return;
                                }
                                
                                Err(e)
                            }
                            Err(e) => Err(AssetLoadError::Other(format!("Task join error: {}", e))),
                        };

                        let _ = result_tx.send(LoadResult {
                            id,
                            expected_type,
                            result: load_result,
                        });
                    });
                } else if priority_queue.is_empty() {
                    // If queue is empty, wait for new request
                    if let Ok(req) = load_request_rx.recv() {
                        priority_queue.push(req);
                    } else {
                        // Channel closed, exit thread
                        break;
                    }
                } else {
                    // Small sleep to avoid busy-waiting when queue has items but we're at capacity
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }
            }
        });

        Self {
            asset_dir: asset_dir.into(),
            handle_allocator: HandleAllocator::new(),
            loaders: Arc::new(RwLock::new(HashMap::new())),
            load_states: Arc::new(RwLock::new(HashMap::new())),
            assets: Arc::new(RwLock::new(HashMap::new())),
            fallbacks: Arc::new(RwLock::new(HashMap::new())),
            placeholders: Arc::new(PlaceholderRegistry::new()),
            load_request_tx,
            load_result_rx,
            runtime,
            thread_count,
            sequence_counter: Arc::new(RwLock::new(0)),
            retry_config,
        }
    }

    pub fn asset_dir(&self) -> &Path {
        &self.asset_dir
    }

    /// Get the configured thread pool size
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    /// Get a reference to the handle allocator
    pub fn handle_allocator(&self) -> &HandleAllocator {
        &self.handle_allocator
    }

    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        self.load_with_priority(path, LoadPriority::Normal)
    }

    /// Load an asset with specified priority
    pub fn load_with_priority<T: Asset>(&self, path: &str, priority: LoadPriority) -> Handle<T> {
        // Path validation to prevent path traversal
        let path_obj = Path::new(path);
        let is_unsafe = path_obj
            .components()
            .any(|c| matches!(c, Component::ParentDir))
            || path_obj.is_absolute();

        if is_unsafe {
            log::error!("Invalid asset path (traversal detected): {}", path);
            // We should probably return a special "Invalid" handle or just the ID but it will never load.
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

        // Insert placeholder asset if available
        if let Some(placeholder) = self.placeholders.get::<T>() {
            let mut assets = self.assets.write().unwrap();
            assets.insert(
                id,
                AssetEntry {
                    asset: placeholder,
                    generation: 0,
                },
            );
            log::debug!("Inserted placeholder for asset: {:?}", id);
        }

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
            // Get next sequence number for stable ordering
            let sequence = {
                let mut counter = self.sequence_counter.write().unwrap();
                let seq = *counter;
                *counter = counter.wrapping_add(1);
                seq
            };
            
            // Send to async loader with priority
            let _ = self.load_request_tx.send(LoadRequest {
                path: full_path,
                id,
                expected_type: TypeId::of::<T>(),
                _extension: extension,
                loader,
                priority,
                sequence,
                retry_attempt: 0,
            });
        } else {
            self.load_states.write().unwrap().insert(
                id,
                LoadState::Failed(format!("No loader for extension {}", extension)),
            );
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
                        // Hot-swap: increment generation to signal update
                        entry.generation + 1
                    } else {
                        0
                    };

                    assets.insert(
                        result.id,
                        AssetEntry {
                            asset: asset_arc,
                            generation: current_gen,
                        },
                    );

                    self.load_states
                        .write()
                        .unwrap()
                        .insert(result.id, LoadState::Loaded);

                    if current_gen > 0 {
                        log::info!("Hot-swapped placeholder with real asset: {:?}", result.id);
                    } else {
                        log::info!("Loaded asset: {:?}", result.id);
                    }
                }
                Err(e) => {
                    log::error!("Failed to load asset {:?}: {}", result.id, e);

                    let fallback = {
                        let fallbacks = self.fallbacks.read().unwrap();
                        fallbacks.get(&result.expected_type).cloned()
                    };

                    if let Some(asset) = fallback {
                        log::warn!("Using fallback for asset {:?}", result.id);
                        let mut assets = self.assets.write().unwrap();
                        assets.insert(
                            result.id,
                            AssetEntry {
                                asset,
                                generation: 0,
                            },
                        );
                        self.load_states
                            .write()
                            .unwrap()
                            .insert(result.id, LoadState::Loaded);
                    } else {
                        self.load_states
                            .write()
                            .unwrap()
                            .insert(result.id, LoadState::Failed(e.to_string()));
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
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
                let path = path.to_path_buf();
                
                // Clone what we need for the async task
                let assets = self.assets.clone();
                let load_states = self.load_states.clone();
                let loaders = self.loaders.clone();

                // Spawn async reload task
                self.runtime.spawn(async move {
                    // Use tokio::fs for non-blocking I/O
                    let bytes = match tokio::fs::read(&path).await {
                        Ok(b) => b,
                        Err(e) => {
                            log::error!("Failed to reload asset {:?}: {}", path, e);
                            return;
                        }
                    };

                    // Find loader
                    let extension = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();

                    let loader = {
                        let loaders = loaders.read().unwrap();
                        loaders.get(&extension).cloned()
                    };

                    if let Some(loader) = loader {
                        // Parse asset in blocking task
                        let path_clone = path.clone();
                        let result = tokio::task::spawn_blocking(move || {
                            loader.load(&bytes, &path_clone)
                        })
                        .await;

                        match result {
                            Ok(Ok(asset_arc)) => {
                                let mut assets_guard = assets.write().unwrap();
                                let current_gen = if let Some(entry) = assets_guard.get(&id) {
                                    entry.generation + 1
                                } else {
                                    0
                                };

                                assets_guard.insert(
                                    id,
                                    AssetEntry {
                                        asset: asset_arc,
                                        generation: current_gen,
                                    },
                                );

                                load_states
                                    .write()
                                    .unwrap()
                                    .insert(id, LoadState::Loaded);
                                log::info!("Reloaded asset: {:?}", path);
                            }
                            Ok(Err(e)) => {
                                log::error!("Failed to reload asset {:?}: {}", path, e);
                                load_states
                                    .write()
                                    .unwrap()
                                    .insert(id, LoadState::Failed(e.to_string()));
                            }
                            Err(e) => {
                                log::error!("Task join error while reloading {:?}: {}", path, e);
                            }
                        }
                    }
                });
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

    pub fn register_fallback<T: Asset>(&mut self, asset: T) {
        let mut fallbacks = self.fallbacks.write().unwrap();
        fallbacks.insert(TypeId::of::<T>(), Arc::new(asset));
    }

    /// Register a placeholder asset that will be displayed while the real asset loads
    pub fn register_placeholder<T: Asset>(&self, placeholder: T) {
        self.placeholders.register(placeholder);
    }

    /// Get the placeholder registry
    pub fn placeholders(&self) -> &PlaceholderRegistry {
        &self.placeholders
    }

    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        let assets = self.assets.read().unwrap();
        assets.get(&handle.id()).and_then(|entry| {
            // Should we check generation?
            // If handle generation < entry generation, it's an old handle but we have newer data. Return newer data.
            // If handle generation > entry generation, we have old data (impossible unless handle comes from future).
            // If mismatch, we can log warning or return None if we strictly enforce versioning.
            // For now, return the asset.
            entry
                .asset
                .clone()
                .downcast::<T>()
                .map_err(|_| "Downcast failed")
                .ok()
        })
    }

    pub fn add<T: Asset>(&self, asset: T) -> Handle<T> {
        let id = AssetId::new();
        let mut assets = self.assets.write().unwrap();

        assets.insert(
            id,
            AssetEntry {
                asset: Arc::new(asset),
                generation: 0,
            },
        );

        self.load_states
            .write()
            .unwrap()
            .insert(id, LoadState::Loaded);

        Handle::new(id, 0)
    }

    /// Get the tokio runtime for spawning async tasks
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    
    /// Get the retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
    
    /// Get current loading progress
    pub fn load_progress(&self) -> LoadProgress {
        let states = self.load_states.read().unwrap();
        
        let mut progress = LoadProgress {
            total: states.len(),
            loaded: 0,
            loading: 0,
            failed: 0,
        };
        
        for state in states.values() {
            match state {
                LoadState::Loaded => progress.loaded += 1,
                LoadState::Loading => progress.loading += 1,
                LoadState::Failed(_) => progress.failed += 1,
                LoadState::NotLoaded => {}
            }
        }
        
        progress
    }
}

/// Check if an I/O error is transient and should be retried
fn is_transient_error(error: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    
    matches!(
        error.kind(),
        ErrorKind::Interrupted
            | ErrorKind::WouldBlock
            | ErrorKind::TimedOut
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
    )
}

/// Check if a parse error is transient and should be retried
fn is_transient_parse_error(error: &AssetLoadError) -> bool {
    // Check if the error is an I/O error that's transient
    match error {
        AssetLoadError::Io(io_error) => is_transient_error(io_error),
        _ => false,
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
