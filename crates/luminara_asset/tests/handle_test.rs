use luminara_asset::{Handle, AssetId, Asset};

struct TestAsset;
impl Asset for TestAsset {
    fn type_name() -> &'static str { "TestAsset" }
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
