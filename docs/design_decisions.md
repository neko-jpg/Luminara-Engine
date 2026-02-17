# Design Decisions and Rationale

This document explains the key architectural and design decisions made in Luminara Engine, along with the rationale, trade-offs, and alternatives considered.

## Table of Contents

1. [ECS Architecture](#ecs-architecture)
2. [Rendering Pipeline](#rendering-pipeline)
3. [Asset Management](#asset-management)
4. [Scripting System](#scripting-system)
5. [AI Integration](#ai-integration)
6. [Math Foundation](#math-foundation)
7. [Database Integration](#database-integration)
8. [Performance Priorities](#performance-priorities)

---

## ECS Architecture

### Decision: Archetype-Based Storage

**Chosen Approach:** Archetype-based ECS with dense component storage

**Rationale:**
- **Cache Efficiency**: Components of the same type stored contiguously in memory
- **Fast Iteration**: Systems iterate over dense arrays, maximizing cache hits
- **SIMD Opportunities**: Contiguous data enables vectorization
- **Predictable Performance**: Linear iteration with minimal indirection

**Alternatives Considered:**

1. **Sparse Set ECS** (like EnTT)
   - Pros: Fast component add/remove, stable entity IDs
   - Cons: More cache misses during iteration, higher memory overhead
   - Why not chosen: Iteration performance is more critical than add/remove

2. **Table-Based ECS** (like Flecs)
   - Pros: Flexible queries, good for complex relationships
   - Cons: More complex implementation, potential performance overhead
   - Why not chosen: Simpler archetype model sufficient for our needs

3. **Object-Oriented Hierarchy**
   - Pros: Familiar to many developers, simple inheritance
   - Cons: Poor cache performance, rigid hierarchies, difficult to extend
   - Why not chosen: Performance and flexibility requirements

**Trade-offs:**
- ✅ Excellent iteration performance
- ✅ Simple mental model
- ❌ Component add/remove requires archetype migration (slower)
- ❌ More memory usage than sparse sets

**Performance Impact:**
- ECS iteration: 312,500 entities/ms
- 1M entities with 5 components: 3.2ms per frame
- Competitive with Bevy, faster than Unity ECS

---

## Rendering Pipeline

### Decision: Forward+ Rendering

**Chosen Approach:** Forward+ (tiled forward rendering) with PBR materials

**Rationale:**
- **Many Lights**: Supports 100+ dynamic lights efficiently
- **Simplicity**: Simpler than deferred rendering, easier to debug
- **Transparency**: Handles transparent objects naturally
- **Modern GPUs**: Leverages compute shaders for light culling

**Alternatives Considered:**

1. **Deferred Rendering**
   - Pros: Constant lighting cost, many lights
   - Cons: No MSAA, limited material variety, transparency issues
   - Why not chosen: Transparency handling is problematic

2. **Forward Rendering (Traditional)**
   - Pros: Simple, good for few lights
   - Cons: Poor scaling with light count
   - Why not chosen: Can't handle 100+ lights efficiently

3. **Clustered Rendering**
   - Pros: Better than Forward+, 3D light culling
   - Cons: More complex, higher overhead for few lights
   - Why not chosen: Forward+ sufficient for target use cases

**Trade-offs:**
- ✅ Handles many lights efficiently
- ✅ Natural transparency support
- ✅ Simpler than deferred
- ❌ More expensive than deferred for very many lights
- ❌ Requires compute shader support

**Performance Impact:**
- 100 dynamic lights: 60 FPS at 1080p
- Light culling: <0.5ms per frame
- Competitive with modern engines

### Decision: Cascaded Shadow Maps

**Chosen Approach:** 4-cascade CSM for directional lights

**Rationale:**
- **Quality**: High detail near camera, acceptable quality far away
- **Performance**: 4 cascades balance quality and cost
- **Industry Standard**: Proven technique used in AAA games

**Alternatives Considered:**

1. **Single Shadow Map**
   - Pros: Simple, fast
   - Cons: Poor quality at distance or near camera
   - Why not chosen: Quality unacceptable

2. **Virtual Shadow Maps** (UE5 style)
   - Pros: Excellent quality, adaptive resolution
   - Cons: Complex, high memory usage, requires modern GPU
   - Why not chosen: Too complex for initial implementation

3. **Ray-Traced Shadows**
   - Pros: Perfect quality
   - Cons: Requires RT hardware, very expensive
   - Why not chosen: Not widely available, too expensive

**Trade-offs:**
- ✅ Good quality/performance balance
- ✅ Works on all GPUs
- ✅ Proven technique
- ❌ Visible cascade transitions (mitigated with blending)
- ❌ Fixed resolution per cascade

**Performance Impact:**
- 4 cascades, 2048x2048: <2ms GPU time
- Acceptable for 60 FPS target

---

## Asset Management

### Decision: Fully Asynchronous Loading

**Chosen Approach:** All asset I/O on background threads, never block main thread

**Rationale:**
- **Smooth Gameplay**: No frame drops during loading
- **Better UX**: Game remains responsive
- **Modern Expectation**: Users expect seamless loading
- **Scalability**: Handles large assets without freezing

**Alternatives Considered:**

1. **Synchronous Loading**
   - Pros: Simple implementation
   - Cons: Freezes game during load
   - Why not chosen: Unacceptable UX

2. **Hybrid Approach** (small assets sync, large async)
   - Pros: Simple for small assets
   - Cons: Inconsistent behavior, still has freezes
   - Why not chosen: Consistency is important

**Trade-offs:**
- ✅ Never blocks main thread
- ✅ Smooth gameplay
- ✅ Better UX
- ❌ More complex implementation
- ❌ Requires placeholder assets

**Performance Impact:**
- Main thread overhead: <0.1ms per frame
- Load time: Texture (2K PNG) = 15ms on background thread
- Zero frame drops during loading

### Decision: Hot Reload Support

**Chosen Approach:** File watching with automatic asset reload

**Rationale:**
- **Iteration Speed**: See changes immediately
- **Artist Workflow**: Artists can iterate without restarting
- **Developer Productivity**: Faster development cycle
- **Industry Standard**: Expected feature in modern engines

**Alternatives Considered:**

1. **Manual Reload**
   - Pros: Simple, no file watching overhead
   - Cons: Slower iteration, manual process
   - Why not chosen: Productivity loss

2. **Full Restart Required**
   - Pros: Simplest implementation
   - Cons: Very slow iteration
   - Why not chosen: Unacceptable for productivity

**Trade-offs:**
- ✅ Fast iteration
- ✅ Better artist workflow
- ✅ Competitive advantage
- ❌ File watching overhead (minimal)
- ❌ State preservation complexity

**Performance Impact:**
- Hot reload time: Texture = 45ms, Script = 95ms
- File watching overhead: <0.01ms per frame
- Excellent iteration speed

---

## Scripting System

### Decision: Dual Runtime (Lua + WASM)

**Chosen Approach:** Support both Lua and WASM scripting

**Rationale:**
- **Flexibility**: Lua for rapid prototyping, WASM for performance
- **Accessibility**: Lua easy to learn, WASM for existing C/C++/Rust code
- **Performance Range**: Cover both quick iteration and production needs
- **Modding**: Lua excellent for user-created content

**Alternatives Considered:**

1. **Lua Only**
   - Pros: Simple, one runtime
   - Cons: Performance limitations
   - Why not chosen: Need performance option

2. **WASM Only**
   - Pros: Best performance, one runtime
   - Cons: Harder to learn, slower iteration
   - Why not chosen: Too high barrier for beginners

3. **JavaScript/TypeScript**
   - Pros: Popular language, large ecosystem
   - Cons: Heavier runtime, less predictable performance
   - Why not chosen: Performance concerns

4. **Python**
   - Pros: Very popular, easy to learn
   - Cons: Slow, GIL issues, large runtime
   - Why not chosen: Performance unacceptable

**Trade-offs:**
- ✅ Flexibility for different use cases
- ✅ Performance when needed
- ✅ Easy prototyping
- ❌ Two runtimes to maintain
- ❌ More complex API surface

**Performance Impact:**
- Lua: 2-5x slower than Rust (acceptable for game logic)
- WASM: 1.2-1.5x slower than Rust (excellent for performance-critical code)
- Hot reload: Lua <100ms, WASM <200ms

### Decision: Sandboxed Execution

**Chosen Approach:** Strict sandboxing with resource limits

**Rationale:**
- **Safety**: Scripts can't crash engine
- **Security**: Untrusted scripts (mods) can't harm system
- **Stability**: Script errors isolated
- **Resource Control**: Prevent runaway scripts

**Alternatives Considered:**

1. **No Sandboxing**
   - Pros: Simpler, full access
   - Cons: Unsafe, scripts can crash engine
   - Why not chosen: Unacceptable for modding

2. **Partial Sandboxing**
   - Pros: Some safety
   - Cons: Inconsistent, still vulnerable
   - Why not chosen: Need complete safety

**Trade-offs:**
- ✅ Safe execution
- ✅ Stable engine
- ✅ Enables modding
- ❌ API restrictions
- ❌ Performance overhead (minimal)

**Performance Impact:**
- Sandbox overhead: <5% for Lua, <2% for WASM
- Acceptable for safety benefits

---

## AI Integration

### Decision: Integrated AI Agent System

**Chosen Approach:** Built-in AI assistance with MCP protocol

**Rationale:**
- **Innovation**: Unique selling point vs other engines
- **Productivity**: AI accelerates development
- **Future-Proof**: AI will be essential for game development
- **Accessibility**: Lowers barrier to entry

**Alternatives Considered:**

1. **No AI Integration**
   - Pros: Simpler, no AI dependencies
   - Cons: Miss opportunity for innovation
   - Why not chosen: AI is the future

2. **External AI Tools**
   - Pros: Separation of concerns
   - Cons: Poor integration, manual workflow
   - Why not chosen: Want seamless experience

3. **Editor-Only AI**
   - Pros: Simpler runtime
   - Cons: Limited use cases
   - Why not chosen: Want runtime AI capabilities too

**Trade-offs:**
- ✅ Unique feature
- ✅ Productivity boost
- ✅ Future-proof
- ❌ Complexity
- ❌ AI API costs
- ❌ Requires internet (for cloud AI)

**Performance Impact:**
- Context generation: <500ms for 10K entities
- Minimal runtime overhead when not in use
- Acceptable for productivity gains

### Decision: Hierarchical World Digest

**Chosen Approach:** Multi-level context with token budget management

**Rationale:**
- **Scalability**: Handle large scenes within token limits
- **Efficiency**: Only send relevant information
- **Cost Control**: Minimize AI API costs
- **Flexibility**: Different detail levels for different queries

**Alternatives Considered:**

1. **Full World Dump**
   - Pros: Complete information
   - Cons: Exceeds token limits, expensive
   - Why not chosen: Not scalable

2. **Fixed Summary**
   - Pros: Simple, predictable
   - Cons: May miss important details
   - Why not chosen: Not flexible enough

**Trade-offs:**
- ✅ Scalable to large scenes
- ✅ Cost-effective
- ✅ Flexible detail levels
- ❌ May miss some context
- ❌ Complex implementation

---

## Math Foundation

### Decision: Projective Geometric Algebra (PGA)

**Chosen Approach:** PGA-based math with Motor transforms

**Rationale:**
- **No Gimbal Lock**: Motors avoid gimbal lock issues
- **Unified Representation**: Rotation + translation in single structure
- **Mathematical Elegance**: Clean geometric operations
- **Innovation**: Differentiation from other engines

**Alternatives Considered:**

1. **Quaternions + Vectors** (Standard)
   - Pros: Well-known, proven, simple
   - Cons: Gimbal lock, separate rotation/translation
   - Why not chosen: Want to innovate

2. **Euler Angles**
   - Pros: Intuitive
   - Cons: Gimbal lock, order-dependent
   - Why not chosen: Gimbal lock unacceptable

3. **Dual Quaternions**
   - Pros: Unified rotation + translation
   - Cons: Less elegant than PGA, harder to understand
   - Why not chosen: PGA more elegant

**Trade-offs:**
- ✅ No gimbal lock
- ✅ Elegant mathematics
- ✅ Unique feature
- ❌ Less familiar to developers
- ❌ Requires education/documentation
- ❌ Slightly more complex

**Performance Impact:**
- Motor operations: Competitive with quaternions (SIMD optimized)
- Benchmark: Motor composition ~20ns, Quat composition ~18ns
- Acceptable overhead for benefits

### Decision: Lie Group Integrators

**Chosen Approach:** Optional Lie group integration for physics

**Rationale:**
- **Stability**: More stable than Euler for high angular velocity
- **Accuracy**: Better energy conservation
- **Innovation**: Advanced feature for demanding users
- **Optional**: Can fall back to Euler

**Alternatives Considered:**

1. **Euler Integration Only**
   - Pros: Simple, fast
   - Cons: Less stable, energy drift
   - Why not chosen: Want better option available

2. **Runge-Kutta (RK4)**
   - Pros: More accurate than Euler
   - Cons: Not as stable as Lie group for rotations
   - Why not chosen: Lie group better for rigid bodies

**Trade-offs:**
- ✅ Better stability
- ✅ Better accuracy
- ✅ Optional (can use Euler)
- ❌ Slightly slower (~10-20% overhead)
- ❌ More complex

**Performance Impact:**
- Lie integrator: ~15% slower than Euler
- Acceptable for improved stability

---

## Database Integration

### Decision: SurrealDB Embedded Mode

**Chosen Approach:** Embedded SurrealDB for asset management and undo/redo

**Rationale:**
- **Graph Queries**: Natural for asset dependencies
- **Embedded**: No external server required
- **WASM Support**: Works in browser
- **Modern**: Future-proof database choice

**Alternatives Considered:**

1. **SQLite**
   - Pros: Proven, lightweight, fast
   - Cons: No graph queries, relational model awkward for assets
   - Why not chosen: Graph model better fit

2. **Custom Format** (RON files)
   - Pros: Simple, human-readable
   - Cons: No queries, no relationships, manual management
   - Why not chosen: Need query capabilities

3. **PostgreSQL**
   - Pros: Powerful, proven
   - Cons: Requires external server, heavy
   - Why not chosen: Want embedded solution

**Trade-offs:**
- ✅ Graph queries
- ✅ Embedded (no server)
- ✅ WASM support
- ❌ Newer technology (less proven)
- ❌ Larger binary size

**Performance Impact:**
- Query time: <10ms for complex graph queries
- Sync latency: <16ms (acceptable for 60 FPS)
- Acceptable for benefits

---

## Performance Priorities

### Decision: 60 FPS Minimum, 120 FPS Target

**Chosen Approach:** Optimize for 120 FPS, guarantee 60 FPS

**Rationale:**
- **Competitive**: Match or exceed other engines
- **Future-Proof**: High refresh rate monitors common
- **Headroom**: 120 FPS target provides margin
- **User Expectation**: 60 FPS is minimum acceptable

**Alternatives Considered:**

1. **30 FPS Target**
   - Pros: Easier to achieve
   - Cons: Unacceptable for modern games
   - Why not chosen: Too low

2. **60 FPS Target Only**
   - Pros: Achievable
   - Cons: No headroom, not competitive
   - Why not chosen: Want to exceed expectations

**Trade-offs:**
- ✅ Competitive performance
- ✅ Future-proof
- ✅ Good user experience
- ❌ Requires aggressive optimization
- ❌ More development effort

**Performance Impact:**
- Achieved: 120+ FPS in typical scenes
- Benchmark: 10K entities at 120 FPS
- Exceeds target

### Decision: Aggressive Batching and Instancing

**Chosen Approach:** Automatic draw call batching and GPU instancing

**Rationale:**
- **Performance**: Massive improvement for repeated objects
- **Automatic**: No manual work required
- **Scalability**: Handles large scenes
- **Industry Standard**: Expected optimization

**Alternatives Considered:**

1. **Manual Batching**
   - Pros: Developer control
   - Cons: Error-prone, tedious
   - Why not chosen: Want automatic optimization

2. **No Batching**
   - Pros: Simple
   - Cons: Poor performance
   - Why not chosen: Unacceptable performance

**Trade-offs:**
- ✅ Massive performance improvement (10x+)
- ✅ Automatic
- ✅ Scalable
- ❌ Complexity in renderer
- ❌ Some edge cases

**Performance Impact:**
- Without: 1000 objects = 1000 draw calls, 45 FPS
- With batching: 1000 objects = 87 draw calls, 120 FPS
- With instancing: 1000 objects = 12 draw calls, 240 FPS
- Excellent results

---

## Summary

These design decisions reflect our priorities:

1. **Performance**: Industry-leading performance through aggressive optimization
2. **Innovation**: Unique features (PGA, AI integration) for differentiation
3. **Developer Experience**: Fast iteration, hot reload, good tooling
4. **Flexibility**: Multiple options (Lua/WASM, Euler/Lie) for different needs
5. **Future-Proof**: Modern technologies (wgpu, SurrealDB, AI) for longevity

Each decision involves trade-offs, but we believe the chosen approaches best serve our goals of creating a high-performance, innovative, and developer-friendly game engine.

## Further Reading

- [Architecture Documentation](architecture/) - Detailed architecture docs
- [Performance Benchmarks](audit/checkpoint_30_final_report.md) - Performance results
- [API Documentation](api/) - API reference
- [Migration Guides](migration/) - Migrating from other engines
