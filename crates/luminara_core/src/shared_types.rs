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
pub struct App {
    pub runner: Option<Box<dyn FnOnce(App)>>,
}

impl Default for App {
    fn default() -> Self {
        Self { runner: None }
    }
}

impl App {
    pub fn set_runner(&mut self, runner: impl FnOnce(App) + 'static) {
        self.runner = Some(Box::new(runner));
    }

    pub fn update(&mut self) {
        // Run systems for each stage (stub)
    }

    // Minimal way to get a resource mutably for the runner
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        // This is a skeleton, so we can't really store/retrieve resources yet.
        // AI-1 will implement the real storage.
        None
    }
}

pub trait IntoSystem {}

// Add ResMut for system params
pub struct ResMut<T: ?Sized>(pub std::marker::PhantomData<T>);

pub struct Events<T> {
    events: Vec<T>,
}

impl<T: Send + Sync + 'static> Resource for Events<T> {}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn update(&mut self) {
        self.events.clear();
    }
}

impl<T: ?Sized> std::ops::Deref for ResMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unimplemented!("This is a skeleton")
    }
}
impl<T: ?Sized> std::ops::DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unimplemented!("This is a skeleton")
    }
}

// Implement IntoSystem for functions to allow compilation
impl<F> IntoSystem for F {}

impl AppInterface for App {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }
    fn add_system(&mut self, _stage: CoreStage, _system: impl IntoSystem) -> &mut Self {
        self
    }
    fn insert_resource<R: Resource>(&mut self, _resource: R) -> &mut Self {
        self
    }
    fn run(mut self) {
        if let Some(runner) = self.runner.take() {
            (runner)(self);
        }
    }
}
