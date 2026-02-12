pub type Entity = u64;

pub trait Component: Send + Sync + 'static {
    fn type_name() -> &'static str where Self: Sized;
}

pub trait Resource: Send + Sync + 'static {}

pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn build(&self, app: &mut App);
}

pub trait AppInterface {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self;
    fn add_system(&mut self, stage: CoreStage, system: impl IntoSystem) -> &mut Self;
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

// Placeholder for App and IntoSystem to make it compile
pub struct App;
pub trait IntoSystem {}

// Add Res and ResMut for system params
pub struct Res<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized> std::ops::Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct ResMut<'a, T: ?Sized>(pub &'a mut T);

impl<'a, T: ?Sized> std::ops::Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: ?Sized> std::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

// Implement IntoSystem for functions to allow compilation
impl<F> IntoSystem for F {}

impl AppInterface for App {
    fn add_plugins(&mut self, _plugin: impl Plugin) -> &mut Self {
        self
    }
    fn add_system(&mut self, _stage: CoreStage, _system: impl IntoSystem) -> &mut Self {
        self
    }
    fn insert_resource<R: Resource>(&mut self, _resource: R) -> &mut Self {
        self
    }
    fn run(self) {}
}
