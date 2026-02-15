use crate::entity::Entity;
use std::alloc::Layout;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(Entity),
    #[error("Component not registered/found")]
    ComponentError,
    #[error("Archetype integrity error: {0}")]
    ArchetypeError(String),
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Allocation error: {0:?}")]
    AllocationError(Layout),
}
