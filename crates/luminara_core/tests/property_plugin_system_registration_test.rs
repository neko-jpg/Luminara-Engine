use luminara_core::{App, AppInterface, Plugin};
use luminara_core::shared_types::CoreStage;
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// Property Test 24: Plugin System Registration
// Validates: Requirements 9.2
// ============================================================================

/// Test plugin that adds systems to specific stages
#[derive(Clone)]
struct SystemRegisteringPlugin {
    name: String,
    systems_to_add: Vec<(CoreStage, String)>, // (stage, system_name)
}

impl SystemRegisteringPlugin {
    fn new(
        name: &str,
        systems_to_add: Vec<(CoreStage, String)>,
        _tracker: Arc<Mutex<Vec<(String, CoreStage, String)>>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            systems_to_add,
        }
    }
}

impl Plugin for SystemRegisteringPlugin {
    fn build(&self, app: &mut App) {
        for (stage, _system_name) in &self.systems_to_add {
            // Add a dummy system (just an empty function)
            fn dummy_system(_world: &mut luminara_core::world::World) {}
            
            app.add_system::<luminara_core::system::ExclusiveMarker>(*stage, dummy_system);
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Strategy for generating CoreStage values
fn core_stage_strategy() -> impl Strategy<Value = CoreStage> {
    prop_oneof![
        Just(CoreStage::Startup),
        Just(CoreStage::PreUpdate),
        Just(CoreStage::Update),
        Just(CoreStage::FixedUpdate),
        Just(CoreStage::PostUpdate),
        Just(CoreStage::PreRender),
        Just(CoreStage::Render),
        Just(CoreStage::PostRender),
    ]
}

/// Strategy for generating system names
fn system_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z_][a-z0-9_]{3,20}_system").unwrap()
}

/// Strategy for generating a list of (stage, system_name) pairs
fn systems_to_add_strategy() -> impl Strategy<Value = Vec<(CoreStage, String)>> {
    prop::collection::vec(
        (core_stage_strategy(), system_name_strategy()),
        1..10
    )
}

/// Strategy for generating plugin names
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{3,20}Plugin").unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Property 24: Plugin System Registration**
    /// 
    /// For any plugin that adds systems to specific stages, those systems should
    /// be present in the schedule at the specified stages after the plugin is built.
    /// 
    /// **Validates: Requirements 9.2**
    #[test]
    fn prop_plugin_system_registration(
        plugin_name in plugin_name_strategy(),
        systems_to_add in systems_to_add_strategy()
    ) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        
        // Count systems per stage before plugin registration
        let mut expected_counts: std::collections::HashMap<CoreStage, usize> = std::collections::HashMap::new();
        for (stage, _) in &systems_to_add {
            *expected_counts.entry(*stage).or_insert(0) += 1;
        }
        
        // Register plugin that adds systems
        let plugin = SystemRegisteringPlugin::new(&plugin_name, systems_to_add.clone(), tracker.clone());
        app.add_plugins(plugin);
        
        // Verify systems are present in the schedule
        for (stage, expected_count) in expected_counts {
            let actual_count = app.schedule.system_count(stage);
            prop_assert!(
                actual_count >= expected_count,
                "Stage {:?} should have at least {} systems, but has {}",
                stage,
                expected_count,
                actual_count
            );
        }
        
        // Verify all stages with systems have systems registered
        for (stage, _) in &systems_to_add {
            prop_assert!(
                app.schedule.has_systems(*stage),
                "Stage {:?} should have systems registered",
                stage
            );
        }
    }
    
    /// **Property 24 (variant): Multiple Plugins System Registration**
    /// 
    /// For any set of plugins that each add systems to various stages, all systems
    /// from all plugins should be present in the schedule after all plugins are built.
    /// 
    /// **Validates: Requirements 9.2**
    #[test]
    fn prop_multiple_plugins_system_registration(
        plugin_count in 1usize..5,
        systems_per_plugin in 1usize..5
    ) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        
        let mut total_expected_counts: std::collections::HashMap<CoreStage, usize> = std::collections::HashMap::new();
        
        // Register multiple plugins
        for i in 0..plugin_count {
            let plugin_name = format!("Plugin{}", i);
            
            // Generate systems for this plugin
            let mut systems_to_add = Vec::new();
            for j in 0..systems_per_plugin {
                let stage = match j % 4 {
                    0 => CoreStage::Update,
                    1 => CoreStage::PreUpdate,
                    2 => CoreStage::PostUpdate,
                    _ => CoreStage::FixedUpdate,
                };
                let system_name = format!("system_{}_{}", i, j);
                systems_to_add.push((stage, system_name));
                *total_expected_counts.entry(stage).or_insert(0) += 1;
            }
            
            let plugin = SystemRegisteringPlugin::new(&plugin_name, systems_to_add, tracker.clone());
            app.add_plugins(plugin);
        }
        
        // Verify all systems are present in the schedule
        for (stage, expected_count) in total_expected_counts {
            let actual_count = app.schedule.system_count(stage);
            prop_assert_eq!(
                actual_count,
                expected_count,
                "Stage {:?} should have exactly {} systems from all plugins",
                stage,
                expected_count
            );
        }
    }
    
    /// **Property 24 (variant): Plugin Adds Systems to All Stages**
    /// 
    /// For any plugin that adds systems to all available stages, each stage should
    /// have at least one system registered.
    /// 
    /// **Validates: Requirements 9.2**
    #[test]
    fn prop_plugin_adds_to_all_stages(plugin_name in plugin_name_strategy()) {
        let tracker = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        
        // Create a plugin that adds one system to each stage
        let all_stages = vec![
            CoreStage::Startup,
            CoreStage::PreUpdate,
            CoreStage::Update,
            CoreStage::FixedUpdate,
            CoreStage::PostUpdate,
            CoreStage::PreRender,
            CoreStage::Render,
            CoreStage::PostRender,
        ];
        
        let systems_to_add: Vec<(CoreStage, String)> = all_stages
            .iter()
            .enumerate()
            .map(|(i, stage)| (*stage, format!("system_{}", i)))
            .collect();
        
        let plugin = SystemRegisteringPlugin::new(&plugin_name, systems_to_add, tracker.clone());
        app.add_plugins(plugin);
        
        // Verify each stage has at least one system
        for stage in all_stages {
            prop_assert!(
                app.schedule.has_systems(stage),
                "Stage {:?} should have at least one system",
                stage
            );
            prop_assert!(
                app.schedule.system_count(stage) >= 1,
                "Stage {:?} should have at least 1 system",
                stage
            );
        }
    }
}

// Additional edge case tests

#[test]
fn test_plugin_adds_single_system() {
    // **Validates: Requirements 9.2**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();
    
    let systems_to_add = vec![(CoreStage::Update, "test_system".to_string())];
    let plugin = SystemRegisteringPlugin::new("TestPlugin", systems_to_add, tracker.clone());
    
    app.add_plugins(plugin);
    
    assert!(
        app.schedule.has_systems(CoreStage::Update),
        "Update stage should have systems"
    );
    assert_eq!(
        app.schedule.system_count(CoreStage::Update),
        1,
        "Update stage should have exactly 1 system"
    );
}

#[test]
fn test_plugin_adds_multiple_systems_same_stage() {
    // **Validates: Requirements 9.2**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();
    
    let systems_to_add = vec![
        (CoreStage::Update, "system_1".to_string()),
        (CoreStage::Update, "system_2".to_string()),
        (CoreStage::Update, "system_3".to_string()),
    ];
    let plugin = SystemRegisteringPlugin::new("TestPlugin", systems_to_add, tracker.clone());
    
    app.add_plugins(plugin);
    
    assert_eq!(
        app.schedule.system_count(CoreStage::Update),
        3,
        "Update stage should have exactly 3 systems"
    );
}

#[test]
fn test_plugin_adds_systems_to_different_stages() {
    // **Validates: Requirements 9.2**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();
    
    let systems_to_add = vec![
        (CoreStage::PreUpdate, "pre_system".to_string()),
        (CoreStage::Update, "update_system".to_string()),
        (CoreStage::PostUpdate, "post_system".to_string()),
    ];
    let plugin = SystemRegisteringPlugin::new("TestPlugin", systems_to_add, tracker.clone());
    
    app.add_plugins(plugin);
    
    assert_eq!(app.schedule.system_count(CoreStage::PreUpdate), 1);
    assert_eq!(app.schedule.system_count(CoreStage::Update), 1);
    assert_eq!(app.schedule.system_count(CoreStage::PostUpdate), 1);
}

#[test]
fn test_empty_plugin_no_systems() {
    // **Validates: Requirements 9.2**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();
    
    let systems_to_add = vec![];
    let plugin = SystemRegisteringPlugin::new("EmptyPlugin", systems_to_add, tracker.clone());
    
    app.add_plugins(plugin);
    
    // Verify no systems were added
    let all_stages = vec![
        CoreStage::Startup,
        CoreStage::PreUpdate,
        CoreStage::Update,
        CoreStage::FixedUpdate,
        CoreStage::PostUpdate,
        CoreStage::PreRender,
        CoreStage::Render,
        CoreStage::PostRender,
    ];
    
    for stage in all_stages {
        assert_eq!(
            app.schedule.system_count(stage),
            0,
            "Stage {:?} should have no systems from empty plugin",
            stage
        );
    }
}

#[test]
fn test_multiple_plugins_accumulate_systems() {
    // **Validates: Requirements 9.2**
    let tracker = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new();
    
    // First plugin adds 2 systems to Update
    let plugin1 = SystemRegisteringPlugin::new(
        "Plugin1",
        vec![
            (CoreStage::Update, "system_1".to_string()),
            (CoreStage::Update, "system_2".to_string()),
        ],
        tracker.clone(),
    );
    app.add_plugins(plugin1);
    
    assert_eq!(app.schedule.system_count(CoreStage::Update), 2);
    
    // Second plugin adds 3 more systems to Update
    let plugin2 = SystemRegisteringPlugin::new(
        "Plugin2",
        vec![
            (CoreStage::Update, "system_3".to_string()),
            (CoreStage::Update, "system_4".to_string()),
            (CoreStage::Update, "system_5".to_string()),
        ],
        tracker.clone(),
    );
    app.add_plugins(plugin2);
    
    // Total should be 5 systems
    assert_eq!(
        app.schedule.system_count(CoreStage::Update),
        5,
        "Update stage should accumulate systems from multiple plugins"
    );
}
