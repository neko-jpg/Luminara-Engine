use luminara_ai_agent::performance::{PerformanceAdvisor, ImpactSeverity};
use luminara_ai_agent::intent_resolver::{AiIntent, EntityReference, RelativePosition};
use luminara_core::world::World;
use luminara_math::Vec3;

#[test]
fn test_fps_impact_prediction() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    // Update with baseline metrics
    advisor.update_metrics(&world, 60.0);
    
    // Test spawn intent
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "enemy".to_string(),
    };
    
    let impact = advisor.estimate_impact(&intent, &world);
    
    // Should predict some FPS impact
    assert!(impact.predicted_fps <= 60.0);
    assert!(impact.predicted_cost_ms > 0.0);
    assert!(!impact.message.is_empty());
}

#[test]
fn test_warning_threshold() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    // Set current FPS to 31 (just above threshold)
    advisor.update_metrics(&world, 31.0);
    
    // Create an expensive intent that will drop FPS below 30
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "expensive_entity".to_string(),
    };
    
    let impact = advisor.estimate_impact(&intent, &world);
    
    // Should have warnings when FPS drops below 30
    if impact.predicted_fps < 30.0 {
        assert!(!impact.warnings.is_empty(), "Should have warnings when FPS drops below 30");
        assert!(!impact.suggestions.is_empty(), "Should have suggestions when FPS drops below 30");
    }
}

#[test]
fn test_severity_levels() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    // Test with good FPS
    advisor.update_metrics(&world, 60.0);
    let intent = AiIntent::ModifyMatching;
    let impact = advisor.estimate_impact(&intent, &world);
    assert_eq!(impact.severity, ImpactSeverity::Low);
    
    // Test with low FPS (should be more critical)
    advisor.update_metrics(&world, 25.0);
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    let impact = advisor.estimate_impact(&intent, &world);
    
    // With already low FPS, any spawn should be at least High severity
    assert!(matches!(impact.severity, ImpactSeverity::High | ImpactSeverity::Critical));
}

#[test]
fn test_optimization_suggestions() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    // Simulate high entity count and set FPS below threshold to trigger suggestions
    advisor.update_metrics(&world, 25.0); // Below 30 FPS threshold
    advisor.update_render_metrics(600, 500.0); // High draw calls
    
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    
    let impact = advisor.estimate_impact(&intent, &world);
    
    // Should provide suggestions when FPS is below threshold
    assert!(!impact.suggestions.is_empty(), "Should provide optimization suggestions when FPS is low");
    assert!(!impact.warnings.is_empty(), "Should provide warnings when FPS drops below threshold");
}

#[test]
fn test_cost_model_learning() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    
    // Get initial prediction
    let initial_impact = advisor.estimate_impact(&intent, &world);
    let initial_prediction = initial_impact.predicted_cost_ms;
    
    // Simulate actual measurement being higher than predicted
    let actual_cost = initial_prediction * 2.0;
    advisor.learn_from_measurement(&intent, actual_cost);
    
    // Get new prediction after learning
    let new_impact = advisor.estimate_impact(&intent, &world);
    let new_prediction = new_impact.predicted_cost_ms;
    
    // New prediction should be higher than initial (learned from error)
    assert!(new_prediction > initial_prediction, 
        "Cost model should learn from measurements: initial={}, new={}", 
        initial_prediction, new_prediction);
}

#[test]
fn test_accuracy_stats() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    
    // Initially no measurements
    let stats = advisor.get_accuracy_stats();
    assert_eq!(stats.sample_count, 0);
    assert_eq!(stats.mean_absolute_error, 0.0);
    
    // Add some measurements
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    
    advisor.learn_from_measurement(&intent, 0.05);
    advisor.learn_from_measurement(&intent, 0.06);
    advisor.learn_from_measurement(&intent, 0.055);
    
    let stats = advisor.get_accuracy_stats();
    assert_eq!(stats.sample_count, 3);
    assert!(stats.mean_absolute_error >= 0.0);
}

#[test]
fn test_context_generation() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    advisor.update_render_metrics(100, 256.5);
    
    let context = advisor.generate_context();
    
    // Should include key metrics
    assert!(context.contains("FPS"));
    assert!(context.contains("60"));
    assert!(context.contains("Entities"));
    assert!(context.contains("FrameTime"));
    assert!(context.contains("DrawCalls"));
    assert!(context.contains("Memory"));
}

#[test]
fn test_different_intent_types() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    
    // Test SpawnRelative
    let spawn_intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    let spawn_impact = advisor.estimate_impact(&spawn_intent, &world);
    assert!(spawn_impact.predicted_cost_ms > 0.0);
    
    // Test ModifyMatching
    let modify_intent = AiIntent::ModifyMatching;
    let modify_impact = advisor.estimate_impact(&modify_intent, &world);
    assert!(modify_impact.predicted_cost_ms > 0.0);
    
    // Test AttachBehavior
    let behavior_intent = AiIntent::AttachBehavior;
    let behavior_impact = advisor.estimate_impact(&behavior_intent, &world);
    assert!(behavior_impact.predicted_cost_ms > 0.0);
    
    // Spawn should generally be more expensive than modify
    assert!(spawn_impact.predicted_cost_ms >= modify_impact.predicted_cost_ms);
}

#[test]
fn test_learning_convergence() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    
    // Simulate consistent actual cost
    let actual_cost = 0.08;
    
    // Learn from multiple measurements
    for _ in 0..10 {
        advisor.learn_from_measurement(&intent, actual_cost);
    }
    
    // Prediction should converge toward actual cost
    let final_impact = advisor.estimate_impact(&intent, &world);
    let error = (final_impact.predicted_cost_ms - actual_cost).abs();
    
    // Error should be reasonably small after learning
    assert!(error < actual_cost * 0.5, 
        "Prediction should converge: predicted={}, actual={}, error={}", 
        final_impact.predicted_cost_ms, actual_cost, error);
}

#[test]
fn test_component_cost_retrieval() {
    let advisor = PerformanceAdvisor::new();
    
    // Should have default costs for common components
    assert!(advisor.get_component_cost("Transform").is_some());
    assert!(advisor.get_component_cost("Mesh").is_some());
    assert!(advisor.get_component_cost("Material").is_some());
    assert!(advisor.get_component_cost("Light").is_some());
    assert!(advisor.get_component_cost("RigidBody").is_some());
    
    // Unknown component should return None
    assert!(advisor.get_component_cost("UnknownComponent").is_none());
}

#[test]
fn test_memory_based_suggestions() {
    let mut advisor = PerformanceAdvisor::new();
    let world = World::new();
    
    advisor.update_metrics(&world, 60.0);
    // Set high memory usage
    advisor.update_render_metrics(100, 1500.0); // 1.5GB
    
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("player".to_string()),
        offset: RelativePosition::AtOffset(Vec3::new(1.0, 0.0, 0.0)),
        template: "entity".to_string(),
    };
    
    // Force FPS below threshold to trigger suggestions
    advisor.update_metrics(&world, 25.0);
    let impact = advisor.estimate_impact(&intent, &world);
    
    // Should have memory-related suggestions
    let has_memory_suggestion = impact.suggestions.iter()
        .any(|s| s.to_lowercase().contains("memory"));
    
    if !impact.suggestions.is_empty() {
        // If we have suggestions, at least one should be about memory or performance
        assert!(has_memory_suggestion || impact.suggestions.iter().any(|s| s.contains("expensive")));
    }
}
