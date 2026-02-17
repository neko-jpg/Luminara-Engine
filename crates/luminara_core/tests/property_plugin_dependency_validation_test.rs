use luminara_core::{App, AppInterface, Plugin, PluginDependency};
use proptest::prelude::*;

// ============================================================================
// Property Test 18: Plugin Dependency Validation
// Validates: Requirements 16.1
// ============================================================================

/// Test plugin that can have configurable dependencies
#[derive(Clone, Debug)]
struct TestPlugin {
    name: String,
    version: String,
    dependencies: Vec<PluginDependency>,
}

impl TestPlugin {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            dependencies: Vec::new(),
        }
    }

    fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    fn with_dependency(mut self, dep: PluginDependency) -> Self {
        self.dependencies.push(dep);
        self
    }
}

impl Plugin for TestPlugin {
    fn build(&self, _app: &mut App) {}

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

/// Strategy for generating valid plugin names
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{2,15}_plugin").unwrap()
}

/// Strategy for generating version strings
fn version_strategy() -> impl Strategy<Value = String> {
    (0u32..5, 0u32..10, 0u32..20).prop_map(|(major, minor, patch)| {
        format!("{}.{}.{}", major, minor, patch)
    })
}

/// Strategy for generating version constraints
fn version_constraint_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        // No version constraint
        Just(None),
        // Exact version
        version_strategy().prop_map(Some),
        // >= constraint
        version_strategy().prop_map(|v| Some(format!(">={}", v))),
        // ^ constraint (caret)
        (0u32..5, 0u32..10).prop_map(|(major, minor)| Some(format!("^{}.{}", major, minor))),
    ]
}

/// Strategy for generating a plugin with random dependencies
fn plugin_with_deps_strategy(
    available_plugins: Vec<String>,
) -> impl Strategy<Value = TestPlugin> {
    let name_strat = plugin_name_strategy();
    let version_strat = version_strategy();
    
    (name_strat, version_strat, prop::collection::vec(
        (
            prop::sample::select(available_plugins.clone()),
            version_constraint_strategy(),
        ),
        0..3, // 0-3 dependencies per plugin
    ))
    .prop_map(|(name, version, deps)| {
        let mut plugin = TestPlugin::new(name).with_version(version);
        for (dep_name, constraint) in deps {
            let dep = if let Some(constraint) = constraint {
                PluginDependency::with_version(dep_name, constraint)
            } else {
                PluginDependency::new(dep_name)
            };
            plugin = plugin.with_dependency(dep);
        }
        plugin
    })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 18: Plugin Dependency Validation**
    /// **Validates: Requirements 16.1**
    ///
    /// For any plugin with unsatisfied dependencies, the plugin system should:
    /// 1. Fail to load the plugin
    /// 2. Provide a clear error listing missing dependencies
    /// 3. Not crash or panic
    /// 4. Allow other plugins to continue loading
    #[test]
    fn property_plugin_with_unsatisfied_dependencies_fails_to_load(
        plugin_name in plugin_name_strategy(),
        dependency_name in plugin_name_strategy(),
        version_constraint in version_constraint_strategy(),
    ) {
        // Ensure plugin and dependency have different names
        prop_assume!(plugin_name != dependency_name);

        let mut app = App::new();

        // Create a plugin with a dependency that is NOT loaded
        let dep = if let Some(constraint) = version_constraint {
            PluginDependency::with_version(&dependency_name, constraint)
        } else {
            PluginDependency::new(&dependency_name)
        };

        let plugin = TestPlugin::new(&plugin_name).with_dependency(dep);

        // Try to load the plugin without loading its dependency
        app.add_plugins(plugin);

        // Property: Plugin should NOT be loaded due to missing dependency
        prop_assert!(!app.has_plugin(&plugin_name), 
            "Plugin '{}' should not be loaded when dependency '{}' is missing", 
            plugin_name, dependency_name);
    }

    /// **Property 18: Plugin Dependency Validation - Satisfied Dependencies**
    /// **Validates: Requirements 16.1**
    ///
    /// For any plugin with satisfied dependencies, the plugin system should:
    /// 1. Successfully load the plugin
    /// 2. Load plugins in the correct order (dependencies first)
    #[test]
    fn property_plugin_with_satisfied_dependencies_loads_successfully(
        plugin_name in plugin_name_strategy(),
        dependency_name in plugin_name_strategy(),
        plugin_version in version_strategy(),
        dep_version in version_strategy(),
    ) {
        // Ensure plugin and dependency have different names
        prop_assume!(plugin_name != dependency_name);

        let mut app = App::new();

        // Load the dependency first
        let dependency = TestPlugin::new(&dependency_name).with_version(&dep_version);
        app.add_plugins(dependency);

        // Create a plugin that depends on the loaded plugin (no version constraint)
        let plugin = TestPlugin::new(&plugin_name)
            .with_version(&plugin_version)
            .with_dependency(PluginDependency::new(&dependency_name));

        // Load the plugin
        app.add_plugins(plugin);

        // Property: Both plugins should be loaded
        prop_assert!(app.has_plugin(&dependency_name), 
            "Dependency '{}' should be loaded", dependency_name);
        prop_assert!(app.has_plugin(&plugin_name), 
            "Plugin '{}' should be loaded when dependency is satisfied", plugin_name);

        // Property: Dependency should be loaded before dependent plugin
        let order = app.plugin_order();
        let dep_idx = order.iter().position(|n| n == &dependency_name);
        let plugin_idx = order.iter().position(|n| n == &plugin_name);
        
        prop_assert!(dep_idx.is_some() && plugin_idx.is_some());
        prop_assert!(dep_idx.unwrap() < plugin_idx.unwrap(),
            "Dependency '{}' should be loaded before plugin '{}'", 
            dependency_name, plugin_name);
    }

    /// **Property 18: Plugin Dependency Validation - Version Constraints**
    /// **Validates: Requirements 16.1**
    ///
    /// For plugins with version constraints:
    /// 1. Plugin loads if dependency version satisfies constraint
    /// 2. Plugin fails to load if dependency version doesn't satisfy constraint
    #[test]
    fn property_version_constraint_validation(
        plugin_name in plugin_name_strategy(),
        dependency_name in plugin_name_strategy(),
        dep_major in 1u32..5,
        dep_minor in 0u32..10,
        dep_patch in 0u32..20,
        required_major in 1u32..5,
        required_minor in 0u32..10,
    ) {
        // Ensure plugin and dependency have different names
        prop_assume!(plugin_name != dependency_name);

        let dep_version = format!("{}.{}.{}", dep_major, dep_minor, dep_patch);
        let required_version = format!(">={}.{}.0", required_major, required_minor);

        let mut app = App::new();

        // Load dependency with specific version
        let dependency = TestPlugin::new(&dependency_name).with_version(&dep_version);
        app.add_plugins(dependency);

        // Create plugin with version constraint
        let plugin = TestPlugin::new(&plugin_name)
            .with_dependency(PluginDependency::with_version(&dependency_name, &required_version));

        // Load the plugin
        app.add_plugins(plugin);

        // Determine if version should satisfy constraint
        let should_satisfy = dep_major > required_major || 
            (dep_major == required_major && dep_minor >= required_minor);

        // Property: Plugin loads if and only if version constraint is satisfied
        if should_satisfy {
            prop_assert!(app.has_plugin(&plugin_name),
                "Plugin '{}' should load when dependency version {} satisfies constraint {}",
                plugin_name, dep_version, required_version);
        } else {
            prop_assert!(!app.has_plugin(&plugin_name),
                "Plugin '{}' should NOT load when dependency version {} doesn't satisfy constraint {}",
                plugin_name, dep_version, required_version);
        }
    }

    /// **Property 18: Plugin Dependency Validation - Multiple Dependencies**
    /// **Validates: Requirements 16.1**
    ///
    /// For plugins with multiple dependencies:
    /// 1. Plugin loads only if ALL dependencies are satisfied
    /// 2. Plugin fails to load if ANY dependency is missing
    #[test]
    fn property_multiple_dependencies_all_or_nothing(
        plugin_name in plugin_name_strategy(),
        dep1_name in plugin_name_strategy(),
        dep2_name in plugin_name_strategy(),
        load_dep1 in prop::bool::ANY,
        load_dep2 in prop::bool::ANY,
    ) {
        // Ensure all names are unique
        prop_assume!(plugin_name != dep1_name);
        prop_assume!(plugin_name != dep2_name);
        prop_assume!(dep1_name != dep2_name);

        let mut app = App::new();

        // Conditionally load dependencies
        if load_dep1 {
            app.add_plugins(TestPlugin::new(&dep1_name));
        }
        if load_dep2 {
            app.add_plugins(TestPlugin::new(&dep2_name));
        }

        // Create plugin with two dependencies
        let plugin = TestPlugin::new(&plugin_name)
            .with_dependency(PluginDependency::new(&dep1_name))
            .with_dependency(PluginDependency::new(&dep2_name));

        app.add_plugins(plugin);

        // Property: Plugin loads if and only if BOTH dependencies are loaded
        let both_loaded = load_dep1 && load_dep2;
        prop_assert_eq!(app.has_plugin(&plugin_name), both_loaded,
            "Plugin '{}' should load only when both dependencies are satisfied (dep1: {}, dep2: {})",
            plugin_name, load_dep1, load_dep2);
    }

    /// **Property 18: Plugin Dependency Validation - No False Positives**
    /// **Validates: Requirements 16.1**
    ///
    /// Plugins without dependencies should always load successfully
    #[test]
    fn property_no_dependencies_always_loads(
        plugin_name in plugin_name_strategy(),
        version in version_strategy(),
    ) {
        let mut app = App::new();

        // Create plugin with no dependencies
        let plugin = TestPlugin::new(&plugin_name).with_version(&version);

        app.add_plugins(plugin);

        // Property: Plugin should always load when it has no dependencies
        prop_assert!(app.has_plugin(&plugin_name),
            "Plugin '{}' with no dependencies should always load", plugin_name);
    }

    /// **Property 18: Plugin Dependency Validation - Caret Constraint**
    /// **Validates: Requirements 16.1**
    ///
    /// Caret (^) constraints should match same major version
    #[test]
    fn property_caret_constraint_same_major_version(
        plugin_name in plugin_name_strategy(),
        dependency_name in plugin_name_strategy(),
        major in 1u32..5,
        dep_minor in 0u32..10,
        dep_patch in 0u32..20,
        req_minor in 0u32..10,
    ) {
        // Ensure plugin and dependency have different names
        prop_assume!(plugin_name != dependency_name);

        let dep_version = format!("{}.{}.{}", major, dep_minor, dep_patch);
        let required_version = format!("^{}.{}", major, req_minor);

        let mut app = App::new();

        // Load dependency
        let dependency = TestPlugin::new(&dependency_name).with_version(&dep_version);
        app.add_plugins(dependency);

        // Create plugin with caret constraint
        let plugin = TestPlugin::new(&plugin_name)
            .with_dependency(PluginDependency::with_version(&dependency_name, &required_version));

        app.add_plugins(plugin);

        // Property: Plugin loads if dependency version >= required version (same major)
        let should_load = dep_minor >= req_minor;
        prop_assert_eq!(app.has_plugin(&plugin_name), should_load,
            "Plugin '{}' with caret constraint {} should {} when dependency version is {}",
            plugin_name, required_version, 
            if should_load { "load" } else { "not load" },
            dep_version);
    }

    /// **Property 18: Plugin Dependency Validation - Error Isolation**
    /// **Validates: Requirements 16.1**
    ///
    /// When one plugin fails to load due to missing dependencies,
    /// other plugins should continue to load normally
    #[test]
    fn property_error_isolation_other_plugins_load(
        good_plugin_name in plugin_name_strategy(),
        bad_plugin_name in plugin_name_strategy(),
        missing_dep_name in plugin_name_strategy(),
    ) {
        // Ensure all names are unique
        prop_assume!(good_plugin_name != bad_plugin_name);
        prop_assume!(good_plugin_name != missing_dep_name);
        prop_assume!(bad_plugin_name != missing_dep_name);

        let mut app = App::new();

        // Load a good plugin with no dependencies
        let good_plugin = TestPlugin::new(&good_plugin_name);
        app.add_plugins(good_plugin);

        // Try to load a bad plugin with missing dependency
        let bad_plugin = TestPlugin::new(&bad_plugin_name)
            .with_dependency(PluginDependency::new(&missing_dep_name));
        app.add_plugins(bad_plugin);

        // Property: Good plugin should be loaded despite bad plugin failure
        prop_assert!(app.has_plugin(&good_plugin_name),
            "Plugin '{}' should load even when another plugin '{}' fails",
            good_plugin_name, bad_plugin_name);
        
        // Property: Bad plugin should not be loaded
        prop_assert!(!app.has_plugin(&bad_plugin_name),
            "Plugin '{}' should not load due to missing dependency '{}'",
            bad_plugin_name, missing_dep_name);
    }
}

// ============================================================================
// Additional Unit Tests for Edge Cases
// ============================================================================

#[test]
fn test_exact_version_match_property() {
    // Test exact version matching with property-based approach
    let mut app = App::new();
    
    let dep = TestPlugin::new("exact_dep").with_version("2.5.3");
    app.add_plugins(dep);
    
    let plugin = TestPlugin::new("exact_plugin")
        .with_dependency(PluginDependency::with_version("exact_dep", "2.5.3"));
    app.add_plugins(plugin);
    
    assert!(app.has_plugin("exact_dep"));
    assert!(app.has_plugin("exact_plugin"));
}

#[test]
fn test_exact_version_mismatch_property() {
    // Test exact version mismatch
    let mut app = App::new();
    
    let dep = TestPlugin::new("exact_dep").with_version("2.5.3");
    app.add_plugins(dep);
    
    let plugin = TestPlugin::new("exact_plugin")
        .with_dependency(PluginDependency::with_version("exact_dep", "2.5.4"));
    app.add_plugins(plugin);
    
    assert!(app.has_plugin("exact_dep"));
    assert!(!app.has_plugin("exact_plugin"));
}

#[test]
fn test_caret_major_version_mismatch() {
    // Test caret constraint with different major version
    let mut app = App::new();
    
    let dep = TestPlugin::new("major_dep").with_version("2.0.0");
    app.add_plugins(dep);
    
    let plugin = TestPlugin::new("major_plugin")
        .with_dependency(PluginDependency::with_version("major_dep", "^1.0"));
    app.add_plugins(plugin);
    
    assert!(app.has_plugin("major_dep"));
    assert!(!app.has_plugin("major_plugin"));
}

#[test]
fn test_dependency_chain() {
    // Test A -> B -> C dependency chain
    let mut app = App::new();
    
    // Load in correct order
    app.add_plugins(TestPlugin::new("plugin_c"));
    app.add_plugins(TestPlugin::new("plugin_b")
        .with_dependency(PluginDependency::new("plugin_c")));
    app.add_plugins(TestPlugin::new("plugin_a")
        .with_dependency(PluginDependency::new("plugin_b")));
    
    assert!(app.has_plugin("plugin_c"));
    assert!(app.has_plugin("plugin_b"));
    assert!(app.has_plugin("plugin_a"));
    
    let order = app.plugin_order();
    assert_eq!(order, vec!["plugin_c", "plugin_b", "plugin_a"]);
}

#[test]
fn test_dependency_chain_broken() {
    // Test A -> B -> C with C missing
    let mut app = App::new();
    
    // Try to load B without C
    app.add_plugins(TestPlugin::new("plugin_b")
        .with_dependency(PluginDependency::new("plugin_c")));
    
    // Try to load A (depends on B which failed to load)
    app.add_plugins(TestPlugin::new("plugin_a")
        .with_dependency(PluginDependency::new("plugin_b")));
    
    // Neither should load
    assert!(!app.has_plugin("plugin_b"));
    assert!(!app.has_plugin("plugin_a"));
}