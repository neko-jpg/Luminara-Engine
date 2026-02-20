//! Global Search Commands
//!
//! Commands for controlling the Global Search functionality.
//! These commands can be triggered from keyboard shortcuts, UI buttons,
//! or menu items.

use crate::core::command_bus::{Command, CommandBus};
use crate::core::state::EditorState;
use std::sync::Arc;
use parking_lot::RwLock;

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
    fn execute(&self, state: &Arc<RwLock<EditorState>>) {
        let mut state = state.write();
        state.toggle_global_search();
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
    fn execute(&self, state: &Arc<RwLock<EditorState>>) {
        let mut state = state.write();
        state.set_global_search_visible(true);
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
    fn execute(&self, state: &Arc<RwLock<EditorState>>) {
        let mut state = state.write();
        state.set_global_search_visible(false);
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
    pub fn execute(&self, state: &Arc<RwLock<EditorState>>) {
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
    command_bus: Arc<RwLock<CommandBus>>,
}

impl GlobalSearchShortcutHandler {
    /// Create a new shortcut handler
    pub fn new(command_bus: Arc<RwLock<CommandBus>>) -> Self {
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
    fn test_toggle_global_search_command() {
        let state = Arc::new(RwLock::new(EditorState::default()));
        
        let cmd = ToggleGlobalSearchCommand::new();
        assert_eq!(cmd.name(), "ToggleGlobalSearch");
        
        // Execute command
        cmd.execute(&state);
        
        // Check state was updated
        let state_guard = state.read();
        assert!(state_guard.global_search_visible);
    }
    
    #[test]
    fn test_open_global_search_command() {
        let state = Arc::new(RwLock::new(EditorState::default()));
        
        let cmd = OpenGlobalSearchCommand::new();
        cmd.execute(&state);
        
        let state_guard = state.read();
        assert!(state_guard.global_search_visible);
    }
    
    #[test]
    fn test_close_global_search_command() {
        let state = Arc::new(RwLock::new(EditorState::default()));
        
        // First open it
        {
            let mut state_guard = state.write();
            state_guard.set_global_search_visible(true);
        }
        
        // Then close it
        let cmd = CloseGlobalSearchCommand::new();
        cmd.execute(&state);
        
        let state_guard = state.read();
        assert!(!state_guard.global_search_visible);
    }
    
    #[test]
    fn test_global_search_command_enum() {
        let state = Arc::new(RwLock::new(EditorState::default()));
        
        // Test Toggle
        GlobalSearchCommand::Toggle.execute(&state);
        {
            let guard = state.read();
            assert!(guard.global_search_visible);
        }
        
        // Test Close
        GlobalSearchCommand::Close.execute(&state);
        {
            let guard = state.read();
            assert!(!guard.global_search_visible);
        }
        
        // Test Open
        GlobalSearchCommand::Open.execute(&state);
        {
            let guard = state.read();
            assert!(guard.global_search_visible);
        }
    }
    
    #[test]
    fn test_is_global_search_shortcut() {
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "k"));
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(false, true, "k"));
        assert!(GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "K"));
        
        assert!(!GlobalSearchShortcutHandler::is_global_search_shortcut(false, false, "k"));
        assert!(!GlobalSearchShortcutHandler::is_global_search_shortcut(true, false, "a"));
    }
}
