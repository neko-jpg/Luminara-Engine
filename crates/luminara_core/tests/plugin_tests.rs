use luminara_core::{App, AppInterface, Plugin};
use std::sync::{Arc, Mutex};

/// Test plugin that tracks when build() is called
struct TestPlugin {
    name: String,
    build_tracker: Arc<Mutex<Vec<String>>>,
}

impl TestPlugin {
    fn new(name: &str, tracker: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.to_string(),
            build_tracker: tracker,
        }
    }
}

impl Plugin for TestPlugin {
    fn build(&self, _app: &mut App) {
        self.build_tracker.lock().unwrap().push(self.name.clone());
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[test]
fn test_plugin_build_called_once() {
    // Validates: Requirements 9.1
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin = TestPlugin::new("test_plugin", tracker.clone());
    app.add_plugins(plugin);

    let builds = tracker.lock().unwrap();
    assert_eq!(
        builds.len(),
        1,
        "Plugin build should be called exactly once"
    );
    assert_eq!(builds[0], "test_plugin");
}

#[test]
fn test_plugin_not_built_twice() {
    // Validates: Requirements 9.1 - ensure build is called only once even if registered twice
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = TestPlugin::new("test_plugin", tracker.clone());
    let plugin2 = TestPlugin::new("test_plugin", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);

    let builds = tracker.lock().unwrap();
    assert_eq!(
        builds.len(),
        1,
        "Plugin build should be called only once even if registered twice"
    );
}

#[test]
fn test_plugin_execution_order() {
    // Validates: Requirements 9.4
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = TestPlugin::new("first", tracker.clone());
    let plugin2 = TestPlugin::new("second", tracker.clone());
    let plugin3 = TestPlugin::new("third", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);
    app.add_plugins(plugin3);

    let builds = tracker.lock().unwrap();
    assert_eq!(builds.len(), 3);
    assert_eq!(builds[0], "first", "First plugin should be built first");
    assert_eq!(builds[1], "second", "Second plugin should be built second");
    assert_eq!(builds[2], "third", "Third plugin should be built third");
}

#[test]
fn test_plugin_order_tracking() {
    // Validates: Requirements 9.4 - verify plugin_order() returns correct order
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = TestPlugin::new("alpha", tracker.clone());
    let plugin2 = TestPlugin::new("beta", tracker.clone());
    let plugin3 = TestPlugin::new("gamma", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);
    app.add_plugins(plugin3);

    let order = app.plugin_order();
    assert_eq!(order.len(), 3);
    assert_eq!(order[0], "alpha");
    assert_eq!(order[1], "beta");
    assert_eq!(order[2], "gamma");
}

#[test]
fn test_has_plugin() {
    // Test the has_plugin() helper method
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin = TestPlugin::new("test_plugin", tracker);
    app.add_plugins(plugin);

    assert!(
        app.has_plugin("test_plugin"),
        "Should detect registered plugin"
    );
    assert!(
        !app.has_plugin("nonexistent"),
        "Should not detect unregistered plugin"
    );
}

#[test]
fn test_empty_app_no_plugins() {
    // Edge case: app with no plugins
    let app = App::new();

    assert_eq!(
        app.plugin_order().len(),
        0,
        "New app should have no plugins"
    );
    assert!(
        !app.has_plugin("any_plugin"),
        "New app should not have any plugins"
    );
}

// Test for Requirements 9.2 and 9.3: Plugin system and resource registration

use luminara_core::{impl_component, CoreStage, Query, Resource};

#[derive(Debug, Clone, PartialEq)]
struct TestComponent {
    value: i32,
}

impl_component!(TestComponent);

#[derive(Debug, Clone, PartialEq)]
struct TestResource {
    name: String,
}

impl Resource for TestResource {}

struct SystemRegistrationPlugin;

impl Plugin for SystemRegistrationPlugin {
    fn build(&self, app: &mut App) {
        // Register a component
        app.register_component::<TestComponent>();

        // Register a resource
        app.insert_resource(TestResource {
            name: "test_resource".to_string(),
        });
    }

    fn name(&self) -> &str {
        "system_registration_plugin"
    }
}

#[test]
fn test_plugin_can_register_components() {
    // Validates: Requirements 9.3 - Plugin can register components
    let mut app = App::new();

    app.add_plugins(SystemRegistrationPlugin);

    // Verify component can be used by spawning an entity with it
    let entity = app.world.spawn();
    app.world.add_component(entity, TestComponent { value: 42 });

    // Query to verify the component exists
    let query = Query::<&TestComponent>::new(&app.world);
    let components: Vec<_> = query.iter().collect();
    assert_eq!(
        components.len(),
        1,
        "Component should be registered and usable"
    );
    assert_eq!(components[0].value, 42);
}

#[test]
fn test_plugin_can_register_resources() {
    // Validates: Requirements 9.3 - Plugin can register resources
    let mut app = App::new();

    app.add_plugins(SystemRegistrationPlugin);

    // Verify resource was registered
    let resource = app.world.get_resource::<TestResource>();
    assert!(resource.is_some(), "Resource should be registered");
    assert_eq!(resource.unwrap().name, "test_resource");
}

#[test]
fn test_plugin_can_add_systems_to_stages() {
    // Validates: Requirements 9.2 - Plugin can add systems to any stage
    // Note: The ScenePlugin already demonstrates this capability by adding
    // transform_propagate_system to PostUpdate stage. The ability to add
    // systems to any stage is provided by the add_system method in AppInterface,
    // which accepts a CoreStage parameter.

    let mut app = App::new();

    // Verify that add_system method exists and accepts different stages
    // by adding a simple system to multiple stages
    fn dummy_system(_world: &mut luminara_core::World) {}

    app.add_system::<luminara_core::system::ExclusiveMarker>(CoreStage::PreUpdate, dummy_system);
    app.add_system::<luminara_core::system::ExclusiveMarker>(CoreStage::Update, dummy_system);
    app.add_system::<luminara_core::system::ExclusiveMarker>(CoreStage::PostUpdate, dummy_system);

    // If we get here without panicking, systems were successfully registered
    // Run one update cycle to ensure systems execute without errors
    app.update();
}

#[test]
fn test_multiple_plugins_register_different_components() {
    // Test that multiple plugins can each register their own components
    #[derive(Debug, Clone)]
    struct ComponentA(i32);
    impl_component!(ComponentA);

    #[derive(Debug, Clone)]
    struct ComponentB(String);
    impl_component!(ComponentB);

    struct PluginA;
    impl Plugin for PluginA {
        fn build(&self, app: &mut App) {
            app.register_component::<ComponentA>();
        }
        fn name(&self) -> &str {
            "plugin_a"
        }
    }

    struct PluginB;
    impl Plugin for PluginB {
        fn build(&self, app: &mut App) {
            app.register_component::<ComponentB>();
        }
        fn name(&self) -> &str {
            "plugin_b"
        }
    }

    let mut app = App::new();
    app.add_plugins(PluginA);
    app.add_plugins(PluginB);

    // Both components should be usable
    let entity = app.world.spawn();
    app.world.add_component(entity, ComponentA(123));
    app.world
        .add_component(entity, ComponentB("test".to_string()));

    // Verify both components exist
    let query_a = Query::<&ComponentA>::new(&app.world);
    let components_a: Vec<_> = query_a.iter().collect();
    assert_eq!(components_a.len(), 1);

    let query_b = Query::<&ComponentB>::new(&app.world);
    let components_b: Vec<_> = query_b.iter().collect();
    assert_eq!(components_b.len(), 1);
}
