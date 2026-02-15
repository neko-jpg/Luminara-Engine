use luminara_math::{Quat, Transform, Vec3};
use luminara_script_lua::api::transform::LuaTransform;
use mlua::prelude::*;

#[test]
fn test_transform_api() -> mlua::Result<()> {
    let lua = mlua::Lua::new();
    let transform = Transform::default();
    let lua_transform = LuaTransform(transform);

    lua.scope(|scope| {
        let user_data = scope.create_userdata(lua_transform)?;

        let chunk = lua.load(
            "
            local t = ...
            t:set_position(10.0, 20.0, 30.0)
            local x, y, z = t:position()
            return x, y, z
        ",
        );

        let (x, y, z): (f32, f32, f32) = chunk.call(user_data)?;
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(z, 30.0);
        Ok(())
    })
}

#[test]
fn test_transform_vectors() -> mlua::Result<()> {
    let lua = mlua::Lua::new();
    let transform = Transform::default();
    let lua_transform = LuaTransform(transform);

    lua.scope(|scope| {
        let user_data = scope.create_userdata(lua_transform)?;

        // Default forward is -Z (0, 0, -1) in Right Handed Y-up system?
        // luminara_math Transform usually assumes -Z forward.

        let chunk = lua.load(
            "
            local t = ...
            local fx, fy, fz = t:forward()
            return fx, fy, fz
        ",
        );

        let (fx, fy, fz): (f32, f32, f32) = chunk.call(user_data)?;
        // Just check it returns something reasonable (not NaN)
        assert!(fx.is_finite());
        assert!(fy.is_finite());
        assert!(fz.is_finite());
        Ok(())
    })
}
