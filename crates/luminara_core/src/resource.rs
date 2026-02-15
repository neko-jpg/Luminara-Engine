use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub trait Resource: Send + Sync + 'static {}

pub struct ResourceMap {
    pub(crate) resources: HashMap<TypeId, RwLock<Box<dyn Any + Send + Sync>>>,
}

impl Default for ResourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMap {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources
            .insert(TypeId::of::<R>(), RwLock::new(Box::new(resource)));
    }

    pub fn get<R: Resource>(&self) -> Option<MappedRwLockReadGuard<R>> {
        self.resources.get(&TypeId::of::<R>()).map(|lock| {
            RwLockReadGuard::map(lock.read(), |boxed| boxed.downcast_ref::<R>().unwrap())
        })
    }

    pub fn get_mut<R: Resource>(&self) -> Option<MappedRwLockWriteGuard<R>> {
        self.resources.get(&TypeId::of::<R>()).map(|lock| {
            RwLockWriteGuard::map(lock.write(), |boxed| boxed.downcast_mut::<R>().unwrap())
        })
    }

    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.resources
            .remove(&TypeId::of::<R>())
            .map(|lock| *lock.into_inner().downcast::<R>().unwrap())
    }
}

pub struct Res<'a, T: Resource> {
    pub(crate) value: MappedRwLockReadGuard<'a, T>,
}

impl<'a, T: Resource> Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct ResMut<'a, T: Resource> {
    pub(crate) value: MappedRwLockWriteGuard<'a, T>,
}

impl<'a, T: Resource> Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T: Resource> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
