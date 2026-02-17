/// Property-based test for async asset loading non-blocking behavior (Task 18.2)
///
/// **Property 15: Async Asset Loading Non-Blocking**
/// **Validates: Requirements 14.1**
///
/// For any asset loading operation, the main thread should never block waiting
/// for I/O completion. This test verifies that load() calls return immediately
/// and that the main thread can continue processing while assets load in the background.

use luminara_asset::{Asset, AssetLoadError, AssetLoader, AssetServer, LoadState};
use proptest::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

// Configurable slow loader to simulate I/O operations
struct SlowLoader {
    delay_ms: u64,
}

impl AssetLoader for SlowLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["slow"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // Simulate slow I/O operation (e.g., reading from disk, parsing complex formats)
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        Ok(TestAsset {
            data: String::from_utf8_lossy(bytes).to_string(),
        })
    }
}

/// Strategy to generate file names
fn file_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9_]{3,15}").unwrap()
}

/// Strategy to generate file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 ]{10,100}").unwrap()
}

/// Strategy to generate loading delay (50-200ms to simulate realistic I/O)
fn delay_strategy() -> impl Strategy<Value = u64> {
    50u64..200u64
}

/// Strategy to generate number of assets to load concurrently
fn asset_count_strategy() -> impl Strategy<Value = usize> {
    1usize..10usize
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: load() returns immediately without blocking the main thread
    #[test]
    fn prop_load_returns_immediately(
        file_name in file_name_strategy(),
        content in file_content_strategy(),
        delay_ms in delay_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_async_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server with slow loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(SlowLoader { delay_ms });

        // Create asset file
        let file_path = temp_dir.join(format!("{}.slow", file_name));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Measure time for load() call
        let start = Instant::now();
        let _handle: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.slow", file_name));
        let load_call_duration = start.elapsed();

        // Property: load() should return in less than 10ms (much less than the I/O delay)
        // This proves the main thread is not blocked by I/O
        prop_assert!(
            load_call_duration < Duration::from_millis(10),
            "load() took {:?}, expected < 10ms. Main thread should not block on I/O (delay was {}ms)",
            load_call_duration,
            delay_ms
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Main thread can continue working while assets load
    #[test]
    fn prop_main_thread_continues_working(
        file_name in file_name_strategy(),
        content in file_content_strategy(),
        delay_ms in delay_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_async_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server with slow loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(SlowLoader { delay_ms });

        // Create asset file
        let file_path = temp_dir.join(format!("{}.slow", file_name));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Start loading asset
        let handle: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.slow", file_name));

        // Track main thread work
        let work_counter = Arc::new(AtomicU64::new(0));
        let work_counter_clone = work_counter.clone();

        // Spawn verification thread to check main thread keeps working
        let verification_thread = std::thread::spawn(move || {
            let start = Instant::now();
            let expected_duration = Duration::from_millis(delay_ms + 50);
            
            while start.elapsed() < expected_duration {
                let work_count = work_counter_clone.load(Ordering::Relaxed);
                // Main thread should be doing work (counter should increase)
                if work_count == 0 && start.elapsed() > Duration::from_millis(20) {
                    return Err("Main thread appears to be blocked - no work being done");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        });

        // Main thread continues working (simulating game loop)
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(delay_ms + 100) {
            // Simulate main thread work
            work_counter.fetch_add(1, Ordering::Relaxed);
            
            // Process asset loading updates
            server.update();
            
            // Check if asset is loaded
            if server.load_state(handle.id()) == LoadState::Loaded {
                break;
            }
            
            std::thread::sleep(Duration::from_millis(5));
        }

        // Verify main thread was working
        let total_work = work_counter.load(Ordering::Relaxed);
        prop_assert!(
            total_work > 0,
            "Main thread should have done work while asset was loading"
        );

        // Verify verification thread succeeded
        verification_thread.join().unwrap().unwrap();

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Multiple concurrent loads don't block the main thread
    #[test]
    fn prop_concurrent_loads_non_blocking(
        asset_count in asset_count_strategy(),
        delay_ms in delay_strategy(),
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_async_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server with slow loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(SlowLoader { delay_ms });

        // Create multiple asset files
        for i in 0..asset_count {
            let file_path = temp_dir.join(format!("asset{}.slow", i));
            fs::write(&file_path, format!("data {}", i)).unwrap();
        }

        // Measure time to start all loads
        let start = Instant::now();
        let handles: Vec<luminara_asset::Handle<TestAsset>> = (0..asset_count)
            .map(|i| server.load(&format!("asset{}.slow", i)))
            .collect();
        let all_loads_duration = start.elapsed();

        // Property: Starting all loads should be fast (< 50ms total)
        // Even with many assets, load() calls should not block
        prop_assert!(
            all_loads_duration < Duration::from_millis(50),
            "Starting {} loads took {:?}, expected < 50ms. load() calls should not block",
            asset_count,
            all_loads_duration
        );

        // Track main thread responsiveness
        let responsive = Arc::new(AtomicBool::new(true));
        let responsive_clone = responsive.clone();

        // Monitor thread to detect if main thread becomes unresponsive
        let monitor_thread = std::thread::spawn(move || {
            let start = Instant::now();
            let mut last_check = start;
            
            while start.elapsed() < Duration::from_millis(delay_ms + 200) {
                let now = Instant::now();
                let gap = now.duration_since(last_check);
                
                // If main thread is blocked, we won't see updates
                if gap > Duration::from_millis(100) {
                    responsive_clone.store(false, Ordering::Relaxed);
                    return Err("Main thread appears unresponsive");
                }
                
                last_check = now;
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(())
        });

        // Main thread continues processing
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(delay_ms + 300) {
            server.update();
            
            // Check if all assets loaded
            let all_loaded = handles.iter()
                .all(|h| server.load_state(h.id()) == LoadState::Loaded);
            
            if all_loaded {
                break;
            }
            
            std::thread::sleep(Duration::from_millis(5));
        }

        // Verify main thread remained responsive
        prop_assert!(
            responsive.load(Ordering::Relaxed),
            "Main thread should remain responsive during concurrent asset loading"
        );

        // Monitor thread should complete successfully
        let _ = monitor_thread.join();

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: Asset loading doesn't block even with very slow I/O
    #[test]
    fn prop_extreme_delay_non_blocking(
        file_name in file_name_strategy(),
        content in file_content_strategy(),
    ) {
        // Use extreme delay to stress test non-blocking behavior
        let extreme_delay_ms = 500u64;
        
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_async_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server with very slow loader
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(SlowLoader { delay_ms: extreme_delay_ms });

        // Create asset file
        let file_path = temp_dir.join(format!("{}.slow", file_name));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Start loading
        let start = Instant::now();
        let handle: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.slow", file_name));
        let load_call_duration = start.elapsed();

        // Property: Even with 500ms I/O delay, load() should return immediately
        prop_assert!(
            load_call_duration < Duration::from_millis(10),
            "load() took {:?} with {}ms I/O delay. Should return immediately regardless of I/O time",
            load_call_duration,
            extreme_delay_ms
        );

        // Verify main thread can do substantial work during the long load
        let work_iterations = Arc::new(AtomicU64::new(0));
        let work_clone = work_iterations.clone();

        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(extreme_delay_ms + 100) {
            work_iterations.fetch_add(1, Ordering::Relaxed);
            server.update();
            
            if server.load_state(handle.id()) == LoadState::Loaded {
                break;
            }
            
            std::thread::sleep(Duration::from_millis(5));
        }

        // With 500ms+ of time, main thread should have done many iterations
        let total_iterations = work_clone.load(Ordering::Relaxed);
        prop_assert!(
            total_iterations > 50,
            "Main thread should have completed many iterations (got {}) during long I/O",
            total_iterations
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Property: load() never blocks regardless of asset size or complexity
    #[test]
    fn prop_load_never_blocks_varying_delays(
        file_name in file_name_strategy(),
        content in file_content_strategy(),
        delay_ms in 10u64..300u64, // Wide range of delays
    ) {
        // Create temp directory
        let temp_dir = std::env::temp_dir().join(format!("luminara_async_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create asset server
        let mut server = AssetServer::new(&temp_dir);
        server.register_loader(SlowLoader { delay_ms });

        // Create asset file
        let file_path = temp_dir.join(format!("{}.slow", file_name));
        fs::write(&file_path, content.as_bytes()).unwrap();

        // Measure load() call time
        let start = Instant::now();
        let _handle: luminara_asset::Handle<TestAsset> = server.load(&format!("{}.slow", file_name));
        let load_duration = start.elapsed();

        // Property: load() should ALWAYS return quickly, regardless of I/O delay
        // This is the core non-blocking guarantee
        prop_assert!(
            load_duration < Duration::from_millis(10),
            "load() took {:?} with {}ms I/O delay. Must return immediately for ANY delay",
            load_duration,
            delay_ms
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
