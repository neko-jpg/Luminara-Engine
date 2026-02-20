//! Property test for Mouse Event Handling
//!
//! **Validates: Requirements 16.5**
//!
//! **Property 46: Mouse Event Handling**
//!
//! This property verifies that the ViewportElement correctly handles mouse events,
//! including button press/release, mouse movement, and drag operations.
//!
//! The property ensures:
//! - Mouse button state transitions (pressed → released) are tracked correctly
//! - Mouse position tracking and delta calculations are accurate
//! - Drag mode transitions (None → Orbit/Pan/Zoom → None) work correctly
//! - Events are properly routed to Luminara's input system

use luminara_editor::viewport::{ViewportElement, SharedRenderTarget, Camera};
use gpui::{px, Point, MouseButton, Pixels};
use proptest::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;

/// Helper to create a test viewport element
fn create_test_viewport() -> ViewportElement {
    let render_target = Arc::new(RwLock::new(SharedRenderTarget::new((800, 600))));
    let camera = Arc::new(RwLock::new(Camera::new()));
    ViewportElement::new(
        render_target,
        camera,
        luminara_editor::viewport::GizmoMode::None,
    )
}

proptest! {
    /// Property 46.1: Mouse Button State Transitions
    ///
    /// **Property**: When a mouse button is pressed and then released,
    /// the viewport should correctly track the state transition.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_mouse_button_state_transitions(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        // Initially not dragging
        prop_assert!(!viewport.is_dragging);
        
        // Start drag (button pressed)
        viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
        prop_assert!(viewport.is_dragging, "Should be dragging after mouse down");
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Orbit);
        prop_assert_eq!(viewport.last_mouse_pos, Some(position));
        
        // Stop drag (button released)
        viewport.stop_drag(MouseButton::Left);
        prop_assert!(!viewport.is_dragging, "Should not be dragging after mouse up");
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
        prop_assert_eq!(viewport.last_mouse_pos, None);
    }

    /// Property 46.2: Mouse Position Tracking
    ///
    /// **Property**: The viewport should accurately track mouse position
    /// during drag operations.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_mouse_position_tracking(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        end_x in 0.0f32..800.0f32,
        end_y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let start_pos = Point::new(px(start_x), px(start_y));
        let end_pos = Point::new(px(end_x), px(end_y));
        
        // Start drag
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        prop_assert_eq!(viewport.last_mouse_pos, Some(start_pos),
            "Last mouse position should be start position");
        
        // Update drag
        viewport.update_drag(end_pos);
        prop_assert_eq!(viewport.last_mouse_pos, Some(end_pos),
            "Last mouse position should be updated to end position");
    }

    /// Property 46.3: Mouse Delta Calculation
    ///
    /// **Property**: The viewport should correctly calculate mouse movement
    /// deltas during drag operations.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_mouse_delta_calculation(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        delta_x in -100.0f32..100.0f32,
        delta_y in -100.0f32..100.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        let start_pos = Point::new(px(start_x), px(start_y));
        let end_x = (start_x + delta_x).clamp(0.0, 800.0);
        let end_y = (start_y + delta_y).clamp(0.0, 600.0);
        let end_pos = Point::new(px(end_x), px(end_y));
        
        // Record initial camera position
        let initial_camera_pos = camera.read().position;
        
        // Start drag and update
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        viewport.update_drag(end_pos);
        
        // Camera should have changed (orbit was applied)
        let final_camera_pos = camera.read().position;
        
        // If there was any movement, camera should have changed
        if delta_x.abs() > 0.01 || delta_y.abs() > 0.01 {
            prop_assert!(
                (final_camera_pos.x - initial_camera_pos.x).abs() > 0.0001 ||
                (final_camera_pos.y - initial_camera_pos.y).abs() > 0.0001 ||
                (final_camera_pos.z - initial_camera_pos.z).abs() > 0.0001,
                "Camera should change when mouse moves during orbit"
            );
        }
    }


    /// Property 46.4: Drag Mode Transitions - Orbit
    ///
    /// **Property**: Left mouse button should activate Orbit drag mode,
    /// and releasing should return to None.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_drag_mode_orbit_transition(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        // Initial state
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
        
        // Start orbit drag
        viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Orbit,
            "Drag mode should be Orbit after left mouse down");
        
        // Stop drag
        viewport.stop_drag(MouseButton::Left);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None,
            "Drag mode should return to None after mouse up");
    }

    /// Property 46.5: Drag Mode Transitions - Pan
    ///
    /// **Property**: Middle mouse button should activate Pan drag mode,
    /// and releasing should return to None.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_drag_mode_pan_transition(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        // Initial state
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
        
        // Start pan drag
        viewport.start_drag(MouseButton::Middle, position, luminara_editor::viewport::DragMode::Pan);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Pan,
            "Drag mode should be Pan after middle mouse down");
        
        // Stop drag
        viewport.stop_drag(MouseButton::Middle);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None,
            "Drag mode should return to None after mouse up");
    }

    /// Property 46.6: Drag Mode Transitions - Zoom
    ///
    /// **Property**: Right mouse button should activate Zoom drag mode,
    /// and releasing should return to None.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_drag_mode_zoom_transition(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        // Initial state
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
        
        // Start zoom drag
        viewport.start_drag(MouseButton::Right, position, luminara_editor::viewport::DragMode::Zoom);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Zoom,
            "Drag mode should be Zoom after right mouse down");
        
        // Stop drag
        viewport.stop_drag(MouseButton::Right);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None,
            "Drag mode should return to None after mouse up");
    }


    /// Property 46.7: Multiple Drag Operations
    ///
    /// **Property**: Multiple sequential drag operations should each
    /// correctly transition through states.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_multiple_drag_operations(
        positions in prop::collection::vec((0.0f32..800.0f32, 0.0f32..600.0f32), 2..10),
    ) {
        let mut viewport = create_test_viewport();
        
        for (x, y) in positions {
            let position = Point::new(px(x), px(y));
            
            // Start drag
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            prop_assert!(viewport.is_dragging, "Should be dragging");
            prop_assert_eq!(viewport.last_mouse_pos, Some(position));
            
            // Stop drag
            viewport.stop_drag(MouseButton::Left);
            prop_assert!(!viewport.is_dragging, "Should not be dragging");
            prop_assert_eq!(viewport.last_mouse_pos, None);
        }
    }

    /// Property 46.8: Drag Without Initial Press
    ///
    /// **Property**: Updating drag without starting should not affect state.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_drag_without_initial_press(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        // Try to update drag without starting
        viewport.update_drag(position);
        
        // Should still not be dragging
        prop_assert!(!viewport.is_dragging);
        prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
    }

    /// Property 46.9: Orbit Camera Behavior
    ///
    /// **Property**: During orbit drag, the camera should rotate around
    /// the target while maintaining distance.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_orbit_camera_behavior(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        delta_x in -50.0f32..50.0f32,
        delta_y in -50.0f32..50.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        // Record initial state
        let initial_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        let start_pos = Point::new(px(start_x), px(start_y));
        let end_x = (start_x + delta_x).clamp(0.0, 800.0);
        let end_y = (start_y + delta_y).clamp(0.0, 600.0);
        let end_pos = Point::new(px(end_x), px(end_y));
        
        // Perform orbit drag
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        viewport.update_drag(end_pos);
        
        // Check distance is maintained (within tolerance)
        let final_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        prop_assert!(
            (final_distance - initial_distance).abs() < 0.1,
            "Camera distance from target should be maintained during orbit (initial: {}, final: {})",
            initial_distance, final_distance
        );
    }


    /// Property 46.10: Pan Camera Behavior
    ///
    /// **Property**: During pan drag, both camera position and target
    /// should move together, maintaining their relative offset.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_pan_camera_behavior(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        delta_x in -50.0f32..50.0f32,
        delta_y in -50.0f32..50.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        // Record initial offset
        let initial_offset = {
            let cam = camera.read();
            cam.position - cam.target
        };
        
        let start_pos = Point::new(px(start_x), px(start_y));
        let end_x = (start_x + delta_x).clamp(0.0, 800.0);
        let end_y = (start_y + delta_y).clamp(0.0, 600.0);
        let end_pos = Point::new(px(end_x), px(end_y));
        
        // Perform pan drag
        viewport.start_drag(MouseButton::Middle, start_pos, luminara_editor::viewport::DragMode::Pan);
        viewport.update_drag(end_pos);
        
        // Check offset is maintained
        let final_offset = {
            let cam = camera.read();
            cam.position - cam.target
        };
        
        prop_assert!(
            (final_offset.x - initial_offset.x).abs() < 0.01 &&
            (final_offset.y - initial_offset.y).abs() < 0.01 &&
            (final_offset.z - initial_offset.z).abs() < 0.01,
            "Camera offset should be maintained during pan"
        );
    }

    /// Property 46.11: Zoom Camera Behavior
    ///
    /// **Property**: During zoom drag, the camera should move closer to
    /// or farther from the target along the view direction.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_zoom_camera_behavior(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        delta_y in -50.0f32..50.0f32,
    ) {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        // Record initial distance
        let initial_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        let start_pos = Point::new(px(start_x), px(start_y));
        let end_y = (start_y + delta_y).clamp(0.0, 600.0);
        let end_pos = Point::new(px(start_x), px(end_y));
        
        // Perform zoom drag
        viewport.start_drag(MouseButton::Right, start_pos, luminara_editor::viewport::DragMode::Zoom);
        viewport.update_drag(end_pos);
        
        // Check distance changed
        let final_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        // If there was vertical movement, distance should change
        if delta_y.abs() > 1.0 {
            prop_assert!(
                (final_distance - initial_distance).abs() > 0.001,
                "Camera distance should change during zoom with delta_y = {}",
                delta_y
            );
        }
        
        // Distance should never go below minimum (0.1)
        prop_assert!(
            final_distance >= 0.1,
            "Camera distance should not go below minimum (0.1), got {}",
            final_distance
        );
    }


    /// Property 46.12: Drag State Consistency
    ///
    /// **Property**: The drag state should remain consistent throughout
    /// a drag operation - if dragging starts, it should remain true until
    /// explicitly stopped.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_drag_state_consistency(
        start_x in 0.0f32..800.0f32,
        start_y in 0.0f32..600.0f32,
        move_sequence in prop::collection::vec((0.0f32..800.0f32, 0.0f32..600.0f32), 2..10),
    ) {
        let mut viewport = create_test_viewport();
        let start_pos = Point::new(px(start_x), px(start_y));
        
        // Start drag
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        prop_assert!(viewport.is_dragging);
        
        // Perform multiple moves - should remain dragging
        for (x, y) in move_sequence {
            viewport.update_drag(Point::new(px(x), px(y)));
            prop_assert!(viewport.is_dragging, "Should remain dragging during move sequence");
            prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Orbit,
                "Drag mode should remain Orbit during move sequence");
        }
        
        // Stop drag
        viewport.stop_drag(MouseButton::Left);
        prop_assert!(!viewport.is_dragging);
    }

    /// Property 46.13: Mouse Position Bounds
    ///
    /// **Property**: The viewport should handle mouse positions at the
    /// boundaries of the viewport area without errors.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_mouse_position_bounds(
        edge_case in prop::sample::select(vec![
            (0.0, 0.0),           // Top-left corner
            (800.0, 0.0),         // Top-right corner
            (0.0, 600.0),         // Bottom-left corner
            (800.0, 600.0),       // Bottom-right corner
            (400.0, 0.0),         // Top edge
            (400.0, 600.0),       // Bottom edge
            (0.0, 300.0),         // Left edge
            (800.0, 300.0),       // Right edge
        ]),
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(edge_case.0), px(edge_case.1));
        
        // Should handle edge positions without panic
        viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
        prop_assert!(viewport.is_dragging);
        prop_assert_eq!(viewport.last_mouse_pos, Some(position));
        
        viewport.stop_drag(MouseButton::Left);
        prop_assert!(!viewport.is_dragging);
    }

    /// Property 46.14: Rapid Button Press/Release
    ///
    /// **Property**: Rapid sequences of button press and release should
    /// be handled correctly without state corruption.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_rapid_button_press_release(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
        repeat_count in 2usize..10usize,
    ) {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(x), px(y));
        
        for _ in 0..repeat_count {
            // Press
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            prop_assert!(viewport.is_dragging);
            
            // Release
            viewport.stop_drag(MouseButton::Left);
            prop_assert!(!viewport.is_dragging);
            prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
        }
    }


    /// Property 46.15: Different Button Combinations
    ///
    /// **Property**: Each mouse button should independently control its
    /// respective drag mode without interference.
    ///
    /// **Validates: Requirements 16.5**
    #[test]
    fn property_different_button_combinations(
        x in 0.0f32..800.0f32,
        y in 0.0f32..600.0f32,
    ) {
        let position = Point::new(px(x), px(y));
        
        // Test Left button -> Orbit
        {
            let mut viewport = create_test_viewport();
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Orbit);
            viewport.stop_drag(MouseButton::Left);
        }
        
        // Test Middle button -> Pan
        {
            let mut viewport = create_test_viewport();
            viewport.start_drag(MouseButton::Middle, position, luminara_editor::viewport::DragMode::Pan);
            prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Pan);
            viewport.stop_drag(MouseButton::Middle);
        }
        
        // Test Right button -> Zoom
        {
            let mut viewport = create_test_viewport();
            viewport.start_drag(MouseButton::Right, position, luminara_editor::viewport::DragMode::Zoom);
            prop_assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Zoom);
            viewport.stop_drag(MouseButton::Right);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_mouse_button_press_release() {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(100.0), px(100.0));
        
        assert!(!viewport.is_dragging);
        
        viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
        assert!(viewport.is_dragging);
        
        viewport.stop_drag(MouseButton::Left);
        assert!(!viewport.is_dragging);
    }

    #[test]
    fn test_mouse_position_tracking() {
        let mut viewport = create_test_viewport();
        let start_pos = Point::new(px(100.0), px(100.0));
        let end_pos = Point::new(px(200.0), px(200.0));
        
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        assert_eq!(viewport.last_mouse_pos, Some(start_pos));
        
        viewport.update_drag(end_pos);
        assert_eq!(viewport.last_mouse_pos, Some(end_pos));
    }

    #[test]
    fn test_drag_mode_orbit() {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(100.0), px(100.0));
        
        viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Orbit);
        
        viewport.stop_drag(MouseButton::Left);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
    }

    #[test]
    fn test_drag_mode_pan() {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(100.0), px(100.0));
        
        viewport.start_drag(MouseButton::Middle, position, luminara_editor::viewport::DragMode::Pan);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Pan);
        
        viewport.stop_drag(MouseButton::Middle);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
    }

    #[test]
    fn test_drag_mode_zoom() {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(100.0), px(100.0));
        
        viewport.start_drag(MouseButton::Right, position, luminara_editor::viewport::DragMode::Zoom);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::Zoom);
        
        viewport.stop_drag(MouseButton::Right);
        assert_eq!(viewport.drag_mode, luminara_editor::viewport::DragMode::None);
    }


    #[test]
    fn test_orbit_maintains_distance() {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        let initial_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        let start_pos = Point::new(px(100.0), px(100.0));
        let end_pos = Point::new(px(150.0), px(120.0));
        
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        viewport.update_drag(end_pos);
        
        let final_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        assert!((final_distance - initial_distance).abs() < 0.1);
    }

    #[test]
    fn test_pan_maintains_offset() {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        let initial_offset = {
            let cam = camera.read();
            cam.position - cam.target
        };
        
        let start_pos = Point::new(px(100.0), px(100.0));
        let end_pos = Point::new(px(150.0), px(120.0));
        
        viewport.start_drag(MouseButton::Middle, start_pos, luminara_editor::viewport::DragMode::Pan);
        viewport.update_drag(end_pos);
        
        let final_offset = {
            let cam = camera.read();
            cam.position - cam.target
        };
        
        assert!((final_offset.x - initial_offset.x).abs() < 0.01);
        assert!((final_offset.y - initial_offset.y).abs() < 0.01);
        assert!((final_offset.z - initial_offset.z).abs() < 0.01);
    }

    #[test]
    fn test_zoom_changes_distance() {
        let mut viewport = create_test_viewport();
        let camera = viewport.camera.clone();
        
        let initial_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        let start_pos = Point::new(px(100.0), px(100.0));
        let end_pos = Point::new(px(100.0), px(150.0));
        
        viewport.start_drag(MouseButton::Right, start_pos, luminara_editor::viewport::DragMode::Zoom);
        viewport.update_drag(end_pos);
        
        let final_distance = {
            let cam = camera.read();
            (cam.position - cam.target).length()
        };
        
        assert!((final_distance - initial_distance).abs() > 0.001);
    }

    #[test]
    fn test_multiple_drag_operations() {
        let mut viewport = create_test_viewport();
        
        for i in 0..5 {
            let x = 100.0 + i as f32 * 10.0;
            let y = 100.0 + i as f32 * 10.0;
            let position = Point::new(px(x), px(y));
            
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            assert!(viewport.is_dragging);
            
            viewport.stop_drag(MouseButton::Left);
            assert!(!viewport.is_dragging);
        }
    }

    #[test]
    fn test_drag_state_consistency() {
        let mut viewport = create_test_viewport();
        let start_pos = Point::new(px(100.0), px(100.0));
        
        viewport.start_drag(MouseButton::Left, start_pos, luminara_editor::viewport::DragMode::Orbit);
        assert!(viewport.is_dragging);
        
        // Multiple updates should maintain dragging state
        for i in 0..10 {
            let x = 100.0 + i as f32 * 5.0;
            let y = 100.0 + i as f32 * 5.0;
            viewport.update_drag(Point::new(px(x), px(y)));
            assert!(viewport.is_dragging);
        }
        
        viewport.stop_drag(MouseButton::Left);
        assert!(!viewport.is_dragging);
    }

    #[test]
    fn test_edge_positions() {
        let mut viewport = create_test_viewport();
        
        let edge_positions = vec![
            (0.0, 0.0),
            (800.0, 0.0),
            (0.0, 600.0),
            (800.0, 600.0),
        ];
        
        for (x, y) in edge_positions {
            let position = Point::new(px(x), px(y));
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            assert!(viewport.is_dragging);
            viewport.stop_drag(MouseButton::Left);
            assert!(!viewport.is_dragging);
        }
    }

    #[test]
    fn test_rapid_button_press_release() {
        let mut viewport = create_test_viewport();
        let position = Point::new(px(100.0), px(100.0));
        
        for _ in 0..20 {
            viewport.start_drag(MouseButton::Left, position, luminara_editor::viewport::DragMode::Orbit);
            assert!(viewport.is_dragging);
            
            viewport.stop_drag(MouseButton::Left);
            assert!(!viewport.is_dragging);
        }
    }
}
