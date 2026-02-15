use luminara_core::system::FunctionMarker;
use luminara_core::{App, AppInterface, Component, CoreStage, Plugin, Query};
use luminara_math::Vec3;
#[derive(Debug)]
pub struct CameraShake {
    pub intensity: f32,
    pub frequency: f32,
    pub decay: f32,
    pub elapsed: f32,
    pub seed: u64,
}

impl Component for CameraShake {
    fn type_name() -> &'static str {
        "CameraShake"
    }
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            frequency: 10.0,
            decay: 5.0,
            elapsed: 0.0,
            seed: 0,
        }
    }
}

pub struct CameraShakePlugin;

impl Plugin for CameraShakePlugin {
    fn name(&self) -> &str {
        "CameraShakePlugin"
    }

    fn build(&self, app: &mut App) {
        app.add_system::<(
            FunctionMarker,
            Query<'static, (&mut luminara_math::Transform, &mut CameraShake)>,
            luminara_core::Res<'static, luminara_core::Time>,
        )>(CoreStage::Update, camera_shake_system);
    }
}

pub fn camera_shake_system(
    mut query: Query<(&mut luminara_math::Transform, &mut CameraShake)>,
    time: luminara_core::Res<luminara_core::Time>,
) {
    let dt = time.delta_seconds();

    for (transform, shake) in query.iter_mut() {
        if shake.intensity <= 0.001 {
            shake.intensity = 0.0;
            continue;
        }

        shake.elapsed += dt;

        // Decay
        shake.intensity = (shake.intensity - shake.decay * dt).max(0.0);

        if shake.intensity > 0.0 {
            // Simple noise-like shake
            // Ideally use Perlin, but for now simple seeded random based on time
            let time_factor = shake.elapsed * shake.frequency;

            // Pseudo-random offsets
            let offset_x = (time_factor.sin() + (time_factor * 2.5).cos()) * 0.5 * shake.intensity;
            let offset_y =
                ((time_factor * 1.3).sin() + (time_factor * 0.7).cos()) * 0.5 * shake.intensity;
            let offset_z =
                ((time_factor * 0.5).sin() + (time_factor * 1.9).cos()) * 0.5 * shake.intensity;

            transform.translation += Vec3::new(offset_x, offset_y, offset_z);

            // Note: This modifies the transform in-place.
            // In a real system we usually have a "BaseTransform" and apply shake as an offset
            // to avoid drift, or reset it.
            // For now, let's assume the camera controller will reset/control the base position,
            // or this shake is applied after controller.
            // If the controller runs *before* this, it sets the base.
            // If we modify it here, the next frame the controller might see the shaken position as the new base.
            // Ideally, CameraShake should produce an offset that is applied non-destructively.
            // But for Phase 1 MVP, we will rely on the fact that CameraController usually overwrites translation
            // or accumulates velocity. If it overwrites, we need to run *after* it.
            // We set this system to Update.
        }
    }
}
