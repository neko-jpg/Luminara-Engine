use luminara_core::entity::Entity;
use luminara_core::world::World;
use mlua::prelude::*;

// Need to duplicate unpack logic or share it.
fn unpack_entity(packed: u64) -> Entity {
    let id = (packed & 0xFFFFFFFF) as u32;
    let generation = (packed >> 32) as u32;
    unsafe { std::mem::transmute(EntityData { id, generation }) }
}

#[repr(C)]
struct EntityData {
    id: u32,
    generation: u32,
}

pub struct LuaPhysics;

impl LuaUserData for LuaPhysics {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("version", |_, _, ()| Ok("0.1.0"));
    }
}

#[derive(Clone, Copy)]
pub struct LuaPhysicsWrapper(pub *mut World);

impl LuaUserData for LuaPhysicsWrapper {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "apply_force",
            |_, this, (entity_packed, x, y, z): (u64, f32, f32, f32)| {
                let _world = unsafe { &mut *this.0 };
                let _entity = unpack_entity(entity_packed);
                // Apply force logic here
                let _ = (x, y, z);
                Ok(())
            },
        );

        methods.add_method(
            "raycast",
            |_, this, (ox, oy, oz, dx, dy, dz, max_dist): (f32, f32, f32, f32, f32, f32, f32)| {
                let _world = unsafe { &mut *this.0 };
                let _ = (ox, oy, oz, dx, dy, dz, max_dist);
                Ok(None::<u64>)
            },
        );
    }
}
