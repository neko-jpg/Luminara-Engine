use luminara_core::{App, AppInterface, Plugin};
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// Property Test 26: Plugin Execution Order
// Validates: Requirements 9.4
// ============================================================================

/// Test plugin that tracks when build() is called and in what order
#[derive(Clone)]
struct OrderTrackingPlugin {
    name: String,
    build_tracker: Arc<Mutex<Vec<String>>>,
}

impl OrderTrackingPlugin {
    fn new(name: &str, tracker: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.to_string(),
            build_tracker: tracker,
        }
    }
}

impl Plugin for OrderTrackingPlugin {
    fn build(&self, _app: &mut App) {
        self.build_tracker.lock().unwrap().push(self.name.clone());
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Strategy for generating plugin names
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{3,20}Plugin").unwrap()
}

/// Strategy for generating a list of unique plugin names
fn plugin_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(plugin_name_strategy(), 2..10)
        .prop_map(|names| {
            // Ensure uniqueness
            let mut unique_names = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for name in names {
                if seen.insert(name.clone()) {
                    unique_names.push(name);
                }
            }
            unique_names
        })
        .prop_filter("Must have at least two unique names", |names| {
            names.len() >= 2
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 26: Plugin Execution Order**
    ///
    /// For any sequence of plugins registered with the engine, the plugins should
    /// be built in the order they were registered.
    ///
    /// **Validates: Requirements 9.4**
    #[test]
    fn prop_plugin_execution_order(plugin_names in plugin_names_strategy()) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();

        // Register all plugins in sequence
        for name in &plugin_names {
            let plugin = OrderTrackingPlugin::new(name, tracker.clone());
            app.add_plugins(plugin);
        }

        // Verify build was called in the same order as registration
        let build_order = tracker.lock().unwrap();

        prop_assert_eq!(
            build_order.len(),
            plugin_names.len(),
            "Number of build calls should match number of registered plugins"
        );

        // Verify exact order match
        for (i, expected_name) in plugin_names.iter().enumerate() {
            prop_assert_eq!(
                &build_order[i],
                expected_name,
                "Plugin at position {} should be '{}', but was '{}'",
                i,
                expected_name,
                build_order[i]
            );
        }

        // Also verify using App's plugin_order() method
        let app_plugin_order = app.plugin_order();
        prop_assert_eq!(
            app_plugin_order.len(),
            plugin_names.len(),
            "App's plugin_order should match number of registered plugins"
        );

        for (i, expected_name) in plugin_names.iter().enumerate() {
            prop_assert_eq!(
                &app_plugin_order[i],
                expected_name,
                "App's plugin_order at position {} should be '{}'",
                i,
                expected_name
            );
        }
    }

    /// **Property 26 (variant): Plugin Order Preserved with Interleaved Operations**
    ///
    /// For any sequence of plugins registered with other operations (like adding
    /// systems or resources) interleaved, the plugin build order should still
    /// match the registration order.
    ///
    /// **Validates: Requirements 9.4**
    #[test]
    fn prop_plugin_order_with_interleaved_operations(
        plugin_names in plugin_names_strategy()
    ) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();

        // Register plugins with interleaved operations
        for (i, name) in plugin_names.iter().enumerate() {
            let plugin = OrderTrackingPlugin::new(name, tracker.clone());
            app.add_plugins(plugin);

            // Interleave with other operations
            if i % 2 == 0 {
                // Add a dummy system
                fn dummy_system(_world: &mut luminara_core::world::World) {}
                app.add_system::<luminara_core::system::ExclusiveMarker>(
                    luminara_core::shared_types::CoreStage::Update,
                    dummy_system
                );
            }
        }

        // Verify build order still matches registration order
        let build_order = tracker.lock().unwrap();

        prop_assert_eq!(
            build_order.len(),
            plugin_names.len(),
            "Build order length should match plugin count despite interleaved operations"
        );

        for (i, expected_name) in plugin_names.iter().enumerate() {
            prop_assert_eq!(
                &build_order[i],
                expected_name,
                "Plugin build order should be preserved with interleaved operations"
            );
        }
    }

    /// **Property 26 (variant): Large Number of Plugins Maintain Order**
    ///
    /// For any large sequence of plugins (up to 50), the build order should
    /// still match the registration order.
    ///
    /// **Validates: Requirements 9.4**
    #[test]
    fn prop_large_plugin_sequence_order(plugin_count in 10usize..50) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();

        // Generate plugin names
        let plugin_names: Vec<String> = (0..plugin_count)
            .map(|i| format!("Plugin{:03}", i))
            .collect();

        // Register all plugins
        for name in &plugin_names {
            let plugin = OrderTrackingPlugin::new(name, tracker.clone());
            app.add_plugins(plugin);
        }

        // Verify order is maintained
        let build_order = tracker.lock().unwrap();

        prop_assert_eq!(
            build_order.len(),
            plugin_count,
            "All plugins should be built"
        );

        for (i, expected_name) in plugin_names.iter().enumerate() {
            prop_assert_eq!(
                &build_order[i],
                expected_name,
                "Plugin order should be maintained even with {} plugins",
                plugin_count
            );
        }
    }
}

// Additional edge case tests

#[test]
fn test_two_plugins_execution_order() {
    // **Validates: Requirements 9.4**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = OrderTrackingPlugin::new("FirstPlugin", tracker.clone());
    let plugin2 = OrderTrackingPlugin::new("SecondPlugin", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);

    let build_order = tracker.lock().unwrap();
    assert_eq!(build_order.len(), 2, "Both plugins should be built");
    assert_eq!(
        build_order[0], "FirstPlugin",
        "First plugin should be built first"
    );
    assert_eq!(
        build_order[1], "SecondPlugin",
        "Second plugin should be built second"
    );
}

#[test]
fn test_three_plugins_execution_order() {
    // **Validates: Requirements 9.4**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = OrderTrackingPlugin::new("PluginA", tracker.clone());
    let plugin2 = OrderTrackingPlugin::new("PluginB", tracker.clone());
    let plugin3 = OrderTrackingPlugin::new("PluginC", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);
    app.add_plugins(plugin3);

    let build_order = tracker.lock().unwrap();
    assert_eq!(build_order.len(), 3);
    assert_eq!(build_order[0], "PluginA");
    assert_eq!(build_order[1], "PluginB");
    assert_eq!(build_order[2], "PluginC");
}

#[test]
fn test_plugin_order_matches_app_plugin_order() {
    // **Validates: Requirements 9.4**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin_names = vec!["Plugin1", "Plugin2", "Plugin3", "Plugin4"];

    for name in &plugin_names {
        let plugin = OrderTrackingPlugin::new(name, tracker.clone());
        app.add_plugins(plugin);
    }

    // Verify build order matches
    let build_order = tracker.lock().unwrap();
    assert_eq!(build_order.len(), plugin_names.len());

    // Verify App's plugin_order() matches
    let app_order = app.plugin_order();
    assert_eq!(app_order.len(), plugin_names.len());

    for (i, name) in plugin_names.iter().enumerate() {
        assert_eq!(
            &build_order[i], name,
            "Build order should match at position {}",
            i
        );
        assert_eq!(
            &app_order[i], name,
            "App plugin_order should match at position {}",
            i
        );
    }
}

#[test]
fn test_plugin_order_with_duplicate_registration() {
    // **Validates: Requirements 9.4**
    // When a plugin is registered twice, it should only be built once (first time)
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin1 = OrderTrackingPlugin::new("Plugin1", tracker.clone());
    let plugin2 = OrderTrackingPlugin::new("Plugin2", tracker.clone());
    let plugin1_duplicate = OrderTrackingPlugin::new("Plugin1", tracker.clone());
    let plugin3 = OrderTrackingPlugin::new("Plugin3", tracker.clone());

    app.add_plugins(plugin1);
    app.add_plugins(plugin2);
    app.add_plugins(plugin1_duplicate); // Should be ignored
    app.add_plugins(plugin3);

    let build_order = tracker.lock().unwrap();
    assert_eq!(
        build_order.len(),
        3,
        "Duplicate plugin should not be built again"
    );
    assert_eq!(build_order[0], "Plugin1");
    assert_eq!(build_order[1], "Plugin2");
    assert_eq!(build_order[2], "Plugin3");

    // Verify App's plugin_order also reflects this
    let app_order = app.plugin_order();
    assert_eq!(app_order.len(), 3);
    assert_eq!(app_order[0], "Plugin1");
    assert_eq!(app_order[1], "Plugin2");
    assert_eq!(app_order[2], "Plugin3");
}

#[test]
fn test_plugin_order_with_many_plugins() {
    // **Validates: Requirements 9.4**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin_count = 20;
    let plugin_names: Vec<String> = (0..plugin_count)
        .map(|i| format!("Plugin{:02}", i))
        .collect();

    for name in &plugin_names {
        let plugin = OrderTrackingPlugin::new(name, tracker.clone());
        app.add_plugins(plugin);
    }

    let build_order = tracker.lock().unwrap();
    assert_eq!(build_order.len(), plugin_count);

    for (i, expected_name) in plugin_names.iter().enumerate() {
        assert_eq!(
            &build_order[i], expected_name,
            "Plugin order should be maintained at position {}",
            i
        );
    }
}

#[test]
fn test_plugin_order_preserved_across_different_operations() {
    // **Validates: Requirements 9.4**
    use luminara_core::Resource;

    #[derive(Debug, Clone)]
    struct TestResource {
        value: i32,
    }

    impl Resource for TestResource {}

    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    // Mix plugin registration with other operations
    let plugin1 = OrderTrackingPlugin::new("Plugin1", tracker.clone());
    app.add_plugins(plugin1);

    app.insert_resource(TestResource { value: 1 });

    let plugin2 = OrderTrackingPlugin::new("Plugin2", tracker.clone());
    app.add_plugins(plugin2);

    fn dummy_system(_world: &mut luminara_core::world::World) {}
    app.add_system::<luminara_core::system::ExclusiveMarker>(
        luminara_core::shared_types::CoreStage::Update,
        dummy_system,
    );

    let plugin3 = OrderTrackingPlugin::new("Plugin3", tracker.clone());
    app.add_plugins(plugin3);

    let build_order = tracker.lock().unwrap();
    assert_eq!(build_order.len(), 3);
    assert_eq!(build_order[0], "Plugin1");
    assert_eq!(build_order[1], "Plugin2");
    assert_eq!(build_order[2], "Plugin3");
}
