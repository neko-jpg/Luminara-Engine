use crate::{App, AppInterface, Plugin, Resource};
use std::collections::HashMap;
use std::sync::Arc;

pub trait ConsoleCommand: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, args: &[&str], app: &mut App) -> String;
}

#[derive(Default)]
pub struct Console {
    pub history: Vec<String>,
    pub input_buffer: String,
    pub is_open: bool,
    commands: HashMap<String, Arc<Box<dyn ConsoleCommand>>>,
}

impl Resource for Console {}

impl Console {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            input_buffer: String::new(),
            is_open: false,
            commands: HashMap::new(),
        }
    }

    pub fn register<C: ConsoleCommand + 'static>(&mut self, command: C) {
        self.commands
            .insert(command.name().to_string(), Arc::new(Box::new(command)));
    }

    pub fn execute(&mut self, input: &str, app: &mut App) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd_name = parts[0];
        let args = &parts[1..];

        let result = if let Some(cmd) = self.commands.get(cmd_name).cloned() {
            // We need to release the borrow on self to pass app to command
            // But we are in `execute(&mut self, app: &mut App)`.
            // The command trait takes `&mut App`.
            // We can't easily hold `cmd` (which was in `self`) while mutating `app` if `self` is not separate from `app` context in ECS.
            // But here `Console` is a Resource.
            // In a real system, the console execution might be deferred or handled via events.
            // For MVP, we pass `app`.
            // `cmd` is Arc<Box>, so we can clone it.
            cmd.execute(args, app)
        } else {
            format!("Unknown command: {}", cmd_name)
        };

        self.history.push(format!("> {}", input));
        self.history.push(result);
    }
}

// System to toggle console? Or just UI?
// We'll leave the UI rendering to `luminara_render`'s overlay,
// this is just the logical backend.

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn name(&self) -> &str {
        "ConsolePlugin"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Console::new());
        // We don't add a system here because console interaction is usually event driven
        // or called from UI system.
    }
}
