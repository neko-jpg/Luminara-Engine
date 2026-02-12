# luminara_input

The unified input system for the Luminara engine.

## Features

- **Multi-device abstraction**: Keyboard, Mouse, Gamepad, and Touch.
- **Unity-like Axis & Action Mapping**: Define virtual axes and actions that map to multiple physical inputs.
- **Axis Ramping**: Support for sensitivity, gravity, and snap for smooth virtual axis movement.
- **Multi-Gamepad Support**: Query specific gamepads for local multiplayer.
- **Cursor Management**: Control cursor visibility and grab mode.
- **AI-Friendly API**: Designed for easy use by both humans and AI assistants.

## Usage

### Plugin Registration

```rust
use luminara_core::shared_types::App;
use luminara_input::plugin::InputPlugin;

fn main() {
    let mut app = App::default();
    app.add_plugins(InputPlugin);
}
```

### Querying Input

```rust
use luminara_input::Input;
use luminara_input::keyboard::Key;

fn my_system(input: Res<Input>) {
    if input.just_pressed(Key::Space) {
        println!("Jump!");
    }

    let move_h = input.axis("horizontal");
    let move_v = input.axis("vertical");
}
```

### Action Mapping

```rust
use luminara_input::{Input, InputMap};

fn my_system(input: Res<Input>, map: Res<InputMap>) {
    if input.action_just_pressed("fire", &map) {
        println!("Pew pew!");
    }
}
```
