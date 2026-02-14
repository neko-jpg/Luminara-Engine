use mlua::prelude::*;
use luminara_input::{Input, keyboard::Key, mouse::MouseButton};

#[derive(Clone, Copy)]
pub struct LuaInput<'a>(pub &'a Input);

impl<'a> LuaUserData for LuaInput<'a> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Key checks
        methods.add_method("is_key_pressed", |_, this, key_name: String| {
            if let Some(key) = parse_key(&key_name) {
                Ok(this.0.pressed(key))
            } else {
                Ok(false)
            }
        });

        methods.add_method("is_key_just_pressed", |_, this, key_name: String| {
            if let Some(key) = parse_key(&key_name) {
                Ok(this.0.just_pressed(key))
            } else {
                Ok(false)
            }
        });

        // Mouse checks
        methods.add_method("is_mouse_button_pressed", |_, this, btn_name: String| {
             if let Some(btn) = parse_mouse_button(&btn_name) {
                Ok(this.0.mouse_pressed(btn))
            } else {
                Ok(false)
            }
        });

        methods.add_method("mouse_position", |_, this, ()| {
            let pos = this.0.mouse_position();
            Ok((pos.x, pos.y))
        });

        methods.add_method("mouse_delta", |_, this, ()| {
            let delta = this.0.mouse_delta();
            Ok((delta.x, delta.y))
        });

        // Axis
        methods.add_method("get_axis", |_, this, axis_name: String| {
            Ok(this.0.axis(&axis_name))
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
