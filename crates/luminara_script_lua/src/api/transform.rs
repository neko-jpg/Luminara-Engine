use mlua::prelude::*;
use luminara_math::Transform;
// luminara_core may not have prelude exposed like that, just import what we need or check structure.
// But we actually only need Transform from math here.

#[derive(Clone, Copy)]
pub struct LuaTransform(pub Transform);

impl LuaUserData for LuaTransform {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("position", |_, this, ()| {
            let p = this.0.translation;
            Ok((p.x, p.y, p.z))
        });

        methods.add_method_mut("set_position", |_, this, (x, y, z): (f32, f32, f32)| {
            this.0.translation = luminara_math::Vec3::new(x, y, z);
            Ok(())
        });

        methods.add_method("rotation", |_, this, ()| {
            let r = this.0.rotation;
            Ok((r.x, r.y, r.z, r.w))
        });

        methods.add_method_mut("set_rotation", |_, this, (x, y, z, w): (f32, f32, f32, f32)| {
            this.0.rotation = luminara_math::Quat::from_xyzw(x, y, z, w);
            Ok(())
        });

        methods.add_method("scale", |_, this, ()| {
            let s = this.0.scale;
            Ok((s.x, s.y, s.z))
        });

        methods.add_method_mut("set_scale", |_, this, (x, y, z): (f32, f32, f32)| {
            this.0.scale = luminara_math::Vec3::new(x, y, z);
            Ok(())
        });

        methods.add_method("forward", |_, this, ()| {
            let f = this.0.forward();
            Ok((f.x, f.y, f.z))
        });

        methods.add_method("right", |_, this, ()| {
            let r = this.0.right();
            Ok((r.x, r.y, r.z))
        });

        methods.add_method("up", |_, this, ()| {
            let u = this.0.up();
            Ok((u.x, u.y, u.z))
        });
    }
}
