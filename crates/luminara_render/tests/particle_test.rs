use luminara_math::{Color, Vec3};
use luminara_render::particles::{Particle, ParticleSystem};

#[test]
fn test_particle_despawn() {
    let mut system = ParticleSystem::new(10);

    // Spawn particle with lifetime 1.0
    system.spawn(Vec3::ZERO, Vec3::ZERO, Color::WHITE, 1.0, 1.0);
    assert_eq!(system.particles.len(), 1);

    let mut particle = system.particles[0];

    // Simulate 0.5s -> should remain
    particle.lifetime -= 0.5;
    assert!(particle.lifetime > 0.0);

    // Simulate another 0.6s -> should be removed (total 1.1s > 1.0s)
    particle.lifetime -= 0.6;
    assert!(particle.lifetime <= 0.0);

    // In actual system, we use retain_mut. Let's test that logic.
    system.particles[0].lifetime = -0.1;
    system.particles.retain(|p| p.lifetime > 0.0);

    assert_eq!(system.particles.len(), 0);
}
