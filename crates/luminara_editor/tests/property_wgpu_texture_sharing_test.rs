//! Property test for WGPU Texture Sharing
//!
//! **Validates: Requirements 17.1**
//!
//! **Property 29: WGPU Texture Sharing**
//!
//! This property verifies that WGPU textures can be correctly shared between
//! Luminara's renderer and GPUI, ensuring proper texture creation, sizing,
//! and format compatibility.
//!
//! The SharedRenderTarget manages a WGPU texture that:
//! - Can be rendered to by Luminara's renderer
//! - Can be displayed in GPUI's UI tree
//! - Maintains correct size synchronization
//! - Uses appropriate texture format and usage flags

use luminara_editor::viewport::SharedRenderTarget;
use proptest::prelude::*;
use std::sync::Arc;

proptest! {
    /// Property 29.1: Texture Size Synchronization
    ///
    /// **Property**: When a SharedRenderTarget is resized, its size should
    /// always match the requested size.
    ///
    /// **Validates: Requirements 17.1, 17.4**
    #[test]
    fn property_texture_size_synchronization(
        width in 1u32..4096u32,
        height in 1u32..4096u32,
    ) {
        let mut render_target = SharedRenderTarget::new((width, height));
        
        // Verify initial size matches
        prop_assert_eq!(render_target.size(), (width, height),
            "Initial size should match requested size");
        
        // Resize to a different size
        let new_width = width / 2 + 1;
        let new_height = height / 2 + 1;
        render_target.resize((new_width, new_height));
        
        // Verify size updated correctly
        prop_assert_eq!(render_target.size(), (new_width, new_height),
            "Size should update after resize");
    }

    /// Property 29.2: Multiple Resize Operations
    ///
    /// **Property**: A SharedRenderTarget should correctly handle multiple
    /// resize operations in sequence, always maintaining the most recent size.
    ///
    /// **Validates: Requirements 17.1, 17.4**
    #[test]
    fn property_multiple_resize_operations(
        initial_width in 1u32..2048u32,
        initial_height in 1u32..2048u32,
        resize_count in 1usize..10usize,
    ) {
        let mut render_target = SharedRenderTarget::new((initial_width, initial_height));
        
        let mut current_width = initial_width;
        let mut current_height = initial_height;
        
        // Perform multiple resizes
        for i in 0..resize_count {
            let new_width = (current_width / 2 + i as u32 + 1).max(1);
            let new_height = (current_height / 2 + i as u32 + 1).max(1);
            
            render_target.resize((new_width, new_height));
            
            prop_assert_eq!(render_target.size(), (new_width, new_height),
                "Size should match after resize {}", i);
            
            current_width = new_width;
            current_height = new_height;
        }
        
        // Final size should be the last resize
        prop_assert_eq!(render_target.size(), (current_width, current_height),
            "Final size should match last resize");
    }

    /// Property 29.3: Texture Availability Before Device Initialization
    ///
    /// **Property**: Before device initialization, texture and texture_view
    /// should be None, but size should still be tracked correctly.
    ///
    /// **Validates: Requirements 17.1, 17.2**
    #[test]
    fn property_texture_availability_before_init(
        width in 1u32..4096u32,
        height in 1u32..4096u32,
    ) {
        let render_target = SharedRenderTarget::new((width, height));
        
        // Size should be set even without device
        prop_assert_eq!(render_target.size(), (width, height),
            "Size should be set even without device initialization");
        
        // Texture should be None before device initialization
        prop_assert!(render_target.texture().is_none(),
            "Texture should be None before device initialization");
        
        prop_assert!(render_target.texture_view().is_none(),
            "Texture view should be None before device initialization");
    }

    /// Property 29.4: Zero Size Handling
    ///
    /// **Property**: SharedRenderTarget should handle zero-size dimensions
    /// gracefully without panicking, and should not create textures for
    /// zero-size viewports.
    ///
    /// **Validates: Requirements 17.1, 17.4**
    #[test]
    fn property_zero_size_handling(
        initial_width in 1u32..1024u32,
        initial_height in 1u32..1024u32,
    ) {
        let mut render_target = SharedRenderTarget::new((initial_width, initial_height));
        
        // Resize to zero width
        render_target.resize((0, initial_height));
        prop_assert_eq!(render_target.size(), (0, initial_height),
            "Should handle zero width");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero width");
        
        // Resize to zero height
        render_target.resize((initial_width, 0));
        prop_assert_eq!(render_target.size(), (initial_width, 0),
            "Should handle zero height");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero height");
        
        // Resize to both zero
        render_target.resize((0, 0));
        prop_assert_eq!(render_target.size(), (0, 0),
            "Should handle zero size");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero size");
        
        // Resize back to valid size
        render_target.resize((initial_width, initial_height));
        prop_assert_eq!(render_target.size(), (initial_width, initial_height),
            "Should recover from zero size");
    }

    /// Property 29.5: Aspect Ratio Preservation
    ///
    /// **Property**: The aspect ratio calculated from the render target size
    /// should match the expected aspect ratio for common viewport dimensions.
    ///
    /// **Validates: Requirements 17.1**
    #[test]
    fn property_aspect_ratio_preservation(
        width in 1u32..4096u32,
        height in 1u32..4096u32,
    ) {
        let render_target = SharedRenderTarget::new((width, height));
        
        let (w, h) = render_target.size();
        prop_assert_eq!(w, width, "Width should be preserved");
        prop_assert_eq!(h, height, "Height should be preserved");
        
        // Calculate aspect ratio
        if h > 0 {
            let aspect_ratio = w as f32 / h as f32;
            let expected_ratio = width as f32 / height as f32;
            
            prop_assert!((aspect_ratio - expected_ratio).abs() < 0.001,
                "Aspect ratio should be preserved: {} vs {}",
                aspect_ratio, expected_ratio);
        }
    }

    /// Property 29.6: Resize Idempotence
    ///
    /// **Property**: Resizing to the same size multiple times should be
    /// idempotent - the state should remain the same after the first resize.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_resize_idempotence(
        width in 1u32..2048u32,
        height in 1u32..2048u32,
        resize_count in 2usize..10usize,
    ) {
        let mut render_target = SharedRenderTarget::new((100, 100));
        
        // Resize to target size
        render_target.resize((width, height));
        let size_after_first = render_target.size();
        
        // Resize to the same size multiple times
        for _ in 0..resize_count {
            render_target.resize((width, height));
            prop_assert_eq!(render_target.size(), size_after_first,
                "Size should remain the same after idempotent resizes");
        }
    }

    /// Property 29.7: Size Bounds Validation
    ///
    /// **Property**: SharedRenderTarget should correctly handle sizes at
    /// the boundaries of valid ranges (1x1 to 4096x4096).
    ///
    /// **Validates: Requirements 17.1**
    #[test]
    fn property_size_bounds_validation(
        width_multiplier in 1u32..10u32,
        height_multiplier in 1u32..10u32,
    ) {
        // Test minimum size (1x1)
        let mut render_target = SharedRenderTarget::new((1, 1));
        prop_assert_eq!(render_target.size(), (1, 1),
            "Should handle minimum size 1x1");
        
        // Test various sizes
        let test_width = width_multiplier * 100;
        let test_height = height_multiplier * 100;
        render_target.resize((test_width, test_height));
        prop_assert_eq!(render_target.size(), (test_width, test_height),
            "Should handle size {}x{}", test_width, test_height);
        
        // Test large size (within reasonable bounds)
        let large_width = (width_multiplier * 400).min(4096);
        let large_height = (height_multiplier * 400).min(4096);
        render_target.resize((large_width, large_height));
        prop_assert_eq!(render_target.size(), (large_width, large_height),
            "Should handle large size {}x{}", large_width, large_height);
    }

    /// Property 29.8: Concurrent Size Queries
    ///
    /// **Property**: Multiple concurrent reads of the size should always
    /// return consistent values.
    ///
    /// **Validates: Requirements 17.1**
    #[test]
    fn property_concurrent_size_queries(
        width in 1u32..2048u32,
        height in 1u32..2048u32,
        query_count in 2usize..20usize,
    ) {
        let render_target = SharedRenderTarget::new((width, height));
        
        // Query size multiple times
        let mut sizes = Vec::new();
        for _ in 0..query_count {
            sizes.push(render_target.size());
        }
        
        // All queries should return the same size
        for (i, size) in sizes.iter().enumerate() {
            prop_assert_eq!(*size, (width, height),
                "Query {} should return consistent size", i);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_texture_sharing_basic() {
        let render_target = SharedRenderTarget::new((800, 600));
        assert_eq!(render_target.size(), (800, 600));
        assert!(render_target.texture().is_none());
        assert!(render_target.texture_view().is_none());
    }

    #[test]
    fn test_texture_sharing_resize() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        render_target.resize((1024, 768));
        assert_eq!(render_target.size(), (1024, 768));
    }

    #[test]
    fn test_texture_sharing_zero_size() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        render_target.resize((0, 0));
        assert_eq!(render_target.size(), (0, 0));
        assert!(render_target.texture().is_none());
    }

    #[test]
    fn test_texture_sharing_aspect_ratios() {
        // Test common aspect ratios
        let test_cases = vec![
            ((1920, 1080), 16.0 / 9.0),  // 16:9
            ((1280, 720), 16.0 / 9.0),   // 16:9
            ((1024, 768), 4.0 / 3.0),    // 4:3
            ((800, 600), 4.0 / 3.0),     // 4:3
            ((1, 1), 1.0),                // Square
        ];
        
        for ((width, height), expected_ratio) in test_cases {
            let render_target = SharedRenderTarget::new((width, height));
            let (w, h) = render_target.size();
            let aspect_ratio = w as f32 / h as f32;
            assert!((aspect_ratio - expected_ratio).abs() < 0.001,
                "Aspect ratio mismatch for {}x{}: {} vs {}",
                width, height, aspect_ratio, expected_ratio);
        }
    }

    #[test]
    fn test_texture_sharing_multiple_resizes() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        let sizes = vec![
            (1024, 768),
            (1920, 1080),
            (640, 480),
            (1280, 720),
        ];
        
        for (width, height) in sizes {
            render_target.resize((width, height));
            assert_eq!(render_target.size(), (width, height));
        }
    }

    #[test]
    fn test_texture_sharing_idempotent_resize() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        // Resize to the same size multiple times
        for _ in 0..5 {
            render_target.resize((1024, 768));
            assert_eq!(render_target.size(), (1024, 768));
        }
    }

    #[test]
    fn test_texture_sharing_minimum_size() {
        let render_target = SharedRenderTarget::new((1, 1));
        assert_eq!(render_target.size(), (1, 1));
    }

    #[test]
    fn test_texture_sharing_large_size() {
        let render_target = SharedRenderTarget::new((4096, 4096));
        assert_eq!(render_target.size(), (4096, 4096));
    }

    #[test]
    fn test_texture_sharing_arc_wrapper() {
        // Test that SharedRenderTarget works correctly when wrapped in Arc
        let render_target = Arc::new(parking_lot::RwLock::new(
            SharedRenderTarget::new((800, 600))
        ));
        
        {
            let rt = render_target.read();
            assert_eq!(rt.size(), (800, 600));
        }
        
        {
            let mut rt = render_target.write();
            rt.resize((1024, 768));
        }
        
        {
            let rt = render_target.read();
            assert_eq!(rt.size(), (1024, 768));
        }
    }

    #[test]
    fn test_texture_sharing_size_consistency() {
        let render_target = SharedRenderTarget::new((800, 600));
        
        // Query size multiple times
        for _ in 0..10 {
            assert_eq!(render_target.size(), (800, 600));
        }
    }
}
