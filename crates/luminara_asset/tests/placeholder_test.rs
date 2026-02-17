use luminara_asset::{Asset, AssetServer, PlaceholderAsset, PlaceholderRegistry};

#[derive(Clone)]
struct TestAsset {
    value: String,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

impl PlaceholderAsset for TestAsset {
    fn create_placeholder() -> Self {
        TestAsset {
            value: "PLACEHOLDER".to_string(),
        }
    }
}

#[test]
fn test_placeholder_registry_basic() {
    let registry = PlaceholderRegistry::new();

    // Register placeholder
    let placeholder = TestAsset::create_placeholder();
    registry.register(placeholder);

    // Retrieve placeholder
    let retrieved = registry.get::<TestAsset>();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, "PLACEHOLDER");
}

#[test]
fn test_placeholder_in_asset_server() {
    let server = AssetServer::new("assets");

    // Register placeholder
    server.register_placeholder(TestAsset::create_placeholder());

    // Verify placeholder is registered
    assert!(server.placeholders().has_placeholder::<TestAsset>());

    // Get placeholder
    let placeholder = server.placeholders().get::<TestAsset>();
    assert!(placeholder.is_some());
    assert_eq!(placeholder.unwrap().value, "PLACEHOLDER");
}

#[test]
fn test_placeholder_display_during_loading() {
    let temp_dir = std::env::temp_dir().join("luminara_test_placeholder");
    std::fs::create_dir_all(&temp_dir).ok();

    let mut server = AssetServer::new(&temp_dir);

    // Register a placeholder
    server.register_placeholder(TestAsset {
        value: "LOADING...".to_string(),
    });

    // Create a test file that will be loaded
    let test_file = temp_dir.join("test.txt");
    std::fs::write(&test_file, b"test data").unwrap();

    // Register a simple loader
    struct TestLoader;
    impl luminara_asset::AssetLoader for TestLoader {
        type Asset = TestAsset;

        fn extensions(&self) -> &[&str] {
            &["txt"]
        }

        fn load(
            &self,
            bytes: &[u8],
            _path: &std::path::Path,
        ) -> Result<Self::Asset, luminara_asset::AssetLoadError> {
            // Simulate slow loading
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok(TestAsset {
                value: String::from_utf8_lossy(bytes).to_string(),
            })
        }
    }

    server.register_loader(TestLoader);

    // Load asset - should immediately return handle with placeholder
    let handle = server.load::<TestAsset>("test.txt");

    // Placeholder should be available immediately
    let asset = server.get(&handle);
    assert!(asset.is_some(), "Placeholder should be available immediately");
    assert_eq!(
        asset.unwrap().value,
        "LOADING...",
        "Should have placeholder value"
    );

    // Wait for real asset to load
    for _ in 0..100 {
        server.update();
        std::thread::sleep(std::time::Duration::from_millis(10));

        if let Some(asset) = server.get(&handle) {
            if asset.value == "test data" {
                // Real asset loaded successfully
                break;
            }
        }
    }

    // Verify real asset is now loaded
    let final_asset = server.get(&handle);
    assert!(final_asset.is_some());
    assert_eq!(final_asset.unwrap().value, "test data");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_hot_swap_without_frame_drops() {
    // This test verifies that hot-swapping doesn't cause frame drops
    // by ensuring the asset is always available (either placeholder or real)

    let temp_dir = std::env::temp_dir().join("luminara_test_hotswap");
    std::fs::create_dir_all(&temp_dir).ok();

    let mut server = AssetServer::new(&temp_dir);

    // Register placeholder
    server.register_placeholder(TestAsset {
        value: "PLACEHOLDER".to_string(),
    });

    // Create test file
    let test_file = temp_dir.join("hotswap.txt");
    std::fs::write(&test_file, b"real asset").unwrap();

    // Register loader
    struct TestLoader;
    impl luminara_asset::AssetLoader for TestLoader {
        type Asset = TestAsset;

        fn extensions(&self) -> &[&str] {
            &["txt"]
        }

        fn load(
            &self,
            bytes: &[u8],
            _path: &std::path::Path,
        ) -> Result<Self::Asset, luminara_asset::AssetLoadError> {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(TestAsset {
                value: String::from_utf8_lossy(bytes).to_string(),
            })
        }
    }

    server.register_loader(TestLoader);

    // Load asset
    let handle = server.load::<TestAsset>("hotswap.txt");

    // Asset should ALWAYS be available (never None)
    let mut frame_count = 0;
    let mut had_placeholder = false;
    let mut had_real_asset = false;

    for _ in 0..200 {
        server.update();

        let asset = server.get(&handle);
        assert!(
            asset.is_some(),
            "Asset should always be available (frame {})",
            frame_count
        );

        if let Some(asset) = asset {
            if asset.value == "PLACEHOLDER" {
                had_placeholder = true;
            } else if asset.value == "real asset" {
                had_real_asset = true;
            }
        }

        frame_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(5));

        if had_real_asset {
            break;
        }
    }

    assert!(had_placeholder, "Should have shown placeholder");
    assert!(had_real_asset, "Should have loaded real asset");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
