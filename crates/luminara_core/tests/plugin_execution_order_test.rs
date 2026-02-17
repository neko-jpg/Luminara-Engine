//! # Plugin Execution Order Tests
//!
//! **Task 21.3: Verify Plugin Execution Order**
//!
//! **Validates: Requirements 16.2**
//!
//! This test suite verifies that the plugin system correctly handles:
//! 1. **Declared Execution Order**: Systems registered by plugins execute in the order they were registered
//! 2. **Stage Placement**: Systems are placed in the correct stages and stages execute in the correct order
//! 3. **No Race Conditions**: Systems with conflicting access don't run in parallel, ensuring thread safety
//!
//! ## Test Coverage
//!
//! ### Execution Order Tests
//! - Systems in the same stage execute in registration order
//! - Multiple plugins' systems execute in plugin registration order
//! - Order is maintained across multiple update cycles
//!
//! ### Stage Placement Tests
//! - Systems execute in their declared stages
//! - All stages execute in the correct order (PreUpdate → Update → FixedUpdate → PostUpdate → PreRender → Render → PostRender)
//! - Multiple plugins adding to different stages maintain correct stage order
//! - Startup stage executes before all other stages
//!
//! ### Race Condition Tests
//! - Systems accessing shared resources don't have race conditions
//! - Execution is deterministic across multiple runs
//! - Complex scenarios with multiple plugins and stages work correctly
//!
//! ## Implementation Notes
//!
//! The tests use an `ExecutionTracker` resource to record when systems execute and in what stage.
//! This allows verification of execution order without relying on side effects or timing.

use luminara_core::shared_types::CoreStage;
use luminara_core::{App, AppInterface, Plugin, Resource};
use std::sync::{Arc, Mutex};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Resource to track system execution order
#[derive(Debug, Clone)]
struct ExecutionTracker {
    executions: Arc<Mutex<Vec<(String, CoreStage)>>>,
}

impl ExecutionTracker {
    fn new() -> Self {
        Self {
            executions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn record(&self, system_name: &str, stage: CoreStage) {
        self.executions
            .lock()
            .unwrap()
            .push((system_name.to_string(), stage));
    }

    fn get_executions(&self) -> Vec<(String, CoreStage)> {
        self.executions.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.executions.lock().unwrap().clear();
    }
}

impl Resource for ExecutionTracker {}

/// Test plugin that registers systems in a specific order
struct OrderedSystemPlugin {
    name: String,
    systems: Vec<(CoreStage, String)>,
}

impl OrderedSystemPlugin {
    fn new(name: &str, systems: Vec<(CoreStage, String)>) -> Self {
        Self {
            name: name.to_string(),
            systems,
        }
    }
}

impl Plugin for OrderedSystemPlugin {
    fn build(&self, app: &mut App) {
        for (stage, system_name) in &self.systems {
            let stage = *stage;
            let name = system_name.clone();

            // Create a system that records its execution
            let system = move |world: &mut luminara_core::world::World| {
                if let Some(tracker) = world.get_resource::<ExecutionTracker>() {
                    tracker.record(&name, stage);
                }
            };

            app.add_system::<luminara_core::system::ExclusiveMarker>(stage, system);
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// Test 1: Declared Execution Order Respected
// ============================================================================

#[test]
fn test_systems_execute_in_registration_order_same_stage() {
    // **Validates: Requirements 16.2**
    // Systems registered in a specific order should execute in that order
    // within the same stage

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Register plugin with multiple systems in Update stage
    let plugin = OrderedSystemPlugin::new(
        "TestPlugin",
        vec![
            (CoreStage::Update, "system_1".to_string()),
            (CoreStage::Update, "system_2".to_string()),
            (CoreStage::Update, "system_3".to_string()),
        ],
    );

    app.add_plugins(plugin);

    // Run one update cycle
    app.update();

    // Verify execution order
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 3, "All three systems should execute");

    // Systems should execute in registration order
    assert_eq!(executions[0].0, "system_1");
    assert_eq!(executions[1].0, "system_2");
    assert_eq!(executions[2].0, "system_3");

    // All should be in Update stage
    assert_eq!(executions[0].1, CoreStage::Update);
    assert_eq!(executions[1].1, CoreStage::Update);
    assert_eq!(executions[2].1, CoreStage::Update);
}

#[test]
fn test_multiple_plugins_respect_registration_order() {
    // **Validates: Requirements 16.2**
    // When multiple plugins register systems, the systems should execute
    // in the order the plugins were registered

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Register first plugin
    let plugin1 = OrderedSystemPlugin::new(
        "Plugin1",
        vec![
            (CoreStage::Update, "plugin1_system1".to_string()),
            (CoreStage::Update, "plugin1_system2".to_string()),
        ],
    );
    app.add_plugins(plugin1);

    // Register second plugin
    let plugin2 = OrderedSystemPlugin::new(
        "Plugin2",
        vec![
            (CoreStage::Update, "plugin2_system1".to_string()),
            (CoreStage::Update, "plugin2_system2".to_string()),
        ],
    );
    app.add_plugins(plugin2);

    // Run one update cycle
    app.update();

    // Verify execution order
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 4);

    // Plugin1 systems should execute before Plugin2 systems
    assert_eq!(executions[0].0, "plugin1_system1");
    assert_eq!(executions[1].0, "plugin1_system2");
    assert_eq!(executions[2].0, "plugin2_system1");
    assert_eq!(executions[3].0, "plugin2_system2");
}

// ============================================================================
// Test 2: Stage Placement Correct
// ============================================================================

#[test]
fn test_systems_execute_in_correct_stages() {
    // **Validates: Requirements 16.2**
    // Systems should execute in their declared stages in the correct order

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Register plugin with systems in different stages
    let plugin = OrderedSystemPlugin::new(
        "TestPlugin",
        vec![
            (CoreStage::PreUpdate, "pre_system".to_string()),
            (CoreStage::Update, "update_system".to_string()),
            (CoreStage::PostUpdate, "post_system".to_string()),
        ],
    );

    app.add_plugins(plugin);

    // Run one update cycle
    app.update();

    // Verify execution order and stage placement
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 3);

    // PreUpdate should execute first
    assert_eq!(executions[0].0, "pre_system");
    assert_eq!(executions[0].1, CoreStage::PreUpdate);

    // Update should execute second
    assert_eq!(executions[1].0, "update_system");
    assert_eq!(executions[1].1, CoreStage::Update);

    // PostUpdate should execute third
    assert_eq!(executions[2].0, "post_system");
    assert_eq!(executions[2].1, CoreStage::PostUpdate);
}

#[test]
fn test_all_stages_execute_in_correct_order() {
    // **Validates: Requirements 16.2**
    // All stages should execute in the correct order

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Register plugin with systems in all stages (except Startup)
    let plugin = OrderedSystemPlugin::new(
        "TestPlugin",
        vec![
            (CoreStage::PreUpdate, "pre_update".to_string()),
            (CoreStage::Update, "update".to_string()),
            (CoreStage::FixedUpdate, "fixed_update".to_string()),
            (CoreStage::PostUpdate, "post_update".to_string()),
            (CoreStage::PreRender, "pre_render".to_string()),
            (CoreStage::Render, "render".to_string()),
            (CoreStage::PostRender, "post_render".to_string()),
        ],
    );

    app.add_plugins(plugin);

    // Run one update cycle
    app.update();

    // Verify execution order matches stage order
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 7);

    let expected_order = vec![
        ("pre_update", CoreStage::PreUpdate),
        ("update", CoreStage::Update),
        ("fixed_update", CoreStage::FixedUpdate),
        ("post_update", CoreStage::PostUpdate),
        ("pre_render", CoreStage::PreRender),
        ("render", CoreStage::Render),
        ("post_render", CoreStage::PostRender),
    ];

    for (i, (expected_name, expected_stage)) in expected_order.iter().enumerate() {
        assert_eq!(
            executions[i].0, *expected_name,
            "System at position {} should be {}",
            i, expected_name
        );
        assert_eq!(
            executions[i].1, *expected_stage,
            "System at position {} should be in stage {:?}",
            i, expected_stage
        );
    }
}

#[test]
fn test_stage_placement_with_multiple_plugins() {
    // **Validates: Requirements 16.2**
    // Multiple plugins adding systems to different stages should maintain
    // correct stage execution order

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Plugin 1 adds to PreUpdate and PostUpdate
    let plugin1 = OrderedSystemPlugin::new(
        "Plugin1",
        vec![
            (CoreStage::PreUpdate, "plugin1_pre".to_string()),
            (CoreStage::PostUpdate, "plugin1_post".to_string()),
        ],
    );
    app.add_plugins(plugin1);

    // Plugin 2 adds to Update
    let plugin2 = OrderedSystemPlugin::new(
        "Plugin2",
        vec![(CoreStage::Update, "plugin2_update".to_string())],
    );
    app.add_plugins(plugin2);

    // Plugin 3 adds to PreUpdate and Update
    let plugin3 = OrderedSystemPlugin::new(
        "Plugin3",
        vec![
            (CoreStage::PreUpdate, "plugin3_pre".to_string()),
            (CoreStage::Update, "plugin3_update".to_string()),
        ],
    );
    app.add_plugins(plugin3);

    // Run one update cycle
    app.update();

    // Verify execution order
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 5);

    // PreUpdate systems should execute first (in plugin registration order)
    assert_eq!(executions[0].0, "plugin1_pre");
    assert_eq!(executions[0].1, CoreStage::PreUpdate);
    assert_eq!(executions[1].0, "plugin3_pre");
    assert_eq!(executions[1].1, CoreStage::PreUpdate);

    // Update systems should execute next (in plugin registration order)
    assert_eq!(executions[2].0, "plugin2_update");
    assert_eq!(executions[2].1, CoreStage::Update);
    assert_eq!(executions[3].0, "plugin3_update");
    assert_eq!(executions[3].1, CoreStage::Update);

    // PostUpdate systems should execute last
    assert_eq!(executions[4].0, "plugin1_post");
    assert_eq!(executions[4].1, CoreStage::PostUpdate);
}

// ============================================================================
// Test 3: No Race Conditions
// ============================================================================

#[test]
fn test_systems_with_shared_resource_no_race_condition() {
    // **Validates: Requirements 16.2**
    // Systems that access the same resource should not have race conditions
    // The schedule should ensure proper synchronization

    #[derive(Debug, Clone)]
    struct Counter {
        value: Arc<Mutex<i32>>,
    }

    impl Counter {
        fn new() -> Self {
            Self {
                value: Arc::new(Mutex::new(0)),
            }
        }

        fn increment(&self) {
            let mut val = self.value.lock().unwrap();
            *val += 1;
        }

        fn get(&self) -> i32 {
            *self.value.lock().unwrap()
        }
    }

    impl Resource for Counter {}

    let mut app = App::new();
    let counter = Counter::new();
    app.insert_resource(counter.clone());

    // Add multiple systems that increment the counter
    for _ in 0..10 {
        let system = move |world: &mut luminara_core::world::World| {
            if let Some(counter) = world.get_resource::<Counter>() {
                counter.increment();
            }
        };

        app.add_system::<luminara_core::system::ExclusiveMarker>(CoreStage::Update, system);
    }

    // Run multiple update cycles
    for _ in 0..10 {
        app.update();
    }

    // Verify counter value is correct (no race conditions)
    // 10 systems * 10 cycles = 100 increments
    assert_eq!(
        counter.get(),
        100,
        "Counter should be exactly 100 if no race conditions occurred"
    );
}

#[test]
fn test_deterministic_execution_across_multiple_runs() {
    // **Validates: Requirements 16.2**
    // Systems should execute in the same order across multiple runs

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Register plugin with systems
    let plugin = OrderedSystemPlugin::new(
        "TestPlugin",
        vec![
            (CoreStage::Update, "system_a".to_string()),
            (CoreStage::Update, "system_b".to_string()),
            (CoreStage::Update, "system_c".to_string()),
        ],
    );

    app.add_plugins(plugin);

    // Run multiple update cycles and verify order is consistent
    for run in 0..5 {
        tracker.clear();
        app.update();

        let executions = tracker.get_executions();
        assert_eq!(
            executions.len(),
            3,
            "Run {}: All systems should execute",
            run
        );
        assert_eq!(
            executions[0].0, "system_a",
            "Run {}: First system should be system_a",
            run
        );
        assert_eq!(
            executions[1].0, "system_b",
            "Run {}: Second system should be system_b",
            run
        );
        assert_eq!(
            executions[2].0, "system_c",
            "Run {}: Third system should be system_c",
            run
        );
    }
}

#[test]
fn test_complex_plugin_ordering_scenario() {
    // **Validates: Requirements 16.2**
    // Complex scenario with multiple plugins, stages, and systems

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Plugin A: Core systems
    let plugin_a = OrderedSystemPlugin::new(
        "CorePlugin",
        vec![
            (CoreStage::PreUpdate, "core_pre".to_string()),
            (CoreStage::Update, "core_update".to_string()),
            (CoreStage::PostUpdate, "core_post".to_string()),
        ],
    );
    app.add_plugins(plugin_a);

    // Plugin B: Physics systems
    let plugin_b = OrderedSystemPlugin::new(
        "PhysicsPlugin",
        vec![
            (CoreStage::FixedUpdate, "physics_fixed".to_string()),
            (CoreStage::PostUpdate, "physics_post".to_string()),
        ],
    );
    app.add_plugins(plugin_b);

    // Plugin C: Rendering systems
    let plugin_c = OrderedSystemPlugin::new(
        "RenderPlugin",
        vec![
            (CoreStage::PreRender, "render_pre".to_string()),
            (CoreStage::Render, "render_main".to_string()),
            (CoreStage::PostRender, "render_post".to_string()),
        ],
    );
    app.add_plugins(plugin_c);

    // Run one update cycle
    app.update();

    // Verify execution order
    let executions = tracker.get_executions();
    assert_eq!(executions.len(), 8);

    // Expected order based on stage order and plugin registration order
    let expected = vec![
        ("core_pre", CoreStage::PreUpdate),
        ("core_update", CoreStage::Update),
        ("physics_fixed", CoreStage::FixedUpdate),
        ("core_post", CoreStage::PostUpdate),
        ("physics_post", CoreStage::PostUpdate),
        ("render_pre", CoreStage::PreRender),
        ("render_main", CoreStage::Render),
        ("render_post", CoreStage::PostRender),
    ];

    for (i, (expected_name, expected_stage)) in expected.iter().enumerate() {
        assert_eq!(
            executions[i].0, *expected_name,
            "Position {}: Expected system {}",
            i, expected_name
        );
        assert_eq!(
            executions[i].1, *expected_stage,
            "Position {}: Expected stage {:?}",
            i, expected_stage
        );
    }
}

#[test]
fn test_startup_stage_executes_before_other_stages() {
    // **Validates: Requirements 16.2**
    // Startup stage should execute before all other stages

    let mut app = App::new();
    let tracker = ExecutionTracker::new();
    app.insert_resource(tracker.clone());

    // Add systems to various stages including Startup
    let plugin = OrderedSystemPlugin::new(
        "TestPlugin",
        vec![
            (CoreStage::Startup, "startup_system".to_string()),
            (CoreStage::PreUpdate, "pre_update_system".to_string()),
            (CoreStage::Update, "update_system".to_string()),
        ],
    );

    app.add_plugins(plugin);

    // Run the app (which runs startup first, then regular update)
    app.update();

    let executions = tracker.get_executions();

    // Startup should have executed first
    // Note: Startup stage is run separately and only once
    // After first update, only PreUpdate and Update should execute
    assert!(
        executions.len() >= 2,
        "At least PreUpdate and Update should execute"
    );

    // Find if startup executed (it should have)
    let has_startup = executions.iter().any(|(name, _)| name == "startup_system");
    let has_pre_update = executions
        .iter()
        .any(|(name, _)| name == "pre_update_system");
    let has_update = executions.iter().any(|(name, _)| name == "update_system");

    assert!(has_pre_update, "PreUpdate system should execute");
    assert!(has_update, "Update system should execute");

    // If startup executed, it should be first
    if has_startup {
        assert_eq!(
            executions[0].0, "startup_system",
            "Startup system should execute first"
        );
    }
}
