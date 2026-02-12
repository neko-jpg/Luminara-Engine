use std::any::{Any, TypeId};
use std::collections::HashMap;

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

pub struct App {
    pub resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl Resource for App {}

pub trait IntoSystem {}

// Simplified Res and ResMut that don't panic if you don't use them (much)
pub struct Res<T: Resource> {
    _marker: std::marker::PhantomData<T>,
}
pub struct ResMut<T: Resource> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Resource> std::ops::Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        panic!("Res::deref is not fully implemented in this skeleton")
    }
}

impl<T: Resource> std::ops::Deref for ResMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        panic!("ResMut::deref is not fully implemented in this skeleton")
    }
}
impl<T: Resource> std::ops::DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        panic!("ResMut::deref_mut is not fully implemented in this skeleton")
    }
}

// Minimal Query stub
pub struct Query<'a, T>(pub std::marker::PhantomData<&'a T>);
impl<'a, T> Query<'a, T> {
    pub fn iter(&self) -> QueryIter<T> {
        QueryIter(std::marker::PhantomData)
    }
}
pub struct QueryIter<T>(pub std::marker::PhantomData<T>);
impl<T> Iterator for QueryIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
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
    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
        self
    }
    fn run(self) {}
}
