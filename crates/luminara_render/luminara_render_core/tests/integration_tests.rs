use luminara_render_core::*;
use luminara_math::*;
use luminara_core::shared_types::{App, AppInterface};

#[test]
fn test_render_plugin_registration() {
    let mut app = App::default();
    app.add_plugins(RenderPlugin);
}

#[test]
fn test_mesh_creation() {
    let mesh = Mesh::triangle();
    assert_eq!(mesh.vertices.len(), 3);

    let cube = Mesh::cube(1.0);
    assert_eq!(cube.vertices.len(), 24);
}

#[test]
fn test_camera_default() {
    let camera = Camera::default();
    assert_eq!(camera.clear_color, Color::BLACK);
    assert!(camera.is_active);
}

#[test]
fn test_transform_matrix() {
    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let matrix = transform.compute_matrix();
    assert_eq!(matrix.w_axis.xyz(), Vec3::new(1.0, 2.0, 3.0));
}
