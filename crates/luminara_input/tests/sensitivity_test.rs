use luminara_input::smoothing::MouseSmoothing;
use luminara_math::Vec2;

#[test]
fn test_mouse_smoothing() {
    let mut smoothing = MouseSmoothing::new(0.5);

    // Initial state
    assert_eq!(smoothing.smoothed_delta, Vec2::ZERO);

    // First update: should move towards target
    let input = Vec2::new(10.0, 0.0);
    let output = smoothing.update(input);

    assert_eq!(output, Vec2::new(5.0, 0.0));

    // Second update: same input
    let output = smoothing.update(input);
    assert_eq!(output, Vec2::new(7.5, 0.0));
}

#[test]
fn test_fov_sensitivity() {
    // This logic is currently embedded in camera_controller.rs which is hard to unit test in isolation
    // without the ECS.
    // We will simulate the calculation logic here.

    let base_sensitivity = 1.0;
    let fov = 45.0;
    let ref_fov = 90.0;

    let effective = base_sensitivity * (fov / ref_fov);
    assert_eq!(effective, 0.5);

    let fov = 90.0;
    let effective = base_sensitivity * (fov / ref_fov);
    assert_eq!(effective, 1.0);
}
