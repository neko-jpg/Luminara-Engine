/// Example demonstrating plugin dependency validation
/// 
/// This example shows how the plugin system validates dependencies
/// and prevents plugins from loading when dependencies are missing.

use luminara_core::{App, AppInterface, Plugin, PluginDependency, Resource};

// Base plugin that provides core functionality
struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        println!("✓ CorePlugin loaded");
        app.insert_resource(CoreResource { initialized: true });
    }

    fn name(&self) -> &str {
        "core_plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

#[derive(Debug, Clone)]
struct CoreResource {
    initialized: bool,
}

impl Resource for CoreResource {}

// Rendering plugin that depends on CorePlugin
struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        println!("✓ RenderingPlugin loaded");
        app.insert_resource(RenderingResource { enabled: true });
    }

    fn name(&self) -> &str {
        "rendering_plugin"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency::with_version("core_plugin", ">=1.0.0")]
    }
}

#[derive(Debug, Clone)]
struct RenderingResource {
    enabled: bool,
}

impl Resource for RenderingResource {}

// Advanced plugin that depends on both CorePlugin and RenderingPlugin
struct AdvancedFeaturesPlugin;

impl Plugin for AdvancedFeaturesPlugin {
    fn build(&self, app: &mut App) {
        println!("✓ AdvancedFeaturesPlugin loaded");
        app.insert_resource(AdvancedResource { level: 3 });
    }

    fn name(&self) -> &str {
        "advanced_features_plugin"
    }

    fn version(&self) -> &str {
        "1.5.0"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency::new("core_plugin"),
            PluginDependency::with_version("rendering_plugin", ">=2.0.0"),
        ]
    }
}

#[derive(Debug, Clone)]
struct AdvancedResource {
    level: u32,
}

impl Resource for AdvancedResource {}

// Plugin with unsatisfied dependency (for demonstration)
struct BrokenPlugin;

impl Plugin for BrokenPlugin {
    fn build(&self, _app: &mut App) {
        println!("✓ BrokenPlugin loaded (this shouldn't happen!)");
    }

    fn name(&self) -> &str {
        "broken_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency::new("nonexistent_plugin")]
    }
}

fn main() {
    println!("=== Plugin Dependency Validation Demo ===\n");

    // Scenario 1: Correct loading order
    println!("Scenario 1: Loading plugins in correct order");
    println!("---------------------------------------------");
    let mut app = App::new();
    app.add_plugins(CorePlugin);
    app.add_plugins(RenderingPlugin);
    app.add_plugins(AdvancedFeaturesPlugin);
    
    println!("\nLoaded plugins:");
    for plugin_name in app.plugin_order() {
        println!("  - {}", plugin_name);
    }
    println!();

    // Scenario 2: Missing dependency
    println!("\nScenario 2: Attempting to load plugin with missing dependency");
    println!("--------------------------------------------------------------");
    let mut app2 = App::new();
    app2.add_plugins(CorePlugin);
    // Skip RenderingPlugin
    println!("Attempting to load AdvancedFeaturesPlugin without RenderingPlugin...");
    app2.add_plugins(AdvancedFeaturesPlugin);
    
    println!("\nLoaded plugins:");
    for plugin_name in app2.plugin_order() {
        println!("  - {}", plugin_name);
    }
    println!("(Note: AdvancedFeaturesPlugin was not loaded due to missing dependency)\n");

    // Scenario 3: Version constraint violation
    println!("\nScenario 3: Version constraint validation");
    println!("------------------------------------------");
    
    struct OldCorePlugin;
    impl Plugin for OldCorePlugin {
        fn build(&self, _app: &mut App) {
            println!("✓ OldCorePlugin (v0.5.0) loaded");
        }
        fn name(&self) -> &str {
            "core_plugin"
        }
        fn version(&self) -> &str {
            "0.5.0"
        }
    }

    struct RequiresNewCorePlugin;
    impl Plugin for RequiresNewCorePlugin {
        fn build(&self, _app: &mut App) {
            println!("✓ RequiresNewCorePlugin loaded");
        }
        fn name(&self) -> &str {
            "requires_new_core"
        }
        fn dependencies(&self) -> Vec<PluginDependency> {
            vec![PluginDependency::with_version("core_plugin", ">=1.0.0")]
        }
    }

    let mut app3 = App::new();
    app3.add_plugins(OldCorePlugin);
    println!("Attempting to load plugin requiring core_plugin >=1.0.0...");
    app3.add_plugins(RequiresNewCorePlugin);
    
    println!("\nLoaded plugins:");
    for plugin_name in app3.plugin_order() {
        println!("  - {}", plugin_name);
    }
    println!("(Note: RequiresNewCorePlugin was not loaded due to version mismatch)\n");

    // Scenario 4: Nonexistent dependency
    println!("\nScenario 4: Nonexistent dependency");
    println!("-----------------------------------");
    let mut app4 = App::new();
    println!("Attempting to load plugin with nonexistent dependency...");
    app4.add_plugins(BrokenPlugin);
    
    println!("\nLoaded plugins:");
    if app4.plugin_order().is_empty() {
        println!("  (none - plugin failed to load)");
    }
    println!();

    println!("=== Demo Complete ===");
    println!("\nKey takeaways:");
    println!("1. Plugins are validated before loading");
    println!("2. Missing dependencies prevent plugin loading");
    println!("3. Version constraints are enforced");
    println!("4. Clear error messages are provided");
    println!("5. Failed plugins don't crash the application");
}
