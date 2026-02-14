use mlua::prelude::*;
use luminara_core::world::World;
use luminara_core::entity::Entity;
use luminara_math::Transform;
use crate::api::transform::LuaTransform;

fn pack_entity(entity: Entity) -> u64 {
    ((entity.generation() as u64) << 32) | (entity.id() as u64)
}

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


#[derive(Clone, Copy)]
pub struct LuaWorld(pub *mut World);

impl LuaUserData for LuaWorld {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("spawn", |_, this, ()| {
            let world = unsafe { &mut *this.0 };
            let entity = world.spawn();
            // Add default Transform to spawned entities so they can be moved
            world.add_component(entity, Transform::default());
            Ok(pack_entity(entity))
        });

        methods.add_method("despawn", |_, this, packed_entity: u64| {
            let world = unsafe { &mut *this.0 };
            let entity = unpack_entity(packed_entity);
            world.despawn(entity);
            Ok(())
        });

        methods.add_method("get_entity", |_, this, packed_entity: u64| {
            let _world = unsafe { &mut *this.0 };
            Ok(packed_entity) // Just verify it exists? Or return wrapper? For now just pass through.
        });

        methods.add_method("get_transform", |_, this, packed_entity: u64| {
            let world = unsafe { &mut *this.0 };
            let entity = unpack_entity(packed_entity);
            if let Some(t) = world.get_component::<Transform>(entity) {
                Ok(Some(LuaTransform(*t)))
            } else {
                Ok(None)
            }
        });

        methods.add_method("set_transform", |_, this, (packed_entity, transform): (u64, LuaTransform)| {
            let world = unsafe { &mut *this.0 };
            let entity = unpack_entity(packed_entity);
            world.add_component(entity, transform.0);
            Ok(())
        });
    }
}
