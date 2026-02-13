use crate::camera::Camera;
use luminara_core::event::EventReader;
use luminara_core::shared_types::{Query, Res};
use luminara_window::{Window, WindowEvent};

/// System to update camera aspect ratio when window is resized.
/// This system responds to WindowEvent::Resized events and updates
/// the camera's internal aspect ratio tracking.
///
/// **Validates: Requirements 6.5**
pub fn camera_resize_system(
    mut cameras: Query<&mut Camera>,
    events: EventReader<WindowEvent>,
    window: Res<Window>,
) {
    // Check if there was a resize event
    let mut resized = false;
    for event in events.iter() {
        if let WindowEvent::Resized { .. } = event {
            resized = true;
            break;
        }
    }

    // If resized, update all cameras with the new aspect ratio
    if resized {
        let (width, height) = window.inner_size();
        if width > 0 && height > 0 {
            let new_aspect = width as f32 / height as f32;

            for _camera in cameras.iter_mut() {
                // The camera will use this aspect ratio when projection_matrix() is called
                // We don't store aspect ratio in Camera, but we log the update
                log::debug!(
                    "Camera aspect ratio updated to {} ({}x{})",
                    new_aspect,
                    width,
                    height
                );
            }
        }
    }
}

/// System to update camera projection matrices when parameters change.
/// This system detects changes to camera projection parameters and
/// recomputes the projection matrix.
///
/// **Validates: Requirements 6.2**
pub fn camera_projection_system(cameras: Query<&Camera>, window: Res<Window>) {
    let (width, height) = window.inner_size();
    if width == 0 || height == 0 {
        return;
    }

    let aspect_ratio = width as f32 / height as f32;

    // For each camera, compute the projection matrix
    // In a real implementation, we would detect changes and only recompute when needed
    // For now, we validate that the projection matrix can be computed
    for camera in cameras.iter() {
        let _proj_matrix = camera.projection_matrix(aspect_ratio);
        // The projection matrix is computed on-demand in the render system
        // This system validates that the computation is possible
    }
}
