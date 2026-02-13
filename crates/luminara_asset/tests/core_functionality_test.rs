/// Test to verify all core components of task 4.1 are implemented
/// 
/// Requirements from task 4.1:
/// - AssetServer implementation
/// - Handle<T> type-safe asset references
/// - AssetStorage for loaded assets
/// - Asset trait
/// - Handle allocator

use luminara_asset::{Asset, AssetId, AssetLoader, AssetLoadError, AssetServer, AssetStorage, Handle, HandleAllocator};
use std::path::Path;

// Test asset type
#[derive(Debug, Clone, PartialEq)]
struct TestAsset {
    data: String,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

// Test asset loader
struct TestAssetLoader;

impl AssetLoader for TestAssetLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["test"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        let data = String::from_utf8(bytes.to_vec())
            .map_err(|e| AssetLoadError::Parse(e.to_string()))?;
        Ok(TestAsset { data })
    }
}

#[test]
fn test_asset_trait_implementation() {
    // Verify Asset trait is implemented
    assert_eq!(TestAsset::type_name(), "TestAsset");
}

#[test]
fn test_handle_type_safety() {
    // Verify Handle<T> provides type-safe asset references
    let allocator = HandleAllocator::new();
    let id = allocator.allocate();
    
    let handle: Handle<TestAsset> = Handle::new(id);
    
    // Verify handle stores the correct ID
    assert_eq!(handle.id(), id);
    
    // Verify handles can be cloned
    let handle2 = handle.clone();
    assert_eq!(handle.id(), handle2.id());
    
    // Verify handles can be compared
    assert_eq!(handle, handle2);
}

#[test]
fn test_asset_storage() {
    // Verify AssetStorage can store and retrieve assets
    let mut storage: AssetStorage<TestAsset> = AssetStorage::new();
    
    let id = AssetId::new();
    let asset = TestAsset {
        data: "test data".to_string(),
    };
    
    // Insert asset
    storage.insert(id, asset.clone());
    
    // Verify contains
    assert!(storage.contains(id));
    
    // Retrieve asset
    let retrieved = storage.get(id).unwrap();
    assert_eq!(retrieved.data, "test data");
    
    // Verify mutable access
    let retrieved_mut = storage.get_mut(id).unwrap();
    retrieved_mut.data = "modified".to_string();
    assert_eq!(storage.get(id).unwrap().data, "modified");
    
    // Remove asset
    let removed = storage.remove(id).unwrap();
    assert_eq!(removed.data, "modified");
    assert!(!storage.contains(id));
}

#[test]
fn test_handle_allocator() {
    // Verify HandleAllocator can generate unique IDs
    let allocator = HandleAllocator::new();
    
    // Test unique ID generation
    let id1 = allocator.allocate();
    let id2 = allocator.allocate();
    assert_ne!(id1, id2);
    
    // Test path-based ID generation (deterministic)
    let path_id1 = allocator.id_for_path("test/path.txt");
    let path_id2 = allocator.id_for_path("test/path.txt");
    assert_eq!(path_id1, path_id2);
    
    // Test different paths produce different IDs
    let path_id3 = allocator.id_for_path("test/other.txt");
    assert_ne!(path_id1, path_id3);
    
    // Test sequential ID generation
    let seq1 = allocator.allocate_sequential();
    let seq2 = allocator.allocate_sequential();
    assert_eq!(seq1 + 1, seq2);
}

#[test]
fn test_asset_server_integration() {
    // Verify AssetServer integrates all components
    let temp_dir = std::env::temp_dir().join("luminara_asset_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // Create test asset file
    let test_file = temp_dir.join("test.test");
    std::fs::write(&test_file, b"test content").unwrap();
    
    // Create asset server
    let mut server = AssetServer::new(&temp_dir);
    server.register_loader(TestAssetLoader);
    
    // Verify handle allocator is accessible
    let allocator = server.handle_allocator();
    let _id = allocator.allocate();
    
    // Load asset
    let handle: Handle<TestAsset> = server.load("test.test");
    
    // Verify asset is loaded
    let asset = server.get(&handle).unwrap();
    assert_eq!(asset.data, "test content");
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_all_requirements_met() {
    // This test verifies all requirements from task 4.1 are met:
    // 1. AssetServer is implemented ✓
    // 2. Handle<T> type-safe asset references ✓
    // 3. AssetStorage for loaded assets ✓
    // 4. Asset trait ✓
    // 5. Handle allocator ✓
    
    // All components compile and work together
    let _server: AssetServer = AssetServer::new("assets");
    let _storage: AssetStorage<TestAsset> = AssetStorage::new();
    let _allocator: HandleAllocator = HandleAllocator::new();
    let _handle: Handle<TestAsset> = Handle::new(AssetId::new());
    
    // Asset trait is implemented
    assert_eq!(TestAsset::type_name(), "TestAsset");
}
