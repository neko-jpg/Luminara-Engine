use luminara_core::time::Time;
use mlua::prelude::*;

#[derive(Clone, Copy)]
pub struct LuaTime<'a>(pub &'a Time);

impl<'a> LuaUserData for LuaTime<'a> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("delta_seconds", |_, this, ()| Ok(this.0.delta_seconds()));

        methods.add_method("elapsed_seconds", |_, this, ()| {
            Ok(this.0.elapsed_seconds())
        });

        methods.add_method("frame_count", |_, this, ()| Ok(this.0.frame_count()));
    }
}
