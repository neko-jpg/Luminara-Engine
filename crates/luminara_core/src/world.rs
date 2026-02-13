use crate::archetype::ArchetypeStorage;
use crate::bundle::Bundle;
use crate::change_detection::{ComponentTicks, Tick};
use crate::component::Component;
use crate::entity::{Entity, EntityAllocator};
use crate::event::{Event, Events};
use crate::resource::{Resource, ResourceMap};
use std::alloc::Layout;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;

/// The collection of all data in the ECS engine.
/// Holds entities, components, resources, and events.
pub struct World {
    pub(crate) entities: EntityAllocator,
    pub(crate) archetypes: ArchetypeStorage,
    pub(crate) resources: ResourceMap,
    pub(crate) event_bus: UnsafeCell<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    pub(crate) component_info: HashMap<TypeId, ComponentInfo>,
    pub(crate) change_tick: Tick,
    pub(crate) last_change_tick: Tick,
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

pub struct ComponentInfo {
    pub layout: Layout,
    pub drop_fn: Option<unsafe fn(*mut u8)>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::default(),
            archetypes: ArchetypeStorage::new(),
            resources: ResourceMap::new(),
            event_bus: UnsafeCell::new(HashMap::new()),
            component_info: HashMap::new(),
            change_tick: Tick(1),
            last_change_tick: Tick(0),
        }
    }

    pub fn increment_tick(&mut self) {
        self.change_tick.increment();
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.entities.iter_alive().collect()
    }

    /// Spawns a new empty entity and returns its ID.
    pub fn spawn(&mut self) -> Entity {
        self.spawn_bundle(())
    }

    /// Spawns a new entity with a bundle of components.
    /// This is more efficient than spawning and adding components individually.
    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = self.entities.spawn();
        B::register_components(self);

        let types = B::component_ids();
        let mut layouts = HashMap::new();
        for &t in &types {
            let info = self.component_info.get(&t).unwrap();
            layouts.insert(t, (info.layout, info.drop_fn));
        }

        let archetype_id = self.archetypes.get_archetype_id(types, &layouts);
        let ticks = ComponentTicks {
            added: self.change_tick,
            changed: self.change_tick,
        };
        let components = bundle.get_components(&self.component_info, ticks);

        let index = {
            let archetype = self.archetypes.get_archetype_mut(archetype_id).unwrap();
            unsafe {
                archetype.push(entity, components.clone());
            }
            archetype.len() - 1
        };

        // Clean up temporary boxes from bundle
        for (type_id, (ptr, _)) in components {
            let info = self.component_info.get(&type_id).unwrap();
            unsafe {
                if info.layout.size() > 0 {
                    std::alloc::dealloc(ptr as *mut u8, info.layout);
                }
            }
        }

        self.archetypes
            .set_entity_location(entity, archetype_id, index);
        entity
    }

    pub fn add_bundle<B: Bundle>(&mut self, entity: Entity, bundle: B) {
        B::register_components(self);
        let (old_archetype_id, old_index) = self
            .archetypes
            .get_entity_location(entity)
            .expect("Entity not found");

        let mut new_types = self
            .archetypes
            .get_archetype(old_archetype_id)
            .unwrap()
            .types()
            .to_vec();
        let bundle_types = B::component_ids();

        for t in &bundle_types {
            if !new_types.contains(t) {
                new_types.push(*t);
            }
        }

        let mut layouts = HashMap::new();
        for &t in &new_types {
            let info = self.component_info.get(&t).unwrap();
            layouts.insert(t, (info.layout, info.drop_fn));
        }

        let new_archetype_id = self.archetypes.get_archetype_id(new_types, &layouts);
        if old_archetype_id == new_archetype_id {
            let ticks = ComponentTicks {
                added: self.change_tick,
                changed: self.change_tick,
            };
            let components = bundle.get_components(&self.component_info, ticks);
            let archetype = self.archetypes.get_archetype_mut(old_archetype_id).unwrap();
            for (type_id, (ptr, _)) in components {
                let target_ptr = archetype.get_component_mut_ptr(type_id, old_index).unwrap();
                let info = self.component_info.get(&type_id).unwrap();
                unsafe {
                    if let Some(drop_fn) = info.drop_fn {
                        drop_fn(target_ptr);
                    }
                    std::ptr::copy_nonoverlapping(ptr, target_ptr, info.layout.size());
                    if info.layout.size() > 0 {
                        std::alloc::dealloc(ptr as *mut u8, info.layout);
                    }
                }
            }
            return;
        }

        let (old_archetype, new_archetype) = if old_archetype_id < new_archetype_id {
            let (left, right) = self.archetypes.archetypes.split_at_mut(new_archetype_id);
            (&mut left[old_archetype_id], &mut right[0])
        } else {
            let (left, right) = self.archetypes.archetypes.split_at_mut(old_archetype_id);
            (&mut right[0], &mut left[new_archetype_id])
        };

        let ticks = ComponentTicks {
            added: self.change_tick,
            changed: self.change_tick,
        };
        let components = bundle.get_components(&self.component_info, ticks);

        let (new_index, swapped_entity) = unsafe {
            let new_index =
                old_archetype.transfer_to(old_index, new_archetype, components.clone(), &[]);
            for (type_id, (ptr, _)) in components {
                let info = self.component_info.get(&type_id).unwrap();
                if info.layout.size() > 0 {
                    std::alloc::dealloc(ptr as *mut u8, info.layout);
                }
            }
            let swapped_entity = if old_index < old_archetype.len() {
                Some(old_archetype.entities()[old_index])
            } else {
                None
            };
            (new_index, swapped_entity)
        };

        self.archetypes
            .set_entity_location(entity, new_archetype_id, new_index);
        if let Some(swapped) = swapped_entity {
            self.archetypes
                .set_entity_location(swapped, old_archetype_id, old_index);
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let (old_archetype_id, old_index) = self.archetypes.get_entity_location(entity)?;
        let mut new_types = self
            .archetypes
            .get_archetype(old_archetype_id)
            .unwrap()
            .types()
            .to_vec();
        let type_id = TypeId::of::<T>();

        if let Some(pos) = new_types.iter().position(|&t| t == type_id) {
            new_types.remove(pos);
        } else {
            return None;
        }

        let mut layouts = HashMap::new();
        for &t in &new_types {
            let info = self.component_info.get(&t).unwrap();
            layouts.insert(t, (info.layout, info.drop_fn));
        }

        let new_archetype_id = self.archetypes.get_archetype_id(new_types, &layouts);

        let old_archetype = self.archetypes.get_archetype_mut(old_archetype_id).unwrap();
        let component_ptr = old_archetype.get_component_ptr(type_id, old_index).unwrap();
        let component = unsafe { std::ptr::read(component_ptr as *const T) };

        let (old_archetype, new_archetype) = if old_archetype_id < new_archetype_id {
            let (left, right) = self.archetypes.archetypes.split_at_mut(new_archetype_id);
            (&mut left[old_archetype_id], &mut right[0])
        } else {
            let (left, right) = self.archetypes.archetypes.split_at_mut(old_archetype_id);
            (&mut right[0], &mut left[new_archetype_id])
        };

        let (new_index, swapped_entity) = unsafe {
            let new_index =
                old_archetype.transfer_to(old_index, new_archetype, HashMap::new(), &[type_id]);
            let swapped_entity = if old_index < old_archetype.len() {
                Some(old_archetype.entities()[old_index])
            } else {
                None
            };
            (new_index, swapped_entity)
        };

        self.archetypes
            .set_entity_location(entity, new_archetype_id, new_index);
        if let Some(swapped) = swapped_entity {
            self.archetypes
                .set_entity_location(swapped, old_archetype_id, old_index);
        }

        Some(component)
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some((archetype_id, index)) = self.archetypes.get_entity_location(entity) {
            let archetype = self.archetypes.get_archetype_mut(archetype_id).unwrap();
            let swapped_entity = archetype.swap_remove(index);
            if swapped_entity != entity {
                self.archetypes
                    .set_entity_location(swapped_entity, archetype_id, index);
            }
            self.archetypes.remove_entity_location(entity);
            self.entities.despawn(entity);
            true
        } else {
            false
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get::<R>()
    }

    pub fn get_resource_mut<R: Resource>(&self) -> Option<&mut R> {
        self.resources.get_mut::<R>()
    }

    pub fn add_event<E: Event>(&self, event: E) {
        self.get_events_mut::<E>().send(event);
    }

    /// # Safety
    /// Interior mutability for events.
    #[allow(clippy::mut_from_ref)]
    pub fn get_events_mut<E: Event>(&self) -> &mut Events<E> {
        unsafe {
            let bus = &mut *self.event_bus.get();
            let any = bus
                .entry(TypeId::of::<E>())
                .or_insert_with(|| Box::new(Events::<E>::default()));
            any.downcast_mut::<Events<E>>().unwrap()
        }
    }

    pub fn get_events<E: Event>(&self) -> Option<&Events<E>> {
        unsafe {
            let bus = &*self.event_bus.get();
            bus.get(&TypeId::of::<E>())
                .and_then(|any| any.downcast_ref::<Events<E>>())
        }
    }

    pub fn is_component_registered<T: Component>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_info.contains_key(&type_id)
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.component_info
            .entry(type_id)
            .or_insert_with(|| ComponentInfo {
                layout: Layout::new::<T>(),
                drop_fn: Some(|ptr| unsafe { std::ptr::drop_in_place(ptr as *mut T) }),
            });
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.register_component::<T>();
        let (old_archetype_id, old_index) = self
            .archetypes
            .get_entity_location(entity)
            .expect("Entity not found");

        let mut new_types = self
            .archetypes
            .get_archetype(old_archetype_id)
            .unwrap()
            .types()
            .to_vec();
        let type_id = TypeId::of::<T>();
        if new_types.contains(&type_id) {
            let archetype = self.archetypes.get_archetype_mut(old_archetype_id).unwrap();
            let ptr = archetype.get_component_mut_ptr(type_id, old_index).unwrap();
            unsafe {
                std::ptr::drop_in_place(ptr as *mut T);
                std::ptr::write(ptr as *mut T, component);
            }
            return;
        }

        new_types.push(type_id);

        let mut layouts = HashMap::new();
        for &t in &new_types {
            let info = self.component_info.get(&t).unwrap();
            layouts.insert(t, (info.layout, info.drop_fn));
        }

        let new_archetype_id = self.archetypes.get_archetype_id(new_types, &layouts);

        let (old_archetype, new_archetype) = if old_archetype_id < new_archetype_id {
            let (left, right) = self.archetypes.archetypes.split_at_mut(new_archetype_id);
            (&mut left[old_archetype_id], &mut right[0])
        } else {
            let (left, right) = self.archetypes.archetypes.split_at_mut(old_archetype_id);
            (&mut right[0], &mut left[new_archetype_id])
        };

        let mut new_components = HashMap::new();
        let ticks = ComponentTicks {
            added: self.change_tick,
            changed: self.change_tick,
        };
        new_components.insert(type_id, (&component as *const T as *const u8, ticks));

        let (new_index, swapped_entity) = unsafe {
            let new_index =
                old_archetype.transfer_to(old_index, new_archetype, new_components, &[]);
            std::mem::forget(component);

            let swapped_entity = if old_index < old_archetype.len() {
                Some(old_archetype.entities()[old_index])
            } else {
                None
            };
            (new_index, swapped_entity)
        };

        self.archetypes
            .set_entity_location(entity, new_archetype_id, new_index);
        if let Some(swapped) = swapped_entity {
            self.archetypes
                .set_entity_location(swapped, old_archetype_id, old_index);
        }
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let (archetype_id, index) = self.archetypes.get_entity_location(entity)?;
        let archetype = self.archetypes.get_archetype(archetype_id)?;
        let ptr = archetype.get_component_ptr(TypeId::of::<T>(), index)?;
        Some(unsafe { &*(ptr as *const T) })
    }

    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut<'_> {
        EntityMut {
            world: self,
            entity,
        }
    }

    /// # Safety
    /// Interior mutability for components.
    #[allow(clippy::mut_from_ref)]
    pub fn get_component_mut<T: Component>(&self, entity: Entity) -> Option<&mut T> {
        let (archetype_id, index) = self.archetypes.get_entity_location(entity)?;
        let tick = self.change_tick;
        unsafe {
            let world_ptr = self as *const World as *mut World;
            let archetype = (*world_ptr).archetypes.get_archetype_mut(archetype_id)?;
            let type_id = TypeId::of::<T>();
            let ptr = archetype.get_component_mut_ptr(type_id, index)?;
            archetype.columns.get_mut(&type_id).unwrap().ticks[index].changed = tick;
            Some(&mut *(ptr as *mut T))
        }
    }
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.world.add_component(self.entity, component);
        self
    }

    pub fn insert_bundle<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        self.world.add_bundle(self.entity, bundle);
        self
    }

    pub fn remove<T: Component>(&mut self) -> &mut Self {
        self.world.remove_component::<T>(self.entity);
        self
    }

    pub fn despawn(self) {
        self.world.despawn(self.entity);
    }
}
