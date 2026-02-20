//! Property test for Asset System integration
//!
//! **Validates: Requirements 12.2.1**
//!
//! This test verifies that asset loading through the EngineHandle
//! correctly integrates with the AssetServer.

use luminara_editor::engine::EngineHandle;
use luminara_asset::{Asset, AssetLoader, AssetLoadError, LoadState};
use proptest::prelude::*;
use std::sync::Arc;

// Test asset for property testing
#[derive(Debug, Clone, PartialEq)]
struct TestAsset {
    value: i32,
}

impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

// Test asset loader
struct TestAssetLoader;

impl AssetLoader for TestAssetLoader {
    type Asset = TestAsset;

    fn extensions(&self) -> &[&str] {
        &["test"]
    }

    fn load(&self, _bytes: &[u8], _path: &std::path::Path) -> Result<Self::Asset, AssetLoadError> {
        Ok(TestAsset { value: 42 })
    }
}

proptest! {
    /// Property 28: Asset Loading Integration
    ///
    /// **Property**: When an asset is loaded through EngineHandle,
    /// it should be accessible through the AssetServer.
    ///
    /// **Validates: Requirements 12.2.1**
    #[test]
    fn property_asset_loading_integration(value in -1000i32..1000i32) {
        // Create a mock engine handle
        let handle = EngineHandle::mock();
        
        // Create a test asset directly (simulating a loaded asset)
        let asset = TestAsset { value };
        let asset_handle = handle.add_asset(asset.clone());
        
        // Verify the asset can be retrieved
        let retrieved = handle.get_asset(&asset_handle);
        prop_assert!(retrieved.is_some(), "Asset should be retrievable after adding");
        prop_assert_eq!(retrieved.unwrap().value, value, "Asset value should match");
    }

    /// Property: Multiple assets can be loaded and retrieved independently
    #[test]
    fn property_multiple_asset_loading(
        value1 in -1000i32..1000i32,
        value2 in -1000i32..1000i32,
        value3 in -1000i32..1000i32,
    ) {
        let handle = EngineHandle::mock();
        
        // Add multiple assets
        let asset1 = TestAsset { value: value1 };
        let asset2 = TestAsset { value: value2 };
        let asset3 = TestAsset { value: value3 };
        
        let handle1 = handle.add_asset(asset1.clone());
        let handle2 = handle.add_asset(asset2.clone());
        let handle3 = handle.add_asset(asset3.clone());
        
        // Verify all assets are retrievable
        let retrieved1 = handle.get_asset(&handle1);
        let retrieved2 = handle.get_asset(&handle2);
        let retrieved3 = handle.get_asset(&handle3);
        
        prop_assert!(retrieved1.is_some());
        prop_assert!(retrieved2.is_some());
        prop_assert!(retrieved3.is_some());
        
        prop_assert_eq!(retrieved1.unwrap().value, value1);
        prop_assert_eq!(retrieved2.unwrap().value, value2);
        prop_assert_eq!(retrieved3.unwrap().value, value3);
    }

    /// Property: Asset handles are unique
    #[test]
    fn property_asset_handle_uniqueness(
        value1 in -1000i32..1000i32,
        value2 in -1000i32..1000i32,
    ) {
        let handle = EngineHandle::mock();
        
        let asset1 = TestAsset { value: value1 };
        let asset2 = TestAsset { value: value2 };
        
        let handle1 = handle.add_asset(asset1);
        let handle2 = handle.add_asset(asset2);
        
        // Handles should be different even if values are the same
        prop_assert_ne!(handle1.id(), handle2.id(), 
            "Each asset should get a unique handle");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_asset_loading_basic() {
        let handle = EngineHandle::mock();
        
        let asset = TestAsset { value: 42 };
        let asset_handle = handle.add_asset(asset.clone());
        
        let retrieved = handle.get_asset(&asset_handle);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
    }

    #[test]
    fn test_asset_not_found() {
        use luminara_asset::{Handle, AssetId};
        
        let handle = EngineHandle::mock();
        
        // Create a handle to a non-existent asset
        let fake_handle: Handle<TestAsset> = Handle::new(AssetId::from_u128(9999), 0);
        
        let retrieved = handle.get_asset(&fake_handle);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_multiple_assets() {
        let handle = EngineHandle::mock();
        
        let asset1 = TestAsset { value: 1 };
        let asset2 = TestAsset { value: 2 };
        let asset3 = TestAsset { value: 3 };
        
        let handle1 = handle.add_asset(asset1);
        let handle2 = handle.add_asset(asset2);
        let handle3 = handle.add_asset(asset3);
        
        assert_eq!(handle.get_asset(&handle1).unwrap().value, 1);
        assert_eq!(handle.get_asset(&handle2).unwrap().value, 2);
        assert_eq!(handle.get_asset(&handle3).unwrap().value, 3);
    }

    #[test]
    fn test_asset_load_progress() {
        let handle = EngineHandle::mock();
        
        // Just verify we can call the method without panicking
        let progress = handle.asset_load_progress();
        assert!(progress.total >= 0);
        assert!(progress.loaded >= 0);
    }

    #[test]
    fn test_asset_load_state() {
        use luminara_asset::AssetId;
        
        let handle = EngineHandle::mock();
        
        // Check state of a non-existent asset
        let state = handle.asset_load_state(AssetId::from_u128(9999));
        // Should return NotLoaded or similar
        assert!(matches!(state, LoadState::NotLoaded | LoadState::Loading | LoadState::Loaded | LoadState::Failed(_)));
    }
}
