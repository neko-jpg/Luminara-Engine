//! Spectacular scene setups for impressive demonstrations

use crate::advanced_effects::*;
use luminara::asset::AssetServer;
use luminara::prelude::*;
use luminara_physics::{Collider, ColliderShape, RigidBody, RigidBodyType};
use luminara_render::{PbrMaterial, PointLight};
use std::f32::consts::PI;

/// Helper: add a cube mesh to an entity via AssetServer
fn add_cube_mesh(world: &mut World, entity: Entity, size: f32) {
    let mesh_h = world
        .get_resource::<AssetServer>()
        .map(|a| a.add(Mesh::cube(size)));
    if let Some(h) = mesh_h {
        world.add_component(entity, h);
    }
}

/// Helper: add a sphere mesh to an entity via AssetServer
fn add_sphere_mesh(world: &mut World, entity: Entity, radius: f32, segments: u32) {
    let mesh_h = world
        .get_resource::<AssetServer>()
        .map(|a| a.add(Mesh::sphere(radius, segments)));
    if let Some(h) = mesh_h {
        world.add_component(entity, h);
    }
}

// ============================================================================
// Domino Chain Setup
// ============================================================================

pub fn create_domino_chain(world: &mut World) {
    let domino_count = 100;
    let _spacing = 1.5;
    let domino_height = 2.0;

    // Create a winding path
    for i in 0..domino_count {
        let t = i as f32 / domino_count as f32;
        let angle = t * PI * 4.0; // 2 full circles
        let radius = 15.0 + t * 5.0; // Expanding spiral

        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = domino_height * 0.5;

        let rotation = Quat::from_rotation_y(angle + PI * 0.5);

        let domino = world.spawn();
        world.add_component(domino, Name::new(format!("Domino_{}", i)));
        world.add_component(
            domino,
            Transform {
                translation: Vec3::new(x, y, z),
                rotation,
                scale: Vec3::new(0.3, domino_height, 1.0),
            },
        );

        // Add box mesh for domino
        add_cube_mesh(world, domino, 1.0);

        // Alternating colors for visual appeal
        let color = if i % 2 == 0 {
            Color::rgb(0.9, 0.2, 0.2)
        } else {
            Color::rgb(0.2, 0.2, 0.9)
        };

        world.add_component(
            domino,
            PbrMaterial {
                albedo: color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.3,
                roughness: 0.6,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.2, color.g * 0.2, color.b * 0.2),
            },
        );

        world.add_component(
            domino,
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 0.5,
                linear_damping: 0.1,
                angular_damping: 0.2,
                gravity_scale: 1.0,
            },
        );

        world.add_component(
            domino,
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(0.15, domino_height * 0.5, 0.5),
                },
                friction: 0.8,
                restitution: 0.1,
                is_sensor: false,
            },
        );

        world.add_component(
            domino,
            DominoPiece {
                index: i,
                fallen: false,
            },
        );

        // Add trail effect to every 5th domino
        if i % 5 == 0 {
            world.add_component(
                domino,
                TrailEffect {
                    color: color,
                    lifetime: 1.0,
                    spawn_rate: 10.0,
                    accumulator: 0.0,
                    last_position: Vec3::new(x, y, z),
                },
            );
        }
    }
}

// ============================================================================
// Pendulum Array
// ============================================================================

pub fn create_pendulum_array(world: &mut World) {
    let count = 15;
    let spacing = 3.0;
    let base_height = 15.0;

    for i in 0..count {
        let x = (i as f32 - count as f32 * 0.5) * spacing;
        let anchor = Vec3::new(x, base_height, -10.0);
        let length = 8.0 + (i as f32 * 0.3);
        let initial_angle = (i as f32 * 0.2).sin() * 0.5;

        // Anchor point visualization
        let anchor_entity = world.spawn();
        world.add_component(anchor_entity, Name::new(format!("PendulumAnchor_{}", i)));
        world.add_component(
            anchor_entity,
            Transform {
                translation: anchor,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.3),
            },
        );

        // Add small sphere mesh for anchor
        add_sphere_mesh(world, anchor_entity, 0.3, 12);

        world.add_component(
            anchor_entity,
            PbrMaterial {
                albedo: Color::rgb(0.3, 0.3, 0.3),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.9,
                roughness: 0.1,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );

        // Pendulum bob
        let bob = world.spawn();
        world.add_component(bob, Name::new(format!("PendulumBob_{}", i)));

        let bob_x = anchor.x + length * initial_angle.sin();
        let bob_y = anchor.y - length * initial_angle.cos();

        world.add_component(
            bob,
            Transform {
                translation: Vec3::new(bob_x, bob_y, anchor.z),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.8),
            },
        );

        // Add sphere mesh for pendulum bob
        add_sphere_mesh(world, bob, 0.8, 16);

        let hue = i as f32 / count as f32;
        let color = Color::hsl(hue * 360.0, 0.8, 0.5);

        world.add_component(
            bob,
            PbrMaterial {
                albedo: color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.8,
                roughness: 0.2,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.5, color.g * 0.5, color.b * 0.5),
            },
        );

        world.add_component(
            bob,
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 2.0,
                linear_damping: 0.01,
                angular_damping: 0.01,
                gravity_scale: 1.0,
            },
        );

        world.add_component(
            bob,
            Collider {
                shape: ColliderShape::Sphere { radius: 0.8 },
                friction: 0.3,
                restitution: 0.9,
                is_sensor: false,
            },
        );

        world.add_component(
            bob,
            Pendulum {
                anchor,
                length,
                angle: initial_angle,
                angular_velocity: 0.0,
                mass: 2.0,
            },
        );

        world.add_component(
            bob,
            TrailEffect {
                color,
                lifetime: 0.8,
                spawn_rate: 20.0,
                accumulator: 0.0,
                last_position: Vec3::new(bob_x, bob_y, anchor.z),
            },
        );

        // Add light to bob
        world.add_component(
            bob,
            PointLight {
                color,
                intensity: 0.5,
                range: 8.0,
                cast_shadows: false,
            },
        );
    }
}

// ============================================================================
// Rotating Platform Maze
// ============================================================================

pub fn create_rotating_platforms(world: &mut World) {
    let platforms = [
        // (position, scale, axis, speed)
        (
            Vec3::new(-15.0, 3.0, 10.0),
            Vec3::new(8.0, 0.5, 2.0),
            Vec3::Y,
            0.5,
        ),
        (
            Vec3::new(-5.0, 5.0, 10.0),
            Vec3::new(6.0, 0.5, 2.0),
            Vec3::Y,
            -0.7,
        ),
        (
            Vec3::new(5.0, 7.0, 10.0),
            Vec3::new(8.0, 0.5, 2.0),
            Vec3::Y,
            0.6,
        ),
        (
            Vec3::new(15.0, 9.0, 10.0),
            Vec3::new(6.0, 0.5, 2.0),
            Vec3::Y,
            -0.8,
        ),
        (
            Vec3::new(0.0, 11.0, 10.0),
            Vec3::new(10.0, 0.5, 2.0),
            Vec3::Y,
            0.4,
        ),
    ];

    for (i, (pos, scale, axis, speed)) in platforms.iter().enumerate() {
        let platform = world.spawn();
        world.add_component(platform, Name::new(format!("RotatingPlatform_{}", i)));
        world.add_component(
            platform,
            Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: *scale,
            },
        );

        // Add box mesh for platform
        add_cube_mesh(world, platform, 1.0);

        let hue = (i as f32 / platforms.len() as f32) * 360.0;
        let color = Color::hsl(hue, 0.7, 0.6);

        world.add_component(
            platform,
            PbrMaterial {
                albedo: color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.6,
                roughness: 0.3,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.3, color.g * 0.3, color.b * 0.3),
            },
        );

        world.add_component(
            platform,
            RigidBody {
                body_type: RigidBodyType::Kinematic,
                mass: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
            },
        );

        world.add_component(
            platform,
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5),
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        );

        world.add_component(
            platform,
            RotatingPlatform {
                axis: *axis,
                speed: *speed,
                radius: scale.x.max(scale.z),
            },
        );

        // Add edge lights
        for j in 0..4 {
            let angle = j as f32 * PI * 0.5;
            let light_offset = Vec3::new(
                angle.cos() * scale.x * 0.4,
                scale.y * 0.6,
                angle.sin() * scale.z * 0.4,
            );

            let light = world.spawn();
            world.add_component(light, Name::new(format!("PlatformLight_{}_{}", i, j)));
            world.add_component(
                light,
                Transform::from_xyz(
                    pos.x + light_offset.x,
                    pos.y + light_offset.y,
                    pos.z + light_offset.z,
                ),
            );
            world.add_component(
                light,
                PointLight {
                    color,
                    intensity: 0.6,
                    range: 5.0,
                    cast_shadows: false,
                },
            );
            world.add_component(
                light,
                PulsatingLight {
                    base_intensity: 0.6,
                    pulse_speed: 2.0 + j as f32 * 0.5,
                    pulse_amount: 0.4,
                    phase: j as f32 * PI * 0.5,
                },
            );
        }
    }
}

// ============================================================================
// Orbital System (planets and moons)
// ============================================================================

pub fn create_orbital_system(world: &mut World) {
    let center = Vec3::new(0.0, 15.0, 0.0);

    // Central "sun"
    let sun = world.spawn();
    world.add_component(sun, Name::new("CentralSun"));
    world.add_component(
        sun,
        Transform {
            translation: center,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.5),
        },
    );

    // Add sphere mesh for sun
    add_sphere_mesh(world, sun, 1.0, 24);

    world.add_component(
        sun,
        PbrMaterial {
            albedo: Color::rgb(0.1, 0.1, 0.1),
            albedo_texture: None,
            normal_texture: None,
            metallic: 0.0,
            roughness: 0.1,
            metallic_roughness_texture: None,
            emissive: Color::rgb(5.0, 3.0, 0.5),
        },
    );

    world.add_component(
        sun,
        PointLight {
            color: Color::rgb(1.0, 0.9, 0.6),
            intensity: 3.0,
            range: 50.0,
            cast_shadows: true,
        },
    );

    world.add_component(
        sun,
        PulsatingLight {
            base_intensity: 3.0,
            pulse_speed: 1.0,
            pulse_amount: 0.5,
            phase: 0.0,
        },
    );

    // Orbiting "planets"
    let planet_configs = [
        (5.0, 0.8, 0.8, Color::rgb(0.3, 0.5, 0.9)),
        (8.0, 0.6, 1.0, Color::rgb(0.9, 0.3, 0.3)),
        (11.0, 0.4, 1.2, Color::rgb(0.3, 0.9, 0.5)),
        (14.0, 0.3, 0.7, Color::rgb(0.9, 0.7, 0.3)),
        (17.0, 0.25, 0.9, Color::rgb(0.7, 0.3, 0.9)),
    ];

    for (i, (radius, speed, size, color)) in planet_configs.iter().enumerate() {
        let planet = world.spawn();
        world.add_component(planet, Name::new(format!("Planet_{}", i)));

        let initial_angle = (i as f32 / planet_configs.len() as f32) * PI * 2.0;
        let x = center.x + radius * initial_angle.cos();
        let z = center.z + radius * initial_angle.sin();

        world.add_component(
            planet,
            Transform {
                translation: Vec3::new(x, center.y, z),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(*size),
            },
        );

        // Add sphere mesh for planet
        add_sphere_mesh(world, planet, 1.0, 16);

        world.add_component(
            planet,
            PbrMaterial {
                albedo: *color,
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.7,
                roughness: 0.3,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 0.3, color.g * 0.3, color.b * 0.3),
            },
        );

        world.add_component(
            planet,
            OrbitalMotion {
                center,
                radius: *radius,
                speed: *speed,
                angle: initial_angle,
                height_offset: i as f32 * 0.5,
                vertical_speed: 0.3 + i as f32 * 0.1,
            },
        );

        world.add_component(
            planet,
            TrailEffect {
                color: *color,
                lifetime: 1.5,
                spawn_rate: 15.0,
                accumulator: 0.0,
                last_position: Vec3::new(x, center.y, z),
            },
        );

        world.add_component(
            planet,
            PointLight {
                color: *color,
                intensity: 0.4,
                range: 6.0,
                cast_shadows: false,
            },
        );
    }
}

// ============================================================================
// Magnetic Field Demonstration
// ============================================================================

pub fn create_magnetic_field_demo(world: &mut World) {
    // Create magnetic field sources
    let field_positions = [
        (Vec3::new(-20.0, 8.0, 20.0), true, Color::rgb(0.2, 0.5, 1.0)), // Attractor
        (Vec3::new(20.0, 8.0, 20.0), false, Color::rgb(1.0, 0.3, 0.2)), // Repeller
    ];

    for (i, (pos, attract, color)) in field_positions.iter().enumerate() {
        let field = world.spawn();
        let name = if *attract {
            "MagneticAttractor"
        } else {
            "MagneticRepeller"
        };
        world.add_component(field, Name::new(format!("{}_{}", name, i)));
        world.add_component(
            field,
            Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(1.5),
            },
        );

        // Add sphere mesh for magnetic field source
        add_sphere_mesh(world, field, 1.0, 16);

        world.add_component(
            field,
            PbrMaterial {
                albedo: Color::rgb(0.1, 0.1, 0.1),
                albedo_texture: None,
                normal_texture: None,
                metallic: 0.9,
                roughness: 0.1,
                metallic_roughness_texture: None,
                emissive: Color::rgb(color.r * 2.0, color.g * 2.0, color.b * 2.0),
            },
        );

        world.add_component(
            field,
            MagneticField {
                strength: 50.0,
                radius: 15.0,
                attract: *attract,
            },
        );

        world.add_component(
            field,
            PointLight {
                color: *color,
                intensity: 1.5,
                range: 20.0,
                cast_shadows: false,
            },
        );

        world.add_component(
            field,
            PulsatingLight {
                base_intensity: 1.5,
                pulse_speed: 2.0,
                pulse_amount: 0.8,
                phase: 0.0,
            },
        );

        // Spawn particles around the field
        for j in 0..20 {
            let angle = (j as f32 / 20.0) * PI * 2.0;
            let offset_radius = 3.0 + (j as f32 * 0.2);
            let particle_pos = *pos
                + Vec3::new(
                    angle.cos() * offset_radius,
                    (j as f32 * 0.3).sin() * 2.0,
                    angle.sin() * offset_radius,
                );

            let particle = world.spawn();
            world.add_component(particle, Name::new(format!("MagneticParticle_{}_{}", i, j)));
            world.add_component(
                particle,
                Transform {
                    translation: particle_pos,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(0.3),
                },
            );

            // Add small sphere mesh for particle
            add_sphere_mesh(world, particle, 0.3, 12);

            world.add_component(
                particle,
                PbrMaterial {
                    albedo: *color,
                    albedo_texture: None,
                    normal_texture: None,
                    metallic: 0.8,
                    roughness: 0.2,
                    metallic_roughness_texture: None,
                    emissive: Color::rgb(color.r * 0.5, color.g * 0.5, color.b * 0.5),
                },
            );

            world.add_component(
                particle,
                RigidBody {
                    body_type: RigidBodyType::Dynamic,
                    mass: 0.2,
                    linear_damping: 0.5,
                    angular_damping: 0.5,
                    gravity_scale: 0.0,
                },
            );

            world.add_component(
                particle,
                Collider {
                    shape: ColliderShape::Sphere { radius: 0.3 },
                    friction: 0.3,
                    restitution: 0.8,
                    is_sensor: false,
                },
            );

            world.add_component(
                particle,
                TrailEffect {
                    color: *color,
                    lifetime: 0.5,
                    spawn_rate: 10.0,
                    accumulator: 0.0,
                    last_position: particle_pos,
                },
            );
        }
    }
}
