//! Level of Detail (LOD) system
//! Optimizes rendering by reducing detail for distant objects
#![allow(dead_code)]

use luminara::prelude::*;

/// LOD component - defines multiple detail levels for an entity
#[derive(Debug, Clone)]
pub struct LodComponent {
    /// LOD levels with distance thresholds
    pub levels: Vec<LodLevel>,
    /// Currently active LOD level
    pub current_level: usize,
}

impl Component for LodComponent {
    fn type_name() -> &'static str {
        "LodComponent"
    }
}

#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Maximum distance for this LOD level
    pub max_distance: f32,
    /// Mesh handle for this level (simplified mesh)
    pub mesh_handle: Option<String>,
    /// Whether to render at this level
    pub visible: bool,
}

impl LodComponent {
    pub fn new() -> Self {
        Self {
            levels: vec![
                LodLevel {
                    max_distance: 20.0,
                    mesh_handle: Some("high_detail".to_string()),
                    visible: true,
                },
                LodLevel {
                    max_distance: 50.0,
                    mesh_handle: Some("medium_detail".to_string()),
                    visible: true,
                },
                LodLevel {
                    max_distance: 100.0,
                    mesh_handle: Some("low_detail".to_string()),
                    visible: true,
                },
            ],
            current_level: 0,
        }
    }

    /// Update LOD level based on distance to camera
    pub fn update(&mut self, distance: f32) {
        for (i, level) in self.levels.iter().enumerate() {
            if distance <= level.max_distance {
                self.current_level = i;
                return;
            }
        }
        // If beyond all levels, use the last one
        self.current_level = self.levels.len().saturating_sub(1);
    }
}

/// System to update LOD levels based on camera distance
pub fn lod_update_system(world: &mut World) {
    // Get camera position
    let camera_pos = {
        let mut pos = Vec3::ZERO;
        let query = Query::<(&Camera, &Transform)>::new(world);
        for (cam, transform) in query.iter() {
            if cam.is_active {
                pos = Vec3::new(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                );
                break;
            }
        }
        pos
    };

    // Update LOD for all entities with LodComponent
    let mut query = Query::<(&Transform, &mut LodComponent)>::new(world);
    for (transform, lod) in query.iter_mut() {
        let distance = (transform.translation - camera_pos).length();
        lod.update(distance);
    }
}
