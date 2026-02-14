/// Property-based test for asset load and retrieve (Task 4.6)
///
/// **Property 4: Asset Load and Retrieve**
/// **Validates: Requirements 4.3, 4.4**
///
/// For any valid asset path, requesting the asset should return a handle,
/// and using that handle should eventually provide access to the loaded
/// asset data once loading completes.
use luminara_asset::{Asset, AssetLoadError, AssetLoader, AssetServer, LoadState};
use proptest::prelude::*;
use std::fs;
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
        &["txt", "json", "ron"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        let data =
            String::from_utf8(bytes.to_vec()).map_err(|e| AssetLoadError::Parse(e.to_string()))?;
        Ok(TestAsset { data })
    }
}

fn wait_for_load(server: &AssetServer, handle: &luminara_asset::Handle<TestAsset>) {
    let start = std::time::Instant::now();
    while server.load_state(handle.id()) == luminara_asset::LoadState::Loading {
        server.update();
        if start.elapsed() > std::time::Duration::from_secs(1) {
            panic!("Timeout waiting for asset load");
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// Strategy to generate valid file names (no path traversal)
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

/// Strategy to generate subdirectory paths
fn subdirectory_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::string::string_regex("[a-z]{3,8}").unwrap(), 0..3)
        .prop_map(|parts| parts.join("/"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: For any valid asset path, load returns a handle
    #[test]
    fn prop_load_returns_handle(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset
        let asset_path = format!("{}.{}", file_name, extension);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Verify handle is returned (not null/invalid)
        // The handle should have a valid ID
        prop_assert!(handle.id() != luminara_asset::AssetId::default());

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: For any valid asset, the handle provides access to loaded data
    #[test]
    fn prop_handle_provides_access_to_loaded_data(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset
        let asset_path = format!("{}.{}", file_name, extension);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        wait_for_load(&server, &handle);

        // Verify asset is accessible via handle
        let loaded_asset = server.get(&handle);
        prop_assert!(loaded_asset.is_some(), "Asset should be accessible via handle");

        // Verify the loaded data matches the original content
        let asset = loaded_asset.unwrap();
        prop_assert_eq!(&asset.data, &content, "Loaded asset data should match original content");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: For any valid asset in subdirectories, load and retrieve works
    #[test]
    fn prop_load_from_subdirectories(
        subdirs in subdirectory_strategy(),
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        let full_dir = if subdirs.is_empty() {
            temp_dir.clone()
        } else {
            temp_dir.join(&subdirs)
        };
        fs::create_dir_all(&full_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create asset file in subdirectory
        let file_path = full_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset with subdirectory path
        let asset_path = if subdirs.is_empty() {
            format!("{}.{}", file_name, extension)
        } else {
            format!("{}/{}.{}", subdirs, file_name, extension)
        };
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        wait_for_load(&server, &handle);

        // Verify asset is accessible
        let loaded_asset = server.get(&handle);
        prop_assert!(loaded_asset.is_some(), "Asset in subdirectory should be accessible");

        let asset = loaded_asset.unwrap();
        prop_assert_eq!(&asset.data, &content, "Loaded asset data should match original content");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Loading the same asset multiple times returns the same handle
    #[test]
    fn prop_same_asset_same_handle(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset multiple times
        let asset_path = format!("{}.{}", file_name, extension);
        let handle1: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        let handle2: luminara_asset::Handle<TestAsset> = server.load(&asset_path);
        let handle3: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Verify all handles are equal (same asset ID)
        prop_assert_eq!(handle1.id(), handle2.id(), "Multiple loads should return same handle ID");
        prop_assert_eq!(handle2.id(), handle3.id(), "Multiple loads should return same handle ID");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Load state transitions correctly for valid assets
    #[test]
    fn prop_load_state_transitions(
        file_name in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content in file_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create asset file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Load the asset
        let asset_path = format!("{}.{}", file_name, extension);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        wait_for_load(&server, &handle);

        // Check load state - should be Loaded after synchronous load
        let state = server.load_state(handle.id());
        prop_assert_eq!(state, LoadState::Loaded, "Asset should be in Loaded state after load completes");

        // Verify asset is accessible
        let loaded_asset = server.get(&handle);
        prop_assert!(loaded_asset.is_some(), "Loaded asset should be accessible");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Different assets have different handles
    #[test]
    fn prop_different_assets_different_handles(
        file_name1 in valid_file_name_strategy(),
        file_name2 in valid_file_name_strategy(),
        extension in file_extension_strategy(),
        content1 in file_content_strategy(),
        content2 in file_content_strategy(),
    ) {
        prop_assume!(file_name1 != file_name2);

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_load_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create two different asset files
        let file_path1 = temp_dir.join(format!("{}.{}", file_name1, extension));
        let file_path2 = temp_dir.join(format!("{}.{}", file_name2, extension));
        fs::write(&file_path1, content1.as_bytes()).unwrap();
        fs::write(&file_path2, content2.as_bytes()).unwrap();

        // Load both assets
        let asset_path1 = format!("{}.{}", file_name1, extension);
        let asset_path2 = format!("{}.{}", file_name2, extension);
        let handle1: luminara_asset::Handle<TestAsset> = server.load(&asset_path1);
        let handle2: luminara_asset::Handle<TestAsset> = server.load(&asset_path2);

        wait_for_load(&server, &handle1);
        wait_for_load(&server, &handle2);

        // Verify handles are different
        prop_assert_ne!(handle1.id(), handle2.id(), "Different assets should have different handles");

        // Verify both assets are accessible with correct data
        let asset1 = server.get(&handle1).unwrap();
        let asset2 = server.get(&handle2).unwrap();
        prop_assert_eq!(&asset1.data, &content1);
        prop_assert_eq!(&asset2.data, &content2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
