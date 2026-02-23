//! Viewport Window Module
//!
//! This module provides shared state for communication between the GPUI editor
//! and a separate 3D viewport window.
//!
//! # Usage
//!
//! For a separate 3D viewport window, use the `hybrid_editor` example which demonstrates
//! running the engine's 3D rendering in a separate window while GPUI handles the editor UI.

use luminara_math::Vec3;
use parking_lot::RwLock;
use std::sync::Arc;

/// Shared state between the GPUI editor and the viewport window
///
/// This state is shared between the GPUI editor thread and the viewport window thread.
/// The editor writes camera updates to this state, and the viewport window reads from it.
#[derive(Clone)]
pub struct ViewportWindowState {
    /// Camera position
    pub position: Vec3,
    /// Camera target (look-at point)  
    pub target: Vec3,
    /// Camera up vector
    pub up: Vec3,
    /// Field of view in degrees
    pub fov: f32,
    /// Generation counter for change detection
    pub generation: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for ViewportWindowState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 45.0,
            generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl ViewportWindowState {
    /// Update camera and increment generation
    pub fn set_camera(&mut self, position: Vec3, target: Vec3, up: Vec3, fov: f32) {
        self.position = position;
        self.target = target;
        self.up = up;
        self.fov = fov;
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Release);
    }

    /// Check if camera has changed since last check
    pub fn has_changed(&self, last_gen: u64) -> bool {
        self.generation.load(std::sync::atomic::Ordering::Acquire) != last_gen
    }
}

/// Create a new viewport window state for communication between editor and viewport
pub fn create_viewport_state() -> Arc<RwLock<ViewportWindowState>> {
    Arc::new(RwLock::new(ViewportWindowState::default()))
}
