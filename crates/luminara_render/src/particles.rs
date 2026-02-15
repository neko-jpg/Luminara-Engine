use crate::GpuContext;
use luminara_core::system::FunctionMarker;
use luminara_core::{App, AppInterface, Component, CoreStage, Plugin, Query, Res, ResMut};
use luminara_math::{Color, Mat4, Vec3};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ParticleEmitter {
    pub rate: f32, // Particles per second
    pub accumulator: f32,
    pub direction: Vec3,
    pub spread: f32,
    pub speed: f32,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
}

impl Component for ParticleEmitter {
    fn type_name() -> &'static str {
        "ParticleEmitter"
    }
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            rate: 10.0,
            accumulator: 0.0,
            direction: Vec3::Y,
            spread: 0.5,
            speed: 5.0,
            color: Color::WHITE,
            size: 0.1,
            lifetime: 2.0,
        }
    }
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub capacity: usize,
    pub instance_buffer: Option<wgpu::Buffer>,
}

impl luminara_core::Resource for ParticleSystem {}

impl ParticleSystem {
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),
            capacity,
            instance_buffer: None,
        }
    }

    pub fn spawn(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        color: Color,
        size: f32,
        lifetime: f32,
    ) {
        if self.particles.len() < self.capacity {
            self.particles.push(Particle {
                position,
                velocity,
                color,
                size,
                lifetime,
                max_lifetime: lifetime,
            });
        }
    }
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn name(&self) -> &str {
        "ParticlePlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleSystem::new(10000));
        app.add_system::<(
            FunctionMarker,
            ResMut<'static, ParticleSystem>,
            Query<'static, (&mut ParticleEmitter, &luminara_math::Transform)>,
            Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, particle_emitter_system);

        app.add_system::<(
            FunctionMarker,
            ResMut<'static, ParticleSystem>,
            Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, particle_update_system);
    }
}

pub fn particle_emitter_system(
    mut particle_system: ResMut<ParticleSystem>,
    mut emitters: Query<(&mut ParticleEmitter, &luminara_math::Transform)>,
    time: Res<luminara_core::Time>,
) {
    let dt = time.delta_seconds();

    for (emitter, transform) in emitters.iter_mut() {
        emitter.accumulator += emitter.rate * dt;

        while emitter.accumulator >= 1.0 {
            emitter.accumulator -= 1.0;

            // Simple random spread (deterministic for now for simplicity, ideally use rand)
            // Using system time or similar as seed in a real engine
            let offset = Vec3::new(
                (emitter.accumulator * 123.45).sin() * emitter.spread,
                (emitter.accumulator * 678.90).cos() * emitter.spread,
                (emitter.accumulator * 321.01).sin() * emitter.spread,
            );

            let velocity = (emitter.direction + offset).normalize() * emitter.speed;

            particle_system.spawn(
                transform.translation,
                velocity,
                emitter.color,
                emitter.size,
                emitter.lifetime,
            );
        }
    }
}

pub fn particle_update_system(
    mut particle_system: ResMut<ParticleSystem>,
    time: Res<luminara_core::Time>,
) {
    let dt = time.delta_seconds();
    let gravity = Vec3::new(0.0, -9.81, 0.0);

    // Update particles
    particle_system.particles.retain_mut(|p| {
        p.lifetime -= dt;
        if p.lifetime <= 0.0 {
            return false;
        }

        p.velocity += gravity * dt;
        p.position += p.velocity * dt;

        true
    });
}

// Rendering integration would go here (GPU instancing)
// For Phase 1, we can implement a basic `render_particles` system
// that uploads instance data to a buffer.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleInstance {
    model_matrix_0: [f32; 4],
    model_matrix_1: [f32; 4],
    model_matrix_2: [f32; 4],
    model_matrix_3: [f32; 4],
    color: [f32; 4],
}

pub fn particle_render_prepare_system(
    mut particle_system: ResMut<ParticleSystem>,
    gpu: Res<GpuContext>,
) {
    if particle_system.particles.is_empty() {
        return;
    }

    let instances: Vec<ParticleInstance> = particle_system
        .particles
        .iter()
        .map(|p| {
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::splat(p.size),
                luminara_math::Quat::IDENTITY,
                p.position,
            );
            let cols = transform.to_cols_array_2d();
            ParticleInstance {
                model_matrix_0: cols[0],
                model_matrix_1: cols[1],
                model_matrix_2: cols[2],
                model_matrix_3: cols[3],
                color: [
                    p.color.r,
                    p.color.g,
                    p.color.b,
                    p.color.a * (p.lifetime / p.max_lifetime),
                ],
            }
        })
        .collect();

    let buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    particle_system.instance_buffer = Some(buffer);
}
