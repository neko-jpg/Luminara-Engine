use luminara_math::Transform;
use luminara_script_lua::api::transform::LuaTransform;
use mlua::prelude::*;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_component_api_type_safety(x: f32, y: f32, z: f32) -> TestResult {
    // NaN or Inf can cause issues in math comparison or logic, and we don't necessarily support them
    // as "valid" inputs for game logic usually, but here the test fails because `math.abs(-inf - -inf)` etc.
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return TestResult::discard();
    }

    let lua = mlua::Lua::new();
    let transform = Transform::default();
    let lua_transform = LuaTransform(transform);

    let result: mlua::Result<bool> = lua.scope(|scope| {
        let user_data = scope.create_userdata(lua_transform)?;

        let chunk = lua.load(
            "
            local t, x, y, z = ...
            t:set_position(x, y, z)
            local p_x, p_y, p_z = t:position()

            -- Test type safety: calling with string should error
            local status, err = pcall(function() t:set_position(\"invalid\", 0, 0) end)

            -- Floating point comparison in Lua
            local close = math.abs(p_x - x) < 0.001

            return close and (status == false)
        ",
        );

        chunk.call::<_, bool>((user_data, x, y, z))
    });

    match result {
        Ok(success) => TestResult::from_bool(success),
        Err(_) => TestResult::failed(),
    }
}
