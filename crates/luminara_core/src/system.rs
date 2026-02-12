use crate::event::{Event, EventReader, EventWriter};
use crate::query::{Query, QueryFilter, WorldQuery};
use crate::resource::{Res, ResMut, Resource};
use crate::world::World;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

#[derive(Default, Clone, Debug)]
pub struct SystemAccess {
    pub resources_read: HashSet<TypeId>,
    pub resources_write: HashSet<TypeId>,
    pub components_read: HashSet<TypeId>,
    pub components_write: HashSet<TypeId>,
    pub exclusive: bool,
}

pub trait System: Send + Sync {
    fn run(&mut self, world: &World);
    fn run_exclusive(&mut self, world: &mut World) {
        self.run(world);
    }
    fn name(&self) -> &str {
        "AnonymousSystem"
    }
    fn access(&self) -> SystemAccess {
        SystemAccess::default()
    }
}

pub trait SystemParam: Send + Sync + 'static {
    type Item<'w>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w>;
    fn add_access(access: &mut SystemAccess);
}

impl<T: Resource> SystemParam for Res<'static, T> {
    type Item<'w> = Res<'w, T>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        Res {
            value: world.get_resource::<T>().expect("Resource not found"),
        }
    }
    fn add_access(access: &mut SystemAccess) {
        access.resources_read.insert(TypeId::of::<T>());
    }
}

impl<T: Resource> SystemParam for ResMut<'static, T> {
    type Item<'w> = ResMut<'w, T>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        ResMut {
            value: world.get_resource_mut::<T>().expect("Resource not found"),
        }
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
        access
            .resources_write
            .insert(TypeId::of::<crate::event::Events<E>>());
    }
}

impl SystemParam for World {
    type Item<'w> = &'w World;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        world
    }
    fn add_access(_access: &mut SystemAccess) {}
}

impl<E: Event> SystemParam for EventReader<'static, E> {
    type Item<'w> = EventReader<'w, E>;
    fn get_param<'w>(world: &'w World) -> Self::Item<'w> {
        EventReader::new(world.get_events_mut::<E>())
    }
    fn add_access(access: &mut SystemAccess) {
        access
            .resources_read
            .insert(TypeId::of::<crate::event::Events<E>>());
    }
}

pub trait IntoSystem<Marker> {
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

pub struct FunctionMarker;
pub struct ExclusiveMarker;

macro_rules! impl_system_func {
    ($($param:ident),*) => {
        #[allow(non_snake_case)]
        impl<Func, $($param),*> System for FunctionSystem<Func, ($($param,)*)>
        where
            Func: for<'a> FnMut($($param::Item<'a>),*) + Send + Sync + 'static,
            $($param: SystemParam),*
        {
            fn run(&mut self, world: &World) {
                (self.f)($($param::get_param(world)),*);
            }
            fn access(&self) -> SystemAccess {
                self.system_access.clone()
            }
        }

        #[allow(non_snake_case)]
        impl<Func, $($param),*> IntoSystem<(FunctionMarker, $($param),*)> for Func
        where
            Func: for<'a> FnMut($($param::Item<'a>),*) + Send + Sync + 'static,
            $($param: SystemParam),*
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

impl<F> IntoSystem<ExclusiveMarker> for F
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    type System = ExclusiveFunctionSystem<F>;
    fn into_system(self) -> Self::System {
        ExclusiveFunctionSystem { f: self }
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

impl<F> IntoSystem<ExclusiveMarker> for ExclusiveFunctionSystem<F>
where
    Self: System + 'static,
{
    type System = Self;
    fn into_system(self) -> Self::System {
        self
    }
}

pub struct ExclusiveFunctionSystem<F> {
    f: F,
}

impl<F> System for ExclusiveFunctionSystem<F>
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn run(&mut self, _world: &World) {
        panic!("Exclusive systems must be run with run_exclusive");
    }
    fn run_exclusive(&mut self, world: &mut World) {
        (self.f)(world);
    }
    fn access(&self) -> SystemAccess {
        SystemAccess {
            exclusive: true,
            ..Default::default()
        }
    }
}

unsafe impl<F, P> Send for FunctionSystem<F, P>
where
    F: Send,
    P: Send,
{
}
unsafe impl<F, P> Sync for FunctionSystem<F, P>
where
    F: Sync,
    P: Sync,
{
}
unsafe impl<F> Send for ExclusiveFunctionSystem<F> where F: Send {}
unsafe impl<F> Sync for ExclusiveFunctionSystem<F> where F: Sync {}
