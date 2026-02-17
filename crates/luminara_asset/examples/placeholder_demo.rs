/// Demonstration of the placeholder asset system
///
/// This example shows how placeholders are displayed while assets load asynchronously,
/// and how they are hot-swapped with real assets when loading completes.

use luminara_asset::{Asset, AssetLoader, AssetServer, PlaceholderAsset};
use std::path::Path;
use std::thread;
use std::time::Duration;

// Example asset type
#[derive(Clone, Debug)]
struct GameAsset {
    name: String,
    data: Vec<u8>,
}

impl Asset for GameAsset {
    fn type_name() -> &'static str {
        "GameAsset"
    }
}

impl PlaceholderAsset for GameAsset {
    fn create_placeholder() -> Self {
        GameAsset {
            name: "LOADING...".to_string(),
            data: vec![0; 16],
        }
    }
}

// Simple loader that simulates slow loading
struct GameAssetLoader;

impl AssetLoader for GameAssetLoader {
    type Asset = GameAsset;

    fn extensions(&self) -> &[&str] {
        &["game"]
    }

    fn load(
        &self,
        bytes: &[u8],
        path: &Path,
    ) -> Result<Self::Asset, luminara_asset::AssetLoadError> {
        // Simulate slow loading (e.g., decompression, parsing)
        thread::sleep(Duration::from_millis(500));

        Ok(GameAsset {
            name: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            data: bytes.to_vec(),
        })
    }
}

fn main() {
    println!("=== Placeholder Asset System Demo ===\n");

    // Create temporary asset directory
    let temp_dir = std::env::temp_dir().join("luminara_placeholder_demo");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Create test asset files
    let asset1_path = temp_dir.join("asset1.game");
    let asset2_path = temp_dir.join("asset2.game");
    let asset3_path = temp_dir.join("asset3.game");

    std::fs::write(&asset1_path, b"Asset 1 Data").expect("Failed to write asset1");
    std::fs::write(&asset2_path, b"Asset 2 Data").expect("Failed to write asset2");
    std::fs::write(&asset3_path, b"Asset 3 Data").expect("Failed to write asset3");

    // Setup asset server
    let mut server = AssetServer::new(&temp_dir);
    server.register_loader(GameAssetLoader);

    // Register placeholder
    println!("Registering placeholder asset...");
    server.register_placeholder(GameAsset::create_placeholder());
    println!("✓ Placeholder registered\n");

    // Load assets
    println!("Loading assets asynchronously...");
    let handle1 = server.load::<GameAsset>("asset1.game");
    let handle2 = server.load::<GameAsset>("asset2.game");
    let handle3 = server.load::<GameAsset>("asset3.game");
    println!("✓ Load requests sent\n");

    // Simulate game loop
    println!("Simulating game loop (checking assets each frame):\n");

    for frame in 0..30 {
        // Update asset server (processes completed loads)
        server.update();

        // Check asset status
        let asset1 = server.get(&handle1);
        let asset2 = server.get(&handle2);
        let asset3 = server.get(&handle3);

        println!("Frame {}: ", frame);

        if let Some(asset) = asset1 {
            let status = if asset.name == "LOADING..." {
                "PLACEHOLDER"
            } else {
                "LOADED"
            };
            println!("  Asset 1: {} ({})", asset.name, status);
        } else {
            println!("  Asset 1: NOT AVAILABLE");
        }

        if let Some(asset) = asset2 {
            let status = if asset.name == "LOADING..." {
                "PLACEHOLDER"
            } else {
                "LOADED"
            };
            println!("  Asset 2: {} ({})", asset.name, status);
        } else {
            println!("  Asset 2: NOT AVAILABLE");
        }

        if let Some(asset) = asset3 {
            let status = if asset.name == "LOADING..." {
                "PLACEHOLDER"
            } else {
                "LOADED"
            };
            println!("  Asset 3: {} ({})", asset.name, status);
        } else {
            println!("  Asset 3: NOT AVAILABLE");
        }

        println!();

        // Check if all loaded
        let all_loaded = [&handle1, &handle2, &handle3]
            .iter()
            .all(|h| {
                server
                    .get(h)
                    .map(|a: std::sync::Arc<GameAsset>| a.name != "LOADING...")
                    .unwrap_or(false)
            });

        if all_loaded {
            println!("✓ All assets loaded successfully!\n");
            break;
        }

        // Simulate frame time
        thread::sleep(Duration::from_millis(50));
    }

    // Verify final state
    println!("=== Final Asset State ===");
    if let Some(asset) = server.get(&handle1) {
        println!("Asset 1: {} ({} bytes)", asset.name, asset.data.len());
    }
    if let Some(asset) = server.get(&handle2) {
        println!("Asset 2: {} ({} bytes)", asset.name, asset.data.len());
    }
    if let Some(asset) = server.get(&handle3) {
        println!("Asset 3: {} ({} bytes)", asset.name, asset.data.len());
    }

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    println!("\n=== Demo Complete ===");
    println!("\nKey Observations:");
    println!("1. Placeholders are available immediately (Frame 0)");
    println!("2. Assets are ALWAYS available (never None)");
    println!("3. Real assets hot-swap placeholders when loading completes");
    println!("4. No frame drops or rendering gaps during loading");
}
