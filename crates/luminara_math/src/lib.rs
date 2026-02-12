//! # Luminara Math
//!
//! Provides mathematical primitives and utilities for the Luminara Engine.
//! Powered by `glam`.

pub use glam::{self, Mat4, Quat, Vec2, Vec3, Vec4};
pub use glam::{IVec2, IVec3, IVec4};
pub use glam::{Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};

use luminara_core::Component;
use serde::{Deserialize, Serialize};

/// A color represented by RGBA components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(rgba: [f32; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }
}

/// Represents the position, rotation, and scale of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn mul_transform(&self, other: &Self) -> Self {
        let mat = self.compute_matrix() * other.compute_matrix();
        let (scale, rotation, translation) = mat.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Component for Transform {
    fn type_name() -> &'static str {
        "Transform"
    }
}
