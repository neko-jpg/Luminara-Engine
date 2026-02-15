use crate::command::{CommandBuffer, DrawCommand, GizmoType};
use luminara_core::shared_types::Resource;
use luminara_math::{Color, Mat4, Vec3};
use std::collections::HashMap;

// ── Gizmo Category System ──────────────────────────────────────────────────

/// Manages gizmo categories for toggling visibility of debug visualization groups.
#[derive(Debug)]
pub struct GizmoCategories {
    /// Map of category name -> enabled flag.
    categories: HashMap<String, bool>,
}

impl Resource for GizmoCategories {}

impl Default for GizmoCategories {
    fn default() -> Self {
        let mut categories = HashMap::new();
        categories.insert("physics".to_string(), true);
        categories.insert("collision".to_string(), true);
        categories.insert("camera".to_string(), true);
        categories.insert("bounds".to_string(), true);
        categories.insert("skeleton".to_string(), true);
        categories.insert("navigation".to_string(), true);
        categories.insert("default".to_string(), true);
        Self { categories }
    }
}

impl GizmoCategories {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a category enabled/disabled.
    pub fn set_enabled(&mut self, category: &str, enabled: bool) {
        self.categories.insert(category.to_string(), enabled);
    }

    /// Toggle a category on/off. Returns the new state.
    pub fn toggle(&mut self, category: &str) -> bool {
        let entry = self.categories.entry(category.to_string()).or_insert(true);
        *entry = !*entry;
        *entry
    }

    /// Check if a category is enabled (defaults to true if unknown).
    pub fn is_enabled(&self, category: &str) -> bool {
        *self.categories.get(category).unwrap_or(&true)
    }

    /// Enable all categories.
    pub fn enable_all(&mut self) {
        for v in self.categories.values_mut() {
            *v = true;
        }
    }

    /// Disable all categories.
    pub fn disable_all(&mut self) {
        for v in self.categories.values_mut() {
            *v = false;
        }
    }

    /// List all registered categories and their states.
    pub fn list(&self) -> Vec<(&str, bool)> {
        self.categories
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect()
    }
}

// ── Gizmo Drawing Helpers ──────────────────────────────────────────────────

pub struct Gizmos;

impl Gizmos {
    /// Draw a line with default category.
    pub fn line(buffer: &mut CommandBuffer, start: Vec3, end: Vec3, color: Color) {
        Self::line_cat(buffer, start, end, color, "default");
    }

    /// Draw a line with a specific category.
    pub fn line_cat(
        buffer: &mut CommandBuffer,
        start: Vec3,
        end: Vec3,
        color: Color,
        _category: &str,
    ) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Line {
                start: [start.x, start.y, start.z],
                end: [end.x, end.y, end.z],
            },
            transform: Mat4::IDENTITY,
            color,
        });
    }

    /// Draw a sphere with default category.
    pub fn sphere(buffer: &mut CommandBuffer, position: Vec3, radius: f32, color: Color) {
        Self::sphere_cat(buffer, position, radius, color, "default");
    }

    /// Draw a sphere with a specific category.
    pub fn sphere_cat(
        buffer: &mut CommandBuffer,
        position: Vec3,
        radius: f32,
        color: Color,
        _category: &str,
    ) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Sphere { radius },
            transform: Mat4::from_translation(position),
            color,
        });
    }

    /// Draw a cube/box with default category.
    pub fn cube(buffer: &mut CommandBuffer, position: Vec3, half_extents: Vec3, color: Color) {
        Self::cube_cat(buffer, position, half_extents, color, "default");
    }

    /// Draw a cube/box with a specific category.
    pub fn cube_cat(
        buffer: &mut CommandBuffer,
        position: Vec3,
        half_extents: Vec3,
        color: Color,
        _category: &str,
    ) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Box {
                half_extents: [half_extents.x, half_extents.y, half_extents.z],
            },
            transform: Mat4::from_translation(position),
            color,
        });
    }

    /// Draw a capsule gizmo.
    pub fn capsule(
        buffer: &mut CommandBuffer,
        position: Vec3,
        radius: f32,
        height: f32,
        color: Color,
    ) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Capsule { radius, height },
            transform: Mat4::from_translation(position),
            color,
        });
    }

    /// Draw an arrow gizmo.
    pub fn arrow(buffer: &mut CommandBuffer, start: Vec3, end: Vec3, color: Color) {
        buffer.push(DrawCommand::DrawGizmo {
            gizmo: GizmoType::Arrow {
                start: [start.x, start.y, start.z],
                end: [end.x, end.y, end.z],
            },
            transform: Mat4::IDENTITY,
            color,
        });
    }

    /// Draw a 3D grid on the XZ plane.
    pub fn grid(buffer: &mut CommandBuffer, center: Vec3, size: f32, divisions: u32, color: Color) {
        let half = size * 0.5;
        let step = size / divisions as f32;
        for i in 0..=divisions {
            let offset = -half + i as f32 * step;
            // Lines along Z axis
            Self::line(
                buffer,
                center + Vec3::new(offset, 0.0, -half),
                center + Vec3::new(offset, 0.0, half),
                color,
            );
            // Lines along X axis
            Self::line(
                buffer,
                center + Vec3::new(-half, 0.0, offset),
                center + Vec3::new(half, 0.0, offset),
                color,
            );
        }
    }

    /// Draw a wireframe circle in the XZ plane.
    pub fn circle(
        buffer: &mut CommandBuffer,
        center: Vec3,
        radius: f32,
        segments: u32,
        color: Color,
    ) {
        let step = std::f32::consts::TAU / segments as f32;
        for i in 0..segments {
            let a0 = i as f32 * step;
            let a1 = (i + 1) as f32 * step;
            Self::line(
                buffer,
                center + Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius),
                center + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius),
                color,
            );
        }
    }

    /// Draw a coordinate axes gizmo (RGB = XYZ).
    pub fn axes(buffer: &mut CommandBuffer, origin: Vec3, length: f32) {
        Self::arrow(
            buffer,
            origin,
            origin + Vec3::new(length, 0.0, 0.0),
            Color::rgb(1.0, 0.0, 0.0),
        );
        Self::arrow(
            buffer,
            origin,
            origin + Vec3::new(0.0, length, 0.0),
            Color::rgb(0.0, 1.0, 0.0),
        );
        Self::arrow(
            buffer,
            origin,
            origin + Vec3::new(0.0, 0.0, length),
            Color::rgb(0.0, 0.0, 1.0),
        );
    }
}
