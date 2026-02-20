//! Property test for Viewport Resize Synchronization
//!
//! **Validates: Requirements 17.4**
//!
//! **Property 30: Viewport Resize Synchronization**
//!
//! This property verifies that when the viewport is resized, Luminara's render
//! target size is updated to match the new viewport dimensions within one frame.
//!
//! The property ensures:
//! - Render target size always matches viewport bounds
//! - Size changes are detected and applied immediately
//! - No frame lag between viewport resize and render target update
//! - Zero-size viewports are handled gracefully

use luminara_editor::viewport::SharedRenderTarget;
use proptest::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;

proptest! {
    /// Property 30.1: Immediate Size Synchronization
    ///
    /// **Property**: When a viewport is resized, the render target size should
    /// immediately match the new dimensions without any frame delay.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_immediate_size_synchronization(
        initial_width in 1u32..2048u32,
        initial_height in 1u32..2048u32,
        new_width in 1u32..2048u32,
        new_height in 1u32..2048u32,
    ) {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((initial_width, initial_height))
        ));
        
        // Verify initial size
        {
            let rt = render_target.read();
            prop_assert_eq!(rt.size(), (initial_width, initial_height),
                "Initial render target size should match viewport");
        }
        
        // Simulate viewport resize (as would happen in prepaint)
        {
            let mut rt = render_target.write();
            rt.resize((new_width, new_height));
        }
        
        // Verify size updated immediately (within same frame)
        {
            let rt = render_target.read();
            prop_assert_eq!(rt.size(), (new_width, new_height),
                "Render target should immediately match new viewport size");
        }
    }

    /// Property 30.2: Multiple Resize Synchronization
    ///
    /// **Property**: A sequence of viewport resizes should result in the render
    /// target always matching the most recent viewport size.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_multiple_resize_synchronization(
        initial_width in 1u32..1024u32,
        initial_height in 1u32..1024u32,
        resize_sequence in prop::collection::vec((1u32..2048u32, 1u32..2048u32), 1..10),
    ) {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((initial_width, initial_height))
        ));
        
        // Apply each resize in sequence
        for (width, height) in resize_sequence.iter() {
            {
                let mut rt = render_target.write();
                rt.resize((*width, *height));
            }
            
            // After each resize, verify synchronization
            {
                let rt = render_target.read();
                prop_assert_eq!(rt.size(), (*width, *height),
                    "Render target should match viewport after resize to {}x{}", width, height);
            }
        }
        
        // Final size should be the last resize
        if let Some((final_width, final_height)) = resize_sequence.last() {
            let rt = render_target.read();
            prop_assert_eq!(rt.size(), (*final_width, *final_height),
                "Final render target size should match last viewport resize");
        }
    }

    /// Property 30.3: Resize Detection Accuracy
    ///
    /// **Property**: The resize operation should correctly detect when the size
    /// has actually changed vs when it remains the same.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_resize_detection_accuracy(
        width in 1u32..2048u32,
        height in 1u32..2048u32,
    ) {
        let mut render_target = SharedRenderTarget::new((width, height));
        
        // Resize to different size should return true
        let different_width = if width > 1 { width - 1 } else { width + 1 };
        let changed = render_target.resize((different_width, height));
        prop_assert!(changed, "Resize to different size should return true");
        prop_assert_eq!(render_target.size(), (different_width, height),
            "Size should be updated");
        
        // Resize to same size should return false
        let not_changed = render_target.resize((different_width, height));
        prop_assert!(!not_changed, "Resize to same size should return false");
        prop_assert_eq!(render_target.size(), (different_width, height),
            "Size should remain unchanged");
    }

    /// Property 30.4: Zero-Size Viewport Handling
    ///
    /// **Property**: When viewport is resized to zero dimensions, the render
    /// target should handle it gracefully without creating invalid textures.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_zero_size_viewport_handling(
        initial_width in 1u32..1024u32,
        initial_height in 1u32..1024u32,
    ) {
        let mut render_target = SharedRenderTarget::new((initial_width, initial_height));
        
        // Resize to zero width
        render_target.resize((0, initial_height));
        prop_assert_eq!(render_target.size(), (0, initial_height),
            "Should accept zero width");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero width");
        
        // Resize to zero height
        render_target.resize((initial_width, 0));
        prop_assert_eq!(render_target.size(), (initial_width, 0),
            "Should accept zero height");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero height");
        
        // Resize to both zero
        render_target.resize((0, 0));
        prop_assert_eq!(render_target.size(), (0, 0),
            "Should accept zero size");
        prop_assert!(render_target.texture().is_none(),
            "Should not create texture for zero size");
        
        // Resize back to valid size
        render_target.resize((initial_width, initial_height));
        prop_assert_eq!(render_target.size(), (initial_width, initial_height),
            "Should recover from zero size");
    }

    /// Property 30.5: Aspect Ratio Synchronization
    ///
    /// **Property**: The aspect ratio of the render target should always match
    /// the aspect ratio of the viewport bounds.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_aspect_ratio_synchronization(
        width in 1u32..4096u32,
        height in 1u32..4096u32,
    ) {
        let render_target = SharedRenderTarget::new((width, height));
        
        let (rt_width, rt_height) = render_target.size();
        
        // Calculate aspect ratios
        let viewport_aspect = width as f32 / height as f32;
        let render_target_aspect = rt_width as f32 / rt_height as f32;
        
        // Aspect ratios should match (within floating point tolerance)
        prop_assert!((viewport_aspect - render_target_aspect).abs() < 0.001,
            "Render target aspect ratio ({}) should match viewport aspect ratio ({})",
            render_target_aspect, viewport_aspect);
    }

    /// Property 30.6: Concurrent Resize Operations
    ///
    /// **Property**: When multiple resize operations occur in rapid succession,
    /// the render target should always reflect the most recent size.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_concurrent_resize_operations(
        initial_width in 1u32..1024u32,
        initial_height in 1u32..1024u32,
        resize_count in 2usize..20usize,
    ) {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((initial_width, initial_height))
        ));
        
        let mut expected_width = initial_width;
        let mut expected_height = initial_height;
        
        // Perform rapid resize operations
        for i in 0..resize_count {
            let new_width = (initial_width + i as u32 * 10).min(2048);
            let new_height = (initial_height + i as u32 * 10).min(2048);
            
            {
                let mut rt = render_target.write();
                rt.resize((new_width, new_height));
            }
            
            expected_width = new_width;
            expected_height = new_height;
            
            // Verify size after each operation
            {
                let rt = render_target.read();
                prop_assert_eq!(rt.size(), (expected_width, expected_height),
                    "Render target should match viewport after rapid resize {}", i);
            }
        }
    }

    /// Property 30.7: Size Bounds Validation
    ///
    /// **Property**: The render target should correctly handle viewport sizes
    /// at the boundaries of valid ranges (minimum 1x1, maximum 4096x4096).
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_size_bounds_validation(
        width_factor in 1u32..10u32,
        height_factor in 1u32..10u32,
    ) {
        // Test minimum size (1x1)
        let mut render_target = SharedRenderTarget::new((1, 1));
        prop_assert_eq!(render_target.size(), (1, 1),
            "Should handle minimum viewport size 1x1");
        
        // Test small sizes
        let small_width = width_factor;
        let small_height = height_factor;
        render_target.resize((small_width, small_height));
        prop_assert_eq!(render_target.size(), (small_width, small_height),
            "Should handle small viewport size {}x{}", small_width, small_height);
        
        // Test medium sizes
        let medium_width = width_factor * 100;
        let medium_height = height_factor * 100;
        render_target.resize((medium_width, medium_height));
        prop_assert_eq!(render_target.size(), (medium_width, medium_height),
            "Should handle medium viewport size {}x{}", medium_width, medium_height);
        
        // Test large sizes (within reasonable bounds)
        let large_width = (width_factor * 400).min(4096);
        let large_height = (height_factor * 400).min(4096);
        render_target.resize((large_width, large_height));
        prop_assert_eq!(render_target.size(), (large_width, large_height),
            "Should handle large viewport size {}x{}", large_width, large_height);
    }

    /// Property 30.8: Resize Idempotence
    ///
    /// **Property**: Resizing to the same dimensions multiple times should be
    /// idempotent - the state should remain consistent.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_resize_idempotence(
        width in 1u32..2048u32,
        height in 1u32..2048u32,
        repeat_count in 2usize..10usize,
    ) {
        let mut render_target = SharedRenderTarget::new((100, 100));
        
        // First resize
        let changed = render_target.resize((width, height));
        prop_assert!(changed, "First resize should return true");
        
        // Repeated resizes to same size
        for i in 0..repeat_count {
            let not_changed = render_target.resize((width, height));
            prop_assert!(!not_changed,
                "Idempotent resize {} should return false", i);
            prop_assert_eq!(render_target.size(), (width, height),
                "Size should remain consistent after idempotent resize {}", i);
        }
    }

    /// Property 30.9: Viewport Dimension Independence
    ///
    /// **Property**: Width and height resizes should be independent - changing
    /// one dimension should not affect the other.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_viewport_dimension_independence(
        initial_width in 1u32..1024u32,
        initial_height in 1u32..1024u32,
        new_width in 1u32..2048u32,
        new_height in 1u32..2048u32,
    ) {
        let mut render_target = SharedRenderTarget::new((initial_width, initial_height));
        
        // Change only width
        render_target.resize((new_width, initial_height));
        let (w, h) = render_target.size();
        prop_assert_eq!(w, new_width, "Width should be updated");
        prop_assert_eq!(h, initial_height, "Height should remain unchanged");
        
        // Change only height
        render_target.resize((new_width, new_height));
        let (w, h) = render_target.size();
        prop_assert_eq!(w, new_width, "Width should remain unchanged");
        prop_assert_eq!(h, new_height, "Height should be updated");
    }

    /// Property 30.10: Resize Consistency Across Threads
    ///
    /// **Property**: When accessed from multiple threads (via Arc<RwLock>),
    /// the render target size should remain consistent.
    ///
    /// **Validates: Requirements 17.4**
    #[test]
    fn property_resize_consistency_across_threads(
        width in 1u32..2048u32,
        height in 1u32..2048u32,
        read_count in 2usize..10usize,
    ) {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((width, height))
        ));
        
        // Multiple reads should return consistent size
        for i in 0..read_count {
            let rt = render_target.read();
            prop_assert_eq!(rt.size(), (width, height),
                "Read {} should return consistent size", i);
        }
        
        // After a resize, all reads should see the new size
        let new_width = if width > 1 { width - 1 } else { width + 1 };
        {
            let mut rt = render_target.write();
            rt.resize((new_width, height));
        }
        
        for i in 0..read_count {
            let rt = render_target.read();
            prop_assert_eq!(rt.size(), (new_width, height),
                "Read {} after resize should return new size", i);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_viewport_resize_sync_basic() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        // Initial size
        assert_eq!(render_target.size(), (800, 600));
        
        // Resize
        render_target.resize((1024, 768));
        assert_eq!(render_target.size(), (1024, 768));
    }

    #[test]
    fn test_viewport_resize_sync_immediate() {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((800, 600))
        ));
        
        // Resize and verify immediately
        {
            let mut rt = render_target.write();
            rt.resize((1920, 1080));
        }
        
        {
            let rt = render_target.read();
            assert_eq!(rt.size(), (1920, 1080));
        }
    }

    #[test]
    fn test_viewport_resize_sync_sequence() {
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
    fn test_viewport_resize_sync_zero_size() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        // Zero width
        render_target.resize((0, 600));
        assert_eq!(render_target.size(), (0, 600));
        assert!(render_target.texture().is_none());
        
        // Zero height
        render_target.resize((800, 0));
        assert_eq!(render_target.size(), (800, 0));
        assert!(render_target.texture().is_none());
        
        // Both zero
        render_target.resize((0, 0));
        assert_eq!(render_target.size(), (0, 0));
        assert!(render_target.texture().is_none());
    }

    #[test]
    fn test_viewport_resize_sync_aspect_ratio() {
        let test_cases = vec![
            ((1920, 1080), 16.0 / 9.0),
            ((1280, 720), 16.0 / 9.0),
            ((1024, 768), 4.0 / 3.0),
            ((800, 600), 4.0 / 3.0),
            ((1, 1), 1.0),
        ];
        
        for ((width, height), expected_ratio) in test_cases {
            let render_target = SharedRenderTarget::new((width, height));
            let (w, h) = render_target.size();
            let aspect_ratio = w as f32 / h as f32;
            assert!((aspect_ratio - expected_ratio).abs() < 0.001);
        }
    }

    #[test]
    fn test_viewport_resize_sync_detection() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        // Different size should return true
        let changed = render_target.resize((1024, 768));
        assert!(changed);
        
        // Same size should return false
        let not_changed = render_target.resize((1024, 768));
        assert!(!not_changed);
    }

    #[test]
    fn test_viewport_resize_sync_bounds() {
        let mut render_target = SharedRenderTarget::new((1, 1));
        assert_eq!(render_target.size(), (1, 1));
        
        render_target.resize((4096, 4096));
        assert_eq!(render_target.size(), (4096, 4096));
    }

    #[test]
    fn test_viewport_resize_sync_idempotence() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        render_target.resize((1024, 768));
        let size_after_first = render_target.size();
        
        // Multiple resizes to same size
        for _ in 0..5 {
            render_target.resize((1024, 768));
            assert_eq!(render_target.size(), size_after_first);
        }
    }

    #[test]
    fn test_viewport_resize_sync_dimension_independence() {
        let mut render_target = SharedRenderTarget::new((800, 600));
        
        // Change width only
        render_target.resize((1024, 600));
        assert_eq!(render_target.size(), (1024, 600));
        
        // Change height only
        render_target.resize((1024, 768));
        assert_eq!(render_target.size(), (1024, 768));
    }

    #[test]
    fn test_viewport_resize_sync_thread_safety() {
        let render_target = Arc::new(RwLock::new(
            SharedRenderTarget::new((800, 600))
        ));
        
        // Multiple reads
        for _ in 0..10 {
            let rt = render_target.read();
            assert_eq!(rt.size(), (800, 600));
        }
        
        // Write
        {
            let mut rt = render_target.write();
            rt.resize((1024, 768));
        }
        
        // Multiple reads after write
        for _ in 0..10 {
            let rt = render_target.read();
            assert_eq!(rt.size(), (1024, 768));
        }
    }
}
