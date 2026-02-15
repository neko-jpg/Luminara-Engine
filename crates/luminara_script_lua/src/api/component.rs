use crate::api::world::LuaWorld;
use luminara_core::world::World;
use mlua::prelude::*;

// "Component: get/set with type safety"
// This is hard in Lua since it's dynamic.
// We typically use string names for components or specialized methods.
// e.g. entity:get_component("Transform") -> LuaTransform

// We can extend LuaWorld or LuaEntity (if we had one) to support this.
// For now, let's assume `LuaWorld` handles it or we have a `LuaComponent` helper.

// Let's implement `LuaComponent` which acts as a bridge.

pub struct LuaComponent;

impl LuaComponent {
    // Helper to register component accessors?
    // In Rust ECS, components are generic. Lua needs dynamic dispatch.
    // We likely need a registry that maps component names to functions that know how to get/set that component on an entity.

    // For MVP, we manually implement accessors for known components like Transform.
    // Dynamic access requires reflection which might be available via `TypeRegistry` in `luminara_scene`.
}

// Let's add component methods to LuaWorld for now.
// Actually, `api/world.rs` defined `LuaWorld`.
// We can use extension traits or just add them there.
// But to keep it modular, let's keep it here conceptually.

// Since `LuaWorld` is defined in another module, we can't `impl LuaUserData` again.
// We should probably modify `LuaWorld` in `api/world.rs` to include component methods,
// OR expose a separate `Component` API object: `Component.get(world, entity, "Transform")`.

pub struct LuaComponentAPI;

impl LuaUserData for LuaComponentAPI {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get", |lua, _this, (world_ud, entity_id, comp_name): (LuaUserDataRef<LuaWorld>, u64, String)| {
             // Access world from UserData
             // This requires `LuaWorld` to be accessible here.
             // And we need to match `comp_name`.

             let world = unsafe { &mut *world_ud.0 };

             match comp_name.as_str() {
                 "Transform" => {
                     // Get Transform component
                     // if let Some(t) = world.get::<Transform>(entity) ...
                     // Return LuaTransform wrapper
                     Ok(mlua::Value::Nil) // Placeholder
                 },
                 _ => Ok(mlua::Value::Nil)
             }
         });
    }
}
