use luminara_core::{App, AppInterface, Plugin, PluginDependency};

// Test plugins with dependencies

struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "base_plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

struct DependentPlugin;

impl Plugin for DependentPlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "dependent_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency::new("base_plugin")]
    }
}

struct VersionedDependentPlugin;

impl Plugin for VersionedDependentPlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "versioned_dependent_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency::with_version("base_plugin", ">=1.0.0")]
    }
}

struct MultiDependentPlugin;

impl Plugin for MultiDependentPlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "multi_dependent_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency::new("base_plugin"),
            PluginDependency::new("dependent_plugin"),
        ]
    }
}

struct OldVersionPlugin;

impl Plugin for OldVersionPlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "old_version_plugin"
    }

    fn version(&self) -> &str {
        "0.5.0"
    }
}

struct RequiresNewerVersionPlugin;

impl Plugin for RequiresNewerVersionPlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "requires_newer_version_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency::with_version("old_version_plugin", ">=1.0.0")]
    }
}

#[test]
fn test_plugin_with_satisfied_dependency() {
    // Validates: Requirements 16.1
    let mut app = App::new();

    // Load base plugin first
    app.add_plugins(BasePlugin);

    // Load dependent plugin - should succeed
    app.add_plugins(DependentPlugin);

    // Both plugins should be registered
    assert!(app.has_plugin("base_plugin"));
    assert!(app.has_plugin("dependent_plugin"));

    let order = app.plugin_order();
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], "base_plugin");
    assert_eq!(order[1], "dependent_plugin");
}

#[test]
fn test_plugin_with_missing_dependency() {
    // Validates: Requirements 16.1, 16.6
    let mut app = App::new();

    // Try to load dependent plugin without base plugin
    app.add_plugins(DependentPlugin);

    // Dependent plugin should NOT be registered due to missing dependency
    assert!(!app.has_plugin("dependent_plugin"));

    // Plugin order should be empty
    assert_eq!(app.plugin_order().len(), 0);
}

#[test]
fn test_plugin_with_version_constraint_satisfied() {
    // Validates: Requirements 16.1
    let mut app = App::new();

    // Load base plugin with version 1.0.0
    app.add_plugins(BasePlugin);

    // Load plugin requiring >=1.0.0 - should succeed
    app.add_plugins(VersionedDependentPlugin);

    assert!(app.has_plugin("base_plugin"));
    assert!(app.has_plugin("versioned_dependent_plugin"));
}

#[test]
fn test_plugin_with_version_constraint_not_satisfied() {
    // Validates: Requirements 16.1, 16.6
    let mut app = App::new();

    // Load old version plugin (0.5.0)
    app.add_plugins(OldVersionPlugin);

    // Try to load plugin requiring >=1.0.0 - should fail
    app.add_plugins(RequiresNewerVersionPlugin);

    assert!(app.has_plugin("old_version_plugin"));
    // Plugin with unsatisfied version constraint should NOT be loaded
    assert!(!app.has_plugin("requires_newer_version_plugin"));
}

#[test]
fn test_plugin_with_multiple_dependencies() {
    // Validates: Requirements 16.1
    let mut app = App::new();

    // Load all dependencies
    app.add_plugins(BasePlugin);
    app.add_plugins(DependentPlugin);

    // Load plugin with multiple dependencies - should succeed
    app.add_plugins(MultiDependentPlugin);

    assert!(app.has_plugin("base_plugin"));
    assert!(app.has_plugin("dependent_plugin"));
    assert!(app.has_plugin("multi_dependent_plugin"));

    let order = app.plugin_order();
    assert_eq!(order.len(), 3);
}

#[test]
fn test_plugin_with_partial_dependencies() {
    // Validates: Requirements 16.1, 16.6
    let mut app = App::new();

    // Load only one of two required dependencies
    app.add_plugins(BasePlugin);

    // Try to load plugin with multiple dependencies - should fail
    app.add_plugins(MultiDependentPlugin);

    assert!(app.has_plugin("base_plugin"));
    assert!(!app.has_plugin("dependent_plugin"));
    // Plugin with missing dependencies should NOT be loaded
    assert!(!app.has_plugin("multi_dependent_plugin"));
}

#[test]
fn test_dependency_validation_error_message() {
    // Validates: Requirements 16.1, 16.6
    use luminara_core::PluginError;

    let app = App::new();

    // Validate dependencies for a plugin without its dependencies loaded
    let result = app.validate_plugin_dependencies(&DependentPlugin);

    assert!(result.is_err());

    if let Err(PluginError::MissingDependencies {
        plugin_name,
        missing,
    }) = result
    {
        assert_eq!(plugin_name, "dependent_plugin");
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].name, "base_plugin");
    } else {
        panic!("Expected MissingDependencies error");
    }
}

#[test]
fn test_version_mismatch_error_message() {
    // Validates: Requirements 16.1, 16.6
    use luminara_core::PluginError;

    let mut app = App::new();
    app.add_plugins(OldVersionPlugin);

    // Validate dependencies for a plugin with version mismatch
    let result = app.validate_plugin_dependencies(&RequiresNewerVersionPlugin);

    assert!(result.is_err());

    if let Err(PluginError::VersionMismatch {
        plugin_name,
        dependency,
        required,
        found,
    }) = result
    {
        assert_eq!(plugin_name, "requires_newer_version_plugin");
        assert_eq!(dependency, "old_version_plugin");
        assert_eq!(required, ">=1.0.0");
        assert_eq!(found, "0.5.0");
    } else {
        panic!("Expected VersionMismatch error");
    }
}

#[test]
fn test_plugin_load_order_with_dependencies() {
    // Validates: Requirements 16.1, 16.2
    let mut app = App::new();

    // Load plugins in correct order
    app.add_plugins(BasePlugin);
    app.add_plugins(DependentPlugin);
    app.add_plugins(MultiDependentPlugin);

    let order = app.plugin_order();
    assert_eq!(order.len(), 3);

    // Verify order: base -> dependent -> multi_dependent
    assert_eq!(order[0], "base_plugin");
    assert_eq!(order[1], "dependent_plugin");
    assert_eq!(order[2], "multi_dependent_plugin");
}

#[test]
fn test_exact_version_match() {
    // Test exact version matching
    struct ExactVersionPlugin;

    impl Plugin for ExactVersionPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "exact_version_plugin"
        }

        fn version(&self) -> &str {
            "2.0.0"
        }
    }

    struct RequiresExactPlugin;

    impl Plugin for RequiresExactPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "requires_exact_plugin"
        }

        fn dependencies(&self) -> Vec<PluginDependency> {
            vec![PluginDependency::with_version("exact_version_plugin", "2.0.0")]
        }
    }

    let mut app = App::new();
    app.add_plugins(ExactVersionPlugin);
    app.add_plugins(RequiresExactPlugin);

    assert!(app.has_plugin("exact_version_plugin"));
    assert!(app.has_plugin("requires_exact_plugin"));
}

#[test]
fn test_caret_version_constraint() {
    // Test caret (^) version constraint - compatible with same major version
    struct CaretVersionPlugin;

    impl Plugin for CaretVersionPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "caret_version_plugin"
        }

        fn version(&self) -> &str {
            "1.5.0"
        }
    }

    struct RequiresCaretPlugin;

    impl Plugin for RequiresCaretPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "requires_caret_plugin"
        }

        fn dependencies(&self) -> Vec<PluginDependency> {
            vec![PluginDependency::with_version("caret_version_plugin", "^1.0")]
        }
    }

    let mut app = App::new();
    app.add_plugins(CaretVersionPlugin);
    app.add_plugins(RequiresCaretPlugin);

    assert!(app.has_plugin("caret_version_plugin"));
    assert!(app.has_plugin("requires_caret_plugin"));
}

#[test]
fn test_caret_version_constraint_major_mismatch() {
    // Test caret constraint fails with different major version
    struct MajorVersionPlugin;

    impl Plugin for MajorVersionPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "major_version_plugin"
        }

        fn version(&self) -> &str {
            "2.0.0"
        }
    }

    struct RequiresMajor1Plugin;

    impl Plugin for RequiresMajor1Plugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "requires_major1_plugin"
        }

        fn dependencies(&self) -> Vec<PluginDependency> {
            vec![PluginDependency::with_version("major_version_plugin", "^1.0")]
        }
    }

    let mut app = App::new();
    app.add_plugins(MajorVersionPlugin);
    app.add_plugins(RequiresMajor1Plugin);

    assert!(app.has_plugin("major_version_plugin"));
    // Should fail due to major version mismatch
    assert!(!app.has_plugin("requires_major1_plugin"));
}

#[test]
fn test_no_dependencies_plugin() {
    // Test that plugins without dependencies work normally
    struct NoDepsPlugin;

    impl Plugin for NoDepsPlugin {
        fn build(&self, _app: &mut App) {}

        fn name(&self) -> &str {
            "no_deps_plugin"
        }
    }

    let mut app = App::new();
    app.add_plugins(NoDepsPlugin);

    assert!(app.has_plugin("no_deps_plugin"));
}

#[test]
fn test_error_display_format() {
    // Validates: Requirements 16.6
    use luminara_core::PluginError;

    let error = PluginError::MissingDependencies {
        plugin_name: "test_plugin".to_string(),
        missing: vec![
            PluginDependency::new("dep1"),
            PluginDependency::with_version("dep2", ">=1.0.0"),
        ],
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("test_plugin"));
    assert!(error_msg.contains("dep1"));
    assert!(error_msg.contains("dep2"));
    assert!(error_msg.contains(">=1.0.0"));

    let version_error = PluginError::VersionMismatch {
        plugin_name: "test_plugin".to_string(),
        dependency: "dep".to_string(),
        required: ">=2.0.0".to_string(),
        found: "1.0.0".to_string(),
    };

    let version_msg = format!("{}", version_error);
    assert!(version_msg.contains("test_plugin"));
    assert!(version_msg.contains("dep"));
    assert!(version_msg.contains(">=2.0.0"));
    assert!(version_msg.contains("1.0.0"));
}
