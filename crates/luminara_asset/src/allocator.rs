use crate::AssetId;
use std::sync::atomic::{AtomicU64, Ordering};

/// Handle allocator for generating unique asset IDs
/// 
/// This allocator provides two strategies:
/// 1. Path-based IDs using UUID v5 (deterministic, same path = same ID)
/// 2. Sequential IDs for runtime-generated assets
pub struct HandleAllocator {
    next_id: AtomicU64,
}

impl HandleAllocator {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
        }
    }

    /// Allocate a new unique asset ID
    pub fn allocate(&self) -> AssetId {
        AssetId::new()
    }

    /// Get a deterministic ID for a given path
    pub fn id_for_path(&self, path: &str) -> AssetId {
        AssetId::from_path(path)
    }

    /// Allocate a sequential ID (useful for runtime-generated assets)
    pub fn allocate_sequential(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for HandleAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_unique_ids() {
        let allocator = HandleAllocator::new();
        let id1 = allocator.allocate();
        let id2 = allocator.allocate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_path_based_ids_deterministic() {
        let allocator = HandleAllocator::new();
        let id1 = allocator.id_for_path("textures/test.png");
        let id2 = allocator.id_for_path("textures/test.png");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_paths_different_ids() {
        let allocator = HandleAllocator::new();
        let id1 = allocator.id_for_path("textures/test1.png");
        let id2 = allocator.id_for_path("textures/test2.png");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_sequential_ids() {
        let allocator = HandleAllocator::new();
        let id1 = allocator.allocate_sequential();
        let id2 = allocator.allocate_sequential();
        assert_eq!(id1 + 1, id2);
    }
}
