use luminara_asset::*;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
struct MyAsset {
    data: String,
}

impl Asset for MyAsset {
    fn type_name() -> &'static str {
        "MyAsset"
    }
}

struct MyLoader;
impl AssetLoader for MyLoader {
    type Asset = MyAsset;
    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
    fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
        Ok(MyAsset {
            data: String::from_utf8_lossy(bytes).to_string(),
        })
    }
}

#[test]
fn test_handle_type_safety() {
    let id = AssetId::new();
    let _handle: Handle<MyAsset> = Handle::new(id);
    // This would fail to compile if types didn't match:
    // let _other: Handle<u32> = Handle::new(id);
}

#[test]
fn test_asset_storage_crud() {
    let mut storage = AssetStorage::<MyAsset>::new();
    let id = AssetId::new();
    let asset = MyAsset {
        data: "hello".to_string(),
    };

    storage.insert(id, asset);
    assert!(storage.contains(id));
    assert_eq!(storage.get(id).unwrap().data, "hello");

    storage.get_mut(id).unwrap().data = "world".to_string();
    assert_eq!(storage.get(id).unwrap().data, "world");

    let removed = storage.remove(id).unwrap();
    assert_eq!(removed.data, "world");
    assert!(!storage.contains(id));
}

#[test]
fn test_asset_id_generation_consistency() {
    let path = "assets/models/cube.obj";
    let id1 = AssetId::from_path(path);
    let id2 = AssetId::from_path(path);
    let id3 = AssetId::from_path("assets/models/sphere.obj");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_asset_server_loading() {
    let test_dir = std::env::temp_dir().join("luminara_test_assets");
    fs::create_dir_all(&test_dir).unwrap();
    let file_path = test_dir.join("test.txt");
    fs::write(&file_path, "asset data").unwrap();

    let mut server = AssetServer::new(&test_dir);
    server.register_loader(MyLoader);

    let handle: Handle<MyAsset> = server.load("test.txt");
    assert_eq!(server.load_state(handle.id()), LoadState::Loaded);

    // Verify asset retrieval
    let asset = server.get(&handle).expect("Asset should be retrievable");
    assert_eq!(asset.data, "asset data");

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_asset_server_missing_file() {
    let server = AssetServer::new("non_existent_dir");
    let handle: Handle<MyAsset> = server.load("missing.txt");
    match server.load_state(handle.id()) {
        LoadState::Failed(err) => {
            assert!(err.contains("No loader") || err.contains("entity not found"))
        }
        _ => panic!("Should have failed"),
    }
}

#[test]
fn test_multiple_extensions() {
    struct MultiLoader;
    impl AssetLoader for MultiLoader {
        type Asset = MyAsset;
        fn extensions(&self) -> &[&str] {
            &["txt", "md"]
        }
        fn load(&self, bytes: &[u8], _path: &Path) -> Result<Self::Asset, AssetLoadError> {
            Ok(MyAsset {
                data: String::from_utf8_lossy(bytes).to_string(),
            })
        }
    }

    let mut server = AssetServer::new("assets");
    server.register_loader(MultiLoader);
    // Just verify they are registered (load would fail due to missing file, but that's fine for this test)
    let handle1: Handle<MyAsset> = server.load("test.txt");
    let handle2: Handle<MyAsset> = server.load("test.md");

    // If they were not registered, state would be Failed("No loader for extension ...")
    // If they were registered, it would be Failed("... No such file or directory ...")
    match server.load_state(handle1.id()) {
        LoadState::Failed(err) => assert!(!err.contains("No loader")),
        _ => {}
    }
    match server.load_state(handle2.id()) {
        LoadState::Failed(err) => assert!(!err.contains("No loader")),
        _ => {}
    }
}
