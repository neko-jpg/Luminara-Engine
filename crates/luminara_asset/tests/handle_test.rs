use luminara_asset::{Asset, AssetId, Handle};

struct TestAsset;
impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

#[test]
fn test_handle_generation_consistency() {
    let id = AssetId::new();
    let handle1 = Handle::<TestAsset>::new(id, 1);
    let handle2 = Handle::<TestAsset>::new(id, 2);

    assert_eq!(handle1.id(), handle2.id());
    assert_ne!(handle1.generation(), handle2.generation());

    // Test cloning preserves generation
    let handle1_clone = handle1.clone();
    assert_eq!(handle1.generation(), handle1_clone.generation());
}

#[test]
fn test_handle_ron_serialization() {
    let id = AssetId::new();
    let original = Handle::<TestAsset>::new(id, 5);
    
    // Serialize to RON
    let ron_string = ron::to_string(&original).expect("Failed to serialize Handle to RON");
    
    // Deserialize from RON
    let deserialized: Handle<TestAsset> = ron::from_str(&ron_string).expect("Failed to deserialize Handle from RON");
    
    // ID should match, generation defaults to 0 after deserialization
    assert_eq!(original.id(), deserialized.id());
}

#[test]
fn test_handle_binary_serialization() {
    let id = AssetId::new();
    let original = Handle::<TestAsset>::new(id, 5);
    
    // Serialize to binary
    let binary = bincode::serialize(&original).expect("Failed to serialize Handle to binary");
    
    // Deserialize from binary
    let deserialized: Handle<TestAsset> = bincode::deserialize(&binary).expect("Failed to deserialize Handle from binary");
    
    // ID should match, generation defaults to 0 after deserialization
    assert_eq!(original.id(), deserialized.id());
}

#[test]
fn test_asset_id_ron_serialization() {
    let original = AssetId::new();
    
    // Serialize to RON
    let ron_string = ron::to_string(&original).expect("Failed to serialize AssetId to RON");
    
    // Deserialize from RON
    let deserialized: AssetId = ron::from_str(&ron_string).expect("Failed to deserialize AssetId from RON");
    
    assert_eq!(original, deserialized);
}

#[test]
fn test_asset_id_binary_serialization() {
    let original = AssetId::new();
    
    // Serialize to binary
    let binary = bincode::serialize(&original).expect("Failed to serialize AssetId to binary");
    
    // Deserialize from binary
    let deserialized: AssetId = bincode::deserialize(&binary).expect("Failed to deserialize AssetId from binary");
    
    assert_eq!(original, deserialized);
}

#[test]
fn test_asset_id_from_path_serialization() {
    let path = "assets/textures/test.png";
    let original = AssetId::from_path(path);
    
    // Test RON
    let ron_string = ron::to_string(&original).expect("Failed to serialize path-based AssetId to RON");
    let deserialized: AssetId = ron::from_str(&ron_string).expect("Failed to deserialize path-based AssetId from RON");
    assert_eq!(original, deserialized);
    
    // Test binary
    let binary = bincode::serialize(&original).expect("Failed to serialize path-based AssetId to binary");
    let deserialized: AssetId = bincode::deserialize(&binary).expect("Failed to deserialize path-based AssetId from binary");
    assert_eq!(original, deserialized);
    
    // Verify deterministic ID generation from path
    let same_path_id = AssetId::from_path(path);
    assert_eq!(original, same_path_id);
}
