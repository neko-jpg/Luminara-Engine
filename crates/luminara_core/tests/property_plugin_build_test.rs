use luminara_core::{App, AppInterface, Plugin};
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// Property Test 23: Plugin Build Invocation
// Validates: Requirements 9.1
// ============================================================================

/// Test plugin that tracks when build() is called
#[derive(Clone)]
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

/// Strategy for generating plugin names
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{3,20}Plugin").unwrap()
}

/// Strategy for generating a list of unique plugin names
fn plugin_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(plugin_name_strategy(), 1..10)
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
        .prop_filter("Must have at least one unique name", |names| {
            !names.is_empty()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 23: Plugin Build Invocation**
    ///
    /// For any plugin registered with the engine, the engine should call the
    /// plugin's build method exactly once during initialization.
    ///
    /// **Validates: Requirements 9.1**
    #[test]
    fn prop_plugin_build_invocation(plugin_names in plugin_names_strategy()) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();

        // Register all plugins
        for name in &plugin_names {
            let plugin = TestPlugin::new(name, tracker.clone());
            app.add_plugins(plugin);
        }

        // Verify build was called exactly once for each plugin
        let builds = tracker.lock().unwrap();
        prop_assert_eq!(
            builds.len(),
            plugin_names.len(),
            "Each plugin's build method should be called exactly once"
        );

        // Verify each plugin name appears exactly once in the build tracker
        for name in &plugin_names {
            let count = builds.iter().filter(|&n| n == name).count();
            prop_assert_eq!(
                count,
                1,
                "Plugin '{}' build should be called exactly once, but was called {} times",
                name,
                count
            );
        }

        // Verify the order matches registration order
        for (i, name) in plugin_names.iter().enumerate() {
            prop_assert_eq!(
                &builds[i],
                name,
                "Plugin build order should match registration order"
            );
        }
    }

    /// **Property 23 (variant): Plugin Build Not Called Twice**
    ///
    /// For any plugin registered multiple times with the same name, the engine
    /// should call the plugin's build method only once (idempotency).
    ///
    /// **Validates: Requirements 9.1**
    #[test]
    fn prop_plugin_build_not_called_twice(
        plugin_name in plugin_name_strategy(),
        registration_count in 2usize..5
    ) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();

        // Register the same plugin multiple times
        for _ in 0..registration_count {
            let plugin = TestPlugin::new(&plugin_name, tracker.clone());
            app.add_plugins(plugin);
        }

        // Verify build was called only once despite multiple registrations
        let builds = tracker.lock().unwrap();
        prop_assert_eq!(
            builds.len(),
            1,
            "Plugin build should be called only once even when registered {} times",
            registration_count
        );

        prop_assert_eq!(
            &builds[0],
            &plugin_name,
            "The single build call should be for the correct plugin"
        );
    }

    /// **Property 23 (variant): Empty App Has No Plugin Builds**
    ///
    /// For any app with no plugins registered, no build methods should be called.
    ///
    /// **Validates: Requirements 9.1**
    #[test]
    fn prop_empty_app_no_builds(_dummy in 0..10) {
        let app = App::new();

        // Verify app has no plugins registered
        prop_assert_eq!(
            app.plugin_order().len(),
            0,
            "Empty app should have no plugins in its order list"
        );
    }
}

// Additional edge case tests

#[test]
fn test_single_plugin_build_invocation() {
    // **Validates: Requirements 9.1**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin = TestPlugin::new("SinglePlugin", tracker.clone());
    app.add_plugins(plugin);

    let builds = tracker.lock().unwrap();
    assert_eq!(
        builds.len(),
        1,
        "Single plugin should have build called once"
    );
    assert_eq!(builds[0], "SinglePlugin");
}

#[test]
fn test_many_plugins_build_invocation() {
    // **Validates: Requirements 9.1**
    // Test with a larger number of plugins to ensure scalability
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();

    let plugin_count = 50;
    let plugin_names: Vec<String> = (0..plugin_count).map(|i| format!("Plugin{}", i)).collect();

    for name in &plugin_names {
        let plugin = TestPlugin::new(name, tracker.clone());
        app.add_plugins(plugin);
    }

    let builds = tracker.lock().unwrap();
    assert_eq!(
        builds.len(),
        plugin_count,
        "All {} plugins should have build called exactly once",
        plugin_count
    );

    // Verify order
    for (i, name) in plugin_names.iter().enumerate() {
        assert_eq!(
            &builds[i], name,
            "Plugin build order should match registration order"
        );
    }
}

#[test]
fn test_plugin_build_with_app_state() {
    // **Validates: Requirements 9.1**
    // Verify that plugin build can interact with app state
    use luminara_core::Resource;

    #[derive(Debug, Clone, PartialEq)]
    struct BuildCounter {
        count: usize,
    }

    impl Resource for BuildCounter {}

    struct CountingPlugin {
        name: String,
    }

    impl Plugin for CountingPlugin {
        fn build(&self, app: &mut App) {
            // Increment counter in app resources
            let mut exists = false;
            if let Some(mut counter) = app.world.get_resource_mut::<BuildCounter>() {
                counter.count += 1;
                exists = true;
            }

            if !exists {
                app.insert_resource(BuildCounter { count: 1 });
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    let mut app = App::new();

    // Register multiple plugins
    app.add_plugins(CountingPlugin {
        name: "Plugin1".to_string(),
    });
    app.add_plugins(CountingPlugin {
        name: "Plugin2".to_string(),
    });
    app.add_plugins(CountingPlugin {
        name: "Plugin3".to_string(),
    });

    // Verify counter was incremented for each plugin
    let counter = app.world.get_resource::<BuildCounter>().unwrap();
    assert_eq!(
        counter.count, 3,
        "Build should be called once for each plugin"
    );
}
