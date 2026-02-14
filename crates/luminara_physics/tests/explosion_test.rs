#[test]
fn test_explosion_force_falloff() {
    // F = force / dist^2
    let force_mag = 1000.0_f32;

    // Dist = 1
    let dist1 = 1.0_f32;
    let f1 = force_mag / (dist1 * dist1);

    // Dist = 2
    let dist2 = 2.0_f32;
    let f2 = force_mag / (dist2 * dist2);

    // Inverse square: f2 should be f1 / 4
    assert!((f2 - f1 / 4.0).abs() < 0.001);

    // Check our specific implementation logic
    // let strength = force_mag / dist_sq.max(1.0);
    // let impulse = dir * strength;

    let calc_impulse = |dist: f32| -> f32 {
        let dist_sq = dist * dist;
        force_mag / dist_sq.max(1.0)
    };

    let i1 = calc_impulse(1.0);
    let i2 = calc_impulse(2.0);

    assert_eq!(i1, 1000.0);
    assert_eq!(i2, 250.0);

    // Dist < 1 (clamped)
    let i05 = calc_impulse(0.5);
    assert_eq!(i05, 1000.0); // Clamped at 1.0 dist_sq
}
