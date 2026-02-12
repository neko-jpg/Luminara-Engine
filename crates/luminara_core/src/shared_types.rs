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
