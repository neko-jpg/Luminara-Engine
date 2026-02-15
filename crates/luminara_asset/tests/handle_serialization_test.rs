use luminara_asset::{Asset, AssetId, Handle};
use serde::{Deserialize, Serialize};

// Mock asset type for testing
#[derive(Debug, Clone)]
struct TestAsset {
    data: String,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

#[test]
fn test_handle_ron_serialization() {
    let asset_id = AssetId::from_path("test/asset.png");
    let handle: Handle<TestAsset> = Handle::new(asset_id, 1);

    // Serialize to RON
    let ron_str = ron::to_string(&handle).expect("Failed to serialize Handle to RON");
    println!("Handle RON: {}", ron_str);

    // Deserialize from RON
    let deserialized: Handle<TestAsset> =
        ron::from_str(&ron_str).expect("Failed to deserialize Handle from RON");

    // Handles should have the same ID
    assert_eq!(handle.id(), deserialized.id());
}

#[test]
fn test_handle_binary_serialization() {
    let asset_id = AssetId::from_path("test/asset.png");
    let handle: Handle<TestAsset> = Handle::new(asset_id, 1);

    // Serialize to binary
    let binary = bincode::serialize(&handle).expect("Failed to serialize Handle to binary");

    // Deserialize from binary
    let deserialized: Handle<TestAsset> =
        bincode::deserialize(&binary).expect("Failed to deserialize Handle from binary");

    // Handles should have the same ID
    assert_eq!(handle.id(), deserialized.id());
}

#[test]
fn test_asset_id_ron_serialization() {
    let asset_id = AssetId::from_path("test/texture.png");

    // Serialize to RON
    let ron_str = ron::to_string(&asset_id).expect("Failed to serialize AssetId to RON");
    println!("AssetId RON: {}", ron_str);

    // Deserialize from RON
    let deserialized: AssetId =
        ron::from_str(&ron_str).expect("Failed to deserialize AssetId from RON");

    assert_eq!(asset_id, deserialized);
}

#[test]
fn test_asset_id_binary_serialization() {
    let asset_id = AssetId::from_path("test/texture.png");

    // Serialize to binary
    let binary = bincode::serialize(&asset_id).expect("Failed to serialize AssetId to binary");

    // Deserialize from binary
    let deserialized: AssetId =
        bincode::deserialize(&binary).expect("Failed to deserialize AssetId from binary");

    assert_eq!(asset_id, deserialized);
}

#[test]
fn test_handle_clone_and_serialize() {
    let asset_id = AssetId::from_path("test/model.gltf");
    let handle: Handle<TestAsset> = Handle::new(asset_id, 2);
    let cloned_handle = handle.clone();

    // Both handles should serialize to the same value
    let ron1 = ron::to_string(&handle).unwrap();
    let ron2 = ron::to_string(&cloned_handle).unwrap();
    assert_eq!(ron1, ron2);

    let binary1 = bincode::serialize(&handle).unwrap();
    let binary2 = bincode::serialize(&cloned_handle).unwrap();
    assert_eq!(binary1, binary2);
}

#[test]
fn test_multiple_handles_different_types() {
    // Create handles for different asset types
    #[derive(Debug, Clone)]
    struct TextureAsset;
    impl Asset for TextureAsset {
        fn type_name() -> &'static str {
            "TextureAsset"
        }
    }

    #[derive(Debug, Clone)]
    struct MeshAsset;
    impl Asset for MeshAsset {
        fn type_name() -> &'static str {
            "MeshAsset"
        }
    }

    let texture_id = AssetId::from_path("texture.png");
    let mesh_id = AssetId::from_path("mesh.obj");

    let texture_handle: Handle<TextureAsset> = Handle::new(texture_id, 0);
    let mesh_handle: Handle<MeshAsset> = Handle::new(mesh_id, 0);

    // Serialize both
    let texture_ron = ron::to_string(&texture_handle).unwrap();
    let mesh_ron = ron::to_string(&mesh_handle).unwrap();

    println!("Texture Handle RON: {}", texture_ron);
    println!("Mesh Handle RON: {}", mesh_ron);

    // Deserialize both
    let texture_deserialized: Handle<TextureAsset> = ron::from_str(&texture_ron).unwrap();
    let mesh_deserialized: Handle<MeshAsset> = ron::from_str(&mesh_ron).unwrap();

    assert_eq!(texture_handle.id(), texture_deserialized.id());
    assert_eq!(mesh_handle.id(), mesh_deserialized.id());
}
