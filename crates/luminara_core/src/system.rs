use crate::world::World;
use crate::resource::{Resource, Res, ResMut};
use crate::query::{WorldQuery, Query, QueryFilter};
use crate::event::{Event, EventWriter, EventReader};
use std::marker::PhantomData;
use std::any::TypeId;
use std::collections::HashSet;

#[derive(Default, Clone, Debug)]
pub struct SystemAccess {
    pub resources_read: HashSet<TypeId>,
    pub resources_write: HashSet<TypeId>,
    pub components_read: HashSet<TypeId>,
    pub components_write: HashSet<TypeId>,
}

pub trait System: Send + Sync {
    fn run(&mut self, world: &World);
    fn name(&self) -> &str {
        "AnonymousSystem"
    }
    fn access(&self) -> SystemAccess {
        SystemAccess::default()
    }
}

pub trait SystemParam: Send + Sync {
    type Item<'w>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w>;
    fn add_access(access: &mut SystemAccess);
}

impl<T: Resource> SystemParam for Res<'static, T> {
    type Item<'w> = Res<'w, T>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        Res { value: world.get_resource::<T>().expect("Resource not found") }
    }
    fn add_access(access: &mut SystemAccess) {
        access.resources_read.insert(TypeId::of::<T>());
    }
}

impl<T: Resource> SystemParam for ResMut<'static, T> {
    type Item<'w> = ResMut<'w, T>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        ResMut { value: world.get_resource_mut::<T>().expect("Resource not found") }
    }
    fn add_access(access: &mut SystemAccess) {
        access.resources_write.insert(TypeId::of::<T>());
    }
}

impl<Q: WorldQuery + 'static, F: QueryFilter + 'static> SystemParam for Query<'static, Q, F> {
    type Item<'w> = Query<'w, Q, F>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        Query::new(world)
    }
    fn add_access(access: &mut SystemAccess) {
        Q::add_access(access);
    }
}

impl<E: Event> SystemParam for EventWriter<'static, E> {
    type Item<'w> = EventWriter<'w, E>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        EventWriter::new(world.get_events_mut::<E>())
    }
    fn add_access(access: &mut SystemAccess) {
        access.resources_write.insert(TypeId::of::<crate::event::Events<E>>());
    }
}

impl<E: Event> SystemParam for EventReader<'static, E> {
    type Item<'w> = EventReader<'w, E>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        EventReader::new(world.get_events::<E>().expect("Events not found"))
    }
    fn add_access(access: &mut SystemAccess) {
        access.resources_read.insert(TypeId::of::<crate::event::Events<E>>());
    }
}

pub trait IntoSystem<Params> {
    type System: System + 'static;
    fn into_system(self) -> Self::System;
}

pub struct FunctionSystem<F, Params> {
    f: F,
    pub(crate) system_access: SystemAccess,
    _marker: PhantomData<Params>,
}

impl<F, Params> FunctionSystem<F, Params> {
    pub fn with_access(mut self, access: SystemAccess) -> Self {
        self.system_access = access;
        self
    }
}

pub struct WorldRefParam;

macro_rules! impl_system_func {
    ($($param:ident),*) => {
        #[allow(non_snake_case)]
        impl<Func, $($param),*> System for FunctionSystem<Func, ($($param,)*)>
        where
            Func: for<'a> FnMut($($param::Item<'a>),*) + Send + Sync + 'static,
            $($param: SystemParam + 'static),*
        {
            fn run(&mut self, world: &World) {
                (self.f)($($param::get_param(world)),*);
            }
            fn access(&self) -> SystemAccess {
                self.system_access.clone()
            }
        }

        #[allow(non_snake_case)]
        impl<Func, $($param),*> IntoSystem<($($param,)*)> for Func
        where
            Func: for<'a> FnMut($($param::Item<'a>),*) + Send + Sync + 'static,
            $($param: SystemParam + 'static),*
        {
            type System = FunctionSystem<Func, ($($param,)*)>;
            fn into_system(self) -> Self::System {
                let mut access = SystemAccess::default();
                $($param::add_access(&mut access);)*
                FunctionSystem {
                    f: self,
                    system_access: access,
                    _marker: PhantomData,
                }
            }
        }
    };
}

impl_system_func!(A);
impl_system_func!(A, B);
impl_system_func!(A, B, C);
impl_system_func!(A, B, C, D);
impl_system_func!(A, B, C, D, E);
impl_system_func!(A, B, C, D, E, F);
impl_system_func!(A, B, C, D, E, F, G);
impl_system_func!(A, B, C, D, E, F, G, H);
impl_system_func!(A, B, C, D, E, F, G, H, I);
impl_system_func!(A, B, C, D, E, F, G, H, I, J);
impl_system_func!(A, B, C, D, E, F, G, H, I, J, K);
impl_system_func!(A, B, C, D, E, F, G, H, I, J, K, L);

impl<F> IntoSystem<WorldRefParam> for F
where
    F: FnMut(&World) + Send + Sync + 'static,
{
    type System = FunctionSystem<F, WorldRefParam>;
    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            system_access: SystemAccess::default(),
            _marker: PhantomData,
        }
    }
}

impl<F> System for FunctionSystem<F, WorldRefParam>
where
    F: FnMut(&World) + Send + Sync + 'static,
{
    fn run(&mut self, world: &World) {
        (self.f)(world);
    }
    fn access(&self) -> SystemAccess {
        self.system_access.clone()
    }
}

impl<F, P> IntoSystem<P> for FunctionSystem<F, P>
where
    Self: System + 'static,
{
    type System = Self;
    fn into_system(self) -> Self::System {
        self
    }
}
