use crate::algebra::motor::Motor;
use crate::{Mat4, Quat, Vec3};
use luminara_core::shared_types::Component;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorTransform {
    pub motor: Motor<f32>,
    pub scale: Vec3,
}

impl Component for MotorTransform {
    fn type_name() -> &'static str {
        "MotorTransform"
    }
}

impl Default for MotorTransform {
    fn default() -> Self {
        Self {
            motor: Motor::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl MotorTransform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            motor: Motor::from_translation(translation.into()),
            scale: Vec3::ONE,
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(rotation, Vec3::ZERO),
            scale: Vec3::ONE,
        }
    }

    pub fn from_translation_rotation(translation: Vec3, rotation: Quat) -> Self {
        Self {
            motor: Motor::from_rotation_translation_glam(rotation, translation),
            scale: Vec3::ONE,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let (r, t) = self.to_rotation_translation();
        Mat4::from_scale_rotation_translation(self.scale, r, t)
    }

    pub fn to_rotation_translation(&self) -> (Quat, Vec3) {
        self.motor.to_rotation_translation_glam()
    }

    pub fn compose(&self, other: &Self) -> Self {
        // Geometric product for composition
        let combined_motor = self.motor.geometric_product(&other.motor);
        Self {
            motor: combined_motor,
            scale: self.scale * other.scale,
        }
    }
}
