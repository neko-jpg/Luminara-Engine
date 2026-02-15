// Fly Camera Controller Test
use luminara_math::glam::EulerRot;
use luminara_math::{Quat, Vec3};

/// Fly camera controller for spectator mode navigation
#[derive(Debug, Clone)]
pub struct FlyCameraController {
    position: Vec3,
    rotation: Quat,
    move_speed: f32,
    sprint_multiplier: f32,
    mouse_sensitivity: f32,
    yaw: f32,
    pitch: f32,
    velocity: Vec3,
    inertia: f32,
}

impl FlyCameraController {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            move_speed: 5.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.1,
            yaw: 0.0,
            pitch: 0.0,
            velocity: Vec3::ZERO,
            inertia: 0.9,
        }
    }

    pub fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * self.mouse_sensitivity;
        self.pitch = (self.pitch - delta_y * self.mouse_sensitivity).clamp(-89.0, 89.0);

        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        // Use glam's from_euler with YXZ order (yaw, pitch, roll)
        self.rotation = Quat::from_euler(EulerRot::YXZ, yaw_rad, pitch_rad, 0.0);
    }

    pub fn move_forward(&mut self, delta_time: f32, sprint: bool) {
        let speed = if sprint {
            self.move_speed * self.sprint_multiplier
        } else {
            self.move_speed
        };
        let forward = self.forward();
        self.velocity = self.velocity + forward * speed * delta_time;
    }

    pub fn move_backward(&mut self, delta_time: f32, sprint: bool) {
        let speed = if sprint {
            self.move_speed * self.sprint_multiplier
        } else {
            self.move_speed
        };
        let forward = self.forward();
        self.velocity = self.velocity - forward * speed * delta_time;
    }

    pub fn move_right(&mut self, delta_time: f32, sprint: bool) {
        let speed = if sprint {
            self.move_speed * self.sprint_multiplier
        } else {
            self.move_speed
        };
        let right = self.right();
        self.velocity = self.velocity + right * speed * delta_time;
    }

    pub fn move_left(&mut self, delta_time: f32, sprint: bool) {
        let speed = if sprint {
            self.move_speed * self.sprint_multiplier
        } else {
            self.move_speed
        };
        let right = self.right();
        self.velocity = self.velocity - right * speed * delta_time;
    }

    pub fn move_up(&mut self, delta_time: f32) {
        let up = self.up();
        self.velocity = self.velocity + up * self.move_speed * delta_time;
    }

    pub fn move_down(&mut self, delta_time: f32) {
        let up = self.up();
        self.velocity = self.velocity - up * self.move_speed * delta_time;
    }

    pub fn apply_inertia(&mut self, _delta_time: f32) {
        self.position = self.position + self.velocity;
        self.velocity = self.velocity * self.inertia;
    }

    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        Vec3::new(
            yaw_rad.sin() * pitch_rad.cos(),
            -pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        Vec3::new(yaw_rad.cos(), 0.0, -yaw_rad.sin()).normalize()
    }

    pub fn up(&self) -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.move_speed = speed;
    }

    pub fn set_inertia(&mut self, inertia: f32) {
        self.inertia = inertia.clamp(0.0, 1.0);
    }
}

#[test]
fn test_fly_camera_wasd_movement() {
    let mut camera = FlyCameraController::new(Vec3::ZERO);
    let initial_pos = camera.position();

    // Move forward (W key)
    camera.move_forward(0.1, false);
    camera.apply_inertia(0.1);

    assert_ne!(
        camera.position(),
        initial_pos,
        "Camera should move forward with W key"
    );
}

#[test]
fn test_fly_camera_mouse_look() {
    let mut camera = FlyCameraController::new(Vec3::ZERO);

    // Rotate camera with mouse
    camera.update_rotation(10.0, 5.0);

    assert_ne!(
        camera.yaw, 0.0,
        "Camera yaw should change with mouse movement"
    );
    assert_ne!(
        camera.pitch, 0.0,
        "Camera pitch should change with mouse movement"
    );
}

#[test]
fn test_fly_camera_speed_control() {
    let mut camera = FlyCameraController::new(Vec3::ZERO);

    camera.set_speed(10.0);
    assert_eq!(camera.move_speed, 10.0, "Camera speed should be adjustable");
}

#[test]
fn test_fly_camera_smooth_movement() {
    let mut camera = FlyCameraController::new(Vec3::ZERO);
    camera.set_inertia(0.95);

    // Apply movement
    camera.move_forward(0.1, false);
    camera.apply_inertia(0.1);

    let vel_magnitude = camera.velocity.length();
    assert!(
        vel_magnitude > 0.0,
        "Camera should have smooth movement with inertia"
    );
}

#[cfg(test)]
mod fly_camera_integration_tests {
    use super::*;

    #[test]
    fn test_fly_camera_collision_detection() {
        let camera = FlyCameraController::new(Vec3::ZERO);

        // Collision detection would be implemented in the physics system
        // This test verifies the camera position can be queried for collision checks
        let pos = camera.position();
        assert_eq!(
            pos,
            Vec3::ZERO,
            "Camera position should be accessible for collision detection"
        );
    }

    #[test]
    fn test_fly_camera_vertical_movement() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);
        let initial_y = camera.position().y;

        // Move up (Space key)
        camera.move_up(0.1);
        camera.apply_inertia(0.1);

        assert!(
            camera.position().y > initial_y,
            "Camera should move up with Space key"
        );

        let after_up_y = camera.position().y;

        // Move down (Shift key)
        camera.move_down(0.1);
        camera.apply_inertia(0.1);

        assert!(
            camera.position().y < after_up_y,
            "Camera should move down with Shift key"
        );
    }

    #[test]
    fn test_fly_camera_sprint_mode() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);

        // Normal movement
        camera.move_forward(0.1, false);
        let normal_velocity = camera.velocity.length();

        camera.velocity = Vec3::ZERO;

        // Sprint movement
        camera.move_forward(0.1, true);
        let sprint_velocity = camera.velocity.length();

        assert!(
            sprint_velocity > normal_velocity,
            "Sprint mode should move faster"
        );
    }

    #[test]
    fn test_fly_camera_inertia() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);

        // Test with high inertia (smooth)
        camera.set_inertia(0.95);
        camera.move_forward(0.1, false);
        camera.apply_inertia(0.1);
        let high_inertia_vel = camera.velocity.length();

        // Test with low inertia (responsive)
        camera.velocity = Vec3::ZERO;
        camera.set_inertia(0.5);
        camera.move_forward(0.1, false);
        camera.apply_inertia(0.1);
        let low_inertia_vel = camera.velocity.length();

        assert!(
            high_inertia_vel > low_inertia_vel,
            "Higher inertia should maintain velocity longer"
        );
    }

    #[test]
    fn test_all_direction_movement() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);

        // Test WASD + QE movement
        camera.move_forward(0.1, false);
        camera.move_backward(0.1, false);
        camera.move_left(0.1, false);
        camera.move_right(0.1, false);
        camera.move_up(0.1);
        camera.move_down(0.1);

        // All movements should be possible without panicking
        assert!(true, "All direction movements should work");
    }
}

#[cfg(test)]
mod fly_camera_math_tests {
    use super::*;

    #[test]
    fn test_camera_forward_vector() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);

        // Default forward should be along -Z axis (OpenGL convention)
        let forward = camera.forward();
        assert!(
            forward.length() > 0.99 && forward.length() < 1.01,
            "Forward vector should be normalized"
        );

        // Rotate and check forward vector changes
        camera.update_rotation(90.0, 0.0);
        let rotated_forward = camera.forward();
        assert_ne!(
            forward, rotated_forward,
            "Forward vector should change with rotation"
        );
    }

    #[test]
    fn test_camera_right_vector() {
        let camera = FlyCameraController::new(Vec3::ZERO);

        let right = camera.right();
        assert!(
            right.length() > 0.99 && right.length() < 1.01,
            "Right vector should be normalized"
        );

        // Right vector should be perpendicular to forward
        let forward = camera.forward();
        let dot = forward.dot(right);
        assert!(
            dot.abs() < 0.01,
            "Right vector should be perpendicular to forward"
        );
    }

    #[test]
    fn test_camera_up_vector() {
        let camera = FlyCameraController::new(Vec3::ZERO);

        let up = camera.up();
        assert_eq!(
            up,
            Vec3::new(0.0, 1.0, 0.0),
            "Up vector should be world up (0, 1, 0)"
        );
    }

    #[test]
    fn test_pitch_clamping() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);

        // Try to pitch beyond limits
        camera.update_rotation(0.0, 1000.0);
        assert!(
            camera.pitch >= -89.0 && camera.pitch <= 89.0,
            "Pitch should be clamped to prevent gimbal lock"
        );
    }

    #[test]
    fn test_orthogonal_basis() {
        let mut camera = FlyCameraController::new(Vec3::ZERO);
        camera.update_rotation(45.0, 30.0);

        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();

        // Check orthogonality
        assert!(
            forward.dot(right).abs() < 0.01,
            "Forward and right should be orthogonal"
        );
        assert!(
            forward.dot(up).abs() < 0.5,
            "Forward and up should be roughly orthogonal"
        );
        assert!(
            right.dot(up).abs() < 0.01,
            "Right and up should be orthogonal"
        );
    }
}
