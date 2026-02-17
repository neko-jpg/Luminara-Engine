# Plugin Testing Framework

## Overview

The plugin testing framework provides utilities for testing Luminara Engine plugins in isolation without requiring a full engine setup. It includes mock engine state, test utilities, and helpers for verifying plugin behavior.

**Validates: Requirements 16.7** - WHEN testing plugins, THE System SHALL provide a plugin testing framework with mock engine state

## Key Components

### MockApp

`MockApp` is a lightweight mock of the `App` structure that tracks all plugin registrations, component registrations, resource insertions, and system additions without requiring a full engine.

```rust
use luminara_core::plugin_test::MockApp;
use luminara_core::AppInterface;

let mut app = MockApp::new();

// Track plugin registration
assert!(!app.has_plugin("my_plugin"));

// Track component registration
app.register_component::<MyComponent>();
assert!(app.has_component::<MyComponent>());

// Track resource registration
app.insert_resource(MyResource::default());
assert!(app.has_resource::<MyResource>());

// Track system registration
app.add_system(CoreStage::Update, my_system);
assert_eq!(app.system_count(CoreStage::Update), 1);
```

### PluginTestContext

`PluginTestContext` provides a high-level interface for testing plugins with common assertions and utilities.

```rust
use luminara_core::plugin_test::PluginTestContext;

let mut ctx = PluginTestContext::new();

// Add plugins
ctx.add_plugin(MyPlugin);

// Verify plugin registration
ctx.assert_plugin_registered("my_plugin");

// Verify plugin order
ctx.assert_plugin_order(&["base_plugin", "my_plugin"]);

// Verify component registration
ctx.assert_component_registered::<MyComponent>();

// Verify resource registration
ctx.assert_resource_registered::<MyResource>();

// Verify system registration
ctx.assert_systems_registered(CoreStage::Update, 2);
```

### MockPluginBuilder

`MockPluginBuilder` is a builder for creating mock plugins for testing purposes.

```rust
use luminara_core::plugin_test::MockPluginBuilder;

let plugin = MockPluginBuilder::new("test_plugin")
    .version("1.0.0")
    .depends_on("base_plugin")
    .depends_on_version("other_plugin", ">=1.0.0")
    .build_fn(|app| {
        app.register_component::<MyComponent>();
        app.insert_resource(MyResource::default());
    })
    .build();
```

## Testing Patterns

### Basic Plugin Testing

Test that a plugin registers correctly:

```rust
#[test]
fn test_my_plugin_registration() {
    let mut ctx = PluginTestContext::new();
    ctx.add_plugin(MyPlugin);
    
    ctx.assert_plugin_registered("my_plugin");
}
```

### Testing Plugin Dependencies

Test that plugin dependencies are validated:

```rust
#[test]
fn test_plugin_dependencies() {
    let mut ctx = PluginTestContext::new();
    
    // Add base plugin first
    ctx.add_plugin(BasePlugin);
    
    // Add dependent plugin
    let result = ctx.add_plugin_with_validation(DependentPlugin);
    assert!(result.is_ok());
    
    ctx.assert_plugin_order(&["base_plugin", "dependent_plugin"]);
}
```

### Testing Plugin Registration Order

Test that plugins are registered in the correct order:

```rust
#[test]
fn test_plugin_order() {
    let mut ctx = PluginTestContext::new();
    
    ctx.add_plugin(FirstPlugin);
    ctx.add_plugin(SecondPlugin);
    ctx.add_plugin(ThirdPlugin);
    
    ctx.assert_plugin_order(&["first", "second", "third"]);
}
```

### Testing Component Registration

Test that a plugin registers components correctly:

```rust
#[test]
fn test_plugin_registers_components() {
    let mut ctx = PluginTestContext::new();
    
    let plugin = MockPluginBuilder::new("test_plugin")
        .build_fn(|app| {
            app.register_component::<MyComponent>();
        })
        .build();
    
    ctx.add_plugin(plugin);
    
    // Note: Component registration through build_fn requires
    // converting MockApp to App, which may not be fully supported
    // in all scenarios. For direct testing, use:
    ctx.app_mut().register_component::<MyComponent>();
    ctx.assert_component_registered::<MyComponent>();
}
```

### Testing Resource Registration

Test that a plugin registers resources correctly:

```rust
#[test]
fn test_plugin_registers_resources() {
    let mut ctx = PluginTestContext::new();
    
    ctx.app_mut().insert_resource(MyResource::default());
    ctx.assert_resource_registered::<MyResource>();
}
```

### Testing System Registration

Test that a plugin registers systems correctly:

```rust
#[test]
fn test_plugin_registers_systems() {
    let mut ctx = PluginTestContext::new();
    
    fn my_system(_world: &mut World) {}
    
    ctx.app_mut().add_system(CoreStage::Update, my_system);
    ctx.assert_systems_registered(CoreStage::Update, 1);
}
```

### Testing Plugin Isolation

Test that plugins can be tested in complete isolation:

```rust
#[test]
fn test_plugin_isolation() {
    let mut ctx1 = PluginTestContext::new();
    let mut ctx2 = PluginTestContext::new();
    
    ctx1.add_plugin(Plugin1);
    ctx2.add_plugin(Plugin2);
    
    // Each context should only have its own plugin
    assert!(ctx1.has_plugin("plugin1"));
    assert!(!ctx1.has_plugin("plugin2"));
    
    assert!(ctx2.has_plugin("plugin2"));
    assert!(!ctx2.has_plugin("plugin1"));
}
```

### Testing Version Constraints

Test that version constraints are validated correctly:

```rust
#[test]
fn test_version_constraints() {
    let mut ctx = PluginTestContext::new();
    
    let base = MockPluginBuilder::new("base")
        .version("1.2.3")
        .build();
    
    let dependent = MockPluginBuilder::new("dependent")
        .depends_on_version("base", ">=1.0.0")
        .build();
    
    ctx.add_plugin(base);
    let result = ctx.add_plugin_with_validation(dependent);
    
    assert!(result.is_ok());
}
```

### Testing Missing Dependencies

Test that missing dependencies are detected:

```rust
#[test]
fn test_missing_dependencies() {
    let mut ctx = PluginTestContext::new();
    
    let dependent = MockPluginBuilder::new("dependent")
        .depends_on("missing_base")
        .build();
    
    let result = ctx.add_plugin_with_validation(dependent);
    assert!(result.is_err());
}
```

### Testing World Access

Test that plugins can interact with the world:

```rust
#[test]
fn test_world_access() {
    let mut ctx = PluginTestContext::new();
    
    // Spawn entity and add component
    let entity = ctx.world_mut().spawn();
    ctx.world_mut().register_component::<MyComponent>();
    ctx.world_mut().add_component(entity, MyComponent { value: 42 });
    
    // Verify using query
    use luminara_core::Query;
    let query = Query::<&MyComponent>::new(ctx.world());
    let components: Vec<_> = query.iter().collect();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].value, 42);
}
```

## Best Practices

### 1. Use PluginTestContext for High-Level Testing

`PluginTestContext` provides convenient assertion methods and should be preferred for most plugin tests:

```rust
let mut ctx = PluginTestContext::new();
ctx.add_plugin(MyPlugin);
ctx.assert_plugin_registered("my_plugin");
```

### 2. Use MockApp for Low-Level Testing

When you need more control or want to test specific behaviors, use `MockApp` directly:

```rust
let mut app = MockApp::new();
app.add_plugins(MyPlugin);
assert!(app.has_plugin("my_plugin"));
```

### 3. Use MockPluginBuilder for Test Plugins

Create test plugins with `MockPluginBuilder` for flexible testing:

```rust
let plugin = MockPluginBuilder::new("test_plugin")
    .version("1.0.0")
    .depends_on("base")
    .build_fn(|app| {
        // Plugin setup
    })
    .build();
```

### 4. Test Plugin Dependencies

Always test that plugin dependencies are correctly declared and validated:

```rust
#[test]
fn test_dependencies() {
    let mut ctx = PluginTestContext::new();
    ctx.add_plugin(BasePlugin);
    
    let result = ctx.add_plugin_with_validation(DependentPlugin);
    assert!(result.is_ok());
}
```

### 5. Test Plugin Isolation

Ensure plugins can be tested independently without side effects:

```rust
#[test]
fn test_isolation() {
    let mut ctx1 = PluginTestContext::new();
    let mut ctx2 = PluginTestContext::new();
    
    ctx1.add_plugin(Plugin1);
    ctx2.add_plugin(Plugin2);
    
    // Verify isolation
    assert!(ctx1.has_plugin("plugin1"));
    assert!(!ctx1.has_plugin("plugin2"));
}
```

### 6. Use Descriptive Test Names

Use descriptive test names that clearly indicate what is being tested:

```rust
#[test]
fn test_my_plugin_registers_transform_component() {
    // Test implementation
}

#[test]
fn test_my_plugin_depends_on_core_plugin() {
    // Test implementation
}
```

### 7. Test Edge Cases

Test edge cases such as missing dependencies, version mismatches, and duplicate registrations:

```rust
#[test]
fn test_missing_dependency_error() {
    let mut ctx = PluginTestContext::new();
    let result = ctx.add_plugin_with_validation(DependentPlugin);
    assert!(result.is_err());
}
```

## Limitations

### Build Function Execution

The current implementation of `MockApp` tracks plugin registrations but does not fully execute the `build` function in the same way as a real `App`. This means that components, resources, and systems registered within a plugin's `build` function may not be tracked by `MockApp`.

**Workaround**: For testing component/resource/system registration, register them directly on the `MockApp`:

```rust
let mut ctx = PluginTestContext::new();
ctx.app_mut().register_component::<MyComponent>();
ctx.assert_component_registered::<MyComponent>();
```

### System Execution

`MockApp` does not execute systems. It only tracks that systems were registered. For testing system behavior, use integration tests with a real `App`.

### World State

While `MockApp` provides access to a `World`, it does not run the full engine loop. For testing that requires the engine loop, use integration tests.

## Examples

See `crates/luminara_core/tests/plugin_testing_framework_test.rs` for comprehensive examples of using the plugin testing framework.

## Related Documentation

- [Plugin System](plugin_dependencies.md)
- [Plugin Execution Order](plugin_execution_order_verification.md)
- [Testing Guide](../../../docs/testing_guide.md)
