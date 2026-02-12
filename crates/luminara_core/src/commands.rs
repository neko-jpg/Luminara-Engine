use crate::bundle::Bundle;
use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;

pub trait Command: Send + Sync + 'static {
    fn apply(self: Box<Self>, world: &mut World);
}

pub struct CommandQueue {
    commands: Vec<Box<dyn Command>>,
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push<C: Command>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    pub fn apply_or_drop_all(&mut self, world: &mut World) {
        for command in self.commands.drain(..) {
            command.apply(world);
        }
    }
}

pub struct Commands<'a> {
    queue: &'a mut CommandQueue,
}

impl<'a> Commands<'a> {
    pub fn new(queue: &'a mut CommandQueue) -> Self {
        Self { queue }
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands<'_, 'a> {
        // We can't know the entity ID until applied, but we can use a "spawn" command
        // that will handle following inserts.
        // For simplicity, let's just push a single command for now.
        let mut entity_cmds = EntityCommands {
            commands: self,
            entity: None,
        };
        entity_cmds.insert_bundle(bundle);
        entity_cmds
    }

    pub fn entity(&mut self, entity: Entity) -> EntityCommands<'_, 'a> {
        EntityCommands {
            commands: self,
            entity: Some(entity),
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(DespawnCommand { entity });
    }
}

pub struct EntityCommands<'a, 'b> {
    commands: &'a mut Commands<'b>,
    entity: Option<Entity>,
}

impl<'a, 'b> EntityCommands<'a, 'b> {
    pub fn id(&self) -> Option<Entity> {
        self.entity
    }

    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        if let Some(entity) = self.entity {
            self.commands
                .queue
                .push(InsertCommand { entity, component });
        } else {
            self.commands.queue.push(SpawnAndInsertCommand {
                component: Some(component),
            });
        }
        self
    }

    pub fn insert_bundle<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        if let Some(entity) = self.entity {
            self.commands
                .queue
                .push(InsertBundleCommand { entity, bundle });
        } else {
            self.commands.queue.push(SpawnBundleCommand { bundle });
        }
        self
    }

    pub fn remove<T: Component>(&mut self) -> &mut Self {
        if let Some(entity) = self.entity {
            self.commands.queue.push(RemoveComponentCommand::<T> {
                entity,
                _marker: std::marker::PhantomData,
            });
        }
        self
    }

    pub fn despawn(&mut self) -> &mut Self {
        if let Some(entity) = self.entity {
            self.commands.queue.push(DespawnCommand { entity });
        }
        self
    }
}

struct DespawnCommand {
    entity: Entity,
}
impl Command for DespawnCommand {
    fn apply(self: Box<Self>, world: &mut World) {
        world.despawn(self.entity);
    }
}

struct InsertCommand<T: Component> {
    entity: Entity,
    component: T,
}
impl<T: Component> Command for InsertCommand<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.add_component(self.entity, self.component);
    }
}

struct SpawnAndInsertCommand<T: Component> {
    component: Option<T>,
}
impl<T: Component> Command for SpawnAndInsertCommand<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        let entity = world.spawn();
        if let Some(c) = self.component {
            world.add_component(entity, c);
        }
    }
}

struct SpawnBundleCommand<B: Bundle> {
    bundle: B,
}
impl<B: Bundle> Command for SpawnBundleCommand<B> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.spawn_bundle(self.bundle);
    }
}

struct InsertBundleCommand<B: Bundle> {
    entity: Entity,
    bundle: B,
}
impl<B: Bundle> Command for InsertBundleCommand<B> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.add_bundle(self.entity, self.bundle);
    }
}

struct RemoveComponentCommand<T: Component> {
    entity: Entity,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Component> Command for RemoveComponentCommand<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.remove_component::<T>(self.entity);
    }
}
