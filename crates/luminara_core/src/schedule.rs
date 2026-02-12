use crate::shared_types::CoreStage;
use crate::system::{System, SystemAccess};
use crate::world::World;
use std::collections::HashMap;

pub struct Schedule {
    stages: HashMap<CoreStage, Vec<Box<dyn System>>>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Schedule {
    pub fn new() -> Self {
        let stages = HashMap::new();
        Self { stages }
    }

    pub fn add_system(&mut self, stage: CoreStage, system: impl System + 'static) {
        self.stages.entry(stage).or_default().push(Box::new(system));
    }

    pub fn run(&mut self, world: &mut World) {
        let stages_order = [
            CoreStage::Startup,
            CoreStage::PreUpdate,
            CoreStage::Update,
            CoreStage::FixedUpdate,
            CoreStage::PostUpdate,
            CoreStage::PreRender,
            CoreStage::Render,
            CoreStage::PostRender,
        ];

        for stage in stages_order {
            if let Some(systems) = self.stages.get_mut(&stage) {
                Self::run_systems(systems, world);
            }
        }
    }

    fn run_systems(systems: &mut [Box<dyn System>], world: &mut World) {
        let mut batches: Vec<Vec<&mut Box<dyn System>>> = Vec::new();
        let mut current_batch: Vec<&mut Box<dyn System>> = Vec::new();
        let mut current_batch_access = SystemAccess::default();

        for system in systems.iter_mut() {
            let access = system.access();

            if Self::conflicts(&current_batch_access, &access)
                || (access.components_write.is_empty()
                    && access.components_read.is_empty()
                    && access.resources_write.is_empty()
                    && access.resources_read.is_empty())
            {
                if !current_batch.is_empty() {
                    batches.push(std::mem::take(&mut current_batch));
                    current_batch_access = SystemAccess::default();
                }
                batches.push(vec![system]);
            } else {
                Self::merge_access(&mut current_batch_access, &access);
                current_batch.push(system);
            }
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        for mut batch in batches {
            if batch.len() == 1 {
                if batch[0].access().exclusive {
                    batch[0].run_exclusive(world);
                } else {
                    batch[0].run(world);
                }
            } else {
                let world_ptr = world as *const World as usize;
                let systems_ptrs: Vec<usize> = batch
                    .iter()
                    .map(|s| *s as *const Box<dyn System> as usize)
                    .collect();

                rayon::scope(|s| {
                    for system_ptr in systems_ptrs {
                        s.spawn(move |_| unsafe {
                            let world = &*(world_ptr as *const World);
                            let system = &mut *(system_ptr as *mut Box<dyn System>);
                            system.run(world);
                        });
                    }
                });
            }
        }
    }

    fn conflicts(a: &SystemAccess, b: &SystemAccess) -> bool {
        if a.resources_write
            .iter()
            .any(|id| b.resources_write.contains(id))
        {
            return true;
        }
        if a.components_write
            .iter()
            .any(|id| b.components_write.contains(id))
        {
            return true;
        }
        if a.resources_read
            .iter()
            .any(|id| b.resources_write.contains(id))
        {
            return true;
        }
        if a.resources_write
            .iter()
            .any(|id| b.resources_read.contains(id))
        {
            return true;
        }
        if a.components_read
            .iter()
            .any(|id| b.components_write.contains(id))
        {
            return true;
        }
        if a.components_write
            .iter()
            .any(|id| b.components_read.contains(id))
        {
            return true;
        }
        false
    }

    fn merge_access(target: &mut SystemAccess, source: &SystemAccess) {
        target.resources_read.extend(&source.resources_read);
        target.resources_write.extend(&source.resources_write);
        target.components_read.extend(&source.components_read);
        target.components_write.extend(&source.components_write);
    }

    pub fn run_startup(&mut self, world: &mut World) {
        if let Some(systems) = self.stages.get_mut(&CoreStage::Startup) {
            Self::run_systems(systems, world);
        }
        self.stages.remove(&CoreStage::Startup);
    }
}
