use crate::change_detection::ComponentTicks;
use crate::entity::Entity;
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

pub type ArchetypeId = usize;
pub type ComponentLayoutMap = HashMap<TypeId, (Layout, Option<unsafe fn(*mut u8)>)>;

pub struct Column {
    data: Vec<u8>,
    pub(crate) ticks: Vec<ComponentTicks>,
    item_layout: Layout,
    drop_fn: Option<unsafe fn(*mut u8)>,
    len: usize,
}

impl Column {
    pub fn new(layout: Layout, drop_fn: Option<unsafe fn(*mut u8)>) -> Self {
        Self {
            data: Vec::new(),
            ticks: Vec::new(),
            item_layout: layout,
            drop_fn,
            len: 0,
        }
    }

    /// # Safety
    /// `ptr` must point to a valid instance of the component type associated with this column.
    pub unsafe fn push(&mut self, ptr: *const u8, ticks: ComponentTicks) {
        let size = self.item_layout.size();
        if size > 0 {
            self.data
                .extend_from_slice(std::slice::from_raw_parts(ptr, size));
        }
        self.ticks.push(ticks);
        self.len += 1;
    }

    /// # Safety
    /// `index` must be within bounds.
    pub unsafe fn swap_remove(&mut self, index: usize) {
        let size = self.item_layout.size();
        let last_index = self.len - 1;

        if size > 0 {
            if index != last_index {
                if let Some(drop_fn) = self.drop_fn {
                    drop_fn(self.data.as_mut_ptr().add(index * size));
                }
                let src = self.data.as_ptr().add(last_index * size);
                let dst = self.data.as_mut_ptr().add(index * size);
                std::ptr::copy_nonoverlapping(src, dst, size);
            } else if let Some(drop_fn) = self.drop_fn {
                drop_fn(self.data.as_mut_ptr().add(index * size));
            }
            self.data.set_len(self.data.len() - size);
        } else if let Some(drop_fn) = self.drop_fn {
            drop_fn(std::ptr::NonNull::dangling().as_ptr());
        }
        self.ticks.swap_remove(index);
        self.len -= 1;
    }

    pub fn get_ptr(&self, index: usize) -> *const u8 {
        let size = self.item_layout.size();
        if size > 0 {
            unsafe { self.data.as_ptr().add(index * size) }
        } else {
            std::ptr::NonNull::dangling().as_ptr()
        }
    }

    pub fn get_mut_ptr(&mut self, index: usize) -> *mut u8 {
        let size = self.item_layout.size();
        if size > 0 {
            unsafe { self.data.as_mut_ptr().add(index * size) }
        } else {
            std::ptr::NonNull::dangling().as_ptr()
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let size = self.item_layout.size();
            for i in 0..self.len() {
                unsafe {
                    let ptr = if size > 0 {
                        self.data.as_mut_ptr().add(i * size)
                    } else {
                        std::ptr::NonNull::dangling().as_ptr()
                    };
                    drop_fn(ptr);
                }
            }
        }
    }
}

pub struct Archetype {
    id: ArchetypeId,
    pub(crate) types: Vec<TypeId>,
    pub(crate) columns: HashMap<TypeId, Column>,
    pub(crate) entities: Vec<Entity>,
}

impl Archetype {
    pub fn new(id: ArchetypeId, mut types: Vec<TypeId>, layouts: ComponentLayoutMap) -> Self {
        types.sort();
        let mut columns = HashMap::new();
        for &type_id in &types {
            let (layout, drop_fn) = layouts.get(&type_id).unwrap();
            columns.insert(type_id, Column::new(*layout, *drop_fn));
        }
        Self {
            id,
            types,
            columns,
            entities: Vec::new(),
        }
    }

    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// # Safety
    /// `components` must contain valid pointers for all component types in this archetype.
    pub unsafe fn push(
        &mut self,
        entity: Entity,
        components: HashMap<TypeId, (*const u8, ComponentTicks)>,
    ) {
        self.entities.push(entity);
        for (type_id, (ptr, ticks)) in components {
            self.columns.get_mut(&type_id).unwrap().push(ptr, ticks);
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> Entity {
        let entity = self.entities.swap_remove(index);
        for column in self.columns.values_mut() {
            unsafe { column.swap_remove(index) };
        }
        entity
    }

    pub fn get_component_ptr(&self, type_id: TypeId, index: usize) -> Option<*const u8> {
        self.columns.get(&type_id).map(|c| c.get_ptr(index))
    }

    pub fn get_component_mut_ptr(&mut self, type_id: TypeId, index: usize) -> Option<*mut u8> {
        self.columns.get_mut(&type_id).map(|c| c.get_mut_ptr(index))
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// # Safety
    /// Structural change. `index` must be valid.
    pub unsafe fn transfer_to(
        &mut self,
        index: usize,
        other: &mut Archetype,
        mut new_components: HashMap<TypeId, (*const u8, ComponentTicks)>,
        skip_drop: &[TypeId],
    ) -> usize {
        let entity = self.entities.swap_remove(index);
        other.entities.push(entity);
        let new_index = other.entities.len() - 1;

        let other_types = other.types.clone();
        for type_id in other_types {
            let other_column = other.columns.get_mut(&type_id).unwrap();
            if let Some((ptr, ticks)) = new_components.remove(&type_id) {
                other_column.push(ptr, ticks);
            } else {
                let self_column = self
                    .columns
                    .get_mut(&type_id)
                    .expect("Component missing during transfer");
                let size = self_column.item_layout.size();
                let ticks = self_column.ticks[index];
                let dst = other_column.allocate_next(ticks);

                if size > 0 {
                    let src = self_column.data.as_ptr().add(index * size);
                    std::ptr::copy_nonoverlapping(src, dst, size);
                }
            }
        }

        // After copying all needed components, perform swap_remove on ALL columns of the old archetype
        for (&type_id, self_column) in self.columns.iter_mut() {
            if !other.columns.contains_key(&type_id) && skip_drop.contains(&type_id) {
                // Manual swap_remove logic without drop
                let size = self_column.item_layout.size();
                let last_index = self_column.len - 1;
                if size > 0 {
                    if index != last_index {
                        let src = self_column.data.as_ptr().add(last_index * size);
                        let dst = self_column.data.as_mut_ptr().add(index * size);
                        std::ptr::copy_nonoverlapping(src, dst, size);
                    }
                    self_column.data.set_len(self_column.data.len() - size);
                }
                self_column.ticks.swap_remove(index);
                self_column.len -= 1;
            } else {
                // If it's being moved to other OR it's being dropped
                // We need to swap_remove it.
                // If it's moved to other, we don't drop.
                if other.columns.contains_key(&type_id) {
                    // Manual swap_remove without drop
                    let size = self_column.item_layout.size();
                    let last_index = self_column.len - 1;
                    if size > 0 {
                        if index != last_index {
                            let src = self_column.data.as_ptr().add(last_index * size);
                            let dst = self_column.data.as_mut_ptr().add(index * size);
                            std::ptr::copy_nonoverlapping(src, dst, size);
                        }
                        self_column.data.set_len(self_column.data.len() - size);
                    }
                    self_column.ticks.swap_remove(index);
                    self_column.len -= 1;
                } else {
                    // Not moved and not skipped -> Drop it
                    self_column.swap_remove(index);
                }
            }
        }

        new_index
    }
}

impl Column {
    /// # Safety
    /// Allocates space for the next component.
    pub unsafe fn allocate_next(&mut self, ticks: ComponentTicks) -> *mut u8 {
        let size = self.item_layout.size();
        self.ticks.push(ticks);
        if size > 0 {
            let old_len = self.data.len();
            self.data.resize(old_len + size, 0);
            self.len += 1;
            self.data.as_mut_ptr().add(old_len)
        } else {
            self.len += 1;
            std::ptr::NonNull::dangling().as_ptr()
        }
    }
}

pub struct ArchetypeStorage {
    pub(crate) archetypes: Vec<Archetype>,
    type_to_archetypes: HashMap<TypeId, Vec<ArchetypeId>>,
    signature_to_archetype: HashMap<Vec<TypeId>, ArchetypeId>,
    entity_location: HashMap<Entity, (ArchetypeId, usize)>,
}

impl Default for ArchetypeStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchetypeStorage {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            type_to_archetypes: HashMap::new(),
            signature_to_archetype: HashMap::new(),
            entity_location: HashMap::new(),
        }
    }

    pub fn get_archetype_id(
        &mut self,
        mut types: Vec<TypeId>,
        layouts: &ComponentLayoutMap,
    ) -> ArchetypeId {
        types.sort();
        if let Some(&id) = self.signature_to_archetype.get(&types) {
            return id;
        }

        let id = self.archetypes.len();
        let archetype = Archetype::new(id, types.clone(), layouts.clone());

        for &type_id in &types {
            self.type_to_archetypes.entry(type_id).or_default().push(id);
        }

        self.signature_to_archetype.insert(types, id);
        self.archetypes.push(archetype);
        id
    }

    pub fn get_archetype(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id)
    }

    pub fn get_archetype_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id)
    }

    pub fn set_entity_location(&mut self, entity: Entity, archetype_id: ArchetypeId, index: usize) {
        self.entity_location.insert(entity, (archetype_id, index));
    }

    pub fn get_entity_location(&self, entity: Entity) -> Option<(ArchetypeId, usize)> {
        self.entity_location.get(&entity).copied()
    }

    pub fn remove_entity_location(&mut self, entity: Entity) {
        self.entity_location.remove(&entity);
    }

    pub fn archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    pub fn archetypes_with_type(&self, type_id: TypeId) -> &[ArchetypeId] {
        self.type_to_archetypes
            .get(&type_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
