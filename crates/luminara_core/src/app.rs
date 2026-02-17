use crate::plugin::{Plugin, PluginError};
use crate::resource::Resource;
use crate::schedule::Schedule;
use crate::shared_types::{AppInterface, CoreStage};
use crate::system::IntoSystem;
use crate::world::World;
use std::collections::{HashMap, HashSet};

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
    /// Track plugin versions for dependency validation
    plugin_versions: HashMap<String, String>,
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
            plugin_versions: HashMap::new(),
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

    /// Validate that all dependencies for a plugin are satisfied
    /// Returns Ok(()) if all dependencies are met, or Err with details of missing dependencies
    pub fn validate_plugin_dependencies(&self, plugin: &dyn Plugin) -> Result<(), PluginError> {
        let dependencies = plugin.dependencies();
        let mut missing = Vec::new();

        for dep in dependencies {
            if !self.registered_plugins.contains(&dep.name) {
                missing.push(dep.clone());
            } else if let Some(required_version) = &dep.version {
                // Check version constraint if specified
                if let Some(found_version) = self.plugin_versions.get(&dep.name) {
                    if !Self::version_satisfies(found_version, required_version) {
                        return Err(PluginError::VersionMismatch {
                            plugin_name: plugin.name().to_string(),
                            dependency: dep.name.clone(),
                            required: required_version.clone(),
                            found: found_version.clone(),
                        });
                    }
                }
            }
        }

        if !missing.is_empty() {
            return Err(PluginError::MissingDependencies {
                plugin_name: plugin.name().to_string(),
                missing,
            });
        }

        Ok(())
    }

    /// Check if a version satisfies a version constraint
    /// Supports simple constraints: ">=X.Y.Z", "^X.Y", "X.Y.Z"
    fn version_satisfies(found: &str, required: &str) -> bool {
        // For now, implement simple exact match and >= comparison
        // A full implementation would use semver crate
        if required.starts_with(">=") {
            let required_ver = &required[2..];
            Self::compare_versions(found, required_ver) >= 0
        } else if required.starts_with('^') {
            // Caret: compatible with version (same major version)
            let required_ver = &required[1..];
            let found_parts: Vec<&str> = found.split('.').collect();
            let required_parts: Vec<&str> = required_ver.split('.').collect();
            
            if found_parts.is_empty() || required_parts.is_empty() {
                return false;
            }
            
            // Major version must match
            found_parts[0] == required_parts[0] && Self::compare_versions(found, required_ver) >= 0
        } else {
            // Exact match
            found == required
        }
    }

    /// Compare two version strings
    /// Returns: -1 if v1 < v2, 0 if v1 == v2, 1 if v1 > v2
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let v2_parts: Vec<u32> = v2
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..v1_parts.len().max(v2_parts.len()) {
            let p1 = v1_parts.get(i).copied().unwrap_or(0);
            let p2 = v2_parts.get(i).copied().unwrap_or(0);

            if p1 < p2 {
                return -1;
            } else if p1 > p2 {
                return 1;
            }
        }

        0
    }
}

impl AppInterface for App {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        let plugin_name = plugin.name().to_string();

        // Only build the plugin if it hasn't been registered yet
        if !self.registered_plugins.contains(&plugin_name) {
            // Validate dependencies before loading
            if let Err(e) = self.validate_plugin_dependencies(&plugin) {
                eprintln!("Error loading plugin '{}': {}", plugin_name, e);
                eprintln!("Plugin '{}' will not be loaded. Please ensure all dependencies are loaded first.", plugin_name);
                return self;
            }

            self.registered_plugins.insert(plugin_name.clone());
            self.plugin_order.push(plugin_name.clone());
            self.plugin_versions.insert(plugin_name, plugin.version().to_string());
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
