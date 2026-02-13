//! # Luminara Math
//!
//! Provides mathematical primitives and utilities for the Luminara Engine.
//! Powered by `glam`.
//!
//! ## Modules
//!
//! - `foundations`: Exact predicates and adaptive precision arithmetic
//! - `algebra`: PGA Motor, Lie group integrators, dual quaternions
//! - `symbolic`: Symbolic computation engine and code generation
//! - `geometry`: BVH, Reeb graph, manifold surfaces, Heat Method
//! - `dynamics`: Spectral fluid solver and FFT utilities
//! - `dsl`: MathDesignCommand DSL for AI integration

pub use glam::{self, EulerRot, Mat4, Quat, Vec2, Vec3, Vec4};
pub use glam::{IVec2, IVec3, IVec4, UVec2};
pub use glam::{Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};

// Mathematical foundation modules
pub mod algebra;
pub mod dsl;
pub mod dynamics;
pub mod foundations;
pub mod geometry;
pub mod symbolic;

use luminara_core::Component;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

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
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

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

// ---------------------------------------------------------------------------
// Rect
// ---------------------------------------------------------------------------

/// An axis-aligned rectangle defined by minimum and maximum corners.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    /// Create a new Rect from min/max corners.
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    /// Create a Rect from a center point and half-extents.
    pub fn from_center_half_size(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Width of the rectangle.
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height of the rectangle.
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Size as (width, height).
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Center point of the rectangle.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Returns true if `point` is inside the rectangle.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        }
    }
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

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

    /// Shorthand for `from_translation(Vec3::new(x, y, z))`.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
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

    /// Create a transform that looks at `target` from the current translation.
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.translation).normalize();
        if forward.length_squared() < f32::EPSILON {
            return self;
        }
        let right = up.cross(forward).normalize();
        let corrected_up = forward.cross(right);
        self.rotation = Quat::from_mat4(&Mat4::from_cols(
            right.extend(0.0),
            corrected_up.extend(0.0),
            forward.extend(0.0),
            Vec4::W,
        ));
        self
    }

    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Convert the transform to a 4x4 transformation matrix.
    /// This is an alias for `compute_matrix()` to match the design specification.
    pub fn to_matrix(&self) -> Mat4 {
        self.compute_matrix()
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

    /// Rotate around the local X axis by `angle` radians.
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_x(angle);
    }

    /// Rotate around the local Y axis by `angle` radians.
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_y(angle);
    }

    /// Rotate around the local Z axis by `angle` radians.
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_z(angle);
    }

    /// Get the forward direction vector (local -Z axis in world space).
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction vector (local X axis in world space).
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction vector (local Y axis in world space).
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
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
