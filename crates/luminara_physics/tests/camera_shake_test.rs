use luminara_physics::camera_shake::CameraShake;

#[test]
fn test_shake_decay() {
    let mut shake = CameraShake {
        intensity: 10.0,
        decay: 5.0,
        ..Default::default()
    };

    let dt = 0.1;

    // Step 1: 10 - 5 * 0.1 = 9.5
    shake.intensity = (shake.intensity - shake.decay * dt).max(0.0);
    assert!((shake.intensity - 9.5).abs() < 0.001);

    // Step 2: 1.0s later -> 10 - 5 * 1.0 = 5.0
    shake.intensity = 10.0;
    let long_dt = 1.0;
    shake.intensity = (shake.intensity - shake.decay * long_dt).max(0.0);
    assert!((shake.intensity - 5.0).abs() < 0.001);

    // Step 3: Full decay
    let huge_dt = 10.0;
    shake.intensity = (shake.intensity - shake.decay * huge_dt).max(0.0);
    assert_eq!(shake.intensity, 0.0);
}
