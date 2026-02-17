# AI Agent System Architecture

## Overview

Luminara Engine features an integrated AI agent system that assists developers through natural language interaction. The AI can understand game state, generate code, make modifications, and provide performance advice - all while maintaining safety through comprehensive verification.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      AI Interface Layer                       │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │    MCP     │  │  Natural   │  │   Visual Feedback    │  │
│  │   Server   │  │  Language  │  │   (Multimodal)       │  │
│  └──────┬─────┘  └──────┬─────┘  └──────────┬───────────┘  │
├─────────┴────────────────┴────────────────────┴──────────────┤
│                      Context Engine                           │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │ Hierarchical│  │  Semantic  │  │  Intent Resolver     │  │
│  │   Digest   │  │   Index    │  │                      │  │
│  └──────┬─────┘  └──────┬─────┘  └──────────┬───────────┘  │
├─────────┴────────────────┴────────────────────┴──────────────┤
│                      Verification Layer                       │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │   Static   │  │  Sandbox   │  │   Dry Run            │  │
│  │  Analysis  │  │  Execution │  │   Verification       │  │
│  └──────┬─────┘  └──────┬─────┘  └──────────┬───────────┘  │
├─────────┴────────────────┴────────────────────┴──────────────┤
│                      Execution Layer                          │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Operation │  │  Command   │  │  Performance         │  │
│  │  Timeline  │  │  Execution │  │  Monitoring          │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Context Engine

Provides AI with understanding of game state within token budgets.

#### Hierarchical World Digest

```rust
pub struct WorldDigest {
    /// L0: High-level summary
    pub summary: WorldSummary,
    /// L1: Entity catalog
    pub catalog: EntityCatalog,
    /// L2: Detailed entity data
    pub details: HashMap<Entity, EntityDetails>,
    /// L3: Full component data
    pub full_data: HashMap<Entity, ComponentData>,
}

pub struct WorldSummary {
    pub entity_count: usize,
    pub archetype_distribution: HashMap<String, usize>,
    pub scene_bounds: Aabb,
    pub active_systems: Vec<String>,
}

// Token budget allocation:
// L0: ~100 tokens (always included)
// L1: ~500 tokens (entity list)
// L2: ~2000 tokens (selected entities)
// L3: ~5000 tokens (full details)
```

**Usage:**
```rust
// Generate digest for AI
let digest = context_engine.generate_digest(&world, DigestLevel::L2);

// AI receives:
// "Scene contains 1,247 entities:
//  - 523 with [Transform, Mesh, Material] (renderable objects)
//  - 342 with [Transform, RigidBody, Collider] (physics objects)
//  - 89 with [Transform, Light] (lights)
//  - ...
//  
//  Selected entities:
//  - Entity(42): Player at (10.5, 0.0, 3.2), health: 85/100
//  - Entity(108): Enemy at (15.0, 0.0, 8.0), state: Patrolling
//  - ..."
```

#### Semantic Entity Index

```rust
pub struct SemanticIndex {
    /// Vector embeddings for entities
    embeddings: HashMap<Entity, Vec<f32>>,
    /// Reverse index: description → entities
    description_index: HashMap<String, Vec<Entity>>,
}

impl SemanticIndex {
    /// Find entities by natural language query
    pub fn search(&self, query: &str, limit: usize) -> Vec<(Entity, f32)> {
        // Convert query to embedding
        let query_embedding = self.embed(query);
        
        // Cosine similarity search
        let mut results = Vec::new();
        for (entity, embedding) in &self.embeddings {
            let similarity = cosine_similarity(&query_embedding, embedding);
            results.push((*entity, similarity));
        }
        
        // Sort by similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        results
    }
}

// Usage:
// AI: "Find the red car"
// → search("red car", 10)
// → Returns entities with high similarity to "red car"
```

**Performance:**
- Index update: <50ms for 10,000 entities
- Search: <10ms for 10,000 entities
- Incremental updates (only changed entities)

#### Intent Resolver

Resolves entity references that may become invalid.

```rust
pub enum EntityReference {
    ById(Entity),
    ByName(String),
    ByTag(String),
    ByComponent(String),
    Nearest { to: Vec3, filter: ComponentFilter },
    Semantic { query: String },
}

impl IntentResolver {
    /// Resolve reference to actual entity
    pub fn resolve(&self, reference: EntityReference, world: &World) -> Result<Entity> {
        match reference {
            EntityReference::ById(entity) => {
                if world.contains(entity) {
                    Ok(entity)
                } else {
                    Err(Error::EntityNotFound(entity))
                }
            }
            EntityReference::ByName(name) => {
                self.find_by_name(&name, world)
                    .ok_or_else(|| Error::EntityNotFoundByName(name))
            }
            EntityReference::Semantic { query } => {
                let results = self.semantic_index.search(&query, 1);
                results.first()
                    .map(|(entity, _)| *entity)
                    .ok_or_else(|| Error::NoMatchingEntity(query))
            }
            // ... other cases
        }
    }
}

// Usage:
// AI: "Move the player to the red building"
// → resolve(Semantic("player"))
// → resolve(Semantic("red building"))
// → execute move command
```

### 2. Code Verification Pipeline

Ensures AI-generated code is safe before execution.

#### Static Analysis

```rust
pub struct StaticAnalyzer {
    lua_analyzer: LuaAnalyzer,
    wasm_analyzer: WasmAnalyzer,
}

pub struct AnalysisResult {
    pub errors: Vec<AnalysisError>,
    pub warnings: Vec<AnalysisWarning>,
    pub complexity: ComplexityMetrics,
}

pub enum AnalysisError {
    InfiniteLoop { line: usize },
    UndefinedVariable { name: String, line: usize },
    TypeError { expected: String, found: String, line: usize },
    UnauthorizedApiCall { function: String, line: usize },
}

// Lua analysis:
// - Parse AST
// - Check for infinite loops (while true without break)
// - Verify variable declarations
// - Type inference where possible
// - API whitelist enforcement

// WASM analysis:
// - Validate module structure
// - Check import restrictions
// - Verify memory limits
// - Analyze control flow
```

**Example:**
```lua
-- AI-generated code
function on_update(entity, dt)
    while true do  -- ERROR: Infinite loop detected
        print("loop")
    end
end

-- Static analyzer catches this before execution
```

#### Sandbox Execution

```rust
pub struct Sandbox {
    limits: SandboxLimits,
    monitor: ExecutionMonitor,
}

pub struct SandboxLimits {
    pub max_execution_time: Duration,
    pub max_memory: usize,
    pub max_entity_spawns: usize,
    pub max_component_modifications: usize,
}

impl Sandbox {
    /// Execute code in sandbox
    pub fn execute(&mut self, code: &str, world: &mut World) -> Result<SandboxResult> {
        // Clone world for sandbox
        let mut sandbox_world = world.clone();
        
        // Execute with limits
        let start = Instant::now();
        let result = self.execute_with_limits(code, &mut sandbox_world)?;
        
        // Check limits
        if start.elapsed() > self.limits.max_execution_time {
            return Err(Error::ExecutionTimeout);
        }
        
        Ok(result)
    }
}

// Default limits:
// - Execution time: 5 seconds
// - Memory: 64MB
// - Entity spawns: 1000
// - Component modifications: 10000
```

#### Dry Run Verification

```rust
pub struct DryRunVerifier {
    snapshot: WorldSnapshot,
}

impl DryRunVerifier {
    /// Verify operation without applying changes
    pub fn verify(&mut self, operation: &Operation, world: &World) -> VerificationResult {
        // Take snapshot
        self.snapshot = world.snapshot();
        
        // Apply operation to snapshot
        let mut test_world = self.snapshot.clone();
        operation.execute(&mut test_world)?;
        
        // Generate diff
        let diff = self.snapshot.diff(&test_world);
        
        // Analyze changes
        let analysis = self.analyze_diff(&diff);
        
        VerificationResult {
            diff,
            analysis,
            safe: analysis.is_safe(),
        }
    }
}

pub struct VerificationResult {
    /// Changes that would be made
    pub diff: WorldDiff,
    /// Analysis of changes
    pub analysis: ChangeAnalysis,
    /// Whether changes are safe
    pub safe: bool,
}

// AI shows user:
// "This operation will:
//  - Modify 5 entities
//  - Add 2 components
//  - Remove 1 entity
//  - Estimated FPS impact: -2 FPS
//  
//  Proceed? [Yes/No]"
```

### 3. Operation Timeline

Git-like operation history with undo/redo.

```rust
pub struct OperationTimeline {
    /// Operation history
    history: Vec<Operation>,
    /// Current position
    current: usize,
    /// Branches
    branches: HashMap<String, usize>,
}

pub struct Operation {
    pub id: OperationId,
    pub timestamp: SystemTime,
    pub description: String,
    pub intent: String,
    pub commands: Vec<Box<dyn Command>>,
    pub inverse_commands: Vec<Box<dyn Command>>,
    pub affected_entities: Vec<Entity>,
    pub metadata: OperationMetadata,
}

impl OperationTimeline {
    /// Record operation
    pub fn record(&mut self, operation: Operation) {
        // Truncate future history if not at end
        self.history.truncate(self.current + 1);
        
        // Add operation
        self.history.push(operation);
        self.current += 1;
    }
    
    /// Undo operation
    pub fn undo(&mut self, world: &mut World) -> Result<()> {
        if self.current == 0 {
            return Err(Error::NothingToUndo);
        }
        
        let operation = &mut self.history[self.current];
        
        // Execute inverse commands
        for cmd in &mut operation.inverse_commands {
            cmd.execute(world)?;
        }
        
        self.current -= 1;
        Ok(())
    }
    
    /// Redo operation
    pub fn redo(&mut self, world: &mut World) -> Result<()> {
        if self.current >= self.history.len() - 1 {
            return Err(Error::NothingToRedo);
        }
        
        self.current += 1;
        let operation = &mut self.history[self.current];
        
        // Execute commands
        for cmd in &mut operation.commands {
            cmd.execute(world)?;
        }
        
        Ok(())
    }
    
    /// Create branch
    pub fn create_branch(&mut self, name: String) {
        self.branches.insert(name, self.current);
    }
    
    /// Switch branch
    pub fn switch_branch(&mut self, name: &str, world: &mut World) -> Result<()> {
        let target = self.branches.get(name)
            .ok_or_else(|| Error::BranchNotFound(name.to_string()))?;
        
        // Undo/redo to target position
        while self.current > *target {
            self.undo(world)?;
        }
        while self.current < *target {
            self.redo(world)?;
        }
        
        Ok(())
    }
}

// Usage:
// AI makes changes → recorded as operation
// User: "undo" → undo last operation
// User: "undo 5" → undo last 5 operations
// User: "create branch experiment" → save current state
// User: "switch branch experiment" → return to saved state
```

**Persistence:**
```rust
// Operations persisted to SurrealDB
// Survives engine restarts
// Can review history across sessions
// Export/import operation history
```

### 4. Performance Advisor

Predicts performance impact of operations.

```rust
pub struct PerformanceAdvisor {
    cost_model: CostModel,
    current_metrics: PerformanceMetrics,
}

pub struct CostModel {
    /// CPU cost per component type
    pub component_costs: HashMap<TypeId, f32>,
    /// GPU cost per component type
    pub gpu_costs: HashMap<TypeId, f32>,
    /// Memory per component type
    pub memory_costs: HashMap<TypeId, usize>,
}

impl PerformanceAdvisor {
    /// Predict FPS impact of operation
    pub fn predict_fps_impact(&self, operation: &Operation) -> FpsImpact {
        let mut cpu_cost = 0.0;
        let mut gpu_cost = 0.0;
        let mut memory_cost = 0;
        
        // Analyze operation
        for cmd in &operation.commands {
            match cmd {
                Command::SpawnEntity { components, .. } => {
                    for component_type in components {
                        cpu_cost += self.cost_model.component_costs
                            .get(component_type)
                            .unwrap_or(&0.0);
                        gpu_cost += self.cost_model.gpu_costs
                            .get(component_type)
                            .unwrap_or(&0.0);
                        memory_cost += self.cost_model.memory_costs
                            .get(component_type)
                            .unwrap_or(&0);
                    }
                }
                // ... other commands
            }
        }
        
        // Estimate FPS impact
        let current_fps = self.current_metrics.fps;
        let current_frame_time = 1000.0 / current_fps;
        let new_frame_time = current_frame_time + cpu_cost + gpu_cost;
        let new_fps = 1000.0 / new_frame_time;
        
        FpsImpact {
            current_fps,
            predicted_fps: new_fps,
            delta_fps: new_fps - current_fps,
            cpu_cost_ms: cpu_cost,
            gpu_cost_ms: gpu_cost,
            memory_cost_mb: memory_cost as f32 / 1024.0 / 1024.0,
        }
    }
    
    /// Suggest optimizations
    pub fn suggest_optimizations(&self, operation: &Operation) -> Vec<Optimization> {
        let mut suggestions = Vec::new();
        
        // Check for instancing opportunities
        if self.can_use_instancing(operation) {
            suggestions.push(Optimization::UseInstancing {
                description: "Use GPU instancing for repeated meshes".to_string(),
                expected_improvement: "5-10x faster rendering".to_string(),
            });
        }
        
        // Check for LOD opportunities
        if self.should_use_lod(operation) {
            suggestions.push(Optimization::UseLod {
                description: "Add LOD levels for distant objects".to_string(),
                expected_improvement: "50% reduction in vertex processing".to_string(),
            });
        }
        
        // Check for excessive entity count
        if self.entity_count_too_high(operation) {
            suggestions.push(Optimization::ReduceEntityCount {
                description: "Consider object pooling or culling".to_string(),
                expected_improvement: "Maintain 60 FPS".to_string(),
            });
        }
        
        suggestions
    }
}

// Usage:
// AI: "I want to spawn 1000 enemies"
// Advisor: "Warning: This will reduce FPS from 60 to 35
//           Suggestions:
//           - Use GPU instancing (expected: 60 FPS)
//           - Add LOD levels (expected: 55 FPS)
//           - Reduce count to 500 (expected: 60 FPS)"
```

**Cost Model Learning:**
```rust
// Cost model learns from actual measurements
impl PerformanceAdvisor {
    /// Update cost model from measurements
    pub fn learn_from_measurement(&mut self, operation: &Operation, actual_impact: FpsImpact) {
        // Compare prediction vs actual
        let prediction_error = (actual_impact.delta_fps - self.predict_fps_impact(operation).delta_fps).abs();
        
        // Update component costs
        for cmd in &operation.commands {
            if let Command::SpawnEntity { components, .. } = cmd {
                for component_type in components {
                    // Adjust cost based on error
                    let current_cost = self.cost_model.component_costs
                        .get(component_type)
                        .unwrap_or(&0.0);
                    let adjusted_cost = current_cost + prediction_error * 0.1;
                    self.cost_model.component_costs.insert(*component_type, adjusted_cost);
                }
            }
        }
    }
}

// Over time, predictions become more accurate
```

### 5. Visual Feedback System

Provides AI with visual understanding of the scene.

```rust
pub struct VisualFeedback {
    capture_size: (u32, u32),
    jpeg_quality: u8,
}

impl VisualFeedback {
    /// Capture viewport as JPEG
    pub fn capture_viewport(&self, renderer: &Renderer) -> Vec<u8> {
        // Render to texture
        let texture = renderer.render_to_texture(self.capture_size);
        
        // Convert to JPEG
        let jpeg = self.encode_jpeg(&texture, self.jpeg_quality);
        
        // Target: <100KB file size
        jpeg
    }
    
    /// Annotate capture with entity information
    pub fn annotate_capture(
        &self,
        image: &[u8],
        world: &World,
        camera: &Camera,
    ) -> Vec<u8> {
        let mut annotated = image.to_vec();
        
        // Project entities to screen space
        for (entity, transform) in world.query::<(Entity, &Transform)>().iter() {
            let screen_pos = camera.world_to_screen(transform.position);
            
            // Draw bounding box
            if let Some(bounds) = world.get::<Aabb>(entity) {
                self.draw_bounds(&mut annotated, bounds, &camera);
            }
            
            // Draw label
            if let Some(name) = world.get::<Name>(entity) {
                self.draw_label(&mut annotated, screen_pos, &name.0);
            }
        }
        
        annotated
    }
    
    /// Generate before/after comparison
    pub fn generate_comparison(&self, before: &[u8], after: &[u8]) -> Vec<u8> {
        // Side-by-side comparison
        let mut comparison = Vec::new();
        
        // Left: before
        // Right: after
        // Diff overlay: highlight changes
        
        comparison
    }
}

// Usage:
// AI: "Show me the scene"
// → capture_viewport()
// → annotate_capture()
// → Send to multimodal LLM
// 
// AI can now see:
// - Entity positions
// - Visual appearance
// - Spatial relationships
// - Lighting conditions
```

**Multimodal Integration:**
```rust
// Send to GPT-4V, Claude 3, or other multimodal LLM
let prompt = format!(
    "Here's the current game scene. {}",
    user_request
);

let response = llm_client.send_multimodal(
    &prompt,
    &[visual_feedback.capture_viewport(renderer)],
).await?;

// AI can now answer questions like:
// - "What's in the scene?"
// - "Is the lighting too dark?"
// - "Are the objects positioned correctly?"
// - "Does this look like a forest?"
```

## AI Interaction Flow

```
1. User Request
   └─> "Add 10 trees around the player"

2. Context Generation
   ├─> Generate world digest
   ├─> Find player entity (semantic search)
   └─> Capture viewport (optional)

3. AI Processing
   ├─> Understand intent
   ├─> Generate operation plan
   └─> Generate code if needed

4. Verification
   ├─> Static analysis
   ├─> Sandbox execution
   ├─> Dry run verification
   └─> Performance prediction

5. User Confirmation
   └─> Show preview and impact

6. Execution
   ├─> Execute operation
   ├─> Record to timeline
   └─> Monitor performance

7. Feedback
   ├─> Update cost model
   └─> Learn from result
```

## Best Practices

### For AI Prompts

- **Be specific**: "Add 10 pine trees" vs "Add trees"
- **Provide context**: "Near the player" vs absolute coordinates
- **Set constraints**: "Without reducing FPS below 60"
- **Use semantic references**: "The red building" vs Entity IDs

### For Performance

- **Batch operations**: Group related changes
- **Use dry run**: Preview before executing
- **Monitor impact**: Check FPS after changes
- **Undo if needed**: Revert problematic changes

### For Safety

- **Review generated code**: Check before executing
- **Use sandbox**: Test in isolated environment
- **Set limits**: Constrain resource usage
- **Create branches**: Experiment safely

## Further Reading

- [AI Integration Guide](../workflows/ai_integration.md) - Using AI features
- [MCP Server API](../api/mcp_server.md) - MCP protocol documentation
- [Performance Optimization](../audit/physics_optimization_guide.md) - Performance tuning
- [Operation Timeline](../workflows/operation_timeline.md) - Using undo/redo

## References

- [Model Context Protocol](https://modelcontextprotocol.io/) - MCP specification
- [GPT-4V](https://openai.com/research/gpt-4v-system-card) - Multimodal AI
- [Claude 3](https://www.anthropic.com/claude) - AI assistant
- [Semantic Search](https://www.pinecone.io/learn/semantic-search/) - Vector search
