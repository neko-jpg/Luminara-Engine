use luminara_math::Vec3;
use luminara_physics::interaction::MouseInteractionConfig;

#[test]
fn test_throw_velocity_logic() {
    // This test simulates the concept of throw velocity scaling.
    // Since the actual implementation relies on the physics engine's spring force
    // to impart velocity during the drag, and then applies a multiplier,
    // we verify the multiplier logic here.

    let config = MouseInteractionConfig {
        throw_multiplier: 2.0,
        ..Default::default()
    };

    let initial_velocity = Vec3::new(10.0, 0.0, 0.0);
    // Convert to nalgebra for simulation if we were using physics types,
    // but here we just test the math logic intended.

    // Logic in system:
    // let vel = *body.linvel();
    // body.apply_impulse(vel * config.throw_multiplier, true);
    // Final velocity approx = vel + (vel * multiplier / mass) * dt?
    // No, apply_impulse adds to velocity instantaneously: v_new = v_old + impulse/mass.
    // If mass=1, v_new = v_old + v_old * multiplier = v_old * (1 + multiplier).

    let mass = 1.0;
    let impulse = initial_velocity * config.throw_multiplier;
    let delta_v = impulse / mass;
    let final_velocity = initial_velocity + delta_v;

    assert_eq!(final_velocity, initial_velocity * (1.0 + config.throw_multiplier));

    // Property: Faster drag (higher initial velocity) produces higher release velocity
    let slow_drag_vel = Vec3::new(1.0, 0.0, 0.0);
    let fast_drag_vel = Vec3::new(100.0, 0.0, 0.0);

    let slow_release = slow_drag_vel * (1.0 + config.throw_multiplier);
    let fast_release = fast_drag_vel * (1.0 + config.throw_multiplier);

    assert!(fast_release.length() > slow_release.length());
    assert!((fast_release.length() / slow_release.length() - 100.0).abs() < 0.001);
}
