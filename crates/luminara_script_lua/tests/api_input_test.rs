use luminara_script_lua::api::input::LuaInput;
use luminara_input::{Input, keyboard::Key};
use mlua::prelude::*;

#[test]
fn test_input_api_basic() -> mlua::Result<()> {
    let lua = mlua::Lua::new();
    let mut input = Input::default();

    // Simulate key press by modifying `pressed` set directly since `KeyboardInput` struct fields are pub
    input.keyboard.pressed.insert(Key::Space);

    let static_input: &'static Input = unsafe { std::mem::transmute(&input) };
    let lua_input = LuaInput(static_input);

    lua.scope(|scope| {
        let user_data = scope.create_userdata(lua_input)?;

        let chunk = lua.load("
            local i = ...
            return i:is_key_pressed(\"Space\")
        ");

        let pressed: bool = chunk.call(user_data)?;
        assert!(pressed);
        Ok(())
    })
}
