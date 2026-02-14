/// Property-based test for asset load error handling (Task 4.8)
///
/// **Property 6: Asset Load Error Handling**
/// **Validates: Requirements 4.7**
///
/// For any invalid asset path or corrupted asset file, the asset server
/// should not crash, should log an error, and should provide a fallback asset.
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

// Test asset loader that can fail on corrupted data
struct TestAssetLoader;

impl AssetLoader for TestAssetLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["txt", "json", "ron"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Attempt to parse as UTF-8
        let data =
            String::from_utf8(bytes.to_vec()).map_err(|e| AssetLoadError::Parse(e.to_string()))?;

        // Additional validation: reject empty data
        if data.trim().is_empty() {
            return Err(AssetLoadError::Parse("Empty asset data".to_string()));
        }

        Ok(TestAsset { data })
    }
}

/// Strategy to generate invalid paths (path traversal attempts)
fn invalid_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("../../../etc/passwd".to_string()),
        Just("..\\..\\windows\\system32".to_string()),
        Just("/etc/passwd".to_string()),
        Just("C:\\Windows\\System32\\config".to_string()),
        Just("../../sensitive_file.txt".to_string()),
        Just("../outside_assets.txt".to_string()),
    ]
}

/// Strategy to generate non-existent file names
fn nonexistent_file_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9_]{10,20}")
        .unwrap()
        .prop_map(|s| format!("{}_nonexistent.txt", s))
}

/// Strategy to generate corrupted file content (invalid UTF-8)
fn corrupted_content_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        // Invalid UTF-8 sequences
        Just(vec![0xFF, 0xFE, 0xFD, 0xFC]),
        Just(vec![0x80, 0x81, 0x82, 0x83]),
        Just(vec![0xC0, 0xC1, 0xF5, 0xF6]),
        // Random binary data
        prop::collection::vec(any::<u8>(), 10..100)
            .prop_filter("Must contain invalid UTF-8", |bytes| {
                String::from_utf8(bytes.clone()).is_err()
            }),
    ]
}

/// Strategy to generate empty or whitespace-only content
fn empty_content_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("   ".to_string()),
        Just("\n\n\n".to_string()),
        Just("\t\t\t".to_string()),
        Just("  \n  \t  \n  ".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Invalid paths (path traversal) should not crash and should fail gracefully
    #[test]
    fn prop_invalid_path_no_crash(invalid_path in invalid_path_strategy()) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Attempt to load with invalid path - should not crash
        let handle: luminara_asset::Handle<TestAsset> = server.load(&invalid_path);

        // Wait for async load if it started
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) {
                // If it takes too long, it might be stuck or failed silently, break loop
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify the load failed gracefully
        let state = server.load_state(handle.id());

        // The asset should either be NotLoaded or Failed, but not Loaded
        prop_assert!(
            matches!(state, LoadState::NotLoaded | LoadState::Failed(_)),
            "Invalid path should result in NotLoaded or Failed state, got: {:?}",
            state
        );

        // Verify asset is not accessible
        let asset = server.get(&handle);
        prop_assert!(asset.is_none(), "Invalid path should not provide accessible asset");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Non-existent files should not crash and should fail gracefully
    #[test]
    fn prop_nonexistent_file_no_crash(file_name in nonexistent_file_strategy()) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Attempt to load non-existent file - should not crash
        let handle: luminara_asset::Handle<TestAsset> = server.load(&file_name);

        // Wait for async load
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify the load failed gracefully
        let state = server.load_state(handle.id());

        prop_assert!(
            matches!(state, LoadState::Failed(_)),
            "Non-existent file should result in Failed state, got: {:?}",
            state
        );

        // Verify asset is not accessible
        let asset = server.get(&handle);
        prop_assert!(asset.is_none(), "Non-existent file should not provide accessible asset");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Corrupted files (invalid UTF-8) should not crash and should fail gracefully
    #[test]
    fn prop_corrupted_file_no_crash(
        file_name in prop::string::string_regex("[a-z0-9_]{5,15}").unwrap(),
        corrupted_content in corrupted_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create corrupted file
        let file_path = temp_dir.join(format!("{}.txt", file_name));
        fs::write(&file_path, &corrupted_content).unwrap();

        // Attempt to load corrupted file - should not crash
        let asset_path = format!("{}.txt", file_name);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Wait for async load
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify the load failed gracefully
        let state = server.load_state(handle.id());

        prop_assert!(
            matches!(state, LoadState::Failed(_)),
            "Corrupted file should result in Failed state, got: {:?}",
            state
        );

        // Verify asset is not accessible
        let asset = server.get(&handle);
        prop_assert!(asset.is_none(), "Corrupted file should not provide accessible asset");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Empty or whitespace-only files should not crash and should fail gracefully
    #[test]
    fn prop_empty_file_no_crash(
        file_name in prop::string::string_regex("[a-z0-9_]{5,15}").unwrap(),
        empty_content in empty_content_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create empty file
        let file_path = temp_dir.join(format!("{}.txt", file_name));
        fs::write(&file_path, empty_content.as_bytes()).unwrap();

        // Attempt to load empty file - should not crash
        let asset_path = format!("{}.txt", file_name);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Wait for async load
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify the load failed gracefully
        let state = server.load_state(handle.id());

        prop_assert!(
            matches!(state, LoadState::Failed(_)),
            "Empty file should result in Failed state, got: {:?}",
            state
        );

        // Verify asset is not accessible
        let asset = server.get(&handle);
        prop_assert!(asset.is_none(), "Empty file should not provide accessible asset");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Unsupported file extensions should not crash and should fail gracefully
    #[test]
    fn prop_unsupported_extension_no_crash(
        file_name in prop::string::string_regex("[a-z0-9_]{5,15}").unwrap(),
        extension in prop::string::string_regex("[a-z]{2,5}").unwrap(),
        content in prop::string::string_regex("[a-zA-Z0-9 ]{10,100}").unwrap(),
    ) {
        // Filter out supported extensions
        prop_assume!(extension != "txt" && extension != "json" && extension != "ron");

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create file with unsupported extension
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Attempt to load file with unsupported extension - should not crash
        let asset_path = format!("{}.{}", file_name, extension);
        let handle: luminara_asset::Handle<TestAsset> = server.load(&asset_path);

        // Wait for async load (though unsupported extensions usually fail immediately)
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify the load failed gracefully
        let state = server.load_state(handle.id());

        prop_assert!(
            matches!(state, LoadState::Failed(_)),
            "Unsupported extension should result in Failed state, got: {:?}",
            state
        );

        // Verify asset is not accessible
        let asset = server.get(&handle);
        prop_assert!(asset.is_none(), "Unsupported extension should not provide accessible asset");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Multiple error scenarios in sequence should not crash
    #[test]
    fn prop_multiple_errors_no_crash(
        valid_file in prop::string::string_regex("[a-z0-9_]{5,10}").unwrap(),
        invalid_path in invalid_path_strategy(),
        nonexistent_file in nonexistent_file_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_error_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        // Create one valid file
        let valid_path = temp_dir.join(format!("{}.txt", valid_file));
        fs::write(&valid_path, "valid content").unwrap();

        // Load valid file first
        let handle1: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.txt", valid_file));

        let start = std::time::Instant::now();
        while server.load_state(handle1.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) { break; }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        prop_assert!(server.get(&handle1).is_some(), "Valid file should load successfully");

        // Then try invalid path - should not crash or affect valid asset
        let handle2: luminara_asset::Handle<TestAsset> = server.load(&invalid_path);

        // Wait for potential async load of invalid path
        let start = std::time::Instant::now();
        while server.load_state(handle2.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) { break; }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        prop_assert!(server.get(&handle2).is_none(), "Invalid path should fail");

        // Verify valid asset is still accessible
        prop_assert!(server.get(&handle1).is_some(), "Valid asset should remain accessible after error");

        // Then try non-existent file - should not crash
        let handle3: luminara_asset::Handle<TestAsset> = server.load(&nonexistent_file);

        let start = std::time::Instant::now();
        while server.load_state(handle3.id()) == LoadState::Loading {
            server.update();
            if start.elapsed() > std::time::Duration::from_secs(1) { break; }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        prop_assert!(server.get(&handle3).is_none(), "Non-existent file should fail");

        // Verify valid asset is still accessible
        prop_assert!(server.get(&handle1).is_some(), "Valid asset should remain accessible after multiple errors");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
