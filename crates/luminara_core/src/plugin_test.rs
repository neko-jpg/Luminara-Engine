/// Plugin testing framework with mock engine state
///
/// This module provides utilities for testing plugins in isolation without requiring
/// a full engine setup. It includes mock engine state, test utilities, and helpers
/// for verifying plugin behavior.
///
/// # Example
///
/// ```
/// use luminara_core::plugin_test::{MockApp, PluginTestContext};
/// use luminara_core::{Plugin, AppInterface};
///
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn build(&self, app: &mut luminara_core::App) {
///         // Plugin implementation
///     }
///     
///     fn name(&self) -> &str {
///         "my_plugin"
///     }
/// }
///
/// #[test]
/// fn test_my_plugin() {
///     let mut ctx = PluginTestContext::new();
///     ctx.add_plugin(MyPlugin);
///     
///     // Verify plugin behavior
///     assert!(ctx.has_plugin("my_plugin"));
/// }
/// ```

use crate::app::App;
use crate::component::Component;
use crate::plugin::{Plugin, PluginDependency, PluginError};
use crate::resource::Resource;
use crate::shared_types::{AppInterface, CoreStage};
use crate::system::IntoSystem;
use crate::world::World;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Mock application for plugin testing
///
/// Provides a lightweight mock of the App structure for testing plugins
/// in isolation. Tracks all plugin registrations, component registrations,
/// resource insertions, and system additions without requiring a full engine.
pub struct MockApp {
    /// Mock world for testing
    pub world: World,
    /// Registered plugins
    registered_plugins: HashSet<String>,
    /// Plugin registration order
    plugin_order: Vec<String>,
    /// Plugin versions
    plugin_versions: HashMap<String, String>,
    /// Registered components (tracked by TypeId)
    registered_components: HashSet<TypeId>,
    /// Registered resources (tracked by TypeId)
    registered_resources: HashSet<TypeId>,
    /// Systems added to each stage
    systems_by_stage: HashMap<CoreStage, Vec<String>>,
    /// Startup systems
    startup_systems: Vec<String>,
}

impl Default for MockApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MockApp {
    /// Create a new mock app for testing
    pub fn new() -> Self {
        Self {
            world: World::new(),
            registered_plugins: HashSet::new(),
            plugin_order: Vec::new(),
            plugin_versions: HashMap::new(),
            registered_components: HashSet::new(),
            registered_resources: HashSet::new(),
            systems_by_stage: HashMap::new(),
            startup_systems: Vec::new(),
        }
    }

    /// Check if a plugin has been registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.registered_plugins.contains(name)
    }

    /// Get the order in which plugins were registered
    pub fn plugin_order(&self) -> &[String] {
        &self.plugin_order
    }

    /// Check if a component type has been registered
    pub fn has_component<C: Component>(&self) -> bool {
        self.registered_components.contains(&TypeId::of::<C>())
    }

    /// Check if a resource type has been registered
    pub fn has_resource<R: Resource>(&self) -> bool {
        self.registered_resources.contains(&TypeId::of::<R>())
    }

    /// Get the number of systems registered for a stage
    pub fn system_count(&self, stage: CoreStage) -> usize {
        self.systems_by_stage.get(&stage).map_or(0, |v| v.len())
    }

    /// Get the number of startup systems
    pub fn startup_system_count(&self) -> usize {
        self.startup_systems.len()
    }

    /// Get all registered component type IDs
    pub fn registered_components(&self) -> &HashSet<TypeId> {
        &self.registered_components
    }

    /// Get all registered resource type IDs
    pub fn registered_resources(&self) -> &HashSet<TypeId> {
        &self.registered_resources
    }

    /// Validate plugin dependencies
    pub fn validate_plugin_dependencies(&self, plugin: &dyn Plugin) -> Result<(), PluginError> {
        let dependencies = plugin.dependencies();
        let mut missing = Vec::new();

        for dep in dependencies {
            if !self.registered_plugins.contains(&dep.name) {
                missing.push(dep.clone());
            } else if let Some(required_version) = &dep.version {
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
    fn version_satisfies(found: &str, required: &str) -> bool {
        if required.starts_with(">=") {
            let required_ver = &required[2..];
            Self::compare_versions(found, required_ver) >= 0
        } else if required.starts_with('^') {
            let required_ver = &required[1..];
            let found_parts: Vec<&str> = found.split('.').collect();
            let required_parts: Vec<&str> = required_ver.split('.').collect();
            
            if found_parts.is_empty() || required_parts.is_empty() {
                return false;
            }
            
            found_parts[0] == required_parts[0] && Self::compare_versions(found, required_ver) >= 0
        } else {
            found == required
        }
    }

    /// Compare two version strings
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

    /// Convert to a real App for integration testing
    pub fn into_app(self) -> App {
        App::new()
    }
}

impl AppInterface for MockApp {
    fn add_plugins(&mut self, plugin: impl Plugin) -> &mut Self {
        let plugin_name = plugin.name().to_string();

        if !self.registered_plugins.contains(&plugin_name) {
            // Validate dependencies
            if let Err(e) = self.validate_plugin_dependencies(&plugin) {
                eprintln!("Error loading plugin '{}': {}", plugin_name, e);
                return self;
            }

            self.registered_plugins.insert(plugin_name.clone());
            self.plugin_order.push(plugin_name.clone());
            self.plugin_versions.insert(plugin_name, plugin.version().to_string());
            
            // Build plugin with mock app
            // Note: This requires converting MockApp to App temporarily
            // For now, we just track the registration
        }

        self
    }

    fn add_system<Marker>(
        &mut self,
        stage: CoreStage,
        _system: impl IntoSystem<Marker>,
    ) -> &mut Self {
        let count = self.systems_by_stage.get(&stage).map_or(0, |v| v.len());
        self.systems_by_stage
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(format!("system_{}", count));
        self
    }

    fn add_startup_system<Marker>(&mut self, _system: impl IntoSystem<Marker>) -> &mut Self {
        self.startup_systems.push(format!("startup_system_{}", self.startup_systems.len()));
        self
    }

    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.registered_resources.insert(TypeId::of::<R>());
        self.world.insert_resource(resource);
        self
    }

    fn register_component<C: Component>(&mut self) -> &mut Self {
        self.registered_components.insert(TypeId::of::<C>());
        self.world.register_component::<C>();
        self
    }

    fn run(self) {
        // Mock implementation - does nothing
    }
}

/// Plugin test context with utilities for testing
///
/// Provides a high-level interface for testing plugins with common
/// assertions and utilities.
pub struct PluginTestContext {
    app: MockApp,
}

impl Default for PluginTestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginTestContext {
    /// Create a new plugin test context
    pub fn new() -> Self {
        Self {
            app: MockApp::new(),
        }
    }

    /// Add a plugin to the test context
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        self.app.add_plugins(plugin);
        self
    }

    /// Add a plugin with dependency validation
    pub fn add_plugin_with_validation(&mut self, plugin: impl Plugin) -> Result<&mut Self, PluginError> {
        self.app.validate_plugin_dependencies(&plugin)?;
        self.app.add_plugins(plugin);
        Ok(self)
    }

    /// Check if a plugin has been registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.app.has_plugin(name)
    }

    /// Get the plugin registration order
    pub fn plugin_order(&self) -> &[String] {
        self.app.plugin_order()
    }

    /// Check if a component type has been registered
    pub fn has_component<C: Component>(&self) -> bool {
        self.app.has_component::<C>()
    }

    /// Check if a resource type has been registered
    pub fn has_resource<R: Resource>(&self) -> bool {
        self.app.has_resource::<R>()
    }

    /// Get the number of systems registered for a stage
    pub fn system_count(&self, stage: CoreStage) -> usize {
        self.app.system_count(stage)
    }

    /// Get the number of startup systems
    pub fn startup_system_count(&self) -> usize {
        self.app.startup_system_count()
    }

    /// Get access to the mock world
    pub fn world(&self) -> &World {
        &self.app.world
    }

    /// Get mutable access to the mock world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.app.world
    }

    /// Assert that a plugin was registered
    pub fn assert_plugin_registered(&self, name: &str) {
        assert!(
            self.has_plugin(name),
            "Expected plugin '{}' to be registered",
            name
        );
    }

    /// Assert that a component type was registered
    pub fn assert_component_registered<C: Component>(&self) {
        assert!(
            self.has_component::<C>(),
            "Expected component '{}' to be registered",
            std::any::type_name::<C>()
        );
    }

    /// Assert that a resource type was registered
    pub fn assert_resource_registered<R: Resource>(&self) {
        assert!(
            self.has_resource::<R>(),
            "Expected resource '{}' to be registered",
            std::any::type_name::<R>()
        );
    }

    /// Assert plugin registration order
    pub fn assert_plugin_order(&self, expected: &[&str]) {
        let actual = self.plugin_order();
        assert_eq!(
            actual.len(),
            expected.len(),
            "Expected {} plugins, found {}",
            expected.len(),
            actual.len()
        );
        for (i, (actual_name, expected_name)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                actual_name, expected_name,
                "Plugin at index {} should be '{}', found '{}'",
                i, expected_name, actual_name
            );
        }
    }

    /// Assert that systems were registered for a stage
    pub fn assert_systems_registered(&self, stage: CoreStage, expected_count: usize) {
        let actual_count = self.system_count(stage);
        assert_eq!(
            actual_count, expected_count,
            "Expected {} systems in stage {:?}, found {}",
            expected_count, stage, actual_count
        );
    }

    /// Get the underlying MockApp
    pub fn app(&self) -> &MockApp {
        &self.app
    }

    /// Get mutable access to the underlying MockApp
    pub fn app_mut(&mut self) -> &mut MockApp {
        &mut self.app
    }
}

/// Builder for creating mock plugins for testing
pub struct MockPluginBuilder {
    name: String,
    version: String,
    dependencies: Vec<PluginDependency>,
    build_fn: Option<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl MockPluginBuilder {
    /// Create a new mock plugin builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            dependencies: Vec::new(),
            build_fn: None,
        }
    }

    /// Set the plugin version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add a dependency
    pub fn dependency(mut self, dep: PluginDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add a dependency by name
    pub fn depends_on(mut self, name: impl Into<String>) -> Self {
        self.dependencies.push(PluginDependency::new(name));
        self
    }

    /// Add a dependency with version constraint
    pub fn depends_on_version(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies.push(PluginDependency::with_version(name, version));
        self
    }

    /// Set the build function
    pub fn build_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut App) + Send + Sync + 'static,
    {
        self.build_fn = Some(Box::new(f));
        self
    }

    /// Build the mock plugin
    pub fn build(self) -> MockPlugin {
        MockPlugin {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies,
            build_fn: self.build_fn,
        }
    }
}

/// Mock plugin for testing
pub struct MockPlugin {
    name: String,
    version: String,
    dependencies: Vec<PluginDependency>,
    build_fn: Option<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl Plugin for MockPlugin {
    fn build(&self, app: &mut App) {
        if let Some(ref f) = self.build_fn {
            f(app);
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        self.dependencies.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_app_tracks_plugins() {
        let mut app = MockApp::new();
        let plugin = MockPluginBuilder::new("test_plugin").build();
        
        app.add_plugins(plugin);
        
        assert!(app.has_plugin("test_plugin"));
        assert_eq!(app.plugin_order(), &["test_plugin"]);
    }

    #[test]
    fn test_plugin_test_context() {
        let mut ctx = PluginTestContext::new();
        let plugin = MockPluginBuilder::new("test_plugin").build();
        
        ctx.add_plugin(plugin);
        
        ctx.assert_plugin_registered("test_plugin");
        ctx.assert_plugin_order(&["test_plugin"]);
    }

    #[test]
    fn test_mock_plugin_with_dependencies() {
        let mut ctx = PluginTestContext::new();
        
        let base_plugin = MockPluginBuilder::new("base").build();
        let dependent_plugin = MockPluginBuilder::new("dependent")
            .depends_on("base")
            .build();
        
        ctx.add_plugin(base_plugin);
        ctx.add_plugin(dependent_plugin);
        
        ctx.assert_plugin_order(&["base", "dependent"]);
    }
}
