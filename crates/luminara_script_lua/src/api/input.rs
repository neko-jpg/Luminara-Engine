use luminara_input::{keyboard::Key, mouse::MouseButton, Input};
use mlua::prelude::*;

#[derive(Clone, Copy)]
pub struct LuaInput(pub *const Input);

impl LuaUserData for LuaInput {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Key checks
        methods.add_method("is_key_pressed", |_, this, key_name: String| {
            let input = unsafe { &*this.0 };
            if let Some(key) = parse_key(&key_name) {
                Ok(input.pressed(key))
            } else {
                Ok(false)
            }
        });

        methods.add_method("is_key_just_pressed", |_, this, key_name: String| {
            let input = unsafe { &*this.0 };
            if let Some(key) = parse_key(&key_name) {
                Ok(input.just_pressed(key))
            } else {
                Ok(false)
            }
        });

        // Mouse checks
        methods.add_method("is_mouse_button_pressed", |_, this, btn_name: String| {
            let input = unsafe { &*this.0 };
            if let Some(btn) = parse_mouse_button(&btn_name) {
                Ok(input.mouse_pressed(btn))
            } else {
                Ok(false)
            }
        });

        methods.add_method("mouse_position", |_, this, ()| {
            let input = unsafe { &*this.0 };
            let pos = input.mouse_position();
            Ok((pos.x, pos.y))
        });

        methods.add_method("mouse_delta", |_, this, ()| {
            let input = unsafe { &*this.0 };
            let delta = input.mouse_delta();
            Ok((delta.x, delta.y))
        });

        // Axis
        methods.add_method("get_axis", |_, this, axis_name: String| {
            let input = unsafe { &*this.0 };
            Ok(input.axis(&axis_name))
        });
    }
}

// Helper to parse key names from string
fn parse_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "w" => Some(Key::W),
        "a" => Some(Key::A),
        "s" => Some(Key::S),
        "d" => Some(Key::D),
        "space" => Some(Key::Space),
        "shift" => Some(Key::LShift),
        "enter" => Some(Key::Enter),
        "escape" => Some(Key::Escape),
        _ => None,
    }
}

fn parse_mouse_button(name: &str) -> Option<MouseButton> {
    match name.to_lowercase().as_str() {
        "left" => Some(MouseButton::Left),
        "right" => Some(MouseButton::Right),
        "middle" => Some(MouseButton::Middle),
        _ => None,
    }
}
