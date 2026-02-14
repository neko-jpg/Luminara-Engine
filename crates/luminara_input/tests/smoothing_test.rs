use luminara_input::smoothing::MouseSmoothing;
use luminara_math::Vec2;

#[test]
fn test_mouse_smoothing() {
    let mut smoothing = MouseSmoothing::new(0.5);

    // Initial state
    assert_eq!(smoothing.smoothed_delta, Vec2::ZERO);

    // First update: should move towards target
    // If factor is 0.5, alpha is 0.5.
    // smoothed = 0.5 * input + 0.5 * previous
    let input = Vec2::new(10.0, 0.0);
    let output = smoothing.update(input);

    assert_eq!(output, Vec2::new(5.0, 0.0));

    // Second update: same input
    // smoothed = 0.5 * 10 + 0.5 * 5 = 5 + 2.5 = 7.5
    let output = smoothing.update(input);
    assert_eq!(output, Vec2::new(7.5, 0.0));

    // Test direction preservation
    let input = Vec2::new(0.0, 10.0);
    let output = smoothing.update(input);
    // smoothed = 0.5 * (0, 10) + 0.5 * (7.5, 0) = (0, 5) + (3.75, 0) = (3.75, 5.0)
    assert!(output.x > 0.0);
    assert!(output.y > 0.0);
}

#[test]
fn test_no_smoothing() {
    let mut smoothing = MouseSmoothing::new(0.0);
    let input = Vec2::new(10.0, 5.0);
    let output = smoothing.update(input);
    assert_eq!(output, input);
}
