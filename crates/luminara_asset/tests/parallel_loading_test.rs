use luminara_asset::{Asset, AssetLoader, AssetServer, LoadPriority, LoadState};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
struct TestAsset {
    data: String,
    load_order: usize,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

struct TestAssetLoader {
    load_counter: Arc<Mutex<usize>>,
    delay_ms: u64,
}

impl AssetLoader for TestAssetLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["test"]
    }

    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, luminara_asset::AssetLoadError> {
        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        
        let mut counter = self.load_counter.lock().unwrap();
        let order = *counter;
        *counter += 1;
        
        Ok(TestAsset {
            data: String::from_utf8_lossy(bytes).to_string(),
            load_order: order,
        })
    }
}

#[test]
fn test_parallel_loading_with_thread_pool() {
    // Create asset server with 4 threads
    let temp_dir = std::env::temp_dir().join("luminara_parallel_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // Create test files
    for i in 0..10 {
        let path = temp_dir.join(format!("asset_{}.test", i));
        std::fs::write(&path, format!("Asset {}", i)).unwrap();
    }
    
    let load_counter = Arc::new(Mutex::new(0));
    let mut server = AssetServer::with_thread_count(&temp_dir, 4);
    
    // Verify thread count is configured
    assert_eq!(server.thread_count(), 4);
    
    // Register loader with small delay to test parallelism
    server.register_loader(TestAssetLoader {
        load_counter: load_counter.clone(),
        delay_ms: 50,
    });
    
    // Load multiple assets
    let handles: Vec<_> = (0..10)
        .map(|i| server.load::<TestAsset>(&format!("asset_{}.test", i)))
        .collect();
    
    // Wait for all assets to load
    let start = std::time::Instant::now();
    loop {
        server.update();
        
        let all_loaded = handles.iter().all(|h| {
            matches!(server.load_state(h.id()), LoadState::Loaded)
        });
        
        if all_loaded {
            break;
        }
        
        if start.elapsed() > Duration::from_secs(10) {
            panic!("Assets took too long to load");
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }
    
    let elapsed = start.elapsed();
    
    // With 4 threads and 10 assets at 50ms each:
    // Sequential would take ~500ms
    // Parallel should take ~150ms (3 batches: 4+4+2)
    // Allow some overhead, but should be significantly faster than sequential
    assert!(elapsed < Duration::from_millis(300), 
        "Parallel loading took too long: {:?}", elapsed);
    
    // Verify all assets loaded
    for handle in &handles {
        let asset = server.get(handle).expect("Asset should be loaded");
        assert!(asset.data.starts_with("Asset "));
    }
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_priority_queue_ordering() {
    let temp_dir = std::env::temp_dir().join("luminara_priority_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // Create test files
    for i in 0..8 {
        let path = temp_dir.join(format!("asset_{}.test", i));
        std::fs::write(&path, format!("Asset {}", i)).unwrap();
    }
    
    let load_counter = Arc::new(Mutex::new(0));
    let mut server = AssetServer::with_thread_count(&temp_dir, 1); // Single thread for strict ordering
    
    server.register_loader(TestAssetLoader {
        load_counter: load_counter.clone(),
        delay_ms: 30, // Longer delay to ensure ordering is visible
    });
    
    // Submit all assets quickly so they queue up before processing starts
    let low1 = server.load_with_priority::<TestAsset>("asset_0.test", LoadPriority::Low);
    let low2 = server.load_with_priority::<TestAsset>("asset_1.test", LoadPriority::Low);
    let normal1 = server.load_with_priority::<TestAsset>("asset_2.test", LoadPriority::Normal);
    let normal2 = server.load_with_priority::<TestAsset>("asset_3.test", LoadPriority::Normal);
    let high1 = server.load_with_priority::<TestAsset>("asset_4.test", LoadPriority::High);
    let high2 = server.load_with_priority::<TestAsset>("asset_5.test", LoadPriority::High);
    let critical1 = server.load_with_priority::<TestAsset>("asset_6.test", LoadPriority::Critical);
    let critical2 = server.load_with_priority::<TestAsset>("asset_7.test", LoadPriority::Critical);
    
    // Small delay to let them all queue up
    std::thread::sleep(Duration::from_millis(20));
    
    // Wait for all to load
    let start = std::time::Instant::now();
    loop {
        server.update();
        
        let all_loaded = [&low1, &low2, &normal1, &normal2, &high1, &high2, &critical1, &critical2]
            .iter()
            .all(|h| matches!(server.load_state(h.id()), LoadState::Loaded));
        
        if all_loaded {
            break;
        }
        
        if start.elapsed() > Duration::from_secs(5) {
            panic!("Assets took too long to load");
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }
    
    // Check load order - higher priority should load first
    let critical1_order = server.get(&critical1).unwrap().load_order;
    let critical2_order = server.get(&critical2).unwrap().load_order;
    let high1_order = server.get(&high1).unwrap().load_order;
    let high2_order = server.get(&high2).unwrap().load_order;
    let normal1_order = server.get(&normal1).unwrap().load_order;
    let normal2_order = server.get(&normal2).unwrap().load_order;
    let low1_order = server.get(&low1).unwrap().load_order;
    let low2_order = server.get(&low2).unwrap().load_order;
    
    println!("Load orders:");
    println!("  Critical: {}, {}", critical1_order, critical2_order);
    println!("  High: {}, {}", high1_order, high2_order);
    println!("  Normal: {}, {}", normal1_order, normal2_order);
    println!("  Low: {}, {}", low1_order, low2_order);
    
    // Critical should load first (both critical assets should be in first 2 positions)
    assert!(critical1_order < 2, "Critical1 should be in first 2 positions");
    assert!(critical2_order < 2, "Critical2 should be in first 2 positions");
    
    // Low should load last (both low assets should be in last 2 positions)
    assert!(low1_order >= 6, "Low1 should be in last 2 positions");
    assert!(low2_order >= 6, "Low2 should be in last 2 positions");
    
    // High priority should load before low priority
    assert!(high1_order < low1_order, "High1 should load before Low1");
    assert!(high1_order < low2_order, "High1 should load before Low2");
    assert!(high2_order < low1_order, "High2 should load before Low1");
    assert!(high2_order < low2_order, "High2 should load before Low2");
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_concurrent_loading_capacity() {
    let temp_dir = std::env::temp_dir().join("luminara_concurrent_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // Create many test files
    let asset_count = 50;
    for i in 0..asset_count {
        let path = temp_dir.join(format!("asset_{}.test", i));
        std::fs::write(&path, format!("Asset {}", i)).unwrap();
    }
    
    let load_counter = Arc::new(Mutex::new(0));
    let mut server = AssetServer::with_thread_count(&temp_dir, 8);
    
    server.register_loader(TestAssetLoader {
        load_counter: load_counter.clone(),
        delay_ms: 20,
    });
    
    // Load all assets at once
    let handles: Vec<_> = (0..asset_count)
        .map(|i| server.load::<TestAsset>(&format!("asset_{}.test", i)))
        .collect();
    
    // Wait for all to load
    let start = std::time::Instant::now();
    loop {
        server.update();
        
        let all_loaded = handles.iter().all(|h| {
            matches!(server.load_state(h.id()), LoadState::Loaded)
        });
        
        if all_loaded {
            break;
        }
        
        if start.elapsed() > Duration::from_secs(10) {
            panic!("Assets took too long to load");
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }
    
    let elapsed = start.elapsed();
    
    // With 8 threads and 50 assets at 20ms each:
    // Sequential would take ~1000ms
    // Parallel should take ~140ms (7 batches: 8*6 + 2)
    // Allow overhead but should be much faster
    assert!(elapsed < Duration::from_millis(400), 
        "Concurrent loading took too long: {:?}", elapsed);
    
    // Verify all loaded correctly
    assert_eq!(handles.len(), asset_count);
    for handle in &handles {
        assert!(server.get(handle).is_some());
    }
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
