/// Tests for asset loading retry logic with exponential backoff
///
/// This test suite verifies that:
/// - Retry logic activates on transient errors
/// - Exponential backoff delays are applied correctly
/// - Error placeholders are used after max retries
/// - Progress tracking works correctly

use luminara_asset::{Asset, AssetLoadError, AssetLoader, AssetServer, LoadState, RetryConfig};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct TestAsset {
    value: String,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

/// Loader that fails a specified number of times before succeeding
struct FlakyLoader {
    fail_count: Arc<AtomicU32>,
    attempts: Arc<AtomicU32>,
}

impl FlakyLoader {
    fn new(fail_count: u32) -> Self {
        Self {
            fail_count: Arc::new(AtomicU32::new(fail_count)),
            attempts: Arc::new(AtomicU32::new(0)),
        }
    }

    fn attempts(&self) -> u32 {
        self.attempts.load(Ordering::SeqCst)
    }
}

impl AssetLoader for FlakyLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["flaky"]
    }

    fn load(&self, _bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
        let remaining_fails = self.fail_count.load(Ordering::SeqCst);

        if remaining_fails > 0 {
            self.fail_count.fetch_sub(1, Ordering::SeqCst);
            // Simulate transient I/O error
            return Err(AssetLoadError::Io(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                format!("Simulated transient error (attempt {})", attempt + 1),
            )));
        }

        Ok(TestAsset {
            value: format!("Loaded after {} attempts", attempt + 1),
        })
    }
}

#[test]
fn test_retry_config_delay_calculation() {
    let config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
    };

    // First attempt has no delay
    assert_eq!(config.delay_for_attempt(0), Duration::ZERO);

    // Exponential backoff: 100ms, 200ms, 400ms
    assert_eq!(config.delay_for_attempt(1), Duration::from_millis(100));
    assert_eq!(config.delay_for_attempt(2), Duration::from_millis(200));
    assert_eq!(config.delay_for_attempt(3), Duration::from_millis(400));

    // Test max delay cap
    let config_with_cap = RetryConfig {
        max_retries: 10,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
        backoff_multiplier: 2.0,
    };

    // Should be capped at 500ms
    assert!(config_with_cap.delay_for_attempt(10) <= Duration::from_millis(500));
}

#[test]
fn test_successful_load_after_retries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let asset_path = temp_dir.path().join("test.flaky");
    std::fs::write(&asset_path, b"test data").unwrap();

    let retry_config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
    };

    let mut server = AssetServer::with_config(temp_dir.path(), 2, retry_config);

    // Loader that fails twice then succeeds
    let loader = FlakyLoader::new(2);
    let loader_clone = FlakyLoader {
        fail_count: loader.fail_count.clone(),
        attempts: loader.attempts.clone(),
    };
    server.register_loader(loader);

    // Load asset
    let handle = server.load::<TestAsset>("test.flaky");

    // Wait for loading to complete
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    loop {
        server.update();
        let state = server.load_state(handle.id());

        match state {
            LoadState::Loaded => break,
            LoadState::Failed(e) => panic!("Load failed: {}", e),
            _ => {
                if start.elapsed() > timeout {
                    panic!("Load timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    // Verify asset loaded successfully
    let asset = server.get(&handle).expect("Asset should be loaded");
    assert!(asset.value.contains("Loaded after"));

    // Verify it took 3 attempts (2 failures + 1 success)
    assert_eq!(loader_clone.attempts(), 3);
}

#[test]
fn test_fallback_after_max_retries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let asset_path = temp_dir.path().join("test.flaky");
    std::fs::write(&asset_path, b"test data").unwrap();

    let retry_config = RetryConfig {
        max_retries: 2,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
    };

    let mut server = AssetServer::with_config(temp_dir.path(), 2, retry_config);

    // Register fallback asset
    server.register_fallback(TestAsset {
        value: "Fallback asset".to_string(),
    });

    // Loader that always fails
    let loader = FlakyLoader::new(10); // More failures than max retries
    server.register_loader(loader);

    // Load asset
    let handle = server.load::<TestAsset>("test.flaky");

    // Wait for loading to complete or fail
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    loop {
        server.update();
        let state = server.load_state(handle.id());

        match state {
            LoadState::Loaded => break,
            LoadState::Failed(_) => {
                // After failure, fallback should be used
                break;
            }
            _ => {
                if start.elapsed() > timeout {
                    panic!("Load timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    // Verify fallback asset is used
    let asset = server.get(&handle).expect("Fallback asset should be available");
    assert_eq!(asset.value, "Fallback asset");
}

#[test]
fn test_progress_tracking() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create multiple test files
    for i in 0..5 {
        let path = temp_dir.path().join(format!("test{}.flaky", i));
        std::fs::write(&path, b"test data").unwrap();
    }

    let retry_config = RetryConfig {
        max_retries: 2,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
    };

    let mut server = AssetServer::with_config(temp_dir.path(), 2, retry_config);

    // Register loader that succeeds immediately
    let loader = FlakyLoader::new(0);
    server.register_loader(loader);

    // Load multiple assets
    let mut handles = Vec::new();
    for i in 0..5 {
        let handle = server.load::<TestAsset>(&format!("test{}.flaky", i));
        handles.push(handle);
    }

    // Check initial progress
    let progress = server.load_progress();
    assert_eq!(progress.total, 5);
    assert_eq!(progress.loading, 5);
    assert_eq!(progress.loaded, 0);
    assert_eq!(progress.failed, 0);

    // Wait for all to load
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    loop {
        server.update();
        let progress = server.load_progress();

        if progress.loaded == 5 {
            break;
        }

        if start.elapsed() > timeout {
            panic!("Load timed out. Progress: {:?}", progress);
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    // Verify final progress
    let progress = server.load_progress();
    assert_eq!(progress.total, 5);
    assert_eq!(progress.loaded, 5);
    assert_eq!(progress.loading, 0);
    assert_eq!(progress.failed, 0);
}

#[test]
fn test_exponential_backoff_timing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let asset_path = temp_dir.path().join("test.flaky");
    std::fs::write(&asset_path, b"test data").unwrap();

    let retry_config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let mut server = AssetServer::with_config(temp_dir.path(), 2, retry_config);

    // Loader that fails twice then succeeds
    let loader = FlakyLoader::new(2);
    server.register_loader(loader);

    // Load asset and measure time
    let start = Instant::now();
    let handle = server.load::<TestAsset>("test.flaky");

    // Wait for loading to complete
    let timeout = Duration::from_secs(5);
    loop {
        server.update();
        let state = server.load_state(handle.id());

        match state {
            LoadState::Loaded => break,
            LoadState::Failed(e) => panic!("Load failed: {}", e),
            _ => {
                if start.elapsed() > timeout {
                    panic!("Load timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    let elapsed = start.elapsed();

    // Should take at least initial_delay + (initial_delay * 2) = 150ms
    // (first retry after 50ms, second retry after 100ms)
    assert!(
        elapsed >= Duration::from_millis(150),
        "Expected at least 150ms with exponential backoff, got {:?}",
        elapsed
    );
}
