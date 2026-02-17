use crate::intent_resolver::AiIntent;
use luminara_core::world::World;
use std::collections::HashMap;

/// Performance Advisor for AI-driven performance-aware decision making
/// 
/// Predicts FPS impact of operations, maintains a cost model, and provides
/// optimization suggestions. Learns from actual measurements to improve accuracy.
pub struct PerformanceAdvisor {
    metrics: PerformanceMetrics,
    cost_model: CostModel,
    budget: PerformanceBudget,
    history: Vec<Measurement>,
}

#[derive(Default, Clone)]
struct PerformanceMetrics {
    current_fps: f32,
    frame_time_ms: f32,
    entity_count: usize,
    draw_calls: usize,
    memory_usage_mb: f32,
}

/// Cost model tracking CPU, GPU, and memory costs per component
#[derive(Clone)]
struct CostModel {
    base_entity_cost_ms: f32,
    component_costs: HashMap<String, ComponentCost>,
    spawn_overhead_ms: f32,
    learning_rate: f32,
}

/// Per-component cost breakdown
#[derive(Clone, Debug)]
pub struct ComponentCost {
    pub cpu_cost_ms: f32,
    pub gpu_cost_ms: f32,
    pub memory_bytes: usize,
}

struct PerformanceBudget {
    min_fps: f32,
    max_frame_time_ms: f32,
    warning_threshold_fps: f32,
}

/// Measurement record for learning
struct Measurement {
    intent_type: String,
    predicted_cost_ms: f32,
    actual_cost_ms: f32,
    entity_count: usize,
}

#[derive(Debug)]
pub struct PerformanceImpact {
    pub severity: ImpactSeverity,
    pub predicted_fps: f32,
    pub predicted_cost_ms: f32,
    pub message: String,
    pub suggestions: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for CostModel {
    fn default() -> Self {
        let mut component_costs = HashMap::new();
        
        // Initialize default costs for common components
        component_costs.insert("Transform".to_string(), ComponentCost {
            cpu_cost_ms: 0.0005,
            gpu_cost_ms: 0.0,
            memory_bytes: 48, // Vec3 * 3
        });
        
        component_costs.insert("Mesh".to_string(), ComponentCost {
            cpu_cost_ms: 0.001,
            gpu_cost_ms: 0.05,
            memory_bytes: 1024,
        });
        
        component_costs.insert("Material".to_string(), ComponentCost {
            cpu_cost_ms: 0.0005,
            gpu_cost_ms: 0.01,
            memory_bytes: 256,
        });
        
        component_costs.insert("Light".to_string(), ComponentCost {
            cpu_cost_ms: 0.002,
            gpu_cost_ms: 0.1,
            memory_bytes: 128,
        });
        
        component_costs.insert("RigidBody".to_string(), ComponentCost {
            cpu_cost_ms: 0.01,
            gpu_cost_ms: 0.0,
            memory_bytes: 512,
        });
        
        component_costs.insert("Collider".to_string(), ComponentCost {
            cpu_cost_ms: 0.005,
            gpu_cost_ms: 0.0,
            memory_bytes: 256,
        });
        
        Self {
            base_entity_cost_ms: 0.001,
            component_costs,
            spawn_overhead_ms: 0.05,
            learning_rate: 0.1, // 10% learning rate for cost model updates
        }
    }
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
            cost_model: CostModel::default(),
            budget: PerformanceBudget {
                min_fps: 30.0,
                max_frame_time_ms: 33.3,
                warning_threshold_fps: 30.0,
            },
            history: Vec::new(),
        }
    }
}

impl PerformanceAdvisor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update current performance metrics from the world
    pub fn update_metrics(&mut self, world: &World, fps: f32) {
        self.metrics.current_fps = fps;
        self.metrics.frame_time_ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };
        self.metrics.entity_count = world.entities().len();
        // Note: draw_calls and memory_usage_mb would be updated from rendering system
    }
    
    /// Update rendering-specific metrics
    pub fn update_render_metrics(&mut self, draw_calls: usize, memory_mb: f32) {
        self.metrics.draw_calls = draw_calls;
        self.metrics.memory_usage_mb = memory_mb;
    }

    /// Estimate performance impact of an AI intent
    /// 
    /// Predicts FPS change based on component costs and provides warnings
    /// when FPS drops below 30.
    pub fn estimate_impact(&self, intent: &AiIntent, world: &World) -> PerformanceImpact {
        let mut cost_ms = 0.0;
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Calculate cost based on intent type
        match intent {
            AiIntent::SpawnRelative { template, .. } => {
                // Base spawn cost
                cost_ms += self.cost_model.spawn_overhead_ms + self.cost_model.base_entity_cost_ms;
                
                // Estimate component costs based on template
                // In a real implementation, we'd parse the template to determine components
                // For now, use a conservative estimate
                cost_ms += 0.01; // Assume average entity with a few components
            }
            AiIntent::ModifyMatching => {
                // Modification is generally cheaper than spawning
                cost_ms += 0.005;
            }
            AiIntent::AttachBehavior => {
                // Attaching behavior (script) has moderate cost
                cost_ms += 0.01;
            }
        }

        // Predict new FPS
        let new_frame_time = self.metrics.frame_time_ms + cost_ms;
        let predicted_fps = if new_frame_time > 0.0 {
            1000.0 / new_frame_time
        } else {
            self.metrics.current_fps
        };

        // Determine severity
        let severity = if predicted_fps < self.budget.min_fps {
            ImpactSeverity::Critical
        } else if predicted_fps < 45.0 {
            ImpactSeverity::High
        } else if cost_ms > 1.0 {
            ImpactSeverity::Medium
        } else {
            ImpactSeverity::Low
        };

        // Generate warnings for FPS drops below 30
        if predicted_fps < self.budget.warning_threshold_fps {
            warnings.push(format!(
                "WARNING: Predicted FPS ({:.1}) drops below minimum threshold ({:.1})",
                predicted_fps, self.budget.warning_threshold_fps
            ));
            
            // Provide optimization suggestions
            suggestions.extend(self.generate_optimization_suggestions(intent, world));
        }

        // Additional suggestions based on severity
        if severity == ImpactSeverity::Critical {
            suggestions.push("Operation too expensive. Consider reducing entity count or simplifying components.".into());
        } else if severity == ImpactSeverity::High {
            suggestions.push("Operation may impact performance. Monitor frame time after execution.".into());
        }

        PerformanceImpact {
            severity,
            predicted_fps,
            predicted_cost_ms: cost_ms,
            message: format!(
                "Estimated cost: {:.4}ms, Predicted FPS: {:.1} (current: {:.1})",
                cost_ms, predicted_fps, self.metrics.current_fps
            ),
            suggestions,
            warnings,
        }
    }

    /// Generate optimization suggestions based on current state
    fn generate_optimization_suggestions(&self, intent: &AiIntent, _world: &World) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Check entity count
        if self.metrics.entity_count > 5000 {
            suggestions.push("Consider using GPU instancing for repeated meshes to reduce draw calls.".into());
        }
        
        if self.metrics.entity_count > 10000 {
            suggestions.push("Implement LOD (Level of Detail) system to reduce complexity of distant objects.".into());
        }
        
        // Check draw calls
        if self.metrics.draw_calls > 500 {
            suggestions.push("High draw call count detected. Consider batching materials or using instancing.".into());
        }
        
        // Intent-specific suggestions
        match intent {
            AiIntent::SpawnRelative { .. } => {
                if self.metrics.entity_count > 1000 {
                    suggestions.push("Many entities already exist. Consider reusing existing entities or using object pooling.".into());
                }
            }
            AiIntent::AttachBehavior => {
                suggestions.push("Scripts have runtime overhead. Consider using native components for performance-critical logic.".into());
            }
            _ => {}
        }
        
        // Memory-based suggestions
        if self.metrics.memory_usage_mb > 1000.0 {
            suggestions.push("High memory usage. Consider unloading unused assets or reducing texture resolution.".into());
        }
        
        suggestions
    }

    /// Generate performance context for AI
    pub fn generate_context(&self) -> String {
        format!(
            "Performance: FPS {:.1}, Entities {}, FrameTime {:.2}ms, DrawCalls {}, Memory {:.1}MB",
            self.metrics.current_fps, 
            self.metrics.entity_count, 
            self.metrics.frame_time_ms,
            self.metrics.draw_calls,
            self.metrics.memory_usage_mb
        )
    }

    /// Learn from actual measurements to improve cost model accuracy
    /// 
    /// Uses exponential moving average to update component costs based on
    /// the difference between predicted and actual performance impact.
    pub fn learn_from_measurement(&mut self, intent: &AiIntent, actual_cost_ms: f32) {
        // Record measurement
        let intent_type = match intent {
            AiIntent::SpawnRelative { .. } => "SpawnRelative",
            AiIntent::ModifyMatching => "ModifyMatching",
            AiIntent::AttachBehavior => "AttachBehavior",
        }.to_string();
        
        // Calculate predicted cost for comparison
        let predicted_cost_ms = match intent {
            AiIntent::SpawnRelative { .. } => {
                self.cost_model.spawn_overhead_ms + self.cost_model.base_entity_cost_ms + 0.01
            }
            AiIntent::ModifyMatching => 0.005,
            AiIntent::AttachBehavior => 0.01,
        };
        
        self.history.push(Measurement {
            intent_type: intent_type.clone(),
            predicted_cost_ms,
            actual_cost_ms,
            entity_count: self.metrics.entity_count,
        });
        
        // Keep history bounded
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        
        // Update cost model using exponential moving average
        let error = actual_cost_ms - predicted_cost_ms;
        
        match intent {
            AiIntent::SpawnRelative { .. } => {
                // Adjust spawn overhead based on error
                let overhead_adjustment = error * 0.5 * self.cost_model.learning_rate;
                self.cost_model.spawn_overhead_ms = (self.cost_model.spawn_overhead_ms + overhead_adjustment).max(0.0);
                
                // Adjust base entity cost
                let base_adjustment = error * 0.5 * self.cost_model.learning_rate;
                self.cost_model.base_entity_cost_ms = (self.cost_model.base_entity_cost_ms + base_adjustment).max(0.0);
            }
            _ => {}
        }
    }
    
    /// Get prediction accuracy statistics
    pub fn get_accuracy_stats(&self) -> AccuracyStats {
        if self.history.is_empty() {
            return AccuracyStats {
                mean_absolute_error: 0.0,
                mean_percentage_error: 0.0,
                sample_count: 0,
            };
        }
        
        let mut total_absolute_error = 0.0;
        let mut total_percentage_error = 0.0;
        
        for measurement in &self.history {
            let absolute_error = (measurement.actual_cost_ms - measurement.predicted_cost_ms).abs();
            total_absolute_error += absolute_error;
            
            if measurement.actual_cost_ms > 0.0 {
                let percentage_error = (absolute_error / measurement.actual_cost_ms) * 100.0;
                total_percentage_error += percentage_error;
            }
        }
        
        AccuracyStats {
            mean_absolute_error: total_absolute_error / self.history.len() as f32,
            mean_percentage_error: total_percentage_error / self.history.len() as f32,
            sample_count: self.history.len(),
        }
    }
    
    /// Get component cost information (for debugging/tuning)
    pub fn get_component_cost(&self, component_name: &str) -> Option<&ComponentCost> {
        self.cost_model.component_costs.get(component_name)
    }
}

/// Statistics about prediction accuracy
#[derive(Debug, Clone)]
pub struct AccuracyStats {
    pub mean_absolute_error: f32,
    pub mean_percentage_error: f32,
    pub sample_count: usize,
}
