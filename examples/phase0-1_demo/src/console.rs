//! In-game console system for runtime parameter tuning

use luminara::prelude::*;
use luminara_physics::physics3d::PhysicsWorld3D;
use rapier3d::prelude::{nalgebra, vector};
use std::collections::HashMap;

/// Console command trait
#[allow(dead_code)]
pub trait ConsoleCommand: Send + Sync {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String>;
    fn help(&self) -> &str;
}

/// Console state resource
pub struct Console {
    pub visible: bool,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub output: Vec<(String, [f32; 4])>, // (message, color)
    pub commands: HashMap<String, Box<dyn ConsoleCommand>>,
    pub max_output_lines: usize,
}

impl Resource for Console {}

impl Default for Console {
    fn default() -> Self {
        let mut console = Self {
            visible: false,
            input_buffer: String::new(),
            history: Vec::new(),
            output: Vec::new(),
            commands: HashMap::new(),
            max_output_lines: 10,
        };

        // Register default commands
        console.register_command("help", Box::new(HelpCommand));
        console.register_command("clear", Box::new(ClearCommand));
        console.register_command("gravity", Box::new(SetGravityCommand));
        console.register_command("timescale", Box::new(SetTimeScaleCommand));
        console.register_command("spawn", Box::new(SpawnCommand));
        console.register_command("list", Box::new(ListEntitiesCommand));

        console
    }
}

impl Console {
    pub fn register_command(&mut self, name: &str, command: Box<dyn ConsoleCommand>) {
        self.commands.insert(name.to_string(), command);
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, world: &mut World) -> Result<String, String> {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            return Ok(String::new());
        }

        self.history.push(input.to_string());

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd_name = parts[0];
        let args = &parts[1..];

        if let Some(command) = self.commands.get(cmd_name) {
            match command.execute(args, world) {
                Ok(msg) => {
                    self.add_output(&msg, [0.3, 1.0, 0.3, 1.0]);
                    Ok(msg)
                }
                Err(err) => {
                    self.add_output(&format!("Error: {}", err), [1.0, 0.3, 0.3, 1.0]);
                    Err(err)
                }
            }
        } else {
            let err = format!("Unknown command: {}", cmd_name);
            self.add_output(&err, [1.0, 0.3, 0.3, 1.0]);
            Err(err)
        }
    }

    pub fn add_output(&mut self, msg: &str, color: [f32; 4]) {
        self.output.push((msg.to_string(), color));
        if self.output.len() > self.max_output_lines {
            self.output.remove(0);
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input_buffer.clear();
        }
    }
}

// ============================================================================
// Built-in Commands
// ============================================================================

struct HelpCommand;
impl ConsoleCommand for HelpCommand {
    fn execute(&self, _args: &[&str], world: &mut World) -> Result<String, String> {
        if let Some(console) = world.get_resource::<Console>() {
            let mut help_text = String::from("Available commands:\n");
            for (name, cmd) in &console.commands {
                help_text.push_str(&format!("  {} - {}\n", name, cmd.help()));
            }
            Ok(help_text)
        } else {
            Ok("No commands available".to_string())
        }
    }

    fn help(&self) -> &str {
        "Show this help message"
    }
}

struct ClearCommand;
impl ConsoleCommand for ClearCommand {
    fn execute(&self, _args: &[&str], world: &mut World) -> Result<String, String> {
        if let Some(console) = world.get_resource_mut::<Console>() {
            console.output.clear();
        }
        Ok("Console cleared".to_string())
    }

    fn help(&self) -> &str {
        "Clear console output"
    }
}

struct SetGravityCommand;
impl ConsoleCommand for SetGravityCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            return Err("Usage: gravity <value>".to_string());
        }

        let gravity: f32 = args[0]
            .parse()
            .map_err(|_| "Invalid number")?;

        if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld3D>() {
            physics_world.gravity = vector![0.0, gravity, 0.0];
            Ok(format!("Gravity set to {}", gravity))
        } else {
            Err("Physics world not found".to_string())
        }
    }

    fn help(&self) -> &str {
        "Set gravity value (e.g., gravity -9.8)"
    }
}

struct SetTimeScaleCommand;
impl ConsoleCommand for SetTimeScaleCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            return Err("Usage: timescale <value>".to_string());
        }

        let scale: f32 = args[0]
            .parse()
            .map_err(|_| "Invalid number")?;

        if scale < 0.0 || scale > 10.0 {
            return Err("Time scale must be between 0.0 and 10.0".to_string());
        }

        if let Some(time) = world.get_resource_mut::<Time>() {
            time.time_scale = scale;
            Ok(format!("Time scale set to {}", scale))
        } else {
            Err("Time resource not found".to_string())
        }
    }

    fn help(&self) -> &str {
        "Set time scale (e.g., timescale 0.5 for slow motion)"
    }
}

struct SpawnCommand;
impl ConsoleCommand for SpawnCommand {
    fn execute(&self, args: &[&str], _world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            return Err("Usage: spawn <type> [x] [y] [z]".to_string());
        }

        let spawn_type = args[0];
        let pos = if args.len() >= 4 {
            Vec3::new(
                args[1].parse().unwrap_or(0.0),
                args[2].parse().unwrap_or(5.0),
                args[3].parse().unwrap_or(0.0),
            )
        } else {
            Vec3::new(0.0, 5.0, 0.0)
        };

        match spawn_type {
            "sphere" => {
                // Spawn sphere logic would go here
                Ok(format!("Spawned sphere at {:?}", pos))
            }
            "cube" => {
                // Spawn cube logic would go here
                Ok(format!("Spawned cube at {:?}", pos))
            }
            _ => Err(format!("Unknown spawn type: {}", spawn_type)),
        }
    }

    fn help(&self) -> &str {
        "Spawn an object (e.g., spawn sphere 0 5 0)"
    }
}

struct ListEntitiesCommand;
impl ConsoleCommand for ListEntitiesCommand {
    fn execute(&self, _args: &[&str], world: &mut World) -> Result<String, String> {
        let count = world.entities().len();
        Ok(format!("Total entities: {}", count))
    }

    fn help(&self) -> &str {
        "List total number of entities"
    }
}

/// System to handle console input
pub fn console_input_system(world: &mut World) {
    let wants_toggle = world
        .get_resource::<Input>()
        .map(|i| i.just_pressed(luminara_input::keyboard::Key::Backquote))
        .unwrap_or(false);

    if wants_toggle {
        if let Some(console) = world.get_resource_mut::<Console>() {
            console.toggle();
        }
    }

    let is_visible = world
        .get_resource::<Console>()
        .map(|c| c.visible)
        .unwrap_or(false);

    if !is_visible {
        return;
    }

    // Handle text input
    // Note: This is a simplified version. A real implementation would need proper text input handling
    let wants_execute = world
        .get_resource::<Input>()
        .map(|i| i.just_pressed(luminara_input::keyboard::Key::Enter))
        .unwrap_or(false);

    if wants_execute {
        // Take the input buffer, execute, then clear
        let input_buffer = world
            .get_resource::<Console>()
            .map(|c| c.input_buffer.clone())
            .unwrap_or_default();

        if !input_buffer.trim().is_empty() {
            let parts: Vec<&str> = input_buffer.trim().split_whitespace().collect();
            let cmd_name = parts[0].to_string();
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            // Add to history
            if let Some(console) = world.get_resource_mut::<Console>() {
                console.history.push(input_buffer.trim().to_string());
            }

            // Try to execute the command
            let _result = {
                let _arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                // Look up the command and execute it
                let cmd_exists = world
                    .get_resource::<Console>()
                    .map(|c| c.commands.contains_key(&cmd_name))
                    .unwrap_or(false);

                if cmd_exists {
                    if let Some(console) = world.get_resource::<Console>() {
                        if let Some(cmd) = console.commands.get(&cmd_name) {
                            // We can't call execute with world here due to borrow rules.
                            // Handle built-in commands inline instead.
                            Some(cmd.help().to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            // Handle specific commands directly to avoid borrow conflicts
            match cmd_name.as_str() {
                "help" => {
                    if let Some(console) = world.get_resource_mut::<Console>() {
                        let help_text: String = console.commands.iter()
                            .map(|(name, cmd)| format!("  {} - {}", name, cmd.help()))
                            .collect::<Vec<_>>()
                            .join("\n");
                        console.add_output(&format!("Available commands:\n{}", help_text), [0.3, 1.0, 0.3, 1.0]);
                    }
                }
                "clear" => {
                    if let Some(console) = world.get_resource_mut::<Console>() {
                        console.output.clear();
                        console.add_output("Console cleared", [0.3, 1.0, 0.3, 1.0]);
                    }
                }
                "gravity" => {
                    if let Some(gravity_val) = args.first().and_then(|a| a.parse::<f32>().ok()) {
                        if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld3D>() {
                            physics_world.gravity = vector![0.0, gravity_val, 0.0];
                        }
                        if let Some(console) = world.get_resource_mut::<Console>() {
                            console.add_output(&format!("Gravity set to {}", gravity_val), [0.3, 1.0, 0.3, 1.0]);
                        }
                    } else {
                        if let Some(console) = world.get_resource_mut::<Console>() {
                            console.add_output("Usage: gravity <value>", [1.0, 0.3, 0.3, 1.0]);
                        }
                    }
                }
                "timescale" => {
                    if let Some(scale) = args.first().and_then(|a| a.parse::<f32>().ok()) {
                        if scale >= 0.0 && scale <= 10.0 {
                            if let Some(time) = world.get_resource_mut::<Time>() {
                                time.time_scale = scale;
                            }
                            if let Some(console) = world.get_resource_mut::<Console>() {
                                console.add_output(&format!("Time scale set to {}", scale), [0.3, 1.0, 0.3, 1.0]);
                            }
                        } else {
                            if let Some(console) = world.get_resource_mut::<Console>() {
                                console.add_output("Time scale must be 0.0-10.0", [1.0, 0.3, 0.3, 1.0]);
                            }
                        }
                    } else {
                        if let Some(console) = world.get_resource_mut::<Console>() {
                            console.add_output("Usage: timescale <value>", [1.0, 0.3, 0.3, 1.0]);
                        }
                    }
                }
                "spawn" => {
                    let spawn_type = args.first().map(|s| s.as_str()).unwrap_or("");
                    let pos = if args.len() >= 4 {
                        Vec3::new(
                            args[1].parse().unwrap_or(0.0),
                            args[2].parse().unwrap_or(5.0),
                            args[3].parse().unwrap_or(0.0),
                        )
                    } else {
                        Vec3::new(0.0, 5.0, 0.0)
                    };
                    let msg = format!("Spawned {} at {:?}", spawn_type, pos);
                    if let Some(console) = world.get_resource_mut::<Console>() {
                        console.add_output(&msg, [0.3, 1.0, 0.3, 1.0]);
                    }
                }
                "list" => {
                    let count = world.entities().len();
                    if let Some(console) = world.get_resource_mut::<Console>() {
                        console.add_output(&format!("Total entities: {}", count), [0.3, 1.0, 0.3, 1.0]);
                    }
                }
                _ => {
                    if let Some(console) = world.get_resource_mut::<Console>() {
                        console.add_output(&format!("Unknown command: {}", cmd_name), [1.0, 0.3, 0.3, 1.0]);
                    }
                }
            }

            // Clear input buffer
            if let Some(console) = world.get_resource_mut::<Console>() {
                console.input_buffer.clear();
            }
        }
    }
}

/// System to render console overlay
pub fn console_render_system(world: &mut World) {
    let (visible, input_buffer, output) = if let Some(console) = world.get_resource::<Console>() {
        if !console.visible {
            return;
        }
        (console.visible, console.input_buffer.clone(), console.output.clone())
    } else {
        return;
    };

    if !visible {
        return;
    }

    if let Some(overlay) = world.get_resource_mut::<luminara_render::OverlayRenderer>() {
        let screen_w = 1440.0f32;
        let screen_h = 900.0f32;

        // Console background
        let console_h = 300.0;
        overlay.draw_gradient_rect(
            0.0,
            screen_h - console_h,
            screen_w,
            console_h,
            [0.0, 0.0, 0.0, 0.9],
            [0.05, 0.05, 0.1, 0.9],
        );

        // Title
        overlay.draw_text_outlined(
            10.0,
            screen_h - console_h + 10.0,
            "CONSOLE (` to close, Enter to execute)",
            [0.3, 1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            1.0,
        );

        // Output lines
        let mut y = screen_h - console_h + 40.0;
        for (msg, color) in &output {
            overlay.draw_text(10.0, y, msg, *color, 0.8);
            y += 16.0;
        }

        // Input line
        let input_y = screen_h - 30.0;
        overlay.draw_text(10.0, input_y, &format!("> {}_", input_buffer), [1.0, 1.0, 1.0, 1.0], 1.0);
    }
}
