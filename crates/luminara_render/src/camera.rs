use luminara_core::shared_types::Component;
use luminara_math::{Color, Mat4};
use serde::{Deserialize, Serialize};
use luminara_reflect_derive::Reflect;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Camera {
    pub projection: Projection,
    pub clear_color: Color,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Projection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { size: f32, near: f32, far: f32 },
}

impl Camera {
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective { fov, near, far } => {
                // Use RH projection: camera looks along -Z, wgpu clip Z ∈ [0, 1]
                let mut mat = Mat4::perspective_rh(fov.to_radians(), aspect_ratio, near, far);
                // Convert [-1, 1] to [0, 1]: Z' = 0.5 * Z + 0.5 * W
                mat.x_axis.z = 0.5 * mat.x_axis.z + 0.5 * mat.x_axis.w;
                mat.y_axis.z = 0.5 * mat.y_axis.z + 0.5 * mat.y_axis.w;
                mat.z_axis.z = 0.5 * mat.z_axis.z + 0.5 * mat.z_axis.w;
                mat.w_axis.z = 0.5 * mat.w_axis.z + 0.5 * mat.w_axis.w;
                mat
            }
            Projection::Orthographic { size, near, far } => {
                let half_size = size / 2.0;
                let mut mat = Mat4::orthographic_rh(
                    -half_size * aspect_ratio,
                    half_size * aspect_ratio,
                    -half_size,
                    half_size,
                    near,
                    far,
                );
                // Convert [-1, 1] to [0, 1]: Z' = 0.5 * Z + 0.5 * W
                mat.x_axis.z = 0.5 * mat.x_axis.z + 0.5 * mat.x_axis.w;
                mat.y_axis.z = 0.5 * mat.y_axis.z + 0.5 * mat.y_axis.w;
                mat.z_axis.z = 0.5 * mat.z_axis.z + 0.5 * mat.z_axis.w;
                mat.w_axis.z = 0.5 * mat.w_axis.z + 0.5 * mat.w_axis.w;
                mat
            }
        }
    }

    /// Compute the view matrix from the camera's global transform.
    /// The view matrix is the inverse of the camera's world transform.
    pub fn view_matrix(&self, global_transform: &luminara_math::Mat4) -> Mat4 {
        global_transform.inverse()
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
    use luminara_math::Vec3;

    #[test]
    fn test_camera_projection_matrix() {
        let camera = Camera::default();
        let mat = camera.projection_matrix(1.6);
        assert_ne!(mat, Mat4::IDENTITY);
    }

    #[test]
    fn test_camera_view_matrix() {
        let camera = Camera::default();

        // Create a transform matrix (camera at position (0, 5, 10))
        let transform = Mat4::from_translation(Vec3::new(0.0, 5.0, 10.0));

        // Get the view matrix
        let view = camera.view_matrix(&transform);

        // View matrix should be the inverse of the transform
        let expected = transform.inverse();

        // Compare matrices element by element with tolerance
        for i in 0..4 {
            for j in 0..4 {
                let diff = (view.col(i)[j] - expected.col(i)[j]).abs();
                assert!(diff < 0.0001, "Matrix mismatch at [{}, {}]", i, j);
            }
        }
    }

    #[test]
    fn test_perspective_projection() {
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: Color::BLACK,
            is_active: true,
        };

        let mat = camera.projection_matrix(16.0 / 9.0);
        assert_ne!(mat, Mat4::IDENTITY);
    }

    #[test]
    fn test_orthographic_projection() {
        let camera = Camera {
            projection: Projection::Orthographic {
                size: 10.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: Color::BLACK,
            is_active: true,
        };

        let mat = camera.projection_matrix(16.0 / 9.0);
        assert_ne!(mat, Mat4::IDENTITY);
    }

    #[test]
    fn test_camera_is_active() {
        let camera = Camera::default();
        assert!(camera.is_active);

        let inactive_camera = Camera {
            is_active: false,
            ..Default::default()
        };
        assert!(!inactive_camera.is_active);
    }

    #[test]
    fn test_camera_clear_color() {
        let camera = Camera {
            clear_color: Color::rgba(0.1, 0.2, 0.3, 1.0),
            ..Default::default()
        };

        assert_eq!(camera.clear_color.r, 0.1);
        assert_eq!(camera.clear_color.g, 0.2);
        assert_eq!(camera.clear_color.b, 0.3);
        assert_eq!(camera.clear_color.a, 1.0);
    }
}
