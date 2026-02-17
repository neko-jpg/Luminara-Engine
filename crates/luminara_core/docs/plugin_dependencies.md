# Plugin Dependency Validation

## Overview

The Luminara Engine plugin system now supports dependency validation to ensure plugins are loaded in the correct order and all required dependencies are satisfied before a plugin is initialized.

## Features

- **Dependency Declaration**: Plugins can declare their dependencies on other plugins
- **Version Constraints**: Support for version constraints (exact match, `>=`, `^`)
- **Clear Error Messages**: Detailed error messages when dependencies are missing or version constraints are not satisfied
- **Automatic Validation**: Dependencies are validated automatically when plugins are loaded
- **Graceful Failure**: Plugins with unsatisfied dependencies fail to load without crashing the application

## Usage

### Basic Dependency Declaration

To declare that your plugin depends on another plugin, override the `dependencies()` method:

```rust
use luminara_core::{Plugin, PluginDependency, App};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        // Plugin initialization code
    }

    fn name(&self) -> &str {
        "my_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency::new("base_plugin"),
            PluginDependency::new("rendering_plugin"),
        ]
    }
}
```

### Version Constraints

You can specify version constraints for dependencies:

```rust
fn dependencies(&self) -> Vec<PluginDependency> {
    vec![
        // Exact version match
        PluginDependency::with_version("base_plugin", "1.0.0"),
        
        // Greater than or equal to
        PluginDependency::with_version("rendering_plugin", ">=2.0.0"),
        
        // Caret constraint (compatible with same major version)
        PluginDependency::with_version("physics_plugin", "^1.5"),
    ]
}
```

### Plugin Versioning

Plugins can declare their version by overriding the `version()` method:

```rust
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        // ...
    }

    fn name(&self) -> &str {
        "my_plugin"
    }

    fn version(&self) -> &str {
        "2.1.0"
    }
}
```

## Version Constraint Syntax

The plugin system supports the following version constraint formats:

- **Exact Match**: `"1.0.0"` - Requires exactly version 1.0.0
- **Greater Than or Equal**: `">=1.0.0"` - Requires version 1.0.0 or higher
- **Caret Constraint**: `"^1.5"` - Requires version 1.5.0 or higher, but less than 2.0.0 (same major version)

## Error Handling

### Missing Dependencies

When a plugin has missing dependencies, it will fail to load and print an error message:

```
Error loading plugin 'my_plugin': Plugin 'my_plugin' has unsatisfied dependencies: 'base_plugin', 'rendering_plugin'
Plugin 'my_plugin' will not be loaded. Please ensure all dependencies are loaded first.
```

### Version Mismatch

When a dependency's version doesn't satisfy the constraint:

```
Error loading plugin 'my_plugin': Plugin 'my_plugin' requires 'base_plugin' version >=2.0.0, but found version 1.5.0
Plugin 'my_plugin' will not be loaded. Please ensure all dependencies are loaded first.
```

## Best Practices

1. **Load Dependencies First**: Always load dependency plugins before dependent plugins
2. **Use Version Constraints**: Specify version constraints when your plugin requires specific features
3. **Keep Dependencies Minimal**: Only declare dependencies that are truly required
4. **Document Dependencies**: Document your plugin's dependencies in your plugin's documentation
5. **Test Dependency Chains**: Test your plugin with various dependency configurations

## Example: Complete Plugin with Dependencies

```rust
use luminara_core::{App, Plugin, PluginDependency, AppInterface};

// Base plugin that other plugins depend on
struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Initialize core systems
        app.insert_resource(CoreResource::default());
    }

    fn name(&self) -> &str {
        "core_plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

// Plugin that depends on CorePlugin
struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize rendering systems
        // Can safely access CoreResource because CorePlugin is guaranteed to be loaded
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

// Plugin that depends on both CorePlugin and RenderingPlugin
struct AdvancedRenderingPlugin;

impl Plugin for AdvancedRenderingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize advanced rendering features
    }

    fn name(&self) -> &str {
        "advanced_rendering_plugin"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency::new("core_plugin"),
            PluginDependency::with_version("rendering_plugin", ">=2.0.0"),
        ]
    }
}

fn main() {
    let mut app = App::new();

    // Load plugins in correct order
    app.add_plugins(CorePlugin);
    app.add_plugins(RenderingPlugin);
    app.add_plugins(AdvancedRenderingPlugin);

    app.run();
}
```

## Implementation Details

### Validation Process

When a plugin is added via `add_plugins()`:

1. The system checks if the plugin is already registered (to prevent duplicate loading)
2. If not registered, it validates all dependencies:
   - Checks that each dependency plugin is already loaded
   - If version constraints are specified, validates that the loaded version satisfies the constraint
3. If validation fails:
   - An error message is printed to stderr
   - The plugin is NOT loaded
   - Other plugins continue to load normally
4. If validation succeeds:
   - The plugin is registered
   - The plugin's `build()` method is called
   - The plugin's version is recorded for future dependency checks

### Version Comparison

Version strings are compared numerically by splitting on `.` and comparing each component:

- `"1.0.0"` < `"1.0.1"` < `"1.1.0"` < `"2.0.0"`
- Missing components are treated as `0`: `"1.5"` is equivalent to `"1.5.0"`

## Testing

The plugin dependency system includes comprehensive tests covering:

- Satisfied dependencies
- Missing dependencies
- Version constraint satisfaction
- Version constraint violations
- Multiple dependencies
- Partial dependencies
- Error message formatting
- Plugin load order with dependencies

Run the tests with:

```bash
cargo test --test plugin_dependency_test --package luminara_core
```

## Future Enhancements

Potential future improvements to the plugin dependency system:

- **Circular Dependency Detection**: Detect and report circular dependencies between plugins
- **Optional Dependencies**: Support for optional dependencies that don't prevent plugin loading
- **Dependency Groups**: Allow plugins to satisfy dependencies as a group
- **Semantic Versioning**: Full semver support with tilde (`~`) and wildcard (`*`) constraints
- **Dependency Resolution**: Automatic ordering of plugins based on dependency graph
