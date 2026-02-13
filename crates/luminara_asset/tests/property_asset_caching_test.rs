/// Property-based test for asset caching (Task 4.9)
///
/// **Property 7: Asset Caching**
/// **Validates: Requirements 4.8**
///
/// For any asset that is loaded multiple times, the asset server should
/// process it only once and return cached data for subsequent requests.

use luminara_asset::{Asset, AssetLoader, AssetLoadError, AssetServer};
use proptest::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

// Test asset loader that tracks load count
struct CountingAssetLoader {
    load_count: Arc<AtomicUsize>,
}

impl CountingAssetLoader {
    fn new(load_count: Arc<AtomicUsize>) -> Self {
        Self { load_count }
    }
}

impl AssetLoader for CountingAssetLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["txt", "json", "ron"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Increment load count to track processing
        self.load_count.fetch_add(1, Ordering::SeqCst);
        
        let data = String::from_utf8(bytes.to_vec())
            .map_err(|e| AssetLoadError::Parse(e.to_string()))?;
        Ok(TestAsset { data })
    }
}

/// Strategy to generate valid file names
fn valid_file_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9_]{3,15}").unwrap()
}

/// Strategy to generate file extensions
fn file_extension_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("txt"), Just("json"), Just("ron"),]
}

/// Strategy to generate file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 ]{10,200}").unwrap()
}

/// Strategy to generate number of load attempts
fn load_count_strategy() -> impl Strategy<Value = usize> {
    2usize..10
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Loading the same asset multiple times processes it only once
    #[test]
    fn prop_asset_loaded_once(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
        num_loads in load_count_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_cache_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create load counter
        let load_count = Arc::new(AtomicUsize::new(0));

        // Create asset server with counting loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(CountingAssetLoader::new(load_count.clone()));

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset multiple times
        let asset_path = format!("{}.{}", file_name, extension);
        let mut handles = Vec::new();
        for _ in 0..num_loads {
            let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
            handles.push(handle);
        }

        // Verify the asset was processed only once
        let actual_load_count = load_count.load(Ordering::SeqCst);
        prop_assert_eq!(
            actual_load_count, 1,
            "Asset should be processed only once, but was processed {} times for {} load requests",
            actual_load_count, num_loads
        );

        // Verify all handles point to the same asset
        for i in 1..handles.len() {
            prop_assert_eq!(
                handles[0].id(), handles[i].id(),
                "All handles should have the same ID"
            );
        }

        // Verify the cached asset is accessible and correct
        let asset = server.get(&handles[0]);
        prop_assert!(asset.is_some(), "Cached asset should be accessible");
        prop_assert_eq!(&asset.unwrap().data, &content, "Cached asset data should match original");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Different assets are processed independently (no cross-contamination)
    #[test]
    fn prop_different_assets_processed_independently(
        file_name1 in valid_file_name_strategy(),
        file_name2 in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content1 in file_content_strategy(),
        content2 in file_content_strategy(),
        num_loads in load_count_strategy(),
    ) {
        prop_assume!(file_name1 != file_name2);

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_cache_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create load counter
        let load_count = Arc::new(AtomicUsize::new(0));

        // Create asset server with counting loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(CountingAssetLoader::new(load_count.clone()));

        // Create two different asset files
        let file_path1 = temp_dir.join(format!("{}.{}", file_name1, extension));
        let file_path2 = temp_dir.join(format!("{}.{}", file_name2, extension));
        fs::write(&file_path1, content1.as_bytes()).unwrap();
        fs::write(&file_path2, content2.as_bytes()).unwrap();

        // Load both assets multiple times
        let asset_path1 = format!("{}.{}", file_name1, extension);
        let asset_path2 = format!("{}.{}", file_name2, extension);
        
        for _ in 0..num_loads {
            let _: luminara_asset::Handle<TestAsset> = server.load(&asset_path1);
            let _: luminara_asset::Handle<TestAsset> = server.load(&asset_path2);
        }

        // Verify both assets were processed exactly once each (total 2 times)
        let actual_load_count = load_count.load(Ordering::SeqCst);
        prop_assert_eq!(
            actual_load_count, 2,
            "Two different assets should be processed once each (2 total), but were processed {} times",
            actual_load_count
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Cached assets return identical data across multiple retrievals
    #[test]
    fn prop_cached_asset_data_consistent(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
        num_retrievals in load_count_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_cache_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create load counter
        let load_count = Arc::new(AtomicUsize::new(0));

        // Create asset server with counting loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(CountingAssetLoader::new(load_count.clone()));

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset once
        let asset_path = format!("{}.{}", file_name, extension);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Retrieve the asset multiple times
        let mut retrieved_data = Vec::new();
        for _ in 0..num_retrievals {
            let asset = server.get(&handle);
            prop_assert!(asset.is_some(), "Cached asset should always be accessible");
            retrieved_data.push(asset.unwrap().data.clone());
        }

        // Verify all retrievals return the same data
        for i in 1..retrieved_data.len() {
            prop_assert_eq!(
                &retrieved_data[0], &retrieved_data[i],
                "All retrievals should return identical data"
            );
        }

        // Verify the data matches the original content
        prop_assert_eq!(&retrieved_data[0], &content, "Cached data should match original content");

        // Verify processing happened only once
        let actual_load_count = load_count.load(Ordering::SeqCst);
        prop_assert_eq!(actual_load_count, 1, "Asset should be processed only once");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Assets in subdirectories are cached correctly
    #[test]
    fn prop_subdirectory_assets_cached(
        subdir in prop::string::string_regex("[a-z]{3,8}").unwrap(),
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
        num_loads in load_count_strategy(),
    ) {
        // Create temp directory with subdirectory
        let temp_dir = std::env::temp_dir().join(format!("luminara_cache_test_{}", uuid::Uuid::new_v4()));
        let full_dir = temp_dir.join(&subdir);
        fs::create_dir_all(&full_dir).unwrap();

        // Create load counter
        let load_count = Arc::new(AtomicUsize::new(0));

        // Create asset server with counting loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(CountingAssetLoader::new(load_count.clone()));

        // Create asset file in subdirectory
        let file_path = full_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset multiple times with subdirectory path
        let asset_path = format!("{}/{}.{}", subdir, file_name, extension);
        for _ in 0..num_loads {
            let _: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        }

        // Verify the asset was processed only once
        let actual_load_count = load_count.load(Ordering::SeqCst);
        prop_assert_eq!(
            actual_load_count, 1,
            "Asset in subdirectory should be processed only once, but was processed {} times",
            actual_load_count
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Caching works correctly with mixed load patterns
    #[test]
    fn prop_mixed_load_pattern_caching(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_cache_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create load counter
        let load_count = Arc::new(AtomicUsize::new(0));

        // Create asset server with counting loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(CountingAssetLoader::new(load_count.clone()));

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        let asset_path = format!("{}.{}", file_name, extension);

        // Load pattern: load, retrieve, load, retrieve, load
        let handle1: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        let asset1 = server.get(&handle1);
        prop_assert!(asset1.is_some());

        let handle2: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        let asset2 = server.get(&handle2);
        prop_assert!(asset2.is_some());

        let handle3: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        let asset3 = server.get(&handle3);
        prop_assert!(asset3.is_some());

        // Verify all handles are the same
        prop_assert_eq!(handle1.id(), handle2.id());
        prop_assert_eq!(handle2.id(), handle3.id());

        // Verify all retrieved data is the same
        prop_assert_eq!(&asset1.unwrap().data, &content);
        prop_assert_eq!(&asset2.unwrap().data, &content);
        prop_assert_eq!(&asset3.unwrap().data, &content);

        // Verify processing happened only once
        let actual_load_count = load_count.load(Ordering::SeqCst);
        prop_assert_eq!(
            actual_load_count, 1,
            "Asset should be processed only once with mixed load pattern, but was processed {} times",
            actual_load_count
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
