//! Scene graph and renderable objects

use crate::camera::Camera;
use crate::light::Light;
use crate::material::{Material, PbrMaterial};
use crate::mesh::Mesh;
use crate::texture::Texture;
use glam::{Mat4, Vec3};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderObject {
    pub id: u64,
    pub transform: Mat4,
    pub mesh: Arc<Mesh>,
    pub material: Arc<dyn Material>,
    pub visible: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

impl RenderObject {
    pub fn new(id: u64, mesh: Arc<Mesh>, material: Arc<dyn Material>) -> Self {
        Self {
            id,
            transform: Mat4::IDENTITY,
            mesh,
            material,
            visible: true,
            cast_shadows: true,
            receive_shadows: true,
        }
    }

    pub fn with_transform(mut self, transform: Mat4) -> Self {
        self.transform = transform;
        self
    }

    pub fn aabb(&self) -> Option<Aabb> {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for vertex in &self.mesh.vertices {
            let world_pos = self.transform * vertex.position.extend(1.0);
            min = min.min(world_pos.xyz());
            max = max.max(world_pos.xyz());
        }

        Some(Aabb { min, max })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    pub fn extents(&self) -> Vec3 {
        self.max - self.min
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub objects: RwLock<HashMap<u64, RenderObject>>,
    pub lights: RwLock<Vec<Light>>,
    pub camera: RwLock<Option<Camera>>,
    pub directional_light: RwLock<Option<Light>>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&self, object: RenderObject) {
        self.objects.write().insert(object.id, object);
    }

    pub fn remove_object(&self, id: u64) {
        self.objects.write().remove(&id);
    }

    pub fn get_object(&self, id: u64) -> Option<RenderObject> {
        self.objects.read().get(&id).cloned()
    }

    pub fn objects(&self) -> Vec<RenderObject> {
        self.objects.read().values().cloned().collect()
    }

    pub fn visible_objects(&self) -> Vec<RenderObject> {
        self.objects
            .read()
            .values()
            .filter(|o| o.visible)
            .cloned()
            .collect()
    }

    pub fn add_light(&self, light: Light) {
        self.lights.write().push(light);
    }

    pub fn lights(&self) -> Vec<Light> {
        self.lights.read().clone()
    }

    pub fn set_directional_light(&self, light: Light) {
        *self.directional_light.write() = Some(light);
    }

    pub fn directional_light(&self) -> Option<Light> {
        self.directional_light.read().clone()
    }

    pub fn set_camera(&self, camera: Camera) {
        *self.camera.write() = Some(camera);
    }

    pub fn camera(&self) -> Option<Camera> {
        self.camera.read().clone()
    }

    pub fn clear(&self) {
        self.objects.write().clear();
        self.lights.write().clear();
        *self.camera.write() = None;
        *self.directional_light.write() = None;
    }
}

#[derive(Debug, Clone)]
pub struct RenderScene {
    pub scene: Arc<Scene>,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub view_projection: Mat4,
}

impl RenderScene {
    pub fn new(scene: Arc<Scene>, camera: &Camera) -> Self {
        Self {
            scene,
            view_matrix: camera.view_matrix,
            projection_matrix: camera.projection.projection_matrix(),
            view_projection: camera.view_projection,
        }
    }
}
