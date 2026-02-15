use luminara_core::resource::Resource;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub data_path: PathBuf,
    pub backend: DbBackend,
    pub cache_size: usize,
    pub namespace: String,
    pub auto_migrate: bool,
    pub strict_mode: bool,
}

impl Resource for DbConfig {}

#[derive(Debug, Clone)]
pub enum DbBackend {
    SurrealKV,
    RocksDb,
    Memory,
    #[cfg(target_arch = "wasm32")]
    IndexedDb,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from(".luminara/db"),
            backend: DbBackend::SurrealKV,
            cache_size: 64 * 1024 * 1024,
            namespace: "luminara".to_string(),
            auto_migrate: true,
            strict_mode: false,
        }
    }
}
