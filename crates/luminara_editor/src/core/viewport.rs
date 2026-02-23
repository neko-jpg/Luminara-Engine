//! Custom WGPU Viewport Element (Vizia version)
//!
//! This module implements a custom Vizia-compatible viewport for Luminara's WGPU renderer.

use luminara_math::Vec3;
use parking_lot::RwLock;
use std::sync::Arc;

pub use crate::rendering::{RenderingServer, ViewportId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    None,
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let offset = self.position - self.target;
        let radius = offset.length();
        let theta = offset.z.atan2(offset.x);
        let phi = (offset.y / radius).acos();
        let new_theta = theta + delta_x;
        let new_phi = (phi + delta_y).clamp(0.01, std::f32::consts::PI - 0.01);
        self.position = self.target
            + Vec3::new(
                radius * new_phi.sin() * new_theta.cos(),
                radius * new_phi.cos(),
                radius * new_phi.sin() * new_theta.sin(),
            );
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        let pan_speed = 0.01 * (self.target - self.position).length();
        self.target += right * delta_x * pan_speed + up * (-delta_y) * pan_speed;
        self.position += right * delta_x * pan_speed + up * (-delta_y) * pan_speed;
    }

    pub fn zoom(&mut self, delta: f32) {
        let direction = self.target - self.position;
        let distance = direction.length();
        let new_distance = (distance * (1.0 + delta * 0.1)).clamp(1.0, 1000.0);
        self.position = self.target - direction.normalize() * new_distance;
    }
}

#[derive(Debug, Clone)]
pub struct ViewportState {
    pub camera: Camera,
    pub gizmo_mode: GizmoMode,
    pub viewport_id: Option<ViewportId>,
    pub rendering_server: Option<Arc<parking_lot::RwLock<RenderingServer>>>,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            gizmo_mode: GizmoMode::None,
            viewport_id: None,
            rendering_server: None,
        }
    }

    pub fn set_camera(&mut self, position: Vec3, target: Vec3, up: Vec3, fov: f32) {
        self.camera.position = position;
        self.camera.target = target;
        self.camera.up = up;
        self.camera.fov = fov;
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new()
    }
}
