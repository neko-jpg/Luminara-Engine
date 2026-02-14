//! Advanced visual effects and interactive elements

use luminara::prelude::*;
use luminara_render::{ParticleEmitter, PointLight};
use luminara_physics::RigidBodyType;

/// Trail effect component for moving objects
#[derive(Debug, Clone)]
pub struct TrailEffect {
    pub color: Color,
    pub lifetime: f32,
    pub spawn_rate: f32,
    pub accumulator: f32,
    pub last_position: Vec3,
}

impl Component for TrailEffect {
    fn type_name() -> &'static str {
        "TrailEffect"
    }
}

/// Rotating platform component
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RotatingPlatform {
    pub axis: Vec3,
    pub speed: f32,
    pub radius: f32,
}

impl Component for RotatingPlatform {
    fn type_name() -> &'static str {
        "RotatingPlatform"
    }
}

/// Pendulum physics component
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Pendulum {
    pub anchor: Vec3,
    pub length: f32,
    pub angle: f32,
    pub angular_velocity: f32,
    pub mass: f32,
}

impl Component for Pendulum {
    fn type_name() -> &'static str {
        "Pendulum"
    }
}

/// Chain reaction trigger
#[derive(Debug, Clone)]
pub struct ChainReactionTrigger {
    pub triggered: bool,
    pub delay: f32,
    pub force: f32,
    pub radius: f32,
}

impl Component for ChainReactionTrigger {
    fn type_name() -> &'static str {
        "ChainReactionTrigger"
    }
}

/// Domino piece
#[derive(Debug, Clone)]
pub struct DominoPiece {
    pub index: u32,
    pub fallen: bool,
}

impl Component for DominoPiece {
    fn type_name() -> &'static str {
        "DominoPiece"
    }
}

/// Orbital movement component
#[derive(Debug, Clone)]
pub struct OrbitalMotion {
    pub center: Vec3,
    pub radius: f32,
    pub speed: f32,
    pub angle: f32,
    pub height_offset: f32,
    pub vertical_speed: f32,
}

impl Component for OrbitalMotion {
    fn type_name() -> &'static str {
        "OrbitalMotion"
    }
}

/// Pulsating light effect
#[derive(Debug, Clone)]
pub struct PulsatingLight {
    pub base_intensity: f32,
    pub pulse_speed: f32,
    pub pulse_amount: f32,
    pub phase: f32,
}

impl Component for PulsatingLight {
    fn type_name() -> &'static str {
        "PulsatingLight"
    }
}

/// Magnetic field effect
#[derive(Debug, Clone)]
pub struct MagneticField {
    pub strength: f32,
    pub radius: f32,
    pub attract: bool, // true = attract, false = repel
}

impl Component for MagneticField {
    fn type_name() -> &'static str {
        "MagneticField"
    }
}

// ============================================================================
// System: Update rotating platforms
// ============================================================================

pub fn rotating_platform_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let mut query = Query::<(&mut Transform, &RotatingPlatform)>::new(world);
    for (transform, platform) in query.iter_mut() {
        let rotation = Quat::from_axis_angle(platform.axis, platform.speed * dt);
        transform.rotation = rotation * transform.rotation;
    }
}

// ============================================================================
// System: Update pendulums
// ============================================================================

pub fn pendulum_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let gravity = 9.8;
    let mut query = Query::<(&mut Transform, &mut Pendulum)>::new(world);
    
    for (transform, pendulum) in query.iter_mut() {
        // Simple pendulum physics
        let angular_acceleration = -(gravity / pendulum.length) * pendulum.angle.sin();
        pendulum.angular_velocity += angular_acceleration * dt;
        pendulum.angular_velocity *= 0.999; // Damping
        pendulum.angle += pendulum.angular_velocity * dt;

        // Update position
        let x = pendulum.anchor.x + pendulum.length * pendulum.angle.sin();
        let y = pendulum.anchor.y - pendulum.length * pendulum.angle.cos();
        let z = pendulum.anchor.z;
        
        transform.translation = Vec3::new(x, y, z);
        transform.rotation = Quat::from_rotation_z(pendulum.angle);
    }
}

// ============================================================================
// System: Update orbital motion
// ============================================================================

pub fn orbital_motion_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let mut query = Query::<(&mut Transform, &mut OrbitalMotion)>::new(world);
    
    for (transform, orbital) in query.iter_mut() {
        orbital.angle += orbital.speed * dt;
        orbital.height_offset += orbital.vertical_speed * dt;
        
        let x = orbital.center.x + orbital.radius * orbital.angle.cos();
        let z = orbital.center.z + orbital.radius * orbital.angle.sin();
        let y = orbital.center.y + orbital.height_offset.sin() * 2.0;
        
        transform.translation = Vec3::new(x, y, z);
        transform.rotation = Quat::from_rotation_y(orbital.angle);
    }
}

// ============================================================================
// System: Pulsating lights
// ============================================================================

pub fn pulsating_light_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let mut query = Query::<(&mut PointLight, &mut PulsatingLight)>::new(world);
    
    for (light, pulse) in query.iter_mut() {
        pulse.phase += pulse.pulse_speed * dt;
        let intensity_mod = (pulse.phase.sin() * 0.5 + 0.5) * pulse.pulse_amount;
        light.intensity = pulse.base_intensity + intensity_mod;
    }
}

// ============================================================================
// System: Trail effects
// ============================================================================

pub fn trail_effect_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let mut spawn_requests = Vec::new();
    
    {
        let mut query = Query::<(Entity, &Transform, &mut TrailEffect)>::new(world);
        
        for (_entity, transform, trail) in query.iter_mut() {
            trail.accumulator += dt;
            
            let distance = (transform.translation - trail.last_position).length();
            
            if trail.accumulator >= 1.0 / trail.spawn_rate && distance > 0.1 {
                spawn_requests.push((transform.translation, trail.color, trail.lifetime));
                trail.last_position = transform.translation;
                trail.accumulator = 0.0;
            }
        }
    }
    
    // Spawn trail particles
    for (pos, color, lifetime) in spawn_requests {
        let particle = world.spawn();
        world.add_component(particle, Name::new("TrailParticle"));
        world.add_component(particle, Transform::from_xyz(pos.x, pos.y, pos.z));
        world.add_component(
            particle,
            ParticleEmitter {
                rate: 5.0,
                accumulator: 0.0,
                direction: Vec3::ZERO,
                spread: 0.1,
                speed: 0.5,
                color,
                size: 0.05,
                lifetime,
            },
        );
    }
}

// ============================================================================
// System: Magnetic field effects
// ============================================================================

pub fn magnetic_field_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    // Collect magnetic field data
    let mut fields = Vec::new();
    {
        let query = Query::<(&Transform, &MagneticField)>::new(world);
        for (transform, field) in query.iter() {
            fields.push((transform.translation, field.strength, field.radius, field.attract));
        }
    }
    
    // Apply forces to dynamic objects
    let mut query = Query::<(&mut Transform, &RigidBody)>::new(world);
    for (transform, rb) in query.iter_mut() {
        if rb.body_type != RigidBodyType::Dynamic {
            continue;
        }
        
        for (field_pos, strength, radius, attract) in &fields {
            let diff = *field_pos - transform.translation;
            let distance = diff.length();
            
            if distance < *radius && distance > 0.1 {
                let direction = diff.normalize();
                let force_magnitude = strength / (distance * distance);
                let force = direction * force_magnitude * if *attract { 1.0 } else { -1.0 };
                
                transform.translation += force * dt;
            }
        }
    }
}

// ============================================================================
// System: Chain reaction triggers
// ============================================================================

pub fn chain_reaction_system(world: &mut World) {
    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_seconds())
        .unwrap_or(1.0 / 60.0);

    let mut triggers = Vec::new();
    
    {
        let mut query = Query::<(Entity, &Transform, &mut ChainReactionTrigger)>::new(world);
        
        for (_entity, transform, trigger) in query.iter_mut() {
            if trigger.triggered {
                trigger.delay -= dt;
                if trigger.delay <= 0.0 {
                    triggers.push((transform.translation, trigger.force, trigger.radius));
                }
            }
        }
    }
    
    // Apply explosion forces
    for (center, force, radius) in triggers {
        let mut query = Query::<(&mut Transform, &RigidBody)>::new(world);
        for (transform, rb) in query.iter_mut() {
            if rb.body_type != RigidBodyType::Dynamic {
                continue;
            }
            
            let diff = transform.translation - center;
            let distance = diff.length();
            
            if distance < radius && distance > 0.1 {
                let direction = diff.normalize();
                let force_magnitude = force * (1.0 - distance / radius);
                transform.translation += direction * force_magnitude * dt;
            }
        }
    }
}

// ============================================================================
// System: Domino chain reaction
// ============================================================================

pub fn domino_system(world: &mut World) {
    let mut fallen_indices = Vec::new();
    
    {
        let query = Query::<(&Transform, &DominoPiece)>::new(world);
        
        for (transform, domino) in query.iter() {
            if !domino.fallen {
                // Check if tilted enough to be considered fallen
                let tilt = transform.rotation.to_euler(EulerRot::XYZ).0.abs();
                if tilt > 0.3 {
                    fallen_indices.push(domino.index);
                }
            }
        }
    }
    
    // Mark fallen dominoes
    if !fallen_indices.is_empty() {
        let mut query = Query::<&mut DominoPiece>::new(world);
        for domino in query.iter_mut() {
            if fallen_indices.contains(&domino.index) {
                domino.fallen = true;
            }
        }
    }
}
