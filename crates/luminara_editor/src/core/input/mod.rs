//! Input System for Luminara Editor
//!
//! This module provides keyboard input handling outside of GPUI's control,
//! allowing commands to be triggered from keyboard shortcuts even when
//! the application window is not focused (if needed).
//!
//! For now, this uses a simple thread-based approach to monitor keyboard
//! input and dispatch commands to the command system.

use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub mod keyboard;

pub use keyboard::KeyboardMonitor;

/// Trait for input event handlers
pub trait InputHandler: Send + Sync {
    /// Handle a key event
    fn handle_key(&self, key: KeyEvent);
}

/// Represents a keyboard key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Character keys
    Char(char),
    /// Function keys (F1-F24)
    F(u8),
    /// Escape
    Escape,
    /// Enter/Return
    Enter,
    /// Tab
    Tab,
    /// Space
    Space,
    /// Backspace
    Backspace,
    /// Delete
    Delete,
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Modifier keys (tracked as flags, not individual events)
    Ctrl,
    Shift,
    Alt,
    Meta, // Windows key / Command key
    /// Unknown key
    Unknown,
}

/// Key event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventType {
    /// Key was pressed
    Press,
    /// Key was released
    Release,
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    /// The key
    pub key: Key,
    /// Event type (press/release)
    pub event_type: KeyEventType,
    /// Whether Ctrl is held
    pub ctrl: bool,
    /// Whether Shift is held
    pub shift: bool,
    /// Whether Alt is held
    pub alt: bool,
    /// Whether Meta (Win/Cmd) is held
    pub meta: bool,
}

impl KeyEvent {
    /// Check if this is a Ctrl+K combination
    pub fn is_ctrl_k(&self) -> bool {
        self.ctrl
            && !self.shift
            && !self.alt
            && !self.meta
            && matches!(self.key, Key::Char('k') | Key::Char('K'))
    }

    /// Check if this is a Cmd+K (Mac) or Ctrl+K (Windows/Linux) combination
    pub fn is_toggle_global_search(&self) -> bool {
        (self.ctrl || self.meta)
            && !self.shift
            && !self.alt
            && matches!(self.key, Key::Char('k') | Key::Char('K'))
    }
}

/// Input manager that coordinates input devices
pub struct InputManager {
    keyboard: KeyboardMonitor,
    handlers: Vec<Box<dyn InputHandler>>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardMonitor::new(),
            handlers: Vec::new(),
        }
    }

    /// Register an input handler
    pub fn register_handler(&mut self, handler: Box<dyn InputHandler>) {
        self.handlers.push(handler);
    }

    /// Start monitoring input in a background thread
    pub fn start_monitoring(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                // Check keyboard state
                if let Some(event) = self.keyboard.poll_event() {
                    // Dispatch to handlers
                    for handler in &self.handlers {
                        handler.handle_key(event.clone());
                    }
                }

                // Small delay to prevent CPU spinning
                thread::sleep(Duration::from_millis(10));
            }
        })
    }
}

/// Command dispatcher that bridges input events to the command system
pub struct CommandDispatcher {
    command_bus: Arc<RwLock<crate::core::command_bus::CommandBus>>,
}

impl CommandDispatcher {
    /// Create a new command dispatcher
    pub fn new(command_bus: Arc<RwLock<crate::core::command_bus::CommandBus>>) -> Self {
        Self { command_bus }
    }
}

impl InputHandler for CommandDispatcher {
    fn handle_key(&self, event: KeyEvent) {
        if event.event_type == KeyEventType::Press {
            // Check for Global Search shortcut (Ctrl+K / Cmd+K)
            if event.is_toggle_global_search() {
                println!("Input: Detected Ctrl+K / Cmd+K");
                let bus = self.command_bus.read();
                let cmd = crate::core::commands::global_search::ToggleGlobalSearchCommand::new();
                bus.publish(&cmd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_is_ctrl_k() {
        let event = KeyEvent {
            key: Key::Char('k'),
            event_type: KeyEventType::Press,
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        };
        assert!(event.is_ctrl_k());
        assert!(event.is_toggle_global_search());
    }

    #[test]
    fn test_key_event_is_cmd_k() {
        let event = KeyEvent {
            key: Key::Char('k'),
            event_type: KeyEventType::Press,
            ctrl: false,
            shift: false,
            alt: false,
            meta: true, // Mac Command key
        };
        assert!(!event.is_ctrl_k()); // Not Ctrl+K
        assert!(event.is_toggle_global_search()); // But is toggle shortcut
    }

    #[test]
    fn test_key_event_not_shortcut() {
        let event = KeyEvent {
            key: Key::Char('a'),
            event_type: KeyEventType::Press,
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        };
        assert!(!event.is_ctrl_k());
        assert!(!event.is_toggle_global_search());
    }
}
