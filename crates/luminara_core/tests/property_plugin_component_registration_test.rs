use luminara_core::{App, AppInterface, Plugin};
use luminara_core::component::Component;
use luminara_core::resource::Resource;
use proptest::prelude::*;

// ============================================================================
// Property Test 25: Plugin Component and Resource Registration
// Validates: Requirements 9.3
// ============================================================================

// Test components with unique TypeIds
#[derive(Debug, Clone, PartialEq)]
struct TestComponent1 {
    value: i32,
}

impl Component for TestComponent1 {
    fn type_name() -> &'static str {
        "TestComponent1"
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TestComponent2 {
    value: f32,
}

impl Component for TestComponent2 {
    fn type_name() -> &'static str {
        "TestComponent2"
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TestComponent3 {
    value: String,
}

impl Component for TestComponent3 {
    fn type_name() -> &'static str {
        "TestComponent3"
    }
}

// Test resources with unique TypeIds
#[derive(Debug, Clone, PartialEq)]
struct TestResource1 {
    count: usize,
}

impl Resource for TestResource1 {}

#[derive(Debug, Clone, PartialEq)]
struct TestResource2 {
    name: String,
}

impl Resource for TestResource2 {}

#[derive(Debug, Clone, PartialEq)]
struct TestResource3 {
    enabled: bool,
}

impl Resource for TestResource3 {}

/// Plugin that registers components and resources
#[derive(Clone)]
struct ComponentRegisteringPlugin {
    name: String,
    register_component1: bool,
    register_component2: bool,
    register_component3: bool,
    register_resource1: bool,
    register_resource2: bool,
    register_resource3: bool,
}

impl ComponentRegisteringPlugin {
    fn new(
        name: &str,
        register_component1: bool,
        register_component2: bool,
        register_component3: bool,
        register_resource1: bool,
        register_resource2: bool,
        register_resource3: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            register_component1,
            register_component2,
            register_component3,
            register_resource1,
            register_resource2,
            register_resource3,
        }
    }
}

impl Plugin for ComponentRegisteringPlugin {
    fn build(&self, app: &mut App) {
        // Register components
        if self.register_component1 {
            app.register_component::<TestComponent1>();
        }
        if self.register_component2 {
            app.register_component::<TestComponent2>();
        }
        if self.register_component3 {
            app.register_component::<TestComponent3>();
        }
        
        // Register resources
        if self.register_resource1 {
            app.insert_resource(TestResource1 { count: 0 });
        }
        if self.register_resource2 {
            app.insert_resource(TestResource2 { name: "test".to_string() });
        }
        if self.register_resource3 {
            app.insert_resource(TestResource3 { enabled: true });
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Strategy for generating plugin names
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{3,20}Plugin").unwrap()
}

/// Strategy for generating boolean flags for component/resource registration
fn registration_flags_strategy() -> impl Strategy<Value = (bool, bool, bool, bool, bool, bool)> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Property 25: Plugin Component and Resource Registration**
    /// 
    /// For any plugin that registers custom components or resources, those types
    /// should be available in the world after the plugin is built.
    /// 
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_plugin_component_and_resource_registration(
        plugin_name in plugin_name_strategy(),
        (reg_c1, reg_c2, reg_c3, reg_r1, reg_r2, reg_r3) in registration_flags_strategy()
    ) {
        let mut app = App::new();
        
        // Create and register plugin
        let plugin = ComponentRegisteringPlugin::new(
            &plugin_name,
            reg_c1, reg_c2, reg_c3,
            reg_r1, reg_r2, reg_r3,
        );
        app.add_plugins(plugin);
        
        // Verify components are registered in the world
        if reg_c1 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent1>(),
                "TestComponent1 should be registered in the world"
            );
        }
        
        if reg_c2 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent2>(),
                "TestComponent2 should be registered in the world"
            );
        }
        
        if reg_c3 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent3>(),
                "TestComponent3 should be registered in the world"
            );
        }
        
        // Verify resources are available in the world
        if reg_r1 {
            prop_assert!(
                app.world.get_resource::<TestResource1>().is_some(),
                "TestResource1 should be available in the world"
            );
        }
        
        if reg_r2 {
            prop_assert!(
                app.world.get_resource::<TestResource2>().is_some(),
                "TestResource2 should be available in the world"
            );
        }
        
        if reg_r3 {
            prop_assert!(
                app.world.get_resource::<TestResource3>().is_some(),
                "TestResource3 should be available in the world"
            );
        }
    }
    
    /// **Property 25 (variant): Multiple Plugins Register Different Components**
    /// 
    /// For any set of plugins that each register different components and resources,
    /// all registered types should be available in the world after all plugins are built.
    /// 
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_multiple_plugins_register_different_types(plugin_count in 1usize..5) {
        let mut app = App::new();
        
        // Register multiple plugins, each registering different components/resources
        for i in 0..plugin_count {
            let plugin_name = format!("Plugin{}", i);
            
            // Each plugin registers a different subset
            let reg_c1 = i % 3 == 0;
            let reg_c2 = i % 3 == 1;
            let reg_c3 = i % 3 == 2;
            let reg_r1 = i % 2 == 0;
            let reg_r2 = i % 2 == 1;
            let reg_r3 = i == 0;
            
            let plugin = ComponentRegisteringPlugin::new(
                &plugin_name,
                reg_c1, reg_c2, reg_c3,
                reg_r1, reg_r2, reg_r3,
            );
            app.add_plugins(plugin);
        }
        
        // Check if any plugin registered each component/resource
        let mut expected_c1 = false;
        let mut expected_c2 = false;
        let mut expected_c3 = false;
        let mut expected_r1 = false;
        let mut expected_r2 = false;
        let mut expected_r3 = false;
        
        for i in 0..plugin_count {
            if i % 3 == 0 { expected_c1 = true; }
            if i % 3 == 1 { expected_c2 = true; }
            if i % 3 == 2 { expected_c3 = true; }
            if i % 2 == 0 { expected_r1 = true; }
            if i % 2 == 1 { expected_r2 = true; }
            if i == 0 { expected_r3 = true; }
        }
        
        // Verify expected components are registered
        if expected_c1 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent1>(),
                "TestComponent1 should be registered"
            );
        }
        
        if expected_c2 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent2>(),
                "TestComponent2 should be registered"
            );
        }
        
        if expected_c3 {
            prop_assert!(
                app.world.is_component_registered::<TestComponent3>(),
                "TestComponent3 should be registered"
            );
        }
        
        // Verify expected resources are available
        if expected_r1 {
            prop_assert!(
                app.world.get_resource::<TestResource1>().is_some(),
                "TestResource1 should be available"
            );
        }
        
        if expected_r2 {
            prop_assert!(
                app.world.get_resource::<TestResource2>().is_some(),
                "TestResource2 should be available"
            );
        }
        
        if expected_r3 {
            prop_assert!(
                app.world.get_resource::<TestResource3>().is_some(),
                "TestResource3 should be available"
            );
        }
    }
    
    /// **Property 25 (variant): Plugin Registers All Types**
    /// 
    /// For any plugin that registers all available components and resources,
    /// all types should be available in the world.
    /// 
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_plugin_registers_all_types(plugin_name in plugin_name_strategy()) {
        let mut app = App::new();
        
        // Create plugin that registers everything
        let plugin = ComponentRegisteringPlugin::new(
            &plugin_name,
            true, true, true,  // All components
            true, true, true,  // All resources
        );
        app.add_plugins(plugin);
        
        // Verify all components are registered
        prop_assert!(
            app.world.is_component_registered::<TestComponent1>(),
            "TestComponent1 should be registered"
        );
        prop_assert!(
            app.world.is_component_registered::<TestComponent2>(),
            "TestComponent2 should be registered"
        );
        prop_assert!(
            app.world.is_component_registered::<TestComponent3>(),
            "TestComponent3 should be registered"
        );
        
        // Verify all resources are available
        prop_assert!(
            app.world.get_resource::<TestResource1>().is_some(),
            "TestResource1 should be available"
        );
        prop_assert!(
            app.world.get_resource::<TestResource2>().is_some(),
            "TestResource2 should be available"
        );
        prop_assert!(
            app.world.get_resource::<TestResource3>().is_some(),
            "TestResource3 should be available"
        );
    }
}

// Additional edge case tests

#[test]
fn test_plugin_registers_single_component() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    let plugin = ComponentRegisteringPlugin::new(
        "TestPlugin",
        true, false, false,  // Only component 1
        false, false, false, // No resources
    );
    app.add_plugins(plugin);
    
    assert!(
        app.world.is_component_registered::<TestComponent1>(),
        "TestComponent1 should be registered"
    );
    assert!(
        !app.world.is_component_registered::<TestComponent2>(),
        "TestComponent2 should not be registered"
    );
    assert!(
        !app.world.is_component_registered::<TestComponent3>(),
        "TestComponent3 should not be registered"
    );
}

#[test]
fn test_plugin_registers_single_resource() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    let plugin = ComponentRegisteringPlugin::new(
        "TestPlugin",
        false, false, false, // No components
        true, false, false,  // Only resource 1
    );
    app.add_plugins(plugin);
    
    assert!(
        app.world.get_resource::<TestResource1>().is_some(),
        "TestResource1 should be available"
    );
    assert!(
        app.world.get_resource::<TestResource2>().is_none(),
        "TestResource2 should not be available"
    );
    assert!(
        app.world.get_resource::<TestResource3>().is_none(),
        "TestResource3 should not be available"
    );
}

#[test]
fn test_plugin_registers_nothing() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    let plugin = ComponentRegisteringPlugin::new(
        "EmptyPlugin",
        false, false, false, // No components
        false, false, false, // No resources
    );
    app.add_plugins(plugin);
    
    // Verify nothing was registered
    assert!(
        !app.world.is_component_registered::<TestComponent1>(),
        "TestComponent1 should not be registered"
    );
    assert!(
        app.world.get_resource::<TestResource1>().is_none(),
        "TestResource1 should not be available"
    );
}

#[test]
fn test_component_can_be_used_after_registration() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    let plugin = ComponentRegisteringPlugin::new(
        "TestPlugin",
        true, false, false,  // Register component 1
        false, false, false,
    );
    app.add_plugins(plugin);
    
    // Verify we can spawn an entity with the registered component
    let entity = app.world.spawn();
    app.world.add_component(entity, TestComponent1 { value: 42 });
    
    let component = app.world.get_component::<TestComponent1>(entity);
    assert!(component.is_some(), "Component should be retrievable");
    assert_eq!(component.unwrap().value, 42, "Component value should match");
}

#[test]
fn test_resource_can_be_accessed_after_registration() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    let plugin = ComponentRegisteringPlugin::new(
        "TestPlugin",
        false, false, false,
        true, false, false,  // Register resource 1
    );
    app.add_plugins(plugin);
    
    // Verify we can access and modify the resource
    {
        let resource = app.world.get_resource::<TestResource1>().unwrap();
        assert_eq!(resource.count, 0, "Initial resource value should be 0");
    }
    
    {
        let resource = app.world.get_resource_mut::<TestResource1>().unwrap();
        resource.count = 100;
    }
    
    {
        let resource = app.world.get_resource::<TestResource1>().unwrap();
        assert_eq!(resource.count, 100, "Modified resource value should be 100");
    }
}

#[test]
fn test_multiple_plugins_accumulate_registrations() {
    // **Validates: Requirements 9.3**
    let mut app = App::new();
    
    // First plugin registers component 1 and resource 1
    let plugin1 = ComponentRegisteringPlugin::new(
        "Plugin1",
        true, false, false,
        true, false, false,
    );
    app.add_plugins(plugin1);
    
    // Second plugin registers component 2 and resource 2
    let plugin2 = ComponentRegisteringPlugin::new(
        "Plugin2",
        false, true, false,
        false, true, false,
    );
    app.add_plugins(plugin2);
    
    // Third plugin registers component 3 and resource 3
    let plugin3 = ComponentRegisteringPlugin::new(
        "Plugin3",
        false, false, true,
        false, false, true,
    );
    app.add_plugins(plugin3);
    
    // Verify all components are registered
    assert!(app.world.is_component_registered::<TestComponent1>());
    assert!(app.world.is_component_registered::<TestComponent2>());
    assert!(app.world.is_component_registered::<TestComponent3>());
    
    // Verify all resources are available
    assert!(app.world.get_resource::<TestResource1>().is_some());
    assert!(app.world.get_resource::<TestResource2>().is_some());
    assert!(app.world.get_resource::<TestResource3>().is_some());
}
