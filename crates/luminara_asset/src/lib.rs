pub mod allocator;
pub mod handle;
pub mod hot_reload;
pub mod loader;
pub mod meta;
pub mod placeholder;
pub mod plugin;
pub mod processor;
pub mod server;
pub mod storage;

pub use allocator::*;
pub use handle::*;
pub use hot_reload::*;
pub use loader::*;
pub use placeholder::*;
pub use plugin::*;
pub use server::*;
pub use storage::*;

// Re-export LoadPriority for convenience
pub use server::LoadPriority;

pub trait Asset: Send + Sync + 'static {
    fn type_name() -> &'static str
    where
        Self: Sized;
}
