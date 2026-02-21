use bevy::prelude::*;
use luminara_math::algebra::{Motor as MathMotor, Rotor as MathRotor};

#[derive(Component, Clone, Copy, Debug)]
pub struct Motor(pub MathMotor<f32>);

impl Default for Motor {
    fn default() -> Self {
        Self(MathMotor::IDENTITY)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Rotor(pub MathRotor<f32>);

impl Default for Rotor {
    fn default() -> Self {
        Self(MathRotor::IDENTITY)
    }
}

pub struct LuminaraCorePlugin;

impl Plugin for LuminaraCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_transform_system);
    }
}

fn sync_transform_system(mut query: Query<(&Motor, &mut Transform)>) {
    for (motor, mut transform) in query.iter_mut() {
        let (rot, trans) = motor.0.to_rotation_translation_glam();
        // Manual conversion to handle potential glam version mismatch
        transform.translation = Vec3::new(trans.x, trans.y, trans.z);
        transform.rotation = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);
    }
}
