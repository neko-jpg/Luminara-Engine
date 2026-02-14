use mlua::prelude::*;
use luminara_core::world::World;

// "Audio: play sounds, volume/pitch, spatial audio"

pub struct LuaAudio;

impl LuaUserData for LuaAudio {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("version", |_, _, ()| {
            Ok("0.1.0")
        });
    }
}

#[derive(Clone, Copy)]
pub struct LuaAudioWrapper(pub *mut World);

impl LuaUserData for LuaAudioWrapper {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("play_sound", |_, this, path: String| {
            let _world = unsafe { &mut *this.0 };
            // Retrieve Audio resource or system and trigger sound
            // For now, placeholder.
            println!("Lua: playing sound {}", path);
            Ok(())
        });

        methods.add_method("set_volume", |_, this, volume: f32| {
            let _world = unsafe { &mut *this.0 };
            Ok(())
        });
    }
}
