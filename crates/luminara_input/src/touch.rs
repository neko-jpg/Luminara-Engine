use luminara_core::shared_types::Resource;
use luminara_math::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TouchInput {
    pub touches: HashMap<u64, TouchPoint>,
}

impl Resource for TouchInput {}

impl TouchInput {
    pub fn handle_event(&mut self, event: &winit::event::Touch) {
        let id = event.id;
        let pos = Vec2::new(event.location.x as f32, event.location.y as f32);
        let phase = match event.phase {
            winit::event::TouchPhase::Started => TouchPhase::Started,
            winit::event::TouchPhase::Moved => TouchPhase::Moved,
            winit::event::TouchPhase::Ended => TouchPhase::Ended,
            winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
        };

        match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                self.touches.insert(
                    id,
                    TouchPoint {
                        id,
                        position: pos,
                        phase,
                    },
                );
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.touches.remove(&id);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    pub id: u64,
    pub position: Vec2,
    pub phase: TouchPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}
