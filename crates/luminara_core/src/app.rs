use crate::world::World;
use crate::schedule::Schedule;
use crate::system::IntoSystem;
use crate::plugin::Plugin;
use crate::shared_types::{CoreStage, AppInterface};
use crate::resource::Resource;

/// The main entry point for a Luminara application.
/// Manages the `World`, `Schedule`, and engine loop.
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    runner: Box<dyn FnOnce(App)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates a new empty `App`.
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::new(),
            runner: Box::new(|mut app| {
                app.schedule.run_startup(&mut app.world);
                loop {
                    app.schedule.run(&mut app.world);
                    // In a real engine, we would have a way to break the loop or handle events.
                    // For now, this is just a placeholder runner.
                    break;
                }
            }),
        }
    }

    /// Adds a system that runs exactly once when the application starts.

    pub fn set_runner(&mut self, runner: impl FnOnce(App) + 'static) -> &mut Self {
        self.runner = Box::new(runner);
        self
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }
}

impl AppInterface for App {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    fn add_system<Params>(&mut self, stage: CoreStage, system: impl IntoSystem<Params>) -> &mut Self {
        self.schedule.add_system(stage, system.into_system());
        self
    }

    fn add_startup_system<Params>(&mut self, system: impl IntoSystem<Params>) -> &mut Self {
        self.schedule.add_system(CoreStage::Startup, system.into_system());
        self
    }

    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    fn run(mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(|_| {}));
        (runner)(self);
    }
}
