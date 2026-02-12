use crate::world::World;
use crate::archetype::{Archetype};
use crate::component::Component;
use crate::change_detection::Tick;
use crate::system::SystemAccess;
use std::marker::PhantomData;
use std::any::TypeId;

pub trait WorldQuery: Send + Sync {
    type Item<'a>;
    type Fetch<'a>;
    fn matches_archetype(archetype: &Archetype) -> bool;
    unsafe fn get_fetch<'a>(archetype: &'a Archetype) -> Self::Fetch<'a>;
    unsafe fn fetch<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a>;
    fn component_ids() -> Vec<TypeId>;
    fn add_access(access: &mut SystemAccess);
}

pub trait QueryFilter: Send + Sync {
    fn matches_archetype(archetype: &Archetype) -> bool;
    fn matches_entity(archetype: &Archetype, index: usize, last_tick: Tick, current_tick: Tick) -> bool;
    fn component_ids() -> Vec<TypeId>;
}

impl QueryFilter for () {
    fn matches_archetype(_: &Archetype) -> bool { true }
    fn matches_entity(_: &Archetype, _: usize, _: Tick, _: Tick) -> bool { true }
    fn component_ids() -> Vec<TypeId> { Vec::new() }
}

pub struct With<T: Component>(PhantomData<T>);
impl<T: Component> QueryFilter for With<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.types().contains(&TypeId::of::<T>())
    }
    fn matches_entity(_: &Archetype, _: usize, _: Tick, _: Tick) -> bool { true }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
}

pub struct Without<T: Component>(PhantomData<T>);
impl<T: Component> QueryFilter for Without<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        !archetype.types().contains(&TypeId::of::<T>())
    }
    fn matches_entity(_: &Archetype, _: usize, _: Tick, _: Tick) -> bool { true }
    fn component_ids() -> Vec<TypeId> { Vec::new() }
}

pub struct Changed<T: Component>(PhantomData<T>);
impl<T: Component> QueryFilter for Changed<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.types().contains(&TypeId::of::<T>())
    }
    fn matches_entity(archetype: &Archetype, index: usize, last_tick: Tick, _current_tick: Tick) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(column) = archetype.columns.get(&type_id) {
            column.ticks[index].changed > last_tick
        } else {
            false
        }
    }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
}

pub struct Added<T: Component>(PhantomData<T>);
impl<T: Component> QueryFilter for Added<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.types().contains(&TypeId::of::<T>())
    }
    fn matches_entity(archetype: &Archetype, index: usize, last_tick: Tick, _current_tick: Tick) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(column) = archetype.columns.get(&type_id) {
            column.ticks[index].added > last_tick
        } else {
            false
        }
    }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
}

pub struct Or<T>(pub T);

macro_rules! impl_query_filter_tuple {
    ($($t:ident),*) => {
        impl<$($t: QueryFilter),*> QueryFilter for Or<($($t,)*)> {
            fn matches_archetype(archetype: &Archetype) -> bool {
                false $(|| $t::matches_archetype(archetype))*
            }
            fn matches_entity(archetype: &Archetype, index: usize, last_tick: Tick, current_tick: Tick) -> bool {
                false $(|| $t::matches_entity(archetype, index, last_tick, current_tick))*
            }
            fn component_ids() -> Vec<TypeId> {
                Vec::new() // Or cannot be used for simple intersection narrowing
            }
        }
    };
}

impl_query_filter_tuple!(A, B);
impl_query_filter_tuple!(A, B, C);

pub struct Query<'w, Q: WorldQuery, F: QueryFilter = ()> {
    pub(crate) world: &'w World,
    pub(crate) _marker: PhantomData<(Q, F)>,
}

impl<'w, Q: WorldQuery, F: QueryFilter> Query<'w, Q, F> {
    pub fn new(world: &'w World) -> Self {
        Self { world, _marker: PhantomData }
    }

    pub fn iter(&self) -> impl Iterator<Item = Q::Item<'_>> {
        let last_tick = self.world.last_change_tick;
        let current_tick = self.world.change_tick;

        let mut types = Q::component_ids();
        types.extend(F::component_ids());

        let archetype_ids = if let Some(first_type) = types.first() {
             let mut ids: std::collections::HashSet<usize> = self.world.archetypes.archetypes_with_type(*first_type).iter().copied().collect();
             for type_id in types.iter().skip(1) {
                 let next_ids: std::collections::HashSet<usize> = self.world.archetypes.archetypes_with_type(*type_id).iter().copied().collect();
                 ids.retain(|id| next_ids.contains(id));
             }
             Some(ids)
        } else {
            None
        };

        let archetypes = if let Some(ids) = archetype_ids {
             ids.into_iter().map(|id| &self.world.archetypes.archetypes[id]).collect::<Vec<_>>()
        } else {
             self.world.archetypes.archetypes().iter().collect::<Vec<_>>()
        };

        archetypes.into_iter()
            .filter(|a| Q::matches_archetype(a) && F::matches_archetype(a))
            .flat_map(move |a| {
                let mut fetch = unsafe { Q::get_fetch(a) };
                (0..a.len())
                    .filter(move |&i| F::matches_entity(a, i, last_tick, current_tick))
                    .map(move |i| unsafe { Q::fetch(&mut fetch, i) })
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = Q::Item<'_>> {
        self.iter()
    }

    pub fn for_each<Func>(&self, mut f: Func)
    where
        Func: FnMut(Q::Item<'_>),
    {
        for item in self.iter() {
            f(item);
        }
    }

    pub fn par_for_each<Func>(&self, f: Func)
    where
        Func: Fn(Q::Item<'_>) + Send + Sync + Clone,
    {
        use rayon::prelude::*;
        let last_tick = self.world.last_change_tick;
        let current_tick = self.world.change_tick;

        // Note: Simple implementation, could be optimized by using par_iter on archetypes
        self.world.archetypes.archetypes().par_iter()
            .filter(|a| Q::matches_archetype(a) && F::matches_archetype(a))
            .for_each(|a| {
                for i in 0..a.len() {
                    if F::matches_entity(a, i, last_tick, current_tick) {
                        unsafe {
                            let mut fetch = Q::get_fetch(a);
                            f(Q::fetch(&mut fetch, i));
                        }
                    }
                }
            });
    }
}

impl<'a, T: Component> WorldQuery for &'a T {
    type Item<'w> = &'w T;
    type Fetch<'w> = *const u8;
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.types().contains(&TypeId::of::<T>())
    }
    unsafe fn get_fetch<'w>(archetype: &'w Archetype) -> Self::Fetch<'w> {
        archetype.get_component_ptr(TypeId::of::<T>(), 0).unwrap_or(std::ptr::null())
    }
    unsafe fn fetch<'w>(fetch: &mut Self::Fetch<'w>, index: usize) -> Self::Item<'w> {
        let ptr = fetch.add(index * std::mem::size_of::<T>());
        &*(ptr as *const T)
    }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
    fn add_access(access: &mut SystemAccess) {
        access.components_read.insert(TypeId::of::<T>());
    }
}

impl<'a, T: Component> WorldQuery for &'a mut T {
    type Item<'w> = &'w mut T;
    type Fetch<'w> = *mut u8;
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.types().contains(&TypeId::of::<T>())
    }
    unsafe fn get_fetch<'w>(archetype: &'w Archetype) -> Self::Fetch<'w> {
        archetype.get_component_ptr(TypeId::of::<T>(), 0).unwrap_or(std::ptr::null()) as *mut u8
    }
    unsafe fn fetch<'w>(fetch: &mut Self::Fetch<'w>, index: usize) -> Self::Item<'w> {
        let ptr = fetch.add(index * std::mem::size_of::<T>());
        &mut *(ptr as *mut T)
    }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
    fn add_access(access: &mut SystemAccess) {
        access.components_write.insert(TypeId::of::<T>());
    }
}

use crate::entity::Entity;
impl WorldQuery for Entity {
    type Item<'w> = Entity;
    type Fetch<'w> = *const Entity;
    fn matches_archetype(_: &Archetype) -> bool { true }
    unsafe fn get_fetch<'w>(archetype: &'w Archetype) -> Self::Fetch<'w> {
        archetype.entities().as_ptr()
    }
    unsafe fn fetch<'w>(fetch: &mut Self::Fetch<'w>, index: usize) -> Self::Item<'w> {
        *fetch.add(index)
    }
    fn component_ids() -> Vec<TypeId> { Vec::new() }
    fn add_access(_: &mut SystemAccess) {}
}

impl<A: WorldQuery, B: WorldQuery> WorldQuery for (A, B) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>);
    type Fetch<'w> = (A::Fetch<'w>, B::Fetch<'w>);
    fn matches_archetype(archetype: &Archetype) -> bool {
        A::matches_archetype(archetype) && B::matches_archetype(archetype)
    }
    unsafe fn get_fetch<'w>(archetype: &'w Archetype) -> Self::Fetch<'w> {
        (A::get_fetch(archetype), B::get_fetch(archetype))
    }
    unsafe fn fetch<'w>(fetch: &mut Self::Fetch<'w>, index: usize) -> Self::Item<'w> {
        (A::fetch(&mut fetch.0, index), B::fetch(&mut fetch.1, index))
    }
    fn component_ids() -> Vec<TypeId> {
        let mut ids = A::component_ids();
        ids.extend(B::component_ids());
        ids
    }
    fn add_access(access: &mut SystemAccess) {
        A::add_access(access);
        B::add_access(access);
    }
}

impl<A: WorldQuery, B: WorldQuery, C: WorldQuery> WorldQuery for (A, B, C) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>, C::Item<'w>);
    type Fetch<'w> = (A::Fetch<'w>, B::Fetch<'w>, C::Fetch<'w>);
    fn matches_archetype(archetype: &Archetype) -> bool {
        A::matches_archetype(archetype) && B::matches_archetype(archetype) && C::matches_archetype(archetype)
    }
    unsafe fn get_fetch<'w>(archetype: &'w Archetype) -> Self::Fetch<'w> {
        (A::get_fetch(archetype), B::get_fetch(archetype), C::get_fetch(archetype))
    }
    unsafe fn fetch<'w>(fetch: &mut Self::Fetch<'w>, index: usize) -> Self::Item<'w> {
        (A::fetch(&mut fetch.0, index), B::fetch(&mut fetch.1, index), C::fetch(&mut fetch.2, index))
    }
    fn component_ids() -> Vec<TypeId> {
        let mut ids = A::component_ids();
        ids.extend(B::component_ids());
        ids.extend(C::component_ids());
        ids
    }
    fn add_access(access: &mut SystemAccess) {
        A::add_access(access);
        B::add_access(access);
        C::add_access(access);
    }
}
