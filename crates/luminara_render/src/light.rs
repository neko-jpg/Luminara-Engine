//! Light system - Directional, Point, and Spot lights

use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
    pub position: Vec3,
    pub direction: Vec3,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub shadows_enabled: bool,
    pub shadow_bias: f32,
    pub shadow_near: f32,
    pub shadow_far: f32,
    pub shadow_resolution: u32,
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self::directional()
    }
}

impl Light {
    pub fn directional() -> Self {
        Self {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            position: Vec3::new(0.0, 10.0, 0.0),
            direction: Vec3::new(-0.5, -1.0, -0.5).normalize(),
            range: 0.0,
            inner_angle: 0.0,
            outer_angle: 0.0,
            shadows_enabled: true,
            shadow_bias: 0.005,
            shadow_near: 0.1,
            shadow_far: 100.0,
            shadow_resolution: 2048,
            enabled: true,
        }
    }

    pub fn point() -> Self {
        Self {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            position: Vec3::ZERO,
            direction: Vec3::ZERO,
            range: 10.0,
            inner_angle: 0.0,
            outer_angle: 0.0,
            shadows_enabled: false,
            shadow_bias: 0.01,
            shadow_near: 0.1,
            shadow_far: 20.0,
            shadow_resolution: 1024,
            enabled: true,
        }
    }

    pub fn spot() -> Self {
        Self {
            light_type: LightType::Spot,
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            position: Vec3::ZERO,
            direction: Vec3::NEG_Y,
            range: 10.0,
            inner_angle: 30.0_f32.to_radians(),
            outer_angle: 45.0_f32.to_radians(),
            shadows_enabled: false,
            shadow_bias: 0.01,
            shadow_near: 0.1,
            shadow_far: 20.0,
            shadow_resolution: 1024,
            enabled: true,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = direction.normalize();
        self
    }

    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = color;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn with_shadows(mut self, enabled: bool) -> Self {
        self.shadows_enabled = enabled;
        self
    }

    pub fn is_directional(&self) -> bool {
        self.light_type == LightType::Directional
    }

    pub fn is_point(&self) -> bool {
        self.light_type == LightType::Point
    }

    pub fn is_spot(&self) -> bool {
        self.light_type == LightType::Spot
    }

    pub fn compute_attenuation(&self, distance: f32) -> f32 {
        if self.range <= 0.0 {
            return 1.0;
        }

        let d = distance.min(self.range);
        let d2 = d * d;
        let range2 = self.range * self.range;

        1.0 - (d2 / range2).powf(2.0)
    }

    pub fn compute_spot_angle_attenuation(&self, spot_dir: Vec3) -> f32 {
        if !self.is_spot() {
            return 1.0;
        }

        let cos_outer = self.outer_angle.cos();
        let cos_inner = self.inner_angle.cos();
        let cos_angle = spot_dir.dot(-self.direction);

        if cos_angle < cos_outer {
            return 0.0;
        }

        let epsilon = cos_inner - cos_outer;
        let t = (cos_angle - cos_outer) / epsilon;
        t * t
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LightBundle {
    pub light: Light,
    pub intensity: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LightData {
    pub position: Vec4,
    pub color: Vec4,
    pub direction: Vec4,
    pub params: Vec4,
    pub shadow: Vec4,
}

impl LightData {
    pub fn from_light(light: &Light) -> Self {
        let (shadow_near, shadow_far, shadow_res, shadow_flags) = if light.shadows_enabled {
            (
                light.shadow_near,
                light.shadow_far,
                light.shadow_resolution as f32,
                1.0,
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        Self {
            position: Vec4::new(
                light.position.x,
                light.position.y,
                light.position.z,
                light.range,
            ),
            color: Vec4::new(
                light.color.x * light.intensity,
                light.color.y * light.intensity,
                light.color.z * light.intensity,
                light.intensity,
            ),
            direction: Vec4::new(
                light.direction.x,
                light.direction.y,
                light.direction.z,
                if light.is_spot() {
                    light.inner_angle.cos()
                } else {
                    0.0
                },
            ),
            params: Vec4::new(
                light.outer_angle.cos(),
                if light.is_spot() { 1.0 } else { 0.0 },
                light.reflectance.unwrap_or(0.0),
                light.metallic.unwrap_or(0.0),
            ),
            shadow: Vec4::new(shadow_near, shadow_far, shadow_res, shadow_flags),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirectionalLightData {
    pub direction: Vec4,
    pub color: Vec4,
    pub shadow_matrix: [f32; 16],
    pub shadow_params: Vec4,
}

impl DirectionalLightData {
    pub fn from_light(light: &Light, view_projection: glam::Mat4) -> Self {
        Self {
            direction: Vec4::new(light.direction.x, light.direction.y, light.direction.z, 0.0),
            color: Vec4::new(
                light.color.x * light.intensity,
                light.color.y * light.intensity,
                light.color.z * light.intensity,
                light.intensity,
            ),
            shadow_matrix: view_projection.to_cols_array(),
            shadow_params: Vec4::new(
                if light.shadows_enabled { 1.0 } else { 0.0 },
                light.shadow_bias,
                light.shadow_resolution as f32,
                0.0,
            ),
        }
    }
}
