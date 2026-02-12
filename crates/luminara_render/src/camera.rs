use luminara_core::shared_types::Component;
use luminara_math::{Color, Mat4};

#[derive(Debug, Clone)]
pub struct Camera {
    pub projection: Projection,
    pub clear_color: Color,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub enum Projection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { size: f32, near: f32, far: f32 },
}

impl Camera {
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective { fov, near, far } => {
                Mat4::perspective_lh(fov.to_radians(), aspect_ratio, near, far)
            }
            Projection::Orthographic { size, near, far } => {
                let half_size = size / 2.0;
                Mat4::orthographic_lh(
                    -half_size * aspect_ratio,
                    half_size * aspect_ratio,
                    -half_size,
                    half_size,
                    near,
                    far,
                )
            }
        }
    }
}

impl Component for Camera {
    fn type_name() -> &'static str {
        "Camera"
    }
}

pub struct Camera3d;
impl Component for Camera3d {
    fn type_name() -> &'static str {
        "Camera3d"
    }
}

pub struct Camera2d;
impl Component for Camera2d {
    fn type_name() -> &'static str {
        "Camera2d"
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: Color::BLACK,
            is_active: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_projection_matrix() {
        let camera = Camera::default();
        let mat = camera.projection_matrix(1.6);
        assert_ne!(mat, Mat4::IDENTITY);
    }
}
