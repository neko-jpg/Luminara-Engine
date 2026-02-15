use luminara_db::models::asset_meta::*;
use serde_json;

#[test]
fn test_asset_meta_serialization() {
    let meta = AssetMeta::default();
    let json = serde_json::to_string(&meta).unwrap();
    println!("Serialized: {}", json);
    let deserialized: AssetMeta = serde_json::from_str(&json).unwrap();
    // success
}
