//! Component Binding Service (Stub)

use std::sync::Arc;
use crate::services::engine_bridge::EngineHandle;

pub struct ComponentBinder;

impl ComponentBinder {
    pub fn new(_engine: Arc<EngineHandle>) -> Self {
        Self
    }
}

pub struct ComponentUpdateCommand;
