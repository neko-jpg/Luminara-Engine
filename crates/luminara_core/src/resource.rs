use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;

pub trait Resource: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Resource for T {}

pub struct ResourceMap {
    pub(crate) resources: HashMap<TypeId, UnsafeCell<Box<dyn Any + Send + Sync>>>,
}

unsafe impl Send for ResourceMap {}
unsafe impl Sync for ResourceMap {}

impl ResourceMap {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), UnsafeCell::new(Box::new(resource)));
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        unsafe {
            self.resources.get(&TypeId::of::<R>())
                .map(|cell| (*cell.get()).downcast_ref::<R>().unwrap())
        }
    }

    pub fn get_mut<R: Resource>(&self) -> Option<&mut R> {
        unsafe {
            self.resources.get(&TypeId::of::<R>())
                .map(|cell| (*cell.get()).downcast_mut::<R>().unwrap())
        }
    }
}

// Res and ResMut will be used in Systems, but for now they can be simple wrappers
// In a real ECS like Bevy, they are handled by the scheduler providing guards.
// For now, I'll just implement the wrappers that take a reference.
pub struct Res<'a, T: Resource> {
    pub(crate) value: &'a T,
}

impl<'a, T: Resource> Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct ResMut<'a, T: Resource> {
    pub(crate) value: &'a mut T,
}

impl<'a, T: Resource> Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Resource> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
