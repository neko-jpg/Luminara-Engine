// ... imports ...
use luminara_core::world::World;
use crate::intent_resolver::AiIntent;
use std::collections::HashMap;

pub struct PerformanceAdvisor {
    metrics: PerformanceMetrics,
    cost_model: CostModel,
    budget: PerformanceBudget,
}

#[derive(Default, Clone)]
struct PerformanceMetrics {
    current_fps: f32,
    frame_time_ms: f32,
    entity_count: usize,
    draw_calls: usize,
    memory_usage_mb: f32,
}

#[derive(Default, Clone)]
struct CostModel {
    base_entity_cost_ms: f32,
    component_costs: HashMap<String, f32>,
    spawn_overhead_ms: f32,
}

struct PerformanceBudget {
    min_fps: f32,
    max_frame_time_ms: f32,
}

#[derive(Debug)]
pub struct PerformanceImpact {
    pub severity: ImpactSeverity,
    pub predicted_fps: f32,
    pub message: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for PerformanceAdvisor {
    fn default() -> Self {
        Self {
            metrics: PerformanceMetrics {
                current_fps: 60.0,
                frame_time_ms: 16.6,
                entity_count: 0,
                draw_calls: 0,
                memory_usage_mb: 0.0,
            },
            cost_model: CostModel {
                base_entity_cost_ms: 0.001,
                component_costs: HashMap::new(),
                spawn_overhead_ms: 0.05,
            },
            budget: PerformanceBudget {
                min_fps: 30.0,
                max_frame_time_ms: 33.3,
            },
        }
    }
}

impl PerformanceAdvisor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_metrics(&mut self, world: &World, fps: f32) {
        self.metrics.current_fps = fps;
        self.metrics.frame_time_ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };
        // Use public API `entities().len()`. `entities_count` is on `EntityAllocator` but accessor returns Vec<Entity>.
        // Wait, `world.entities()` returns `Vec<Entity>`.
        // So `world.entities().len()` is correct.
        self.metrics.entity_count = world.entities().len();
    }

    pub fn estimate_impact(&self, intent: &AiIntent, _world: &World) -> PerformanceImpact {
        let mut cost_ms = 0.0;

        match intent {
            AiIntent::SpawnRelative { .. } => {
                cost_ms += self.cost_model.spawn_overhead_ms + self.cost_model.base_entity_cost_ms;
            },
            _ => {}
        }

        let new_frame_time = self.metrics.frame_time_ms + cost_ms;
        let predicted_fps = if new_frame_time > 0.0 { 1000.0 / new_frame_time } else { 0.0 };

        let severity = if predicted_fps < self.budget.min_fps {
            ImpactSeverity::Critical
        } else if predicted_fps < 45.0 {
            ImpactSeverity::High
        } else if cost_ms > 1.0 {
            ImpactSeverity::Medium
        } else {
            ImpactSeverity::Low
        };

        let mut suggestions = Vec::new();
        if severity == ImpactSeverity::Critical {
            suggestions.push("Operation too expensive. Reduce count or optimize.".into());
        }

        PerformanceImpact {
            severity,
            predicted_fps,
            message: format!("Estimated cost: {:.4}ms", cost_ms),
            suggestions,
        }
    }

    pub fn generate_context(&self) -> String {
        format!(
            "Performance: FPS {:.1}, Entities {}, FrameTime {:.2}ms",
            self.metrics.current_fps,
            self.metrics.entity_count,
            self.metrics.frame_time_ms
        )
    }

    pub fn learn_from_measurement(&mut self, _intent: &AiIntent, _actual_cost_ms: f32) {
        // Placeholder for learning logic
    }
}
