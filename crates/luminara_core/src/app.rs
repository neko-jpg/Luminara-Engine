use crate::plugin::Plugin;
use crate::resource::Resource;
use crate::schedule::Schedule;
use crate::shared_types::{AppInterface, CoreStage};
use crate::system::IntoSystem;
use crate::world::World;
use std::collections::HashSet;

/// The main entry point for a Luminara application.
/// Manages the `World`, `Schedule`, and engine loop.
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    runner: Box<dyn FnOnce(App)>,
    /// Track which plugins have been registered to ensure build is called only once
    registered_plugins: HashSet<String>,
    /// Track plugin registration order for validation
    plugin_order: Vec<String>,
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
                app.schedule.run(&mut app.world);
            }),
            registered_plugins: HashSet::new(),
            plugin_order: Vec::new(),
        }
    }

    pub fn set_runner(&mut self, runner: impl FnOnce(App) + 'static) -> &mut Self {
        self.runner = Box::new(runner);
        self
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }

    /// Get the order in which plugins were registered
    pub fn plugin_order(&self) -> &[String] {
        &self.plugin_order
    }

    /// Check if a plugin has been registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.registered_plugins.contains(name)
    }
}

impl AppInterface for App {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        let plugin_name = plugin.name().to_string();

        // Only build the plugin if it hasn't been registered yet
        if !self.registered_plugins.contains(&plugin_name) {
            self.registered_plugins.insert(plugin_name.clone());
            self.plugin_order.push(plugin_name);
            plugin.build(self);
        }

        self
    }

    fn add_system<Marker>(
        &mut self,
        stage: CoreStage,
        system: impl IntoSystem<Marker>,
    ) -> &mut Self {
        self.schedule.add_system(stage, system.into_system());
        self
    }

    fn add_startup_system<Marker>(&mut self, system: impl IntoSystem<Marker>) -> &mut Self {
        self.schedule
            .add_system(CoreStage::Startup, system.into_system());
        self
    }

    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    fn register_component<C: crate::component::Component>(&mut self) -> &mut Self {
        self.world.register_component::<C>();
        self
    }

    fn run(mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(|_| {}));
        (runner)(self);
    }
}
