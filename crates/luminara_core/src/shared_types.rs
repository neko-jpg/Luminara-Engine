use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

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

pub struct App;
pub trait IntoSystem {}

// Add ResMut for system params
pub struct Res<T: ?Sized>(pub std::marker::PhantomData<T>);

impl<T: ?Sized> std::ops::Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unimplemented!("This is a skeleton")
    }
}

pub struct ResMut<T: ?Sized>(pub std::marker::PhantomData<T>);
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
    fn deref(&self) -> &Self::Target { unimplemented!() }
}
impl<T: ?Sized> std::ops::DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { unimplemented!() }
}

impl<F> IntoSystem for F {}

impl AppInterface for App {
    fn add_plugins(&mut self, _plugin: impl Plugin) -> &mut Self { self }
    fn add_system(&mut self, _stage: CoreStage, _system: impl IntoSystem) -> &mut Self { self }
    fn insert_resource<R: Resource>(&mut self, _resource: R) -> &mut Self { self }
    fn run(self) {}
}

pub struct World {
    entities: HashSet<Entity>,
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any + Send + Sync>>>,
    next_entity: Entity,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: HashSet::new(),
            components: HashMap::new(),
            next_entity: 0,
        }
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.insert(entity);
        entity
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.components
            .entry(TypeId::of::<C>())
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.components
            .get(&TypeId::of::<C>())?
            .get(&entity)?
            .downcast_ref()
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.components
            .get_mut(&TypeId::of::<C>())?
            .get_mut(&entity)?
            .downcast_mut()
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        let boxed = self.components.get_mut(&TypeId::of::<C>())?.remove(&entity)?;
        let component = boxed.downcast::<C>().ok()?;
        Some(*component)
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.entities.iter().cloned().collect()
    }
    fn run(mut self) {
        if let Some(runner) = self.runner.take() {
            (runner)(self);
        }
    }
}

pub struct Query<'a, T, F = ()> {
    _marker: std::marker::PhantomData<(&'a T, F)>,
}

pub struct With<T>(std::marker::PhantomData<T>);
pub struct Without<T>(std::marker::PhantomData<T>);
