use luminara_asset::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
struct TestAsset {
    data: String,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

struct SlowLoader;
impl AssetLoader for SlowLoader {
    type Asset = TestAsset;
    fn extensions(&self) -> &[&str] {
        &["slow"]
    }
    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Simulate slow parsing (e.g., complex mesh processing)
        std::thread::sleep(Duration::from_millis(100));
        Ok(TestAsset {
            data: String::from_utf8_lossy(bytes).to_string(),
        })
    }
}

#[test]
fn test_main_thread_never_blocks() {
    // Setup test directory and file
    let test_dir = std::env::temp_dir().join("luminara_async_test");
    fs::create_dir_all(&test_dir).unwrap();
    let file_path = test_dir.join("slow.slow");
    fs::write(&file_path, "slow asset data").unwrap();

    let mut server = AssetServer::new(&test_dir);
    server.register_loader(SlowLoader);

    // Start loading asset
    let handle: Handle<TestAsset> = server.load("slow.slow");
    
    // Verify that load() returned immediately (main thread didn't block)
    let load_start = Instant::now();
    assert!(load_start.elapsed() < Duration::from_millis(10), 
            "load() should return immediately without blocking");

    // Simulate main thread doing other work while asset loads
    let main_thread_working = Arc::new(AtomicBool::new(true));
    let working_clone = main_thread_working.clone();
    
    // Spawn a thread to verify main thread keeps working
    let verification_thread = std::thread::spawn(move || {
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(150) {
            assert!(working_clone.load(Ordering::Relaxed), 
                    "Main thread should continue working during asset load");
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    // Main thread continues working (simulated by update loop)
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(200) {
        // This simulates the main game loop
        server.update(); // Process completed loads
        main_thread_working.store(true, Ordering::Relaxed);
        
        // Check if asset is loaded
        if server.load_state(handle.id()) == LoadState::Loaded {
            break;
        }
        
        std::thread::sleep(Duration::from_millis(5));
    }

    verification_thread.join().unwrap();

    // Verify asset eventually loaded
    assert_eq!(server.load_state(handle.id()), LoadState::Loaded);
    let asset = server.get(&handle).expect("Asset should be loaded");
    assert_eq!(asset.data, "slow asset data");

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_parallel_asset_loading() {
    // Setup test directory with multiple files
    let test_dir = std::env::temp_dir().join("luminara_parallel_test");
    fs::create_dir_all(&test_dir).unwrap();
    
    let file_count = 10;
    for i in 0..file_count {
        let file_path = test_dir.join(format!("asset{}.slow", i));
        fs::write(&file_path, format!("data {}", i)).unwrap();
    }

    let mut server = AssetServer::new(&test_dir);
    server.register_loader(SlowLoader);

    // Load all assets
    let start = Instant::now();
    let handles: Vec<Handle<TestAsset>> = (0..file_count)
        .map(|i| server.load(&format!("asset{}.slow", i)))
        .collect();

    // All load() calls should return quickly
    assert!(start.elapsed() < Duration::from_millis(50), 
            "All load() calls should return quickly");

    // Wait for all assets to load
    let load_start = Instant::now();
    loop {
        server.update();
        
        let all_loaded = handles.iter()
            .all(|h| server.load_state(h.id()) == LoadState::Loaded);
        
        if all_loaded {
            break;
        }
        
        if load_start.elapsed() > Duration::from_secs(5) {
            panic!("Timeout waiting for parallel loads");
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }

    let total_time = load_start.elapsed();
    
    // With parallel loading, total time should be much less than sequential
    // Sequential would be: 10 assets * 100ms = 1000ms
    // Parallel should be closer to 100-200ms (depending on thread count)
    assert!(total_time < Duration::from_millis(500), 
            "Parallel loading should be faster than sequential. Took: {:?}", total_time);

    // Verify all assets loaded correctly
    for (i, handle) in handles.iter().enumerate() {
        let asset = server.get(handle).expect("Asset should be loaded");
        assert_eq!(asset.data, format!("data {}", i));
    }

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}
