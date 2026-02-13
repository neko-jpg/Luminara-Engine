pub use crate::app::App;
pub use crate::component::Component;
pub use crate::entity::Entity;
pub use crate::event::Events;
pub use crate::plugin::Plugin;
pub use crate::query::Query;
pub use crate::resource::{Res, ResMut, Resource};
pub use crate::system::IntoSystem;
pub use crate::world::World;

pub trait AppInterface {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self;
    fn add_system<Marker>(
        &mut self,
        stage: CoreStage,
        system: impl IntoSystem<Marker>,
    ) -> &mut Self;
    fn add_startup_system<Marker>(&mut self, system: impl IntoSystem<Marker>) -> &mut Self;
    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self;
    fn register_component<C: Component>(&mut self) -> &mut Self;
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
