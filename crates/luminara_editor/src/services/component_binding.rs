//! Component Binding System
//!
//! Provides two-way data binding between UI and ECS components.
//! This enables real-time synchronization between editor panels and the game world.

use luminara_core::{Entity, World, Component};
use luminara_math::{Transform, Vec3, Quat};
use luminara_scene::{Name, Tag};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// A command to update a component in the ECS
#[derive(Debug, Clone)]
pub enum ComponentUpdateCommand {
    /// Update transform position
    SetPosition { entity: Entity, position: Vec3 },
    /// Update transform rotation
    SetRotation { entity: Entity, rotation: Quat },
    /// Update transform scale
    SetScale { entity: Entity, scale: Vec3 },
    /// Update entire transform
    SetTransform { entity: Entity, transform: Transform },
    /// Update entity name
    SetName { entity: Entity, name: String },
    /// Add a tag to entity
    AddTag { entity: Entity, tag: String },
    /// Remove a tag from entity
    RemoveTag { entity: Entity, tag: String },
    /// Toggle entity active state
    SetActive { entity: Entity, active: bool },
}

impl ComponentUpdateCommand {
    /// Execute the update command on the world
    pub fn execute(&self, world: &mut World) -> Result<(), String> {
        match self {
            ComponentUpdateCommand::SetPosition { entity, position } => {
                if let Some(transform) = world.get_component_mut::<Transform>(*entity) {
                    transform.translation = *position;
                    Ok(())
                } else {
                    // Add transform if it doesn't exist
                    world.add_component(*entity, Transform::from_translation(*position))
                        .map_err(|e| format!("Failed to add transform: {}", e))
                }
            }
            ComponentUpdateCommand::SetRotation { entity, rotation } => {
                if let Some(transform) = world.get_component_mut::<Transform>(*entity) {
                    transform.rotation = *rotation;
                    Ok(())
                } else {
                    let mut transform = Transform::IDENTITY;
                    transform.rotation = *rotation;
                    world.add_component(*entity, transform)
                        .map_err(|e| format!("Failed to add transform: {}", e))
                }
            }
            ComponentUpdateCommand::SetScale { entity, scale } => {
                if let Some(transform) = world.get_component_mut::<Transform>(*entity) {
                    transform.scale = *scale;
                    Ok(())
                } else {
                    let mut transform = Transform::IDENTITY;
                    transform.scale = *scale;
                    world.add_component(*entity, transform)
                        .map_err(|e| format!("Failed to add transform: {}", e))
                }
            }
            ComponentUpdateCommand::SetTransform { entity, transform } => {
                world.add_component(*entity, *transform)
                    .map_err(|e| format!("Failed to update transform: {}", e))
            }
            ComponentUpdateCommand::SetName { entity, name } => {
                world.add_component(*entity, Name::new(name.clone()))
                    .map_err(|e| format!("Failed to update name: {}", e))
            }
            ComponentUpdateCommand::AddTag { entity, tag } => {
                let mut tag_component = world.get_component::<Tag>(*entity)
                    .cloned()
                    .unwrap_or_default();
                tag_component.insert(tag.clone());
                world.add_component(*entity, tag_component)
                    .map_err(|e| format!("Failed to add tag: {}", e))
            }
            ComponentUpdateCommand::RemoveTag { entity, tag } => {
                if let Some(tag_component) = world.get_component_mut::<Tag>(*entity) {
                    tag_component.0.remove(tag);
                    Ok(())
                } else {
                    Err("Entity has no tags".to_string())
                }
            }
            ComponentUpdateCommand::SetActive { entity: _, active: _ } => {
                // Active state would be stored in a specific component
                // For now, this is a placeholder
                Ok(())
            }
        }
    }

    /// Get the entity affected by this command
    pub fn entity(&self) -> Entity {
        match self {
            ComponentUpdateCommand::SetPosition { entity, .. } => *entity,
            ComponentUpdateCommand::SetRotation { entity, .. } => *entity,
            ComponentUpdateCommand::SetScale { entity, .. } => *entity,
            ComponentUpdateCommand::SetTransform { entity, .. } => *entity,
            ComponentUpdateCommand::SetName { entity, .. } => *entity,
            ComponentUpdateCommand::AddTag { entity, .. } => *entity,
            ComponentUpdateCommand::RemoveTag { entity, .. } => *entity,
            ComponentUpdateCommand::SetActive { entity, .. } => *entity,
        }
    }
}

/// Component binding manager for tracking UI-ECS synchronization
pub struct ComponentBindingManager {
    pending_updates: Arc<RwLock<Vec<ComponentUpdateCommand>>>,
    update_callbacks: Arc<RwLock<Vec<Box<dyn Fn(&ComponentUpdateCommand) + Send + Sync>>>>,
}

impl ComponentBindingManager {
    pub fn new() -> Self {
        Self {
            pending_updates: Arc::new(RwLock::new(Vec::new())),
            update_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Queue a component update
    pub fn queue_update(&self, command: ComponentUpdateCommand) {
        self.pending_updates.write().push(command);
    }

    /// Apply all pending updates to the world
    pub fn apply_pending_updates(&self, world: &mut World) -> Vec<Result<(), String>> {
        let mut updates = self.pending_updates.write();
        let results: Vec<_> = updates.drain(..).map(|cmd| {
            let result = cmd.execute(world);
            if result.is_ok() {
                // Notify callbacks
                for callback in self.update_callbacks.read().iter() {
                    callback(&cmd);
                }
            }
            result
        }).collect();
        results
    }

    /// Subscribe to update events
    pub fn on_update<F>(&self, callback: F)
    where
        F: Fn(&ComponentUpdateCommand) + Send + Sync + 'static,
    {
        self.update_callbacks.write().push(Box::new(callback));
    }

    /// Get pending update count
    pub fn pending_count(&self) -> usize {
        self.pending_updates.read().len()
    }

    /// Clear all pending updates
    pub fn clear_pending(&self) {
        self.pending_updates.write().clear();
    }
}

impl Default for ComponentBindingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for editable component values
pub trait EditableComponent: Clone + PartialEq {
    type Value;
    
    /// Get the current value
    fn get_value(&self) -> Self::Value;
    
    /// Set a new value
    fn set_value(&mut self, value: Self::Value);
    
    /// Check if the value has changed
    fn has_changed(&self, other: &Self) -> bool {
        self != other
    }
}

/// Editor command for undoable operations
#[derive(Debug, Clone)]
pub struct EditCommand {
    pub name: String,
    pub entity: Entity,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub component_type: String,
}

impl EditCommand {
    pub fn new(
        name: impl Into<String>,
        entity: Entity,
        component_type: impl Into<String>,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            entity,
            component_type: component_type.into(),
            old_value,
            new_value,
        }
    }
}

/// Command history for undo/redo
pub struct CommandHistory {
    commands: Vec<EditCommand>,
    current_index: isize,
    max_size: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: Vec::new(),
            current_index: -1,
            max_size,
        }
    }

    /// Add a new command to history
    pub fn push(&mut self, command: EditCommand) {
        // Remove any commands after current index (redo history)
        if self.current_index < self.commands.len() as isize - 1 {
            self.commands.truncate((self.current_index + 1) as usize);
        }

        self.commands.push(command);

        // Maintain max size
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
        } else {
            self.current_index += 1;
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.current_index >= 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.current_index < self.commands.len() as isize - 1
    }

    /// Get the command to undo
    pub fn undo_command(&self) -> Option<&EditCommand> {
        if self.can_undo() {
            Some(&self.commands[self.current_index as usize])
        } else {
            None
        }
    }

    /// Get the command to redo
    pub fn redo_command(&self) -> Option<&EditCommand> {
        if self.can_redo() {
            Some(&self.commands[(self.current_index + 1) as usize])
        } else {
            None
        }
    }

    /// Move back in history (undo)
    pub fn undo(&mut self) -> Option<&EditCommand> {
        if self.can_undo() {
            let cmd = &self.commands[self.current_index as usize];
            self.current_index -= 1;
            Some(cmd)
        } else {
            None
        }
    }

    /// Move forward in history (redo)
    pub fn redo(&mut self) -> Option<&EditCommand> {
        if self.can_redo() {
            self.current_index += 1;
            Some(&self.commands[self.current_index as usize])
        } else {
            None
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = -1;
    }

    /// Get history length
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(100) // Default 100 commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new(10);
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let cmd = EditCommand::new(
            "Test",
            Entity::from_raw(0),
            "Transform",
            serde_json::json!({"x": 0.0}),
            serde_json::json!({"x": 1.0}),
        );
        history.push(cmd);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo();
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }
}
