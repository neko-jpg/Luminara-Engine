use crate::system::IntoSystem;
use crate::plugin::Plugin;
use crate::resource::Resource;

pub use crate::entity::Entity;
pub use crate::component::Component;

pub trait AppInterface {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self;
    fn add_system<Params>(&mut self, stage: CoreStage, system: impl IntoSystem<Params>) -> &mut Self;
    fn add_startup_system<Params>(&mut self, system: impl IntoSystem<Params>) -> &mut Self;
    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self;
    fn run(self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreStage {
    Startup,
    PreUpdate,
    Update,
    FixedUpdate,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
}
