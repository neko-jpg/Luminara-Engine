# Requirements Document: Phase 2 - Scripting & AI Integration

## Introduction

Phase 2 of the Luminara Engine implements the scripting layer and AI-driven development infrastructure. This phase transforms Luminara from a traditional game engine into an AI-first development platform where AI agents can understand, manipulate, and generate game content through structured APIs and intelligent context management.

The system enables both human developers and AI agents to script game logic through Lua and WASM runtimes, while providing AI agents with sophisticated tools for understanding game state, generating code safely, and making performance-aware decisions.

## Glossary

- **Luminara_Engine**: The core game engine built in Rust with ECS architecture
- **MCP_Server**: Model Context Protocol server implementing Anthropic's standard for AI tool integration
- **Script_Runtime**: The execution environment for Lua or WASM scripts
- **AI_Context_Engine**: System for generating optimal context for AI agents within token budgets
- **World_Digest**: Hierarchical compression of game world state for AI consumption
- **Intent_Resolver**: System that resolves AI operations based on semantic intent rather than concrete values
- **Schema_Discovery**: Progressive system for AI to discover component schemas on-demand
- **Code_Verifier**: Multi-stage pipeline for validating AI-generated code before execution
- **Operation_Timeline**: Immutable log of all AI operations supporting undo/redo/branching
- **Performance_Advisor**: System that predicts performance impact of AI operations
- **Sandbox**: Isolated execution environment for testing scripts safely
- **Entity**: A game object in the ECS system
- **Component**: Data attached to entities defining their properties
- **System**: Logic that operates on entities with specific components
- **Hot_Reload**: Automatic reloading of changed scripts without restarting the engine

## Requirements

### Requirement 1: Lua Scripting Runtime

**User Story:** As a game developer, I want to write game logic in Lua scripts, so that I can rapidly prototype and iterate on gameplay without recompiling the engine.

#### Acceptance Criteria

1. WHEN the Lua runtime initializes, THE Script_Runtime SHALL load the mlua library and register all engine APIs
2. WHEN a Lua script file is loaded, THE Script_Runtime SHALL compile it and cache the bytecode for subsequent executions
3. WHEN a script calls an engine API function, THE Script_Runtime SHALL validate parameters and execute the corresponding Rust function safely
4. WHEN a script file is modified on disk, THE Script_Runtime SHALL detect the change and reload the script within 100ms
5. WHEN a script throws an error, THE Script_Runtime SHALL capture the stack trace and report it with file name and line number
6. WHEN multiple scripts execute in a frame, THE Script_Runtime SHALL execute them in dependency order based on system scheduling
7. THE Script_Runtime SHALL expose Transform, Input, World, Time, and Physics APIs to Lua scripts
8. WHEN a script accesses an Entity, THE Script_Runtime SHALL provide type-safe wrappers preventing invalid operations

### Requirement 2: WASM Scripting Runtime

**User Story:** As a game developer, I want to write high-performance scripts in Rust/C/Go compiled to WASM, so that I can achieve near-native performance for compute-intensive game logic.

#### Acceptance Criteria

1. WHEN the WASM runtime initializes, THE Script_Runtime SHALL create a wasmtime engine with resource limits configured
2. WHEN a WASM module is loaded, THE Script_Runtime SHALL validate it and instantiate it with engine API imports
3. WHEN a WASM script calls an engine API, THE Script_Runtime SHALL marshal data across the WASM boundary safely
4. THE Script_Runtime SHALL enforce memory limits of 64MB per WASM instance
5. THE Script_Runtime SHALL enforce execution time limits preventing infinite loops
6. WHEN a WASM module is recompiled, THE Script_Runtime SHALL hot-reload it preserving script state where possible
7. THE Script_Runtime SHALL provide WIT interface definitions for all engine APIs
8. WHEN a WASM script panics, THE Script_Runtime SHALL isolate the failure and report detailed error information

### Requirement 3: Script Hot Reload System

**User Story:** As a game developer, I want scripts to reload automatically when I save changes, so that I can see results immediately without restarting the game.

#### Acceptance Criteria

1. WHEN the hot reload system starts, THE Script_Runtime SHALL watch all script directories for file changes
2. WHEN a script file is modified, THE Script_Runtime SHALL detect the change within 100ms
3. WHEN reloading a script, THE Script_Runtime SHALL preserve entity references and component data
4. WHEN a script reload fails, THE Script_Runtime SHALL keep the previous version active and log the error
5. WHEN multiple scripts change simultaneously, THE Script_Runtime SHALL reload them in dependency order
6. THE Script_Runtime SHALL provide callbacks for scripts to save/restore state during reload
7. WHEN a script is reloaded, THE Script_Runtime SHALL call on_reload lifecycle hooks if defined

### Requirement 4: MCP Server Implementation

**User Story:** As an AI agent, I want to interact with the Luminara Engine through standardized MCP tools, so that I can create and modify game content programmatically.

#### Acceptance Criteria

1. WHEN the MCP server starts, THE MCP_Server SHALL listen on the configured port and register all available tools
2. WHEN an AI agent connects, THE MCP_Server SHALL provide tool schemas describing all available operations
3. WHEN a tool is invoked, THE MCP_Server SHALL validate input against the schema and return structured errors for invalid input
4. THE MCP_Server SHALL provide tools for: scene.create_entity, scene.modify_component, scene.query_entities, script.create, script.modify, debug.inspect, project.scaffold
5. WHEN a tool executes successfully, THE MCP_Server SHALL return structured results including entity IDs and change summaries
6. WHEN a tool fails, THE MCP_Server SHALL return error details with suggestions for correction
7. THE MCP_Server SHALL maintain a connection to the running engine instance for real-time operations
8. WHEN the engine is not running, THE MCP_Server SHALL operate in offline mode allowing project scaffolding and file operations

### Requirement 5: AI Context Engine - World Digest

**User Story:** As an AI agent, I want to receive optimally compressed game world state within my token budget, so that I can understand the current game state without exceeding context limits.

#### Acceptance Criteria

1. WHEN generating a world digest, THE AI_Context_Engine SHALL analyze the AI query to determine relevant entities
2. WHEN the token budget is limited, THE AI_Context_Engine SHALL prioritize relevant entities with full details over complete entity lists
3. THE AI_Context_Engine SHALL provide L0 (world summary ~500 tokens), L1 (entity catalog ~2000 tokens), L2 (entity details ~5000 tokens), and L3 (full entity ~1000 tokens) digest levels
4. WHEN an AI query mentions specific entities, THE AI_Context_Engine SHALL include those entities at L2 or L3 detail level
5. WHEN generating a digest, THE AI_Context_Engine SHALL include recent changes (last 60 seconds) if token budget allows
6. THE AI_Context_Engine SHALL use semantic indexing to find entities matching natural language descriptions
7. WHEN the world has more than 1000 entities, THE AI_Context_Engine SHALL use spatial and semantic clustering to select representative samples

### Requirement 6: AI Context Engine - Schema Discovery

**User Story:** As an AI agent, I want to discover component schemas progressively, so that I only receive schema information for components I'm actually working with.

#### Acceptance Criteria

1. WHEN an AI agent requests context, THE AI_Context_Engine SHALL include L0 (brief) schemas for all registered components
2. WHEN the current scene uses specific components, THE AI_Context_Engine SHALL include L1 (fields summary) schemas for those components
3. WHEN an AI query indicates intent to modify a component, THE AI_Context_Engine SHALL include L2 (full schema with examples) for that component
4. THE AI_Context_Engine SHALL provide a schema.inspect tool allowing AI to request detailed schema for any component
5. WHEN providing component schemas, THE AI_Context_Engine SHALL include commonly paired components and usage examples
6. THE AI_Context_Engine SHALL categorize components into: Rendering, Physics, Gameplay, Audio, AI, and UI categories
7. WHEN token budget is limited, THE AI_Context_Engine SHALL prioritize schemas for components in the current scene

### Requirement 7: Intent Resolver

**User Story:** As an AI agent, I want my operations to resolve based on semantic intent rather than concrete IDs, so that my commands remain valid even as the game world changes.

#### Acceptance Criteria

1. WHEN an AI operation references an entity by name, THE Intent_Resolver SHALL resolve it to the current entity with that name at execution time
2. WHEN an AI operation uses relative positioning (e.g., "3m in front of player"), THE Intent_Resolver SHALL compute absolute position at execution time
3. WHEN an AI operation uses semantic queries (e.g., "all enemies"), THE Intent_Resolver SHALL resolve to all matching entities at execution time
4. WHEN an entity reference cannot be resolved, THE Intent_Resolver SHALL return an error with suggestions for similar entities
5. THE Intent_Resolver SHALL support entity references by: name, ID, tag, component type, spatial proximity, and semantic description
6. WHEN multiple entities match a semantic query, THE Intent_Resolver SHALL return all matches or the closest match based on context
7. WHEN resolving relative positions, THE Intent_Resolver SHALL support: Forward, Above, AtOffset, RandomInRadius, and RandomReachable

### Requirement 8: Code Verification Pipeline

**User Story:** As a game developer, I want AI-generated code to be automatically verified before execution, so that unsafe or incorrect code cannot crash the engine or corrupt my project.

#### Acceptance Criteria

1. WHEN AI generates a script, THE Code_Verifier SHALL perform static analysis detecting infinite loops, undefined references, and dangerous patterns
2. WHEN static analysis passes, THE Code_Verifier SHALL execute the script in a sandbox with resource limits
3. THE Code_Verifier SHALL enforce sandbox limits: 5 second execution time, 64MB memory, 1000 max entities spawned, 10000 max API calls, 1M max instructions
4. WHEN sandbox execution succeeds, THE Code_Verifier SHALL generate a diff preview showing expected changes
5. WHEN applying verified code, THE Code_Verifier SHALL create a rollback checkpoint before modification
6. WHEN code execution causes runtime errors, THE Code_Verifier SHALL automatically rollback to the checkpoint
7. THE Code_Verifier SHALL monitor the first 2 seconds after applying code for anomalies and rollback if detected
8. WHEN verification fails, THE Code_Verifier SHALL provide fix suggestions based on the detected issues

### Requirement 9: Operation Timeline

**User Story:** As a game developer, I want to undo/redo AI operations selectively, so that I can keep beneficial changes while reverting problematic ones.

#### Acceptance Criteria

1. WHEN an AI operation executes, THE Operation_Timeline SHALL record the operation with its intent, commands, and inverse commands
2. WHEN undoing an operation, THE Operation_Timeline SHALL execute the inverse commands to restore previous state
3. WHEN an operation depends on previous operations, THE Operation_Timeline SHALL detect conflicts before selective undo
4. THE Operation_Timeline SHALL support creating branches from any operation point (Git-like workflow)
5. WHEN switching branches, THE Operation_Timeline SHALL restore world state to that branch's head
6. THE Operation_Timeline SHALL provide operation summaries for the last N operations as AI context
7. WHEN operations conflict, THE Operation_Timeline SHALL suggest resolution strategies to the user
8. THE Operation_Timeline SHALL persist operation history to disk for recovery across sessions

### Requirement 10: Performance Advisor

**User Story:** As an AI agent, I want to understand the performance impact of my operations before executing them, so that I can make performance-aware decisions.

#### Acceptance Criteria

1. WHEN an AI operation would spawn entities, THE Performance_Advisor SHALL estimate the FPS impact based on component costs
2. WHEN estimated FPS drops below 30, THE Performance_Advisor SHALL warn the AI and suggest alternatives
3. THE Performance_Advisor SHALL maintain a cost model with CPU/GPU/memory costs per component type
4. WHEN generating context for AI, THE Performance_Advisor SHALL include current performance metrics (FPS, entity count, draw calls, memory usage)
5. THE Performance_Advisor SHALL provide performance guidelines in AI context (e.g., "Max recommended entities: 5000")
6. WHEN an operation would exceed performance budgets, THE Performance_Advisor SHALL suggest optimizations (GPU instancing, LOD, reduced count)
7. THE Performance_Advisor SHALL learn from actual performance measurements to improve cost model accuracy

### Requirement 11: Visual Feedback System

**User Story:** As an AI agent, I want to capture and analyze visual output of the engine, so that I can make decisions based on how the game looks, not just data.

#### Acceptance Criteria

1. WHEN requested, THE Visual_Feedback_System SHALL capture the current viewport as a low-resolution (512x512) JPEG image
2. WHEN capturing for AI analysis, THE Visual_Feedback_System SHALL optionally overlay entity names, bounding boxes, and light positions
3. THE Visual_Feedback_System SHALL provide a viewport.capture MCP tool for AI agents
4. WHEN AI makes visual changes, THE Visual_Feedback_System SHALL support before/after comparison captures
5. THE Visual_Feedback_System SHALL integrate with multimodal LLMs to analyze visual output
6. WHEN sending images to AI, THE Visual_Feedback_System SHALL include compact scene descriptions alongside the image
7. THE Visual_Feedback_System SHALL compress images to minimize token usage while preserving visual clarity

### Requirement 12: AI CLI Tools

**User Story:** As a game developer, I want to use AI to generate complete game projects from natural language descriptions, so that I can rapidly prototype game ideas.

#### Acceptance Criteria

1. WHEN running `luminara ai generate <prompt>`, THE Luminara_CLI SHALL send the prompt to the configured LLM for game design
2. WHEN the LLM returns a design, THE Luminara_CLI SHALL generate project structure, scene files, and scripts
3. THE Luminara_CLI SHALL scaffold projects with: Cargo.toml, main.rs, assets directory, scenes directory, scripts directory
4. WHEN generating scenes, THE Luminara_CLI SHALL create valid .luminara.ron files with entities and components
5. WHEN generating scripts, THE Luminara_CLI SHALL create Lua scripts with proper lifecycle hooks and engine API usage
6. WHEN generation completes, THE Luminara_CLI SHALL build the project and report any compilation errors
7. THE Luminara_CLI SHALL provide templates for common game types: 2D platformer, 3D FPS, top-down RPG, puzzle game
8. WHEN generation fails, THE Luminara_CLI SHALL provide detailed error messages and partial results

### Requirement 13: Script Sandbox Execution

**User Story:** As a game developer, I want untrusted scripts to run in a sandbox, so that malicious or buggy scripts cannot harm my system or corrupt game state.

#### Acceptance Criteria

1. WHEN a script runs in sandbox mode, THE Script_Runtime SHALL enforce memory limits preventing excessive allocation
2. WHEN a script runs in sandbox mode, THE Script_Runtime SHALL enforce execution time limits preventing infinite loops
3. THE Script_Runtime SHALL provide instruction counting for Lua scripts using mlua hooks
4. WHEN a sandbox limit is exceeded, THE Script_Runtime SHALL terminate the script and return a detailed error
5. THE Script_Runtime SHALL restrict sandbox scripts from: file system access, network access, process spawning, and FFI calls
6. WHEN testing AI-generated scripts, THE Script_Runtime SHALL execute them in sandbox mode by default
7. THE Script_Runtime SHALL allow whitelisting specific scripts for unrestricted execution after verification

### Requirement 14: Engine API Exposure

**User Story:** As a script developer, I want comprehensive engine APIs exposed to scripts, so that I can implement any game logic without dropping to Rust.

#### Acceptance Criteria

1. THE Script_Runtime SHALL expose Transform API for: get/set position, rotation, scale, and computing forward/right/up vectors
2. THE Script_Runtime SHALL expose Input API for: key/button state, axis values, and mouse position/delta
3. THE Script_Runtime SHALL expose World API for: finding entities by name/tag/component, spawning entities, and destroying entities
4. THE Script_Runtime SHALL expose Physics API for: applying forces/impulses, raycasting, and querying collisions
5. THE Script_Runtime SHALL expose Audio API for: playing sounds, setting volume/pitch, and 3D spatial audio
6. THE Script_Runtime SHALL expose Time API for: delta time, total time, and frame count
7. THE Script_Runtime SHALL expose Component API for: getting/setting component values with type safety
8. WHEN scripts call APIs with invalid parameters, THE Script_Runtime SHALL return clear error messages indicating the problem

### Requirement 15: Multi-Agent Orchestration

**User Story:** As a project manager, I want multiple specialized AI agents to collaborate on game development, so that complex projects can be divided among expert agents.

#### Acceptance Criteria

1. WHEN multiple AI agents are registered, THE Agent_Orchestrator SHALL assign each agent a role and permissions
2. THE Agent_Orchestrator SHALL support roles: SceneArchitect, GameplayProgrammer, ArtDirector, QAEngineer, ProjectDirector
3. WHEN a user request arrives, THE Agent_Orchestrator SHALL use the ProjectDirector agent to decompose it into sub-tasks
4. WHEN sub-tasks are independent, THE Agent_Orchestrator SHALL execute them in parallel
5. WHEN sub-tasks have dependencies, THE Agent_Orchestrator SHALL execute them in correct order
6. WHEN agents make conflicting changes, THE Agent_Orchestrator SHALL detect conflicts and apply resolution strategy
7. THE Agent_Orchestrator SHALL provide inter-agent messaging for coordination
8. WHEN all agents complete their tasks, THE Agent_Orchestrator SHALL summarize all changes for user review
