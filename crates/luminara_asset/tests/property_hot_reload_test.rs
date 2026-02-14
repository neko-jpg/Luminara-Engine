/// Property-based test for asset hot-reload detection (Task 4.5)
///
/// **Property 3: Asset Hot-Reload Detection**
/// **Validates: Requirements 4.1, 4.2**
///
/// For any asset file (including scene files) that is modified on disk,
/// the file watcher should detect the change within a reasonable time window
/// and trigger a reload of that asset.
use luminara_asset::{Asset, AssetLoadError, AssetLoader, AssetServer, HotReloadWatcher};
use proptest::prelude::*;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

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
        &["txt", "json", "ron", "scene"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        let data =
            String::from_utf8(bytes.to_vec()).map_err(|e| AssetLoadError::Parse(e.to_string()))?;
        Ok(TestAsset { data })
    }
}

/// Strategy to generate file extensions for various asset types
fn file_extension_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("txt"),
        Just("json"),
        Just("ron"),
        Just("scene"),
        Just("png"),
        Just("jpg"),
        Just("glb"),
    ]
}

/// Strategy to generate file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 ]{10,100}").unwrap()
}

/// Strategy to generate file names
fn file_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{3,10}").unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: For any asset file that is modified, the file watcher detects the change
    #[test]
    fn prop_hot_reload_detects_file_modifications(
        file_name in file_name_strategy(),
        extension in file_extension_strategy(),
        initial_content in file_content_strategy(),
        modified_content in file_content_strategy(),
    ) {
        // Skip if contents are identical
        prop_assume!(initial_content != modified_content);

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_prop_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create watcher
        let watcher = HotReloadWatcher::new(temp_dir.clone()).unwrap();

        // Create initial file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, initial_content.as_bytes()).unwrap();

        // Wait for initial file creation to settle
        thread::sleep(Duration::from_millis(100));

        // Clear any creation events
        while watcher.receiver().try_recv().is_ok() {}

        // Modify the file
        fs::write(&file_path, modified_content.as_bytes()).unwrap();

        // Wait for modification to be detected (reasonable time window)
        thread::sleep(Duration::from_millis(300));

        // Check if modification event was received
        let mut found_modify = false;
        while let Ok(event) = watcher.receiver().try_recv() {
            if event.kind.is_modify() {
                // Verify the event is for our file
                if event.paths.iter().any(|p| p == &file_path) {
                    found_modify = true;
                    break;
                }
            }
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();

        prop_assert!(found_modify, "File watcher should detect modification of {} file", extension);
    }

    /// Property: For any asset file in a subdirectory that is modified, the watcher detects it
    #[test]
    fn prop_hot_reload_detects_nested_file_modifications(
        dir_depth in 1usize..4,
        file_name in file_name_strategy(),
        extension in file_extension_strategy(),
        initial_content in file_content_strategy(),
        modified_content in file_content_strategy(),
    ) {
        prop_assume!(initial_content != modified_content);

        // Create temp directory with nested structure
        let temp_dir = std::env::temp_dir().join(format!("luminara_prop_test_{}", uuid::Uuid::new_v4()));
        let mut nested_dir = temp_dir.clone();
        for i in 0..dir_depth {
            nested_dir = nested_dir.join(format!("level{}", i));
        }
        fs::create_dir_all(&nested_dir).unwrap();

        // Create watcher on root
        let watcher = HotReloadWatcher::new(temp_dir.clone()).unwrap();

        // Create file in nested directory
        let file_path = nested_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, initial_content.as_bytes()).unwrap();

        // Wait for initial file creation to settle
        thread::sleep(Duration::from_millis(100));

        // Clear any creation events
        while watcher.receiver().try_recv().is_ok() {}

        // Modify the file
        fs::write(&file_path, modified_content.as_bytes()).unwrap();

        // Wait for modification to be detected
        thread::sleep(Duration::from_millis(300));

        // Check if modification event was received
        let mut found_modify = false;
        while let Ok(event) = watcher.receiver().try_recv() {
            if event.kind.is_modify() {
                if event.paths.iter().any(|p| p == &file_path) {
                    found_modify = true;
                    break;
                }
            }
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();

        prop_assert!(found_modify, "File watcher should detect modification in nested directories (depth {})", dir_depth);
    }

    /// Property: For any asset file that is modified multiple times, each modification is detected
    #[test]
    fn prop_hot_reload_detects_multiple_modifications(
        file_name in file_name_strategy(),
        extension in file_extension_strategy(),
        contents in prop::collection::vec(file_content_strategy(), 2..5),
    ) {
        // Ensure all contents are unique
        let unique_contents: Vec<_> = contents.iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
        prop_assume!(unique_contents.len() >= 2);

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_prop_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create watcher
        let watcher = HotReloadWatcher::new(temp_dir.clone()).unwrap();

        // Create initial file
        let file_path = temp_dir.join(format!("{}.{}", file_name, extension));
        fs::write(&file_path, contents[0].as_bytes()).unwrap();

        // Wait for initial file creation to settle
        thread::sleep(Duration::from_millis(100));

        // Clear any creation events
        while watcher.receiver().try_recv().is_ok() {}

        let mut modification_count = 0;

        // Modify the file multiple times
        for content in &contents[1..] {
            fs::write(&file_path, content.as_bytes()).unwrap();
            thread::sleep(Duration::from_millis(300));

            // Check for modification event
            while let Ok(event) = watcher.receiver().try_recv() {
                if event.kind.is_modify() {
                    if event.paths.iter().any(|p| p == &file_path) {
                        modification_count += 1;
                        break;
                    }
                }
            }
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();

        // We should detect at least one modification (file systems may coalesce events)
        prop_assert!(modification_count >= 1, "File watcher should detect at least one modification out of {} changes", contents.len() - 1);
    }

    /// Property: Asset server reload is triggered when file watcher detects changes
    #[test]
    fn prop_asset_server_reloads_on_file_change(
        file_name in file_name_strategy(),
        initial_content in file_content_strategy(),
        modified_content in file_content_strategy(),
    ) {
        prop_assume!(initial_content != modified_content);

        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_prop_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server and watcher
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(TestAssetLoader);

        let watcher = HotReloadWatcher::new(temp_dir.clone()).unwrap();

        // Create initial file
        let file_path = temp_dir.join(format!("{}.txt", file_name));
        fs::write(&file_path, initial_content.as_bytes()).unwrap();

        // Load the asset
        let handle: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.txt", file_name));

        // Wait for load
        let start = std::time::Instant::now();
        while server.load_state(handle.id()) == luminara_asset::LoadState::Loading {
            server.update();
            if start.elapsed() > Duration::from_secs(1) {
                // Cannot panic in prop_test, but unwrap later will fail or we can break
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }

        // Verify initial content
        let asset = server.get(&handle).unwrap();
        prop_assert_eq!(&asset.data, &initial_content);

        // Wait for system to settle
        thread::sleep(Duration::from_millis(100));

        // Clear any pending events
        while watcher.receiver().try_recv().is_ok() {}

        // Modify the file
        fs::write(&file_path, modified_content.as_bytes()).unwrap();

        // Wait for modification to be detected
        thread::sleep(Duration::from_millis(300));

        // Simulate hot reload system
        while let Ok(event) = watcher.receiver().try_recv() {
            if event.kind.is_modify() {
                for path in event.paths {
                    server.reload(&path);
                }
            }
        }

        // Wait for reload
        let start = std::time::Instant::now();
        let mut reloaded = false;
        while start.elapsed() < Duration::from_secs(1) {
            server.update();
            let asset = server.get(&handle).unwrap();
            if asset.data == modified_content {
                reloaded = true;
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }

        // Verify the asset was reloaded
        prop_assert!(reloaded, "Asset should be reloaded with new content");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
