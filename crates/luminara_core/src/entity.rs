use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32,
}

impl Entity {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

#[derive(Default)]
pub struct EntityAllocator {
    next_id: u32,
    free_list: Vec<Entity>,
    generations: Vec<u32>,
}

impl EntityAllocator {
    pub fn spawn(&mut self) -> Entity {
        if let Some(mut entity) = self.free_list.pop() {
            // Reusing an ID, generation is already incremented when despawned
            entity.generation = self.generations[entity.id as usize];
            entity
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.generations.push(0);
            Entity { id, generation: 0 }
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let index = entity.id as usize;
        self.generations[index] += 1;
        self.free_list.push(Entity {
            id: entity.id,
            generation: self.generations[index],
        });
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.id as usize;
        index < self.generations.len() && self.generations[index] == entity.generation
    }

    pub fn entities_count(&self) -> usize {
        self.generations.len() - self.free_list.len()
    }

    pub fn iter_alive(&self) -> impl Iterator<Item = Entity> + '_ {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(id, &gen)| {
                let entity = Entity {
                    id: id as u32,
                    generation: gen,
                };
                if self.is_alive(entity) {
                    Some(entity)
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_spawn_despawn() {
        let mut allocator = EntityAllocator::default();
        let e1 = allocator.spawn();
        let e2 = allocator.spawn();

        assert!(allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
        assert_ne!(e1, e2);

        allocator.despawn(e1);
        assert!(!allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));

        let e3 = allocator.spawn();
        assert!(allocator.is_alive(e3));
        assert_eq!(e3.id, e1.id);
        assert_ne!(e3.generation, e1.generation);
    }
}
