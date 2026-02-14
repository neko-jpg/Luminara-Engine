# Design Document: Phase 2 - Scripting & AI Integration

## Overview

Phase 2 transforms Luminara Engine into an AI-first game development platform by implementing two critical layers: a robust scripting system for game logic and an intelligent AI integration layer that enables AI agents to understand, manipulate, and generate game content.

The design follows a layered architecture where the scripting layer provides runtime execution for Lua and WASM scripts with hot-reload capabilities, while the AI integration layer sits above it, providing sophisticated context management, code verification, and intelligent operation resolution. This separation ensures that traditional game development workflows remain unaffected while enabling powerful AI-driven development capabilities.

Key innovations include:
- **Hierarchical World Digest**: Token-budget-aware context generation for AI agents
- **Intent-Based Resolution**: Semantic operation resolution that remains valid as game state changes
- **Multi-Stage Code Verification**: Sandbox testing with automatic rollback on failure
- **Operation Timeline**: Git-like version control for AI operations with selective undo
- **Performance-Aware AI**: Predictive performance impact analysis before operation execution

## Architecture

### System Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Integration Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ MCP Server   │  │ AI Context   │  │ Code Verifier   │   │
│  │              │  │ Engine       │  │                 │   │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘   │
│         │                  │                    │            │
│  ┌──────▼──────────────────▼────────────────────▼────────┐  │
│  │           Intent Resolver & Operation Timeline         │  │
│  └──────────────────────────┬─────────────────────────────┘  │
├────────────────────────────┴──────────────────────────────────┤
│                    Scripting Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ Lua Runtime  │  │ WASM Runtime │  │ Hot Reload      │   │
│  │ (mlua)       │  │ (wasmtime)   │  │ System          │   │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘   │
│         │                  │                    │            │
│  ┌──────▼──────────────────▼────────────────────▼────────┐  │
│  │              Script API Bridge                         │  │
│  │  (Transform, Input, World, Physics, Audio, Time)       │  │
│  └──────────────────────────┬─────────────────────────────┘  │
├────────────────────────────┴──────────────────────────────────┤
│                    Engine Core (Phase 1)                     │
│  ECS, Rendering, Physics, Audio, Assets, Scene Management   │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow

```
AI Agent Request
      │
      ▼
┌─────────────────┐
│   MCP Server    │ ← Validates input, routes to tools
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ AI Context      │ ← Generates optimal context within token budget
│ Engine          │ ← Uses World Digest + Schema Discovery
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Intent Resolver │ ← Resolves semantic references to concrete entities
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Code Verifier   │ ← Static analysis → Sandbox → Dry run
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Operation       │ ← Records operation with inverse for undo
│ Timeline        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Engine Core     │ ← Executes validated operation
└─────────────────┘
```

## Components and Interfaces

### 1. Lua Script Runtime

```rust
// crates/luminara_script_lua/src/runtime.rs

pub struct LuaScriptRuntime {
    /// mlua Lua VM instance
    lua: mlua::Lua,
    /// Loaded scripts indexed by ID
    scripts: HashMap<ScriptId, LoadedScript>,
    /// File watcher for hot reload
    watcher: notify::RecommendedWatcher,
    /// Script lifecycle hooks
    lifecycle_manager: LifecycleManager,
}

pub struct LoadedScript {
    pub id: ScriptId,
    pub path: PathBuf,
    pub source: String,
    pub compiled: mlua::Function,
    pub instance: Option<mlua::Table>,
    pub last_modified: SystemTime,
}

impl LuaScriptRuntime {
    /// Initialize runtime and register engine APIs
    pub fn new() -> Result<Self>;
    
    /// Load and compile a Lua script
    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId>;
    
    /// Execute script lifecycle hook (on_start, on_update, on_destroy)
    pub fn call_lifecycle(&mut self, script_id: ScriptId, hook: &str, args: &[mlua::Value]) -> Result<()>;
    
    /// Reload script preserving state
    pub fn reload_script(&mut self, script_id: ScriptId) -> Result<()>;
    
    /// Register engine API module
    fn register_api(&self, name: &str, module: mlua::Table) -> Result<()>;
}
```

### 2. WASM Script Runtime

```rust
// crates/luminara_script_wasm/src/runtime.rs

pub struct WasmScriptRuntime {
    /// wasmtime engine with configured limits
    engine: wasmtime::Engine,
    /// Store with host state
    store: wasmtime::Store<HostState>,
    /// Loaded WASM modules
    modules: HashMap<ScriptId, WasmModule>,
    /// Linker for host functions
    linker: wasmtime::Linker<HostState>,
}

pub struct WasmModule {
    pub id: ScriptId,
    pub module: wasmtime::Module,
    pub instance: wasmtime::Instance,
    pub memory: wasmtime::Memory,
}

pub struct HostState {
    /// Reference to engine world
    pub world: *mut World,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Execution statistics
    pub stats: ExecutionStats,
}

impl WasmScriptRuntime {
    /// Create runtime with resource limits
    pub fn new(limits: ResourceLimits) -> Result<Self>;
    
    /// Load WASM module from bytes
    pub fn load_module(&mut self, bytes: &[u8]) -> Result<ScriptId>;
    
    /// Call exported function
    pub fn call_function(&mut self, script_id: ScriptId, name: &str, args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>>;
    
    /// Register host function
    fn register_host_fn<Params, Results>(&mut self, module: &str, name: &str, func: impl Fn(Params) -> Results) -> Result<()>;
}
```

### 3. Hot Reload System

```rust
// crates/luminara_script/src/hot_reload.rs

pub struct HotReloadSystem {
    /// File system watcher
    watcher: notify::RecommendedWatcher,
    /// Pending reload events
    events: Receiver<notify::Event>,
    /// Script path to ID mapping
    path_map: HashMap<PathBuf, ScriptId>,
    /// Reload strategies per script
    strategies: HashMap<ScriptId, ReloadStrategy>,
}

pub enum ReloadStrategy {
    /// Preserve all state
    PreserveState,
    /// Call save/restore hooks
    SaveRestore,
    /// Full reset
    Reset,
}

impl HotReloadSystem {
    /// Start watching directories
    pub fn watch(&mut self, paths: &[PathBuf]) -> Result<()>;
    
    /// Process pending file events
    pub fn process_events(&mut self, lua_runtime: &mut LuaScriptRuntime, wasm_runtime: &mut WasmScriptRuntime) -> Result<Vec<ScriptId>>;
    
    /// Reload script with strategy
    fn reload_with_strategy(&mut self, script_id: ScriptId, strategy: ReloadStrategy) -> Result<()>;
}
```

### 4. MCP Server

```rust
// tools/luminara_mcp_server/src/server.rs

pub struct LuminaraMcpServer {
    /// Connection to engine
    engine: EngineConnection,
    /// Registered tools
    tools: HashMap<String, Box<dyn McpTool>>,
    /// Server configuration
    config: McpConfig,
}

pub trait McpTool: Send + Sync {
    /// Tool name (e.g., "scene.create_entity")
    fn name(&self) -> &str;
    
    /// Tool description for AI
    fn description(&self) -> &str;
    
    /// JSON schema for input validation
    fn input_schema(&self) -> serde_json::Value;
    
    /// Execute tool with validated input
    fn execute(&self, input: serde_json::Value, engine: &mut EngineConnection) -> Result<ToolResult>;
}

pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

impl LuminaraMcpServer {
    /// Start MCP server on port
    pub async fn start(port: u16, engine_addr: &str) -> Result<Self>;
    
    /// Register a tool
    pub fn register_tool(&mut self, tool: Box<dyn McpTool>);
    
    /// Handle incoming MCP request
    async fn handle_request(&mut self, request: McpRequest) -> McpResponse;
}
```

### 5. AI Context Engine

```rust
// crates/luminara_ai_agent/src/context_engine.rs

pub struct AiContextEngine {
    /// World digest generator
    digest: WorldDigestEngine,
    /// Schema discovery service
    schema: SchemaDiscoveryService,
    /// Semantic index for entity search
    semantic_index: SemanticIndex,
    /// Change tracker for diffs
    change_tracker: ChangeTracker,
}

pub struct WorldDigestEngine {
    /// Attention estimator for relevance
    attention: AttentionEstimator,
    /// Digest level generators
    generators: Vec<Box<dyn DigestGenerator>>,
}

pub enum DigestLevel {
    WorldSummary,           // ~500 tokens
    EntityCatalog,          // ~2000 tokens
    EntityDetail(Query),    // ~5000 tokens
    FullEntity(Entity),     // ~1000 tokens
}

impl AiContextEngine {
    /// Generate optimal context for AI query
    pub fn generate_context(&self, query: &str, max_tokens: usize, world: &World) -> WorldContext;
    
    /// Get component schema at specified detail level
    pub fn get_schema(&self, component: &str, level: SchemaLevel) -> ComponentSchema;
    
    /// Search entities by semantic description
    pub fn semantic_search(&self, description: &str, world: &World, top_k: usize) -> Vec<(Entity, f32)>;
}

pub struct WorldContext {
    pub summary: String,
    pub entities: Vec<EntityDigest>,
    pub schemas: Vec<ComponentSchema>,
    pub recent_changes: Option<ChangeSet>,
    pub performance_context: PerformanceContext,
    pub token_usage: usize,
}
```

### 6. Intent Resolver

```rust
// crates/luminara_ai_agent/src/intent_resolver.rs

pub struct IntentResolver {
    /// Semantic index for entity resolution
    semantic_index: Arc<SemanticIndex>,
    /// World version tracking
    world_version: WorldVersion,
}

pub enum AiIntent {
    SpawnRelative {
        anchor: EntityReference,
        offset: RelativePosition,
        template: EntityTemplate,
    },
    ModifyMatching {
        query: SemanticQuery,
        mutation: ComponentMutation,
    },
    AttachBehavior {
        target: EntityReference,
        behavior: BehaviorTemplate,
        parameters: HashMap<String, DynamicReference>,
    },
}

pub enum EntityReference {
    ByName(String),
    ById(Entity),
    ByTag(String),
    ByComponent(String),
    Nearest { to: Box<EntityReference>, with_tag: Option<String> },
    Semantic(String),
}

pub enum RelativePosition {
    Forward(f32),
    Above(f32),
    AtOffset(Vec3),
    RandomInRadius(f32),
    RandomReachable { radius: f32 },
}

impl IntentResolver {
    /// Resolve intent to concrete operations
    pub fn resolve(&self, intent: &AiIntent, world: &World) -> Result<Vec<EngineCommand>>;
    
    /// Resolve entity reference to concrete entity
    fn resolve_reference(&self, reference: &EntityReference, world: &World) -> Result<Entity>;
    
    /// Resolve relative position to absolute position
    fn resolve_position(&self, position: &RelativePosition, anchor: &Transform, world: &World) -> Result<Vec3>;
}
```

### 7. Code Verification Pipeline

```rust
// crates/luminara_ai_agent/src/code_verifier.rs

pub struct CodeVerificationPipeline {
    /// Static analyzer
    static_analyzer: StaticAnalyzer,
    /// Sandbox executor
    sandbox: ScriptSandbox,
    /// Dry run executor
    dry_runner: DryRunner,
    /// Rollback manager
    rollback: RollbackManager,
}

pub struct VerificationResult {
    pub passed: bool,
    pub static_issues: Vec<StaticIssue>,
    pub sandbox_result: Option<SandboxResult>,
    pub preview: Option<DiffPreview>,
    pub suggestions: Vec<String>,
}

pub struct SandboxConfig {
    pub max_execution_time: Duration,
    pub max_memory: usize,
    pub max_entities_spawned: usize,
    pub max_api_calls: usize,
    pub max_instructions: u64,
}

impl CodeVerificationPipeline {
    /// Verify and apply AI-generated code
    pub async fn verify_and_apply(&mut self, code: &GeneratedCode, world: &mut World) -> Result<ApplyResult>;
    
    /// Stage 1: Static analysis
    fn static_analysis(&self, code: &GeneratedCode) -> Result<StaticAnalysisResult>;
    
    /// Stage 2: Sandbox execution
    async fn sandbox_execute(&self, code: &GeneratedCode, snapshot: &WorldSnapshot) -> Result<SandboxResult>;
    
    /// Stage 3: Dry run preview
    fn dry_run(&self, code: &GeneratedCode, world: &World) -> DiffPreview;
    
    /// Stage 4: Apply with monitoring
    async fn apply_with_monitoring(&mut self, code: &GeneratedCode, world: &mut World) -> Result<()>;
}
```

### 8. Operation Timeline

```rust
// crates/luminara_ai_agent/src/operation_timeline.rs

pub struct OperationTimeline {
    /// Immutable operation log
    log: OperationLog,
    /// Current head position
    head: OperationId,
    /// Branch management
    branches: HashMap<String, OperationId>,
    /// Periodic snapshots
    snapshots: BTreeMap<OperationId, WorldSnapshot>,
}

pub struct Operation {
    pub id: OperationId,
    pub timestamp: Instant,
    pub ai_prompt: String,
    pub ai_response: String,
    pub commands: Vec<EngineCommand>,
    pub inverse_commands: Vec<EngineCommand>,
    pub change_summary: String,
    pub parent: Option<OperationId>,
    pub tags: Vec<String>,
}

impl OperationTimeline {
    /// Record new operation
    pub fn record(&mut self, operation: Operation) -> OperationId;
    
    /// Undo operation
    pub fn undo(&mut self, operation_id: OperationId, world: &mut World) -> Result<()>;
    
    /// Selective undo with conflict detection
    pub fn selective_undo(&mut self, operation_id: OperationId, world: &mut World) -> Result<SelectiveUndoResult>;
    
    /// Create branch from operation
    pub fn create_branch(&mut self, name: &str, from: OperationId) -> Result<()>;
    
    /// Switch to branch
    pub fn checkout_branch(&mut self, name: &str, world: &mut World) -> Result<()>;
    
    /// Get operation history summary
    pub fn summarize_recent(&self, count: usize) -> String;
}
```

### 9. Performance Advisor

```rust
// crates/luminara_ai_agent/src/performance_advisor.rs

pub struct PerformanceAdvisor {
    /// Current performance metrics
    metrics: PerformanceMetrics,
    /// Component cost model
    cost_model: CostModel,
    /// Performance budget
    budget: PerformanceBudget,
}

pub struct CostModel {
    /// Entity count to frame time curve
    entity_cost_curve: PiecewiseLinear,
    /// Per-component costs
    component_costs: HashMap<String, ComponentCost>,
}

pub struct ComponentCost {
    pub cpu_cost_per_entity: f64,  // μs/entity/frame
    pub gpu_cost_per_entity: f64,  // μs/entity/frame
    pub memory_per_entity: usize,  // bytes
    pub draw_calls_per_entity: f32,
}

pub struct PerformanceImpact {
    pub severity: Severity,
    pub predicted_fps: f32,
    pub message: String,
    pub suggestions: Vec<Suggestion>,
}

impl PerformanceAdvisor {
    /// Estimate impact of AI intent
    pub fn estimate_impact(&self, intent: &AiIntent, world: &World) -> PerformanceImpact;
    
    /// Generate performance context for AI
    pub fn generate_context(&self, world: &World) -> String;
    
    /// Update cost model from measurements
    pub fn update_model(&mut self, measurements: &[Measurement]);
}
```

### 10. Visual Feedback System

```rust
// crates/luminara_ai_agent/src/visual_feedback.rs

pub struct VisualFeedbackSystem {
    /// Screenshot capturer
    capturer: ScreenshotCapturer,
    /// Scene visualizer
    visualizer: SceneVisualizer,
    /// Multimodal LLM client
    llm_client: MultimodalLlmClient,
}

pub struct CaptureConfig {
    pub resolution: (u32, u32),
    pub format: ImageFormat,
    pub annotations: AnnotationConfig,
}

pub struct AnnotationConfig {
    pub show_entity_names: bool,
    pub show_bounding_boxes: bool,
    pub show_light_positions: bool,
    pub highlight_selected: bool,
}

impl VisualFeedbackSystem {
    /// Capture and analyze viewport
    pub async fn capture_and_analyze(&self, world: &World, render_context: &RenderContext, feedback: &str) -> VisualAnalysis;
    
    /// Compare before/after screenshots
    pub async fn compare_before_after(&self, before: &[u8], after: &[u8], description: &str) -> ComparisonResult;
    
    /// Create annotated view
    fn create_annotated_view(&self, screenshot: &[u8], world: &World, config: AnnotationConfig) -> Vec<u8>;
}
```

## Data Models

### Script Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAsset {
    pub id: ScriptId,
    pub name: String,
    pub language: ScriptLanguage,
    pub source: String,
    pub dependencies: Vec<ScriptId>,
    pub metadata: ScriptMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    Lua,
    Wasm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub author: Option<String>,
    pub description: String,
    pub version: String,
    pub lifecycle_hooks: Vec<String>,
    pub required_components: Vec<String>,
}

#[derive(Component)]
pub struct ScriptComponent {
    pub script: Handle<ScriptAsset>,
    pub enabled: bool,
    pub state: ScriptState,
}

#[derive(Debug, Clone)]
pub enum ScriptState {
    Uninitialized,
    Running,
    Paused,
    Error(String),
}
```

### AI Operation Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOperation {
    pub id: OperationId,
    pub timestamp: SystemTime,
    pub intent: AiIntent,
    pub resolved_commands: Vec<EngineCommand>,
    pub result: OperationResult,
    pub performance_impact: PerformanceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineCommand {
    SpawnEntity { template: EntityTemplate, position: Vec3 },
    DestroyEntity { entity: Entity },
    AddComponent { entity: Entity, component: Box<dyn Component> },
    RemoveComponent { entity: Entity, component_type: TypeId },
    ModifyComponent { entity: Entity, component_type: TypeId, mutation: ComponentMutation },
    CreateScript { path: PathBuf, source: String },
    ModifyScript { script_id: ScriptId, source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub entities_created: Vec<Entity>,
    pub entities_modified: Vec<Entity>,
    pub entities_destroyed: Vec<Entity>,
    pub error: Option<String>,
}
```

### Context Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldContext {
    pub summary: WorldSummary,
    pub entities: Vec<EntityDigest>,
    pub schemas: Vec<ComponentSchema>,
    pub recent_changes: Option<ChangeSet>,
    pub performance: PerformanceContext,
    pub token_usage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSummary {
    pub entity_count: usize,
    pub component_types: Vec<String>,
    pub tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDigest {
    pub entity: Entity,
    pub name: Option<String>,
    pub position: Option<Vec3>,
    pub components: Vec<String>,
    pub tags: Vec<String>,
    pub detail_level: DigestLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub name: String,
    pub description: String,
    pub category: String,
    pub fields: Vec<FieldSchema>,
    pub examples: Vec<serde_json::Value>,
    pub commonly_paired_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
    pub constraints: Option<FieldConstraints>,
}
```

## Correctness Properties

