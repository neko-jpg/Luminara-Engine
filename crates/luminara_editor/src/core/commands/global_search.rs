//! Global Search Commands
//!
//! Commands for controlling the Global Search functionality.
use crate::core::command_bus::{Command, CommandBus};
use gpui::Model;
use crate::core::state::EditorStateManager;
use std::sync::Arc;

/// Command to toggle Global Search visibility
#[derive(Debug, Clone, Copy)]
pub struct ToggleGlobalSearchCommand;

impl ToggleGlobalSearchCommand {
    /// Create a new toggle global search command
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToggleGlobalSearchCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ToggleGlobalSearchCommand {
    fn execute(&self, state: &Model<EditorStateManager>) {
        // FIXME: GPUI Models require a context to mutate. 
        // Need to refactor Command trait to accept a generic Context or WindowContext.
        // For now, we leave it as a no-op to fix compiler errors.
    }
    
    fn name(&self) -> &'static str {
        "ToggleGlobalSearch"
    }
}

/// Command to open Global Search
#[derive(Debug, Clone, Copy)]
pub struct OpenGlobalSearchCommand;

impl OpenGlobalSearchCommand {
    /// Create a new open global search command
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenGlobalSearchCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for OpenGlobalSearchCommand {
    fn execute(&self, state: &Model<EditorStateManager>) {
        // FIXME: GPUI Models require a context to mutate.
    }
    
    fn name(&self) -> &'static str {
        "OpenGlobalSearch"
    }
}

/// Command to close Global Search
#[derive(Debug, Clone, Copy)]
pub struct CloseGlobalSearchCommand;

impl CloseGlobalSearchCommand {
    /// Create a new close global search command
    pub fn new() -> Self {
        Self
    }
}

impl Default for CloseGlobalSearchCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for CloseGlobalSearchCommand {
    fn execute(&self, _state: &Model<EditorStateManager>) {
        // FIXME: GPUI Models require a context to mutate.
    }
    
    fn name(&self) -> &'static str {
        "CloseGlobalSearch"
    }
}

/// Enum for convenient command dispatch
#[derive(Debug, Clone, Copy)]
pub enum GlobalSearchCommand {
    /// Toggle global search
    Toggle,
    /// Open global search
    Open,
    /// Close global search
    Close,
}

impl GlobalSearchCommand {
    /// Execute the command
    pub fn execute(&self, state: &Model<EditorStateManager>) {
        match self {
            Self::Toggle => ToggleGlobalSearchCommand::new().execute(state),
            Self::Open => OpenGlobalSearchCommand::new().execute(state),
            Self::Close => CloseGlobalSearchCommand::new().execute(state),
        }
    }
    
    /// Get the command name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Toggle => "Toggle",
            Self::Open => "Open",
            Self::Close => "Close",
        }
    }
}

/// Keyboard shortcut handler for Global Search
///
/// This can be used from a separate keyboard monitoring system
/// outside of GPUI's direct control.
pub struct GlobalSearchShortcutHandler {
    command_bus: Arc<parking_lot::RwLock<CommandBus>>,
}

impl GlobalSearchShortcutHandler {
    /// Create a new shortcut handler
    pub fn new(command_bus: Arc<parking_lot::RwLock<CommandBus>>) -> Self {
        Self { command_bus }
    }
    
    /// Handle Ctrl+K or Cmd+K
    pub fn handle_ctrl_k(&self) {
        let _command = GlobalSearchCommand::Toggle;
        let bus = self.command_bus.read();
        bus.publish(&ToggleGlobalSearchCommand::new());
    }
    
    /// Check if a key combination is the global search shortcut
    /// 
    /// # Arguments
    /// * `ctrl` - Ctrl key pressed
    /// * `cmd` - Command key pressed (Mac)
    /// * `key` - The key that was pressed
    /// 
    /// # Returns
    /// true if this is the global search shortcut
    pub fn is_global_search_shortcut(ctrl: bool, cmd: bool, key: &str) -> bool {
        (ctrl || cmd) && key.eq_ignore_ascii_case("k")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_global_search_shortcut() {
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "k"));
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(false, true, "k"));
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "K"));
        
        assert!(!GlobalSearchShortcutHandler::is_global_search_shortcut(false, false, "k"));
        assert!(!GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "a"));
    }
}
