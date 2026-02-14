# Implementation Plan: Phase 1 Core Engine Features

## Overview

This implementation plan breaks down Phase 1 into incremental, testable tasks. The approach prioritizes scene serialization (for Phase 2 AI integration), then builds rendering, physics, and audio systems. Each major component includes property-based tests to validate correctness properties from the design document.

## Environment Setup

**Rust Execution:** Use WSL Ubuntu environment with cargo at `/home/arat2/.cargo/bin/cargo`
- Command format: `wsl -d ubuntu /home/arat2/.cargo/bin/cargo <command>`

The implementation follows this order:
1. Scene system with serialization (foundation for all other features)
2. Transform hierarchy (required by rendering and physics)
3. Asset pipeline with hot-reload (required by all asset-dependent systems)
4. Camera system (required by rendering)
5. 3D PBR rendering with Forward+ pipeline
6. 2D sprite rendering with batching
7. Physics integration (Rapier 3D/2D)
8. Audio system (kira)
9. Plugin system improvements
10. Phase 1 demo integration

## Tasks

- [x] 1. Set up scene system foundation
  - [x] 1.1 Create scene data structures and serialization
    - Implement `Scene`, `SceneMeta`, `EntityData` structs in `luminara_scene/src/lib.rs`
    - Add serde derives for RON/JSON serialization
    - Implement `ComponentSchema` and `FieldSchema` for AI introspection
    - Create registry for component schemas
    - _Requirements: 1.1, 1.6_
  
  - [x] 1.2 Write property test for scene serialization
    - **Property 1: Scene Round-Trip Consistency**
    - **Validates: Requirements 1.1, 1.2, 1.4, 1.5**
    - Generate random scenes with entities, components, hierarchies
    - Serialize to RON, deserialize, verify equivalence
    - Test with 100+ iterations
  
  - [x] 1.3 Write property test for component schema availability
    - **Property 2: Component Schema Availability**
    - **Validates: Requirements 1.6**
    - For all registered component types, verify schema exists
    - Validate schema contains type name, description, fields

- [x] 2. Implement transform hierarchy system
  - [x] 2.1 Create transform components
    - Implement `Transform` component (local transform) in `luminara_scene/src/transform.rs`
    - Implement `GlobalTransform` component (world transform)
    - Implement `Parent` and `Children` components for hierarchy
    - Add `to_matrix()` method for Transform
    - _Requirements: 5.1, 5.2, 5.3_
  
  - [x] 2.2 Implement transform propagation system
    - Create `transform_propagate_system` that traverses hierarchy
    - Use breadth-first traversal to update GlobalTransform
    - Handle parent-child matrix multiplication
    - Add to `TransformPropagate` stage
    - _Requirements: 5.1, 5.2_
  
  - [x] 2.3 Write property test for transform hierarchy propagation
    - **Property 8: Transform Hierarchy Propagation**
    - **Validates: Requirements 5.1, 5.2, 5.3**
    - Generate random parent-child hierarchies with transforms
    - Verify child world transform = parent world * child local
    - Test with various hierarchy depths
  
  - [x] 2.4 Write property test for child detachment
    - **Property 9: Child Detachment Preserves World Position**
    - **Validates: Requirements 5.5**
    - Generate parent-child pairs, detach child
    - Verify world position unchanged after detachment

- [x] 3. Checkpoint - Verify scene and transform systems
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement asset pipeline with hot-reload
  - [x] 4.1 Create asset server core
    - Implement `AssetServer` in `luminara_asset/src/lib.rs`
    - Implement `Handle<T>` type-safe asset references
    - Create `AssetStorage` for loaded assets
    - Implement `Asset` trait
    - Add handle allocator
    - _Requirements: 4.3, 4.4_
  
  - [x] 4.2 Implement file watcher for hot-reload
    - Add `notify` crate dependency
    - Implement `FileWatcher` using `notify::RecommendedWatcher`
    - Create `asset_hot_reload_system` to poll file events
    - Watch assets directory recursively
    - _Requirements: 4.1, 4.2_
  
  - [x] 4.3 Implement mesh asset loader
    - Add `gltf` crate dependency
    - Implement `Mesh` struct with vertices, indices, AABB
    - Implement `Asset` trait for `Mesh` with GLTF loading
    - Parse vertex positions, normals, UVs, tangents
    - Parse indices
    - _Requirements: 4.5_
  
  - [x] 4.4 Implement texture asset loader
    - Add `image` crate dependency
    - Implement `Texture` struct
    - Implement `Asset` trait for `Texture` with PNG/JPG/HDR loading
    - Support multiple formats via `image` crate
    - _Requirements: 4.6_
  
  - [x] 4.5 Write property test for asset hot-reload detection
    - **Property 3: Asset Hot-Reload Detection**
    - **Validates: Requirements 4.1, 4.2**
    - Create temp asset files, modify them
    - Verify file watcher detects changes
    - Test with various file types
  
  - [x] 4.6 Write property test for asset load and retrieve
    - **Property 4: Asset Load and Retrieve**
    - **Validates: Requirements 4.3, 4.4**
    - Request assets with various paths
    - Verify handles are returned and assets are accessible
  
  - [x] 4.7 Write property test for format support
    - **Property 5: Format Support**
    - **Validates: Requirements 4.5, 4.6**
    - Load valid GLTF/GLB/PNG/JPG/HDR files
    - Verify successful loading and correct format
  
  - [x] 4.8 Write property test for error handling
    - **Property 6: Asset Load Error Handling**
    - **Validates: Requirements 4.7**
    - Test invalid paths, corrupted files
    - Verify no crashes, errors logged, fallback provided
  
  - [x] 4.9 Write property test for asset caching
    - **Property 7: Asset Caching**
    - **Validates: Requirements 4.8**
    - Load same asset multiple times
    - Verify processing happens only once

- [x] 5. Implement camera system
  - [x] 5.1 Create camera component
    - Implement `Camera` component in `luminara_render/src/components.rs`
    - Implement `Projection` enum (Perspective, Orthographic)
    - Add `projection_matrix()` and `view_matrix()` methods
    - Add `is_active` and `clear_color` fields
    - _Requirements: 6.1, 6.4_
  
  - [x] 5.2 Implement camera systems
    - Create `camera_projection_system` to update projection on parameter changes
    - Create `camera_resize_system` to update aspect ratio on window resize
    - Add systems to appropriate stages
    - _Requirements: 6.2, 6.5_
  
  - [x] 5.3 Write property test for projection modes
    - **Property 10: Projection Mode Support**
    - **Validates: Requirements 6.1, 6.4**
    - Generate cameras with random projection parameters
    - Verify valid projection matrices for both modes
  
  - [x] 5.4 Write property test for projection matrix update
    - **Property 11: Projection Matrix Update**
    - **Validates: Requirements 6.2**
    - Change projection parameters
    - Verify matrix is recomputed and differs
  
  - [x] 5.5 Write property test for aspect ratio update
    - **Property 12: Camera Aspect Ratio Update**
    - **Validates: Requirements 6.5**
    - Simulate window resize events
    - Verify camera aspect ratio updates

- [x] 6. Checkpoint - Verify asset and camera systems
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement 3D PBR rendering
  - [x] 7.1 Create PBR material and mesh renderer components
    - Implement `PbrMaterial` struct in `luminara_render/src/components.rs`
    - Add albedo, metallic, roughness, normal map fields
    - Implement `MeshRenderer` component
    - Add cast_shadows and receive_shadows flags
    - _Requirements: 2.1_
  
  - [x] 7.2 Create light components
    - Implement `DirectionalLight` component
    - Implement `PointLight` component
    - Add color, intensity, shadow casting fields
    - _Requirements: 2.2, 2.3_
  
  - [x] 7.3 Implement Forward+ rendering pipeline
    - Create render graph nodes for Forward+ pipeline
    - Implement light culling pass (tile-based)
    - Implement main lighting pass with PBR shading
    - Add WGSL shaders for PBR (albedo, metallic, roughness, normal mapping)
    - _Requirements: 2.1, 2.2, 2.5_
  
  - [x] 7.4 Implement cascaded shadow mapping
    - Create shadow map render pass
    - Implement cascade splitting for directional lights
    - Add shadow sampling in PBR shader
    - _Requirements: 2.3_
  
  - [x] 7.5 Implement post-processing
    - Add gamma correction pass
    - Add tone mapping pass (ACES or Reinhard)
    - _Requirements: 2.4_
  
  - [x] 7.6 Write unit tests for PBR rendering
    - Test shader uniform setup
    - Test shadow map creation
    - Test post-processing pipeline
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 8. Implement 2D sprite rendering with batching
  - [x] 8.1 Create sprite components
    - Implement `Sprite` component in `luminara_render/src/sprite.rs`
    - Add texture, color, rect, flip, anchor fields
    - Implement `ZOrder` component for depth sorting
    - Implement `Anchor` enum
    - _Requirements: 3.2_
  
  - [x] 8.2 Implement sprite batch renderer
    - Create `SpriteBatcher` struct
    - Implement `prepare()` method to build batches
    - Sort sprites by z-order and texture
    - Group sprites by texture into batches
    - Create `SpriteInstance` struct for GPU data
    - _Requirements: 3.1, 3.3_
  
  - [x] 8.3 Create sprite rendering pipeline
    - Add sprite vertex and fragment shaders (WGSL)
    - Implement instanced rendering for sprite batches
    - Set up alpha blending for transparency
    - Support texture atlases with UV rects
    - _Requirements: 3.4, 3.5_
  
  - [x] 8.4 Write property test for sprite batching
    - **Property 13: Sprite Batching by Texture**
    - **Validates: Requirements 3.1**
    - Generate sprites with same/different textures
    - Verify batching groups by texture
  
  - [x] 8.5 Write property test for sprite instance data
    - **Property 14: Sprite Instance Data Correctness**
    - **Validates: Requirements 3.2, 3.4**
    - Generate sprites with various properties
    - Verify instance data has correct transform, color, UVs
  
  - [x] 8.6 Write property test for sprite sorting
    - **Property 15: Sprite Sorting**
    - **Validates: Requirements 3.3**
    - Generate sprites with random z-order and textures
    - Verify sorting by z-order then texture

- [x] 9. Checkpoint - Verify rendering systems
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Integrate Rapier physics (3D and 2D)
  - [x] 10.1 Add Rapier dependencies and create physics plugin
    - Add `rapier3d` and `rapier2d` crate dependencies
    - Create `luminara_physics` crate
    - Create `PhysicsPlugin` for 3D physics
    - Create `Physics2dPlugin` for 2D physics
    - Initialize Rapier worlds in plugin build
    - _Requirements: 7.3_
  
  - [x] 10.2 Create physics components
    - Implement `RigidBody` component in `luminara_physics/src/components.rs`
    - Add body_type, mass, damping, gravity_scale fields
    - Implement `RigidBodyType` enum (Dynamic, Kinematic, Static)
    - Implement `Collider` component
    - Implement `ColliderShape` enum (Box, Sphere, Capsule, Mesh)
    - _Requirements: 7.1, 7.5_
  
  - [x] 10.3 Implement physics simulation systems
    - Create `physics_step_system` to step Rapier simulation
    - Run in `FixedUpdate` stage at 60 Hz
    - Create `physics_sync_system` to sync Rapier state to ECS transforms
    - Run in `PostUpdate` stage
    - Handle NaN detection and error recovery
    - _Requirements: 7.4, 7.7_
  
  - [x] 10.4 Implement collision detection
    - Create `CollisionEvent` struct
    - Emit collision events from Rapier contact events
    - Store events in ECS event bus
    - _Requirements: 7.2, 7.6_
  
  - [x] 10.5 Write property test for collision detection
    - **Property 16: Collision Detection and Events**
    - **Validates: Requirements 7.2, 7.6**
    - Generate overlapping colliders
    - Verify collision events are emitted
  
  - [x] 10.6 Write property test for physics transform sync
    - **Property 17: Physics Transform Synchronization**
    - **Validates: Requirements 7.4, 7.7**
    - Step physics simulation
    - Verify ECS transforms match physics state
  
  - [x] 10.7 Write property test for rigid body types
    - **Property 18: Rigid Body Type Behavior**
    - **Validates: Requirements 7.5**
    - Test Dynamic, Kinematic, Static bodies
    - Verify correct behavior for each type

- [x] 11. Implement audio system with kira
  - [x] 11.1 Add kira dependency and create audio plugin
    - Add `kira` crate dependency
    - Create `luminara_audio` crate
    - Create `AudioPlugin`
    - Initialize kira audio manager in plugin build
    - _Requirements: 8.1_
  
  - [x] 11.2 Create audio components
    - Implement `AudioSource` component in `luminara_audio/src/components.rs`
    - Add clip, volume, pitch, looping, spatial fields
    - Implement `AudioListener` component
    - Create `AudioCommand` enum (Play, Pause, Resume, Stop)
    - _Requirements: 8.2, 8.3, 8.4_
  
  - [x] 11.3 Implement audio asset loading
    - Implement `AudioClip` asset type
    - Load WAV, OGG, MP3 formats via kira
    - _Requirements: 8.1_
  
  - [x] 11.4 Implement audio playback system
    - Create `audio_system` to process AudioCommands
    - Handle play, pause, resume, stop commands
    - Update spatial audio based on source and listener positions
    - Calculate distance attenuation
    - _Requirements: 8.2, 8.3, 8.6_
  
  - [x] 11.5 Write property test for audio file loading
    - **Property 19: Audio File Loading**
    - **Validates: Requirements 8.1**
    - Load valid audio files
    - Verify successful decoding
  
  - [x] 11.6 Write property test for audio playback control
    - **Property 20: Audio Playback Control**
    - **Validates: Requirements 8.2, 8.6**
    - Test play, pause, resume, stop commands
    - Verify correct playback state
  
  - [x] 11.7 Write property test for spatial audio
    - **Property 21: Spatial Audio Distance Attenuation**
    - **Validates: Requirements 8.3**
    - Vary distance between source and listener
    - Verify volume attenuation
  
  - [x] 11.8 Write property test for audio looping
    - **Property 22: Audio Looping**
    - **Validates: Requirements 8.4**
    - Enable looping on audio source
    - Verify audio restarts after finishing

- [x] 12. Checkpoint - Verify physics and audio systems
  - Ensure all tests pass, ask the user if questions arise.

- [x] 13. Enhance plugin system
  - [x] 13.1 Improve plugin trait and registration
    - Ensure `Plugin` trait has `build()` method in `luminara_core/src/plugin.rs`
    - Implement plugin registration in `App`
    - Track plugin execution order
    - _Requirements: 9.1, 9.4_
  
  - [x] 13.2 Enable system and resource registration from plugins
    - Allow plugins to add systems to any stage
    - Allow plugins to register components and resources
    - _Requirements: 9.2, 9.3_
  
  - [x] 13.3 Create DefaultPlugins bundle
    - Bundle ScenePlugin, AssetPlugin, RenderPlugin, PhysicsPlugin, AudioPlugin
    - Ensure correct initialization order
    - _Requirements: 9.5_
  
  - [x] 13.4 Write property test for plugin build invocation
    - **Property 23: Plugin Build Invocation**
    - **Validates: Requirements 9.1**
    - Register plugins, verify build called once
  
  - [x] 13.5 Write property test for plugin system registration
    - **Property 24: Plugin System Registration**
    - **Validates: Requirements 9.2**
    - Plugin adds systems to stages
    - Verify systems present in schedule
  
  - [x] 13.6 Write property test for plugin component registration
    - **Property 25: Plugin Component and Resource Registration**
    - **Validates: Requirements 9.3**
    - Plugin registers components/resources
    - Verify types available in world
  
  - [x] 13.7 Write property test for plugin execution order
    - **Property 26: Plugin Execution Order**
    - **Validates: Requirements 9.4**
    - Register plugins in sequence
    - Verify build order matches registration order

- [x] 14. Create Phase 1 demo scene and integration
  - [x] 14.1 Create demo scene file
    - Create `assets/scenes/phase1_demo.scene.ron`
    - Add camera with perspective projection
    - Add directional light with shadows
    - Add ground plane with static collider
    - Add falling sphere with dynamic rigid body
    - Add PBR materials for objects
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  
  - [x] 14.2 Create demo application
    - Create `examples/phase1_demo/main.rs`
    - Initialize engine with DefaultPlugins
    - Load demo scene from asset pipeline
    - Add background music or sound effects
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  
  - [x] 14.3 Write integration tests for demo
    - Test scene loads without errors
    - Test physics simulation runs
    - Test rendering produces frames
    - Test audio plays
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [x] 15. Final checkpoint - Phase 1 complete
  - Run Phase 1 demo and verify all features work
  - Ensure all property tests pass (26 properties)
  - Verify hot-reload works for assets and scenes
  - Ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional property-based tests and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- Checkpoints ensure incremental validation throughout implementation
- The demo serves as the final integration test for Phase 1
