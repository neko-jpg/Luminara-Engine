# Implementation Plan: Phase 2 - Scripting & AI Integration

## Overview

This implementation plan breaks down Phase 2 into discrete, incremental tasks that build upon Phase 1's foundation. The plan follows a bottom-up approach: first establishing the scripting layer (Lua and WASM runtimes with hot reload), then building the AI integration layer on top (MCP server, context engine, intent resolver, code verifier, operation timeline, performance advisor, and visual feedback).

Each task is designed to be independently testable and includes references to specific requirements. Property-based tests are marked as optional sub-tasks to allow for faster MVP delivery while maintaining the option for comprehensive correctness verification.

## Tasks

- [ ] 1. Set up Phase 2 crate structure and dependencies
  - Create crates: luminara_script, luminara_script_lua, luminara_script_wasm, luminara_ai_agent, tools/luminara_mcp_server
  - Add dependencies: mlua, wasmtime, notify, serde_json, tokio, quickcheck
  - Set up workspace configuration in root Cargo.toml
  - _Requirements: All Phase 2 requirements_


- [ ] 2. Implement Lua Script Runtime core
  - [ ] 2.1 Create LuaScriptRuntime struct with mlua::Lua instance
    - Initialize Lua VM with safe defaults
    - Set up script storage HashMap
    - _Requirements: 1.1_
  
  - [ ] 2.2 Implement script loading and compilation
    - Load Lua source from file
    - Compile to bytecode and cache
    - Store LoadedScript with metadata
    - _Requirements: 1.2_
  
  - [ ]* 2.3 Write property test for script compilation caching
    - **Property 2: Script Compilation Caching**
    - **Validates: Requirements 1.2**
  
  - [ ] 2.4 Implement lifecycle manager for script hooks
    - Support on_start, on_update, on_destroy hooks
    - Call hooks in correct order
    - Handle missing hooks gracefully
    - _Requirements: 1.6_
  
  - [ ]* 2.5 Write property test for lifecycle hook invocation
    - **Property 12: Lifecycle Hook Invocation**
    - **Validates: Requirements 3.6, 3.7**

- [ ] 3. Implement Lua Engine API Bridge
  - [ ] 3.1 Create Transform API module
    - Expose get/set position, rotation, scale
    - Expose forward, right, up vector computation
    - Register module in Lua globals
    - _Requirements: 14.1_
  
  - [ ] 3.2 Create Input API module
    - Expose key/button state queries
    - Expose axis value queries
    - Expose mouse position/delta
    - _Requirements: 14.2_
  
  - [ ] 3.3 Create World API module
    - Expose entity finding by name/tag/component
    - Expose entity spawning and destruction
    - Provide type-safe entity wrappers
    - _Requirements: 14.3_
  
  - [ ] 3.4 Create Physics, Audio, Time, Component API modules
    - Physics: forces, impulses, raycasting, collisions
    - Audio: play sounds, volume/pitch, spatial audio
    - Time: delta time, total time, frame count
    - Component: get/set with type safety
    - _Requirements: 14.4, 14.5, 14.6, 14.7_
  
  - [ ]* 3.5 Write property test for API parameter validation
    - **Property 3: API Parameter Validation**
    - **Validates: Requirements 1.3, 14.8**
  
  - [ ]* 3.6 Write property test for Component API type safety
    - **Property 51: Component API Type Safety**
    - **Validates: Requirements 14.7**
  
  - [ ]* 3.7 Write unit tests for each API module
    - Test Transform operations with valid inputs
    - Test Input queries return correct values
    - Test World operations create/find/destroy entities
    - Test error messages for invalid parameters
    - _Requirements: 14.1-14.8_

- [ ] 4. Implement Hot Reload System
  - [ ] 4.1 Create HotReloadSystem with notify file watcher
    - Set up file system watcher
    - Create event channel for file changes
    - Map file paths to script IDs
    - _Requirements: 3.1_
  
  - [ ] 4.2 Implement reload event processing
    - Detect file modifications within 100ms
    - Trigger script reload on change
    - Handle multiple simultaneous changes
    - _Requirements: 3.2, 3.5_
  
  - [ ] 4.3 Implement state preservation during reload
    - Save entity references before reload
    - Restore references after reload
    - Call save/restore hooks if defined
    - _Requirements: 3.3, 3.6_
  
  - [ ] 4.4 Implement fallback on reload failure
    - Keep previous version active on error
    - Log reload failure details
    - Continue execution with old version
    - _Requirements: 3.4_
  
  - [ ]* 4.5 Write property test for hot reload timing
    - **Property 4: Hot Reload Detection Timing**
    - **Validates: Requirements 1.4, 3.2**
  
  - [ ]* 4.6 Write property test for state preservation
    - **Property 10: Hot Reload State Preservation**
    - **Validates: Requirements 2.6, 3.3**
  
  - [ ]* 4.7 Write property test for fallback on failure
    - **Property 11: Hot Reload Fallback on Failure**
    - **Validates: Requirements 3.4**

- [ ] 5. Checkpoint - Ensure Lua scripting tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 6. Implement WASM Script Runtime core
  - [ ] 6.1 Create WasmScriptRuntime with wasmtime engine
    - Initialize wasmtime engine with resource limits
    - Create store with HostState
    - Set up linker for host functions
    - _Requirements: 2.1_
  
  - [ ] 6.2 Implement WASM module loading and validation
    - Load WASM bytes and validate
    - Instantiate module with imports
    - Store WasmModule with metadata
    - _Requirements: 2.2_
  
  - [ ]* 6.3 Write property test for WASM module validation
    - **Property 7: WASM Module Validation**
    - **Validates: Requirements 2.2**
  
  - [ ] 6.4 Implement WASM-Rust data marshaling
    - Marshal primitive types across boundary
    - Marshal structs via JSON serialization
    - Handle errors gracefully
    - _Requirements: 2.3_
  
  - [ ]* 6.5 Write property test for data marshaling round-trip
    - **Property 8: WASM Data Marshaling Round-Trip**
    - **Validates: Requirements 2.3**
  
  - [ ] 6.6 Implement resource limits enforcement
    - Memory limit: 64MB per instance
    - Execution time limit with timeout
    - Instruction counting for infinite loop detection
    - _Requirements: 2.4, 2.5_
  
  - [ ]* 6.7 Write edge case tests for resource limits
    - Test memory limit enforcement
    - Test execution timeout
    - Test instruction limit
    - _Requirements: 2.4, 2.5_

- [ ] 7. Implement WASM Engine API Bridge
  - [ ] 7.1 Create WIT interface definitions
    - Define luminara-guest.wit with all engine APIs
    - Generate Rust bindings with wit-bindgen
    - Document API usage for guest languages
    - _Requirements: 2.7_
  
  - [ ] 7.2 Register host functions in linker
    - Register Transform, Input, World APIs
    - Register Physics, Audio, Time, Component APIs
    - Implement host function wrappers
    - _Requirements: 2.3_
  
  - [ ] 7.3 Implement WASM error isolation
    - Catch WASM panics and traps
    - Report detailed error information
    - Prevent engine crashes from WASM errors
    - _Requirements: 2.8_
  
  - [ ]* 7.4 Write property test for error isolation
    - **Property 5: Script Error Reporting Completeness**
    - **Validates: Requirements 1.5, 2.8**

- [ ] 8. Implement Script Sandbox System
  - [ ] 8.1 Create ScriptSandbox with resource limits
    - Configure sandbox limits (time, memory, entities, API calls)
    - Set up isolated execution environment
    - Implement resource tracking
    - _Requirements: 13.1, 13.2_
  
  - [ ] 8.2 Implement Lua instruction counting
    - Use mlua hooks for instruction counting
    - Terminate on instruction limit exceeded
    - Report detailed error on termination
    - _Requirements: 13.3_
  
  - [ ]* 8.3 Write property test for instruction counting
    - **Property 49: Sandbox Instruction Counting Accuracy**
    - **Validates: Requirements 13.3**
  
  - [ ] 8.4 Implement capability restrictions
    - Block file system access in sandbox
    - Block network access in sandbox
    - Block process spawning and FFI
    - _Requirements: 13.5_
  
  - [ ] 8.5 Implement script whitelisting
    - Allow marking scripts as trusted
    - Grant unrestricted execution to whitelisted scripts
    - Maintain whitelist persistence
    - _Requirements: 13.7_
  
  - [ ]* 8.6 Write property test for sandbox resource limits
    - **Property 9: Sandbox Resource Limit Enforcement**
    - **Validates: Requirements 2.4, 2.5, 8.3, 13.1, 13.2, 13.4, 13.5**

- [ ] 9. Checkpoint - Ensure WASM scripting tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 10. Implement MCP Server foundation
  - [ ] 10.1 Create LuminaraMcpServer struct
    - Set up TCP server with tokio
    - Create engine connection interface
    - Implement tool registry
    - _Requirements: 4.1_
  
  - [ ] 10.2 Implement MCP protocol handling
    - Parse incoming MCP requests
    - Validate request format
    - Route to appropriate tools
    - Format responses according to MCP spec
    - _Requirements: 4.2_
  
  - [ ] 10.3 Implement tool input validation
    - Validate against JSON schemas
    - Return structured errors for invalid input
    - Provide helpful error messages
    - _Requirements: 4.3_
  
  - [ ]* 10.4 Write property test for tool input validation
    - **Property 13: MCP Tool Input Validation**
    - **Validates: Requirements 4.3**

- [ ] 11. Implement MCP Tools - Scene Operations
  - [ ] 11.1 Implement scene.create_entity tool
    - Define input schema for entity creation
    - Parse entity template from JSON
    - Create entity in world
    - Return entity ID and success message
    - _Requirements: 4.4_
  
  - [ ] 11.2 Implement scene.modify_component tool
    - Define input schema for component modification
    - Resolve entity reference
    - Apply component mutation
    - Return change summary
    - _Requirements: 4.4_
  
  - [ ] 11.3 Implement scene.query_entities tool
    - Define input schema for entity queries
    - Support queries by name, tag, component
    - Return matching entities with details
    - _Requirements: 4.4_
  
  - [ ]* 11.4 Write property test for tool result structure
    - **Property 14: MCP Tool Result Structure**
    - **Validates: Requirements 4.5**
  
  - [ ]* 11.5 Write property test for tool error suggestions
    - **Property 15: MCP Tool Error Suggestions**
    - **Validates: Requirements 4.6, 8.8**

- [ ] 12. Implement MCP Tools - Script Operations
  - [ ] 12.1 Implement script.create tool
    - Define input schema for script creation
    - Write script file to disk
    - Load script into runtime
    - Return script ID
    - _Requirements: 4.4_
  
  - [ ] 12.2 Implement script.modify tool
    - Define input schema for script modification
    - Update script file
    - Trigger hot reload
    - Return reload status
    - _Requirements: 4.4_
  
  - [ ] 12.3 Implement debug.inspect tool
    - Define input schema for inspection queries
    - Support queries: all_entities, entity_by_name, world_stats, render_stats
    - Return structured inspection data
    - _Requirements: 4.4_
  
  - [ ] 12.4 Implement project.scaffold tool
    - Define input schema for project scaffolding
    - Create project directory structure
    - Generate Cargo.toml, main.rs, directories
    - Return project path
    - _Requirements: 4.4_

- [ ] 13. Implement AI Context Engine - World Digest
  - [ ] 13.1 Create WorldDigestEngine struct
    - Set up attention estimator
    - Create digest level generators
    - Initialize semantic index
    - _Requirements: 5.1_
  
  - [ ] 13.2 Implement L0 world summary generator
    - Generate ~500 token summary
    - Include entity count, component types, tags
    - Provide high-level description
    - _Requirements: 5.3_
  
  - [ ] 13.3 Implement L1 entity catalog generator
    - Generate ~2000 token catalog
    - List entities with names, positions, main components
    - Organize by categories
    - _Requirements: 5.3_
  
  - [ ] 13.4 Implement L2 entity detail generator
    - Generate ~5000 token details
    - Include full component data for matching entities
    - Support query-based filtering
    - _Requirements: 5.3_
  
  - [ ] 13.5 Implement L3 full entity generator
    - Generate ~1000 token full entity data
    - Include all components and relationships
    - Include related entities
    - _Requirements: 5.3_
  
  - [ ] 13.6 Implement attention estimator
    - Analyze AI query for entity mentions
    - Determine relevant entities
    - Prioritize entities for detail levels
    - _Requirements: 5.1, 5.4_
  
  - [ ]* 13.7 Write property test for digest relevance prioritization
    - **Property 16: World Digest Relevance Prioritization**
    - **Validates: Requirements 5.1, 5.4**
  
  - [ ]* 13.8 Write property test for token budget compliance
    - **Property 17: World Digest Token Budget Compliance**
    - **Validates: Requirements 5.2, 5.3**


- [ ] 14. Implement AI Context Engine - Semantic Index
  - [ ] 14.1 Create SemanticIndex struct
    - Set up local embedding model (ONNX Runtime)
    - Initialize HNSW vector store
    - Create entity-to-text converter
    - _Requirements: 5.6_
  
  - [ ] 14.2 Implement entity indexing
    - Convert entities to searchable text
    - Generate embeddings for entities
    - Insert into vector store
    - _Requirements: 5.6_
  
  - [ ] 14.3 Implement semantic search
    - Embed natural language queries
    - Search vector store for similar entities
    - Return ranked results with scores
    - _Requirements: 5.6_
  
  - [ ] 14.4 Implement incremental index updates
    - Track dirty entities
    - Update embeddings on entity changes
    - Maintain index consistency
    - _Requirements: 5.6_
  
  - [ ]* 14.5 Write property test for semantic search
    - **Property 19: Semantic Entity Search**
    - **Validates: Requirements 5.6**
  
  - [ ] 14.6 Implement large world sampling
    - Use spatial clustering for >1000 entities
    - Use semantic clustering for representative samples
    - Balance spatial and semantic diversity
    - _Requirements: 5.7_
  
  - [ ]* 14.7 Write property test for large world sampling
    - **Property 20: Large World Sampling**
    - **Validates: Requirements 5.7**

- [ ] 15. Implement AI Context Engine - Schema Discovery
  - [ ] 15.1 Create SchemaDiscoveryService struct
    - Set up component registry
    - Create schema taxonomy (categories)
    - Initialize usage statistics
    - _Requirements: 6.1_
  
  - [ ] 15.2 Implement schema level generation
    - Generate L0 (brief) schemas for all components
    - Generate L1 (fields) schemas for used components
    - Generate L2 (full) schemas for modified components
    - _Requirements: 6.1, 6.2, 6.3_
  
  - [ ] 15.3 Implement schema.inspect tool
    - Allow AI to request detailed schemas
    - Include commonly paired components
    - Include usage examples and caveats
    - _Requirements: 6.4_
  
  - [ ] 15.4 Implement component categorization
    - Categorize into: Rendering, Physics, Gameplay, Audio, AI, UI
    - Maintain category mappings
    - Use categories for schema organization
    - _Requirements: 6.6_
  
  - [ ]* 15.5 Write property test for schema detail progression
    - **Property 21: Schema Detail Level Progression**
    - **Validates: Requirements 6.1, 6.2, 6.3**
  
  - [ ]* 15.6 Write property test for schema completeness
    - **Property 22: Component Schema Completeness**
    - **Validates: Requirements 6.5, 6.6**

- [ ] 16. Implement context generation integration
  - [ ] 16.1 Create AiContextEngine struct
    - Integrate WorldDigestEngine
    - Integrate SchemaDiscoveryService
    - Integrate SemanticIndex
    - Create ChangeTracker
    - _Requirements: 5.1_
  
  - [ ] 16.2 Implement generate_context method
    - Analyze AI query for intent
    - Generate world digest within token budget
    - Include relevant schemas
    - Include recent changes if budget allows
    - _Requirements: 5.1, 5.2, 5.5_
  
  - [ ]* 16.3 Write property test for recent changes inclusion
    - **Property 18: World Digest Recent Changes Inclusion**
    - **Validates: Requirements 5.5**
  
  - [ ]* 16.4 Write property test for schema prioritization
    - **Property 23: Schema Token Budget Prioritization**
    - **Validates: Requirements 6.7**

- [ ] 17. Checkpoint - Ensure AI context engine tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 18. Implement Intent Resolver
  - [ ] 18.1 Create IntentResolver struct
    - Set up semantic index reference
    - Initialize world version tracking
    - _Requirements: 7.1_
  
  - [ ] 18.2 Implement entity reference resolution
    - Support ByName, ById, ByTag, ByComponent references
    - Support Nearest spatial queries
    - Support Semantic natural language references
    - Return errors with suggestions for failed resolutions
    - _Requirements: 7.1, 7.3, 7.4, 7.5_
  
  - [ ]* 18.3 Write property test for entity reference resolution
    - **Property 24: Entity Reference Resolution**
    - **Validates: Requirements 7.1, 7.3, 7.4, 7.5**
  
  - [ ] 18.4 Implement relative position resolution
    - Support Forward, Above, AtOffset positions
    - Support RandomInRadius, RandomReachable positions
    - Compute absolute positions from anchor transforms
    - _Requirements: 7.2, 7.7_
  
  - [ ]* 18.5 Write property test for position resolution
    - **Property 25: Relative Position Resolution**
    - **Validates: Requirements 7.2, 7.7**
  
  - [ ] 18.6 Implement multi-match query handling
    - Return all matches for queries
    - Rank by spatial proximity and semantic relevance
    - Support "closest match" mode
    - _Requirements: 7.6_
  
  - [ ]* 18.7 Write property test for multi-match handling
    - **Property 26: Multi-Match Query Handling**
    - **Validates: Requirements 7.6**
  
  - [ ] 18.8 Implement intent resolution
    - Resolve SpawnRelative intents
    - Resolve ModifyMatching intents
    - Resolve AttachBehavior intents
    - Convert intents to concrete EngineCommands
    - _Requirements: 7.1, 7.2, 7.3_

- [ ] 19. Implement Code Verification Pipeline - Static Analysis
  - [ ] 19.1 Create StaticAnalyzer struct
    - Set up Lua AST parser
    - Create pattern matchers for dangerous code
    - _Requirements: 8.1_
  
  - [ ] 19.2 Implement dangerous pattern detection
    - Detect potential infinite loops (while true, unbounded recursion)
    - Detect undefined variable references
    - Detect system calls and FFI usage
    - Detect resource exhaustion patterns (massive spawns)
    - _Requirements: 8.1_
  
  - [ ] 19.3 Generate fix suggestions
    - Suggest loop bounds for infinite loops
    - Suggest variable declarations for undefined refs
    - Suggest safe alternatives for dangerous patterns
    - _Requirements: 8.8_
  
  - [ ]* 19.4 Write property test for static analysis detection
    - **Property 27: Static Code Analysis Detection**
    - **Validates: Requirements 8.1**

- [ ] 20. Implement Code Verification Pipeline - Sandbox Execution
  - [ ] 20.1 Create ScriptSandbox for verification
    - Create world snapshot for isolated testing
    - Configure resource limits
    - Set up monitoring
    - _Requirements: 8.2_
  
  - [ ] 20.2 Implement sandbox execution
    - Execute script in isolated environment
    - Track resource usage
    - Capture execution results
    - Detect anomalies
    - _Requirements: 8.2, 8.3_
  
  - [ ]* 20.3 Write property test for sandbox execution
    - **Property 28: Sandbox Execution After Static Analysis**
    - **Validates: Requirements 8.2**

- [ ] 21. Implement Code Verification Pipeline - Dry Run & Apply
  - [ ] 21.1 Create DryRunner struct
    - Execute script without committing changes
    - Track all operations performed
    - Generate diff preview
    - _Requirements: 8.4_
  
  - [ ] 21.2 Implement diff preview generation
    - Show entities added, modified, removed
    - Show component changes
    - Estimate performance impact
    - Include warnings
    - _Requirements: 8.4_
  
  - [ ]* 21.3 Write property test for diff preview accuracy
    - **Property 29: Code Verification Diff Preview Accuracy**
    - **Validates: Requirements 8.4**
  
  - [ ] 21.4 Create RollbackManager
    - Create world snapshots as checkpoints
    - Store inverse commands for operations
    - Implement rollback execution
    - _Requirements: 8.5_
  
  - [ ] 21.5 Implement apply with monitoring
    - Create checkpoint before apply
    - Apply verified code
    - Monitor for 2 seconds post-apply
    - Rollback on errors or anomalies
    - _Requirements: 8.5, 8.6, 8.7_
  
  - [ ]* 21.6 Write property test for automatic rollback
    - **Property 30: Automatic Rollback on Failure**
    - **Validates: Requirements 8.5, 8.6, 8.7**

- [ ] 22. Integrate Code Verification Pipeline
  - [ ] 22.1 Create CodeVerificationPipeline struct
    - Integrate StaticAnalyzer
    - Integrate ScriptSandbox
    - Integrate DryRunner
    - Integrate RollbackManager
    - _Requirements: 8.1_
  
  - [ ] 22.2 Implement verify_and_apply method
    - Run static analysis
    - Run sandbox execution
    - Generate dry run preview
    - Apply with monitoring
    - Return verification result
    - _Requirements: 8.1, 8.2, 8.4, 8.5_

- [ ] 23. Checkpoint - Ensure code verification tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 24. Implement Operation Timeline
  - [ ] 24.1 Create OperationTimeline struct
    - Set up immutable operation log
    - Initialize head pointer
    - Create branch management
    - Set up snapshot storage
    - _Requirements: 9.1_
  
  - [ ] 24.2 Implement operation recording
    - Record intent, commands, inverse commands
    - Store timestamp and metadata
    - Generate change summary
    - Link to parent operation
    - _Requirements: 9.1_
  
  - [ ]* 24.3 Write property test for operation recording
    - **Property 31: Operation Recording Completeness**
    - **Validates: Requirements 9.1**
  
  - [ ] 24.4 Implement undo operation
    - Execute inverse commands
    - Restore previous state
    - Update head pointer
    - _Requirements: 9.2_
  
  - [ ]* 24.5 Write property test for undo round-trip
    - **Property 32: Operation Undo Round-Trip**
    - **Validates: Requirements 9.2**
  
  - [ ] 24.6 Implement selective undo with conflict detection
    - Detect dependencies between operations
    - Check for conflicts before undo
    - Suggest resolution strategies
    - _Requirements: 9.3, 9.7_
  
  - [ ]* 24.7 Write property test for conflict detection
    - **Property 33: Selective Undo Conflict Detection**
    - **Validates: Requirements 9.3, 9.7**
  
  - [ ] 24.8 Implement branching
    - Create branches from any operation
    - Switch between branches
    - Restore world state on branch switch
    - _Requirements: 9.4, 9.5_
  
  - [ ]* 24.9 Write property test for branching
    - **Property 34: Operation Timeline Branching**
    - **Validates: Requirements 9.4, 9.5**
  
  - [ ] 24.10 Implement operation history persistence
    - Serialize timeline to disk
    - Load timeline from disk
    - Preserve all operations and branches
    - _Requirements: 9.8_
  
  - [ ]* 24.11 Write property test for persistence round-trip
    - **Property 35: Operation History Persistence Round-Trip**
    - **Validates: Requirements 9.8**
  
  - [ ] 24.12 Implement operation summarization
    - Generate summaries for recent operations
    - Format for AI context inclusion
    - Include operation IDs and descriptions
    - _Requirements: 9.6_

- [ ] 25. Implement Performance Advisor
  - [ ] 25.1 Create PerformanceAdvisor struct
    - Set up performance metrics tracking
    - Initialize cost model
    - Configure performance budget
    - _Requirements: 10.1_
  
  - [ ] 25.2 Implement cost model
    - Define ComponentCost structure
    - Populate costs for all component types
    - Create entity count to frame time curve
    - _Requirements: 10.3_
  
  - [ ] 25.3 Implement impact estimation
    - Estimate FPS impact for spawn operations
    - Estimate memory impact
    - Estimate draw call impact
    - _Requirements: 10.1_
  
  - [ ]* 25.4 Write property test for impact estimation accuracy
    - **Property 36: Performance Impact Estimation Accuracy**
    - **Validates: Requirements 10.1**
  
  - [ ] 25.5 Implement warning generation
    - Warn when estimated FPS < 30
    - Suggest optimizations (GPU instancing, LOD, reduced count)
    - Include specific recommendations
    - _Requirements: 10.2, 10.6_
  
  - [ ]* 25.6 Write property test for warning threshold
    - **Property 37: Performance Warning Threshold**
    - **Validates: Requirements 10.2, 10.6**
  
  - [ ] 25.7 Implement performance context generation
    - Include current FPS, entity count, draw calls, memory
    - Include performance guidelines
    - Format for AI context
    - _Requirements: 10.4_
  
  - [ ]* 25.8 Write property test for context completeness
    - **Property 38: Performance Context Completeness**
    - **Validates: Requirements 10.4**
  
  - [ ] 25.9 Implement cost model learning
    - Collect actual performance measurements
    - Update cost model based on measurements
    - Improve prediction accuracy over time
    - _Requirements: 10.7_
  
  - [ ]* 25.10 Write property test for model learning
    - **Property 39: Performance Model Learning**
    - **Validates: Requirements 10.7**

- [ ] 26. Checkpoint - Ensure operation timeline and performance advisor tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 27. Implement Visual Feedback System
  - [ ] 27.1 Create VisualFeedbackSystem struct
    - Set up screenshot capturer
    - Initialize scene visualizer
    - Configure multimodal LLM client
    - _Requirements: 11.1_
  
  - [ ] 27.2 Implement viewport capture
    - Capture current framebuffer
    - Resize to 512x512
    - Compress to JPEG format
    - _Requirements: 11.1_
  
  - [ ]* 27.3 Write property test for capture format compliance
    - **Property 40: Viewport Capture Format Compliance**
    - **Validates: Requirements 11.1**
  
  - [ ] 27.4 Implement visual annotations
    - Overlay entity names on screenshot
    - Draw bounding boxes for entities
    - Mark light positions
    - Highlight selected entities
    - _Requirements: 11.2_
  
  - [ ]* 27.5 Write property test for annotation presence
    - **Property 41: Visual Annotation Presence**
    - **Validates: Requirements 11.2**
  
  - [ ] 27.6 Implement viewport.capture MCP tool
    - Define tool schema
    - Capture viewport on request
    - Return image data
    - _Requirements: 11.3_
  
  - [ ] 27.7 Implement before/after comparison
    - Capture before state
    - Capture after state
    - Generate comparison data
    - _Requirements: 11.4_
  
  - [ ]* 27.8 Write property test for comparison support
    - **Property 42: Visual Comparison Support**
    - **Validates: Requirements 11.4**
  
  - [ ] 27.9 Implement scene description generation
    - Generate compact text description
    - Include entity counts, types, layout
    - Format for AI context
    - _Requirements: 11.6_
  
  - [ ]* 27.10 Write property test for context inclusion
    - **Property 43: Visual Context Inclusion**
    - **Validates: Requirements 11.6**
  
  - [ ] 27.11 Implement image compression
    - Compress images to minimize tokens
    - Maintain visual clarity
    - Target 50% size reduction
    - _Requirements: 11.7_
  
  - [ ]* 27.12 Write property test for compression efficiency
    - **Property 44: Image Compression Token Efficiency**
    - **Validates: Requirements 11.7**

- [ ] 28. Implement AI CLI Tools
  - [ ] 28.1 Create luminara_cli crate structure
    - Set up CLI argument parsing with clap
    - Create subcommands: ai generate, ai scaffold
    - Configure LLM client
    - _Requirements: 12.1_
  
  - [ ] 28.2 Implement ai generate command
    - Parse user prompt
    - Send to LLM for game design
    - Parse LLM response
    - _Requirements: 12.1_
  
  - [ ] 28.3 Implement project scaffolding
    - Create directory structure
    - Generate Cargo.toml with dependencies
    - Generate main.rs with engine setup
    - Create assets/, scenes/, scripts/ directories
    - _Requirements: 12.2, 12.3_
  
  - [ ]* 28.4 Write property test for project completeness
    - **Property 45: AI Project Generation Completeness**
    - **Validates: Requirements 12.2, 12.3**
  
  - [ ] 28.5 Implement scene file generation
    - Parse game design for entities
    - Generate .luminara.ron files
    - Validate scene file format
    - _Requirements: 12.4_
  
  - [ ]* 28.6 Write property test for scene file validity
    - **Property 46: Generated Scene File Validity**
    - **Validates: Requirements 12.4**
  
  - [ ] 28.7 Implement script generation
    - Parse game design for behaviors
    - Generate Lua scripts with lifecycle hooks
    - Include proper engine API usage
    - _Requirements: 12.5_
  
  - [ ]* 28.8 Write property test for script validity
    - **Property 47: Generated Script Validity**
    - **Validates: Requirements 12.5**
  
  - [ ] 28.9 Implement project build verification
    - Run cargo build on generated project
    - Report compilation errors
    - Verify project runs
    - _Requirements: 12.6_
  
  - [ ] 28.10 Create game templates
    - Create template: 2D platformer
    - Create template: 3D FPS
    - Create template: top-down RPG
    - Create template: puzzle game
    - _Requirements: 12.7_
  
  - [ ] 28.11 Implement error handling
    - Catch generation failures
    - Provide detailed error messages
    - Return partial results when possible
    - _Requirements: 12.8_
  
  - [ ]* 28.12 Write property test for error reporting
    - **Property 48: AI Generation Error Reporting**
    - **Validates: Requirements 12.8**

- [ ] 29. Checkpoint - Ensure visual feedback and CLI tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [ ] 30. Implement Multi-Agent Orchestration
  - [ ] 30.1 Create AgentOrchestrator struct
    - Set up agent registry
    - Initialize responsibility map
    - Create operation serializer
    - Set up message bus
    - _Requirements: 15.1_
  
  - [ ] 30.2 Implement agent registration
    - Register agents with roles and permissions
    - Support roles: SceneArchitect, GameplayProgrammer, ArtDirector, QAEngineer, ProjectDirector
    - Assign permissions based on role
    - _Requirements: 15.1, 15.2_
  
  - [ ]* 30.3 Write property test for role assignment
    - **Property 52: Multi-Agent Role Assignment**
    - **Validates: Requirements 15.1**
  
  - [ ] 30.4 Implement task decomposition
    - Use ProjectDirector to analyze requests
    - Decompose into sub-tasks
    - Assign sub-tasks to appropriate agents
    - _Requirements: 15.3_
  
  - [ ]* 30.5 Write property test for task decomposition
    - **Property 53: Task Decomposition Correctness**
    - **Validates: Requirements 15.3**
  
  - [ ] 30.6 Implement execution planning
    - Build dependency graph for sub-tasks
    - Identify independent tasks for parallelization
    - Order dependent tasks correctly
    - _Requirements: 15.4, 15.5_
  
  - [ ]* 30.7 Write property test for parallelization
    - **Property 54: Independent Task Parallelization**
    - **Validates: Requirements 15.4**
  
  - [ ] 30.8 Implement conflict detection and resolution
    - Detect conflicting changes to same entity/component
    - Apply resolution strategy (last-write-wins, merge, user prompt)
    - Log conflicts for review
    - _Requirements: 15.6_
  
  - [ ]* 30.9 Write property test for conflict detection
    - **Property 55: Agent Conflict Detection and Resolution**
    - **Validates: Requirements 15.6**
  
  - [ ] 30.10 Implement inter-agent messaging
    - Create message bus for agent communication
    - Support message sending and receiving
    - Deliver messages within one cycle
    - _Requirements: 15.7_
  
  - [ ]* 30.11 Write property test for message delivery
    - **Property 56: Inter-Agent Message Delivery**
    - **Validates: Requirements 15.7**
  
  - [ ] 30.12 Implement change summarization
    - Collect changes from all agents
    - Organize by agent role
    - Generate comprehensive summary
    - _Requirements: 15.8_
  
  - [ ]* 30.13 Write property test for summarization
    - **Property 57: Multi-Agent Change Summarization**
    - **Validates: Requirements 15.8**

- [ ] 31. Integration and System Testing
  - [ ] 31.1 Create end-to-end integration tests
    - Test MCP → Intent Resolver → Engine workflow
    - Test Script hot reload workflow
    - Test Code verification workflow
    - Test Operation timeline workflow
    - Test Multi-agent collaboration workflow
    - _Requirements: All Phase 2 requirements_
  
  - [ ] 31.2 Create example Lua scripts
    - Create player controller script
    - Create enemy AI script
    - Create camera follow script
    - Create UI interaction script
    - _Requirements: 1.1-1.8_
  
  - [ ] 31.3 Create example WASM scripts
    - Create performance-critical physics script
    - Create procedural generation script
    - Demonstrate Rust → WASM workflow
    - _Requirements: 2.1-2.8_
  
  - [ ] 31.4 Create MCP client examples
    - Create Python MCP client example
    - Create TypeScript MCP client example
    - Demonstrate AI agent workflows
    - _Requirements: 4.1-4.8_
  
  - [ ] 31.5 Create AI CLI usage examples
    - Document ai generate command usage
    - Provide example prompts
    - Show generated project structure
    - _Requirements: 12.1-12.8_
  
  - [ ] 31.6 Performance testing
    - Test 10,000 scripts execute in < 16ms
    - Test hot reload latency < 100ms
    - Test world digest generation < 500ms for 10k entities
    - Test intent resolution < 10ms
    - Test MCP tool latency < 100ms
    - _Requirements: All Phase 2 requirements_

- [ ] 32. Documentation and Examples
  - [ ] 32.1 Write Lua scripting guide
    - Document all engine APIs
    - Provide code examples
    - Explain lifecycle hooks
    - Cover hot reload workflow
    - _Requirements: 1.1-1.8_
  
  - [ ] 32.2 Write WASM scripting guide
    - Document WIT interface
    - Provide Rust guest examples
    - Explain resource limits
    - Cover compilation workflow
    - _Requirements: 2.1-2.8_
  
  - [ ] 32.3 Write MCP integration guide
    - Document all MCP tools
    - Provide client examples
    - Explain AI context system
    - Cover best practices
    - _Requirements: 4.1-4.8, 5.1-5.7, 6.1-6.7_
  
  - [ ] 32.4 Write AI CLI guide
    - Document ai generate command
    - Provide example prompts
    - Explain project templates
    - Cover customization
    - _Requirements: 12.1-12.8_
  
  - [ ] 32.5 Create Phase 2 demo project
    - Showcase Lua and WASM scripting
    - Demonstrate AI-generated content
    - Show hot reload in action
    - Include MCP client integration
    - _Requirements: All Phase 2 requirements_

- [ ] 33. Final checkpoint - Ensure all Phase 2 tests pass
  - Run full test suite
  - Verify all property tests pass
  - Verify all integration tests pass
  - Verify performance benchmarks meet targets
  - Ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional property-based tests that can be skipped for faster MVP delivery
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties with minimum 100 iterations
- Unit tests validate specific examples and edge cases
- Integration tests validate end-to-end workflows
- The implementation follows a bottom-up approach: scripting layer first, then AI integration layer
- Hot reload is critical for developer experience and should be thoroughly tested
- Code verification pipeline is essential for safety and should not be skipped
- Performance advisor helps AI make informed decisions about resource usage
- Multi-agent orchestration enables complex collaborative workflows
