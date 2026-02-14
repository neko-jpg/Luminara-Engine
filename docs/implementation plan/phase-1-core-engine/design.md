# Design Document - Phase 1: Core Engine Features

## Overview

Phase 1 transforms the Luminara Engine from a basic rendering prototype into a fully functional 3D game engine. Building on Phase 0's ECS foundation, windowing, input, and basic rendering, this phase adds:

- **Scene System**: Serializable scene graphs with transform hierarchies and component schemas for AI integration
- **Advanced Rendering**: Forward+ PBR pipeline with cascaded shadow maps, lighting, and post-processing
- **2D Rendering**: Batched sprite system for efficient 2D game development
- **Asset Pipeline**: Hot-reloadable asset system supporting GLTF meshes, textures, and scene files
- **Physics**: Rapier integration for both 3D and 2D physics simulation
- **Audio**: Kira-based audio system with spatial sound support
- **Plugin Architecture**: Extensible plugin system for modular engine features

The design prioritizes scene serialization as the foundation for Phase 2's AI integration, uses Forward+ rendering for WASM compatibility, and implements hot-reload for both assets and scene files to enable rapid iteration without a GUI editor.

## Architecture

### System Dependencies

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│                    (examples/phase1_demo)                    │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│                      Plugin System                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │  Scene   │ │  Render  │ │ Physics  │ │  Audio   │      │
│  │  Plugin  │ │  Plugin  │ │  Plugin  │ │  Plugin  │      │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘      │
└───────┼────────────┼────────────┼────────────┼─────────────┘
        │            │            │            │
┌───────▼────────────▼────────────▼────────────▼─────────────┐
│                    ECS Core (Phase 0)                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  World, Schedule, Systems, Components, Resources     │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Execution Flow

```
App Startup
    │
    ├─> Load Plugins (DefaultPlugins)
    │   ├─> ScenePlugin: Register scene systems
    │   ├─> AssetPlugin: Start file watcher
    │   ├─> RenderPlugin: Initialize GPU resources
    │   ├─> PhysicsPlugin: Create Rapier world
    │   └─> AudioPlugin: Initialize kira manager
    │
    ├─> Load Initial Scene (from .ron file)
    │   ├─> Deserialize entities and components
    │   ├─> Build transform hierarchy
    │   └─> Load referenced assets
    │
    └─> Enter Main Loop
        │
        ├─> PreUpdate Stage
        │   ├─> Process input events
        │   └─> Check for hot-reload triggers
        │
        ├─> Update Stage
        │   ├─> Run game logic systems
        │   └─> Update audio sources
        │
        ├─> FixedUpdate Stage (60 Hz)
        │   └─> Step physics simulation
        │
        ├─> PostUpdate Stage
        │   └─> Sync physics transforms to ECS
        │
        ├─> TransformPropagate Stage
        │   └─> Update world transforms from hierarchy
        │
        ├─> PreRender Stage
        │   ├─> Frustum culling
        │   └─> Build render batches
        │
        └─> Render Stage
            ├─> Shadow pass (cascaded)
            ├─> Forward+ lighting pass
            ├─> 2D sprite batch pass
            └─> Post-processing
```

## Components and Interfaces

### Scene System Components

```rust
// luminara_scene/src/lib.rs

/// Scene data structure (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub meta: SceneMeta,
    pub entities: Vec<EntityData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMeta {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub name: String,
    pub parent: Option<String>,  // Parent entity name
    pub components: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
}

/// Component schema for AI understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub type_name: String,
    pub description: String,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    pub description: String,
}

/// Scene loader system
pub fn scene_loader_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scenes: Res<Assets<Scene>>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // Load and spawn scenes
}

/// Scene serializer
pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(world: &World) -> Result<Scene, SerializeError> {
        // Serialize all entities and components
    }
    
    pub fn deserialize(scene: &Scene, world: &mut World) -> Result<(), DeserializeError> {
        // Reconstruct entities from scene data
    }
}
```

### Transform Hierarchy Components

```rust
// luminara_scene/src/transform.rs

/// Local transform (relative to parent)
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// World transform (absolute)
#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalTransform {
    matrix: Mat4,
}

impl GlobalTransform {
    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }
}

/// Parent-child relationship
#[derive(Component, Debug, Clone)]
pub struct Parent(pub Entity);

#[derive(Component, Debug, Clone, Default)]
pub struct Children(pub Vec<Entity>);

/// Transform propagation system
pub fn transform_propagate_system(
    mut root_query: Query<(Entity, &Transform, &mut GlobalTransform, Option<&Children>), Without<Parent>>,
    mut child_query: Query<(&Transform, &mut GlobalTransform, &Parent, Option<&Children>)>,
) {
    // Breadth-first traversal to propagate transforms
    for (entity, transform, mut global, children) in root_query.iter_mut() {
        global.matrix = transform.to_matrix();
        
        if let Some(children) = children {
            propagate_recursive(&children.0, &global.matrix, &mut child_query);
        }
    }
}

fn propagate_recursive(
    children: &[Entity],
    parent_matrix: &Mat4,
    child_query: &mut Query<(&Transform, &mut GlobalTransform, &Parent, Option<&Children>)>,
) {
    for &child in children {
        if let Ok((transform, mut global, _, grandchildren)) = child_query.get_mut(child) {
            global.matrix = *parent_matrix * transform.to_matrix();
            
            if let Some(grandchildren) = grandchildren {
                propagate_recursive(&grandchildren.0, &global.matrix, child_query);
            }
        }
    }
}
```

### Rendering Components

```rust
// luminara_render/src/components.rs

/// Mesh renderer component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

/// PBR Material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMaterial {
    pub albedo: Color,
    pub albedo_texture: Option<Handle<Texture>>,
    pub normal_texture: Option<Handle<Texture>>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    pub emissive: Color,
}

/// Light components
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub color: Color,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub shadow_cascade_count: u32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
}

/// Camera component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub projection: Projection,
    pub is_active: bool,
    pub clear_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Projection {
    Perspective {
        fov: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        size: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective { fov, near, far } => {
                Mat4::perspective_rh(fov.to_radians(), aspect_ratio, near, far)
            }
            Projection::Orthographic { size, near, far } => {
                let half_width = size * aspect_ratio * 0.5;
                let half_height = size * 0.5;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, near, far)
            }
        }
    }
    
    pub fn view_matrix(&self, global_transform: &GlobalTransform) -> Mat4 {
        global_transform.matrix().inverse()
    }
}
```

### 2D Sprite Components

```rust
// luminara_render/src/sprite.rs

/// Sprite component for 2D rendering
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: Handle<Texture>,
    pub color: Color,
    pub rect: Option<Rect>,  // For sprite atlases
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: Anchor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Anchor {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Z-order for 2D sorting
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZOrder(pub f32);

/// Sprite batch renderer
pub struct SpriteBatcher {
    batches: Vec<SpriteBatch>,
    max_sprites_per_batch: usize,
}

struct SpriteBatch {
    texture: Handle<Texture>,
    instances: Vec<SpriteInstance>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteInstance {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
    uv_rect: [f32; 4],
}

impl SpriteBatcher {
    pub fn prepare(
        &mut self,
        sprites: Query<(&Sprite, &GlobalTransform, Option<&ZOrder>)>,
    ) {
        self.batches.clear();
        
        // Sort sprites by texture and z-order
        let mut sorted_sprites: Vec<_> = sprites.iter().collect();
        sorted_sprites.sort_by(|(a_sprite, _, a_z), (b_sprite, _, b_z)| {
            let a_z = a_z.map(|z| z.0).unwrap_or(0.0);
            let b_z = b_z.map(|z| z.0).unwrap_or(0.0);
            a_z.partial_cmp(&b_z).unwrap()
                .then_with(|| a_sprite.texture.id().cmp(&b_sprite.texture.id()))
        });
        
        // Build batches
        for (sprite, transform, _) in sorted_sprites {
            self.add_to_batch(sprite, transform);
        }
    }
    
    fn add_to_batch(&mut self, sprite: &Sprite, transform: &GlobalTransform) {
        // Find or create batch for this texture
        let batch = self.batches.iter_mut()
            .find(|b| b.texture == sprite.texture && b.instances.len() < self.max_sprites_per_batch)
            .or_else(|| {
                self.batches.push(SpriteBatch {
                    texture: sprite.texture.clone(),
                    instances: Vec::new(),
                });
                self.batches.last_mut()
            })
            .unwrap();
        
        // Add instance
        batch.instances.push(SpriteInstance {
            transform: transform.matrix().to_cols_array_2d(),
            color: sprite.color.as_rgba_f32(),
            uv_rect: sprite.rect.map(|r| [r.min.x, r.min.y, r.max.x, r.max.y])
                .unwrap_or([0.0, 0.0, 1.0, 1.0]),
        });
    }
}
```

### Physics Components

```rust
// luminara_physics/src/components.rs

/// Rigid body component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RigidBodyType {
    Dynamic,
    Kinematic,
    Static,
}

/// Collider component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub is_sensor: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { half_height: f32, radius: f32 },
    Mesh { vertices: Vec<Vec3>, indices: Vec<[u32; 3]> },
}

/// Collision event
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub started: bool,  // true = collision started, false = collision ended
}

/// Physics sync system
pub fn physics_sync_system(
    mut physics_world: ResMut<RapierPhysicsWorld>,
    mut query: Query<(Entity, &mut Transform, &RigidBody)>,
) {
    // Sync physics state to ECS transforms
    for (entity, mut transform, _) in query.iter_mut() {
        if let Some(body_handle) = physics_world.entity_to_body.get(&entity) {
            if let Some(body) = physics_world.bodies.get(*body_handle) {
                let position = body.translation();
                let rotation = body.rotation();
                
                transform.translation = Vec3::new(position.x, position.y, position.z);
                transform.rotation = Quat::from_xyzw(rotation.i, rotation.j, rotation.k, rotation.w);
            }
        }
    }
}
```

### Audio Components

```rust
// luminara_audio/src/components.rs

/// Audio source component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource {
    pub clip: Handle<AudioClip>,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub spatial: bool,
    pub max_distance: f32,
}

/// Audio listener component (typically on camera)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AudioListener {
    pub enabled: bool,
}

/// Audio playback control
pub enum AudioCommand {
    Play(Entity),
    Pause(Entity),
    Resume(Entity),
    Stop(Entity),
}

/// Audio system
pub fn audio_system(
    mut audio_manager: ResMut<KiraAudioManager>,
    sources: Query<(Entity, &AudioSource, &GlobalTransform)>,
    listener: Query<&GlobalTransform, With<AudioListener>>,
    mut commands: EventReader<AudioCommand>,
) {
    // Process audio commands
    for command in commands.iter() {
        match command {
            AudioCommand::Play(entity) => {
                if let Ok((_, source, transform)) = sources.get(*entity) {
                    audio_manager.play(source, transform);
                }
            }
            // ... handle other commands
        }
    }
    
    // Update spatial audio
    if let Ok(listener_transform) = listener.get_single() {
        for (_, source, transform) in sources.iter() {
            if source.spatial {
                audio_manager.update_spatial(source, transform, listener_transform);
            }
        }
    }
}
```

### Asset System

```rust
// luminara_asset/src/lib.rs

/// Asset server with hot-reload
pub struct AssetServer {
    loader: AssetLoader,
    storage: AssetStorage,
    watcher: FileWatcher,
    handle_allocator: HandleAllocator,
}

impl AssetServer {
    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        let handle = self.handle_allocator.allocate();
        self.loader.load_async(path, handle.clone());
        handle
    }
    
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<&T> {
        self.storage.get(handle)
    }
}

/// Asset handle (type-safe reference)
#[derive(Debug, Clone)]
pub struct Handle<T: Asset> {
    id: AssetId,
    _phantom: PhantomData<T>,
}

/// Asset trait
pub trait Asset: Send + Sync + 'static {
    fn load(bytes: &[u8]) -> Result<Self, AssetLoadError> where Self: Sized;
}

/// Hot-reload watcher
pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    events: Receiver<notify::Event>,
}

impl FileWatcher {
    pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.watch(path, notify::RecursiveMode::Recursive)
    }
    
    pub fn poll_events(&mut self) -> Vec<AssetPath> {
        self.events.try_iter()
            .filter_map(|event| {
                if let notify::EventKind::Modify(_) = event.kind {
                    event.paths.first().map(|p| AssetPath::from(p))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Asset hot-reload system
pub fn asset_hot_reload_system(
    mut asset_server: ResMut<AssetServer>,
    mut watcher: ResMut<FileWatcher>,
) {
    for path in watcher.poll_events() {
        info!("Hot-reloading asset: {:?}", path);
        asset_server.reload(&path);
    }
}
```

## Data Models

### Scene File Format (RON)

```ron
// assets/scenes/demo.scene.ron
Scene(
    meta: SceneMeta(
        name: "Phase 1 Demo Scene",
        version: "1.0.0",
        description: "Demo scene with physics objects",
    ),
    entities: [
        EntityData(
            name: "Camera",
            parent: None,
            components: {
                "Transform": (
                    translation: (0.0, 5.0, 10.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                ),
                "Camera": (
                    projection: Perspective(
                        fov: 60.0,
                        near: 0.1,
                        far: 1000.0,
                    ),
                    is_active: true,
                    clear_color: (0.1, 0.1, 0.15, 1.0),
                ),
                "AudioListener": (
                    enabled: true,
                ),
            },
            tags: ["camera"],
        ),
        EntityData(
            name: "Sun",
            parent: None,
            components: {
                "Transform": (
                    translation: (0.0, 10.0, 0.0),
                    rotation: (0.3826834, 0.0, 0.0, 0.9238795),  // 45 degrees
                    scale: (1.0, 1.0, 1.0),
                ),
                "DirectionalLight": (
                    color: (1.0, 0.95, 0.9, 1.0),
                    intensity: 3.0,
                    cast_shadows: true,
                    shadow_cascade_count: 4,
                ),
            },
            tags: ["light"],
        ),
        EntityData(
            name: "Ground",
            parent: None,
            components: {
                "Transform": (
                    translation: (0.0, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (10.0, 0.5, 10.0),
                ),
                "MeshRenderer": (
                    mesh: "meshes/cube.glb",
                    material: "materials/ground.mat",
                    cast_shadows: false,
                    receive_shadows: true,
                ),
                "Collider": (
                    shape: Box(half_extents: (5.0, 0.25, 5.0)),
                    friction: 0.5,
                    restitution: 0.0,
                    is_sensor: false,
                ),
                "RigidBody": (
                    body_type: Static,
                    mass: 0.0,
                    linear_damping: 0.0,
                    angular_damping: 0.0,
                    gravity_scale: 1.0,
                ),
            },
            tags: ["ground"],
        ),
        EntityData(
            name: "Sphere",
            parent: None,
            components: {
                "Transform": (
                    translation: (0.0, 5.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                ),
                "MeshRenderer": (
                    mesh: "meshes/sphere.glb",
                    material: "materials/metal.mat",
                    cast_shadows: true,
                    receive_shadows: true,
                ),
                "Collider": (
                    shape: Sphere(radius: 0.5),
                    friction: 0.3,
                    restitution: 0.7,
                    is_sensor: false,
                ),
                "RigidBody": (
                    body_type: Dynamic,
                    mass: 1.0,
                    linear_damping: 0.1,
                    angular_damping: 0.1,
                    gravity_scale: 1.0,
                ),
            },
            tags: ["physics"],
        ),
    ],
)
```

### Material File Format (RON)

```ron
// assets/materials/metal.mat.ron
PbrMaterial(
    albedo: (0.8, 0.8, 0.8, 1.0),
    albedo_texture: Some("textures/metal_albedo.png"),
    normal_texture: Some("textures/metal_normal.png"),
    metallic: 0.9,
    roughness: 0.3,
    metallic_roughness_texture: Some("textures/metal_mr.png"),
    emissive: (0.0, 0.0, 0.0, 1.0),
)
```

### Mesh Data Structure

```rust
// luminara_render/src/mesh.rs

/// Mesh asset
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub aabb: AABB,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl Asset for Mesh {
    fn load(bytes: &[u8]) -> Result<Self, AssetLoadError> {
        // Load from GLTF
        let (document, buffers, _) = gltf::import_slice(bytes)?;
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                
                // Read positions
                let positions = reader.read_positions().unwrap();
                let normals = reader.read_normals().unwrap();
                let uvs = reader.read_tex_coords(0).unwrap().into_f32();
                let tangents = reader.read_tangents().unwrap();
                
                for ((((pos, norm), uv), tan)) in positions.zip(normals).zip(uvs).zip(tangents) {
                    vertices.push(Vertex {
                        position: pos,
                        normal: norm,
                        uv,
                        tangent: tan,
                    });
                }
                
                // Read indices
                if let Some(indices_reader) = reader.read_indices() {
                    indices.extend(indices_reader.into_u32());
                }
            }
        }
        
        let aabb = compute_aabb(&vertices);
        
        Ok(Mesh { vertices, indices, aabb })
    }
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Scene Serialization Properties

Property 1: Scene Round-Trip Consistency
*For any* valid scene with entities, components, and hierarchy relationships, serializing to RON/JSON format and then deserializing should produce an equivalent scene structure with all entity data, component values, names, tags, and parent-child relationships preserved.
**Validates: Requirements 1.1, 1.2, 1.4, 1.5**

Property 2: Component Schema Availability
*For any* registered component type in the engine, a ComponentSchema with type name, description, and field metadata should be available for AI introspection.
**Validates: Requirements 1.6**

### Asset Pipeline Properties

Property 3: Asset Hot-Reload Detection
*For any* asset file (including scene files) that is modified on disk, the file watcher should detect the change within a reasonable time window and trigger a reload of that asset.
**Validates: Requirements 4.1, 4.2**

Property 4: Asset Load and Retrieve
*For any* valid asset path, requesting the asset should return a handle, and using that handle should eventually provide access to the loaded asset data once loading completes.
**Validates: Requirements 4.3, 4.4**

Property 5: Format Support
*For any* valid GLTF/GLB mesh file or PNG/JPG/HDR texture file, the asset server should successfully load and process it into the corresponding runtime format (Mesh or Texture).
**Validates: Requirements 4.5, 4.6**

Property 6: Asset Load Error Handling
*For any* invalid asset path or corrupted asset file, the asset server should not crash, should log an error, and should provide a fallback asset.
**Validates: Requirements 4.7**

Property 7: Asset Caching
*For any* asset that is loaded multiple times, the asset server should process it only once and return cached data for subsequent requests.
**Validates: Requirements 4.8**

### Transform Hierarchy Properties

Property 8: Transform Hierarchy Propagation
*For any* entity hierarchy with parent-child relationships, when a parent's local transform changes, all descendant entities' world transforms should be updated such that each child's world transform equals its parent's world transform multiplied by its own local transform.
**Validates: Requirements 5.1, 5.2, 5.3**

Property 9: Child Detachment Preserves World Position
*For any* child entity with a parent, detaching it from the parent should update its local transform such that its world position, rotation, and scale remain unchanged.
**Validates: Requirements 5.5**

### Camera Properties

Property 10: Projection Mode Support
*For any* camera component, setting it to either Perspective or Orthographic projection mode should produce a valid projection matrix with the specified parameters (FOV/size, near, far, aspect ratio).
**Validates: Requirements 6.1, 6.4**

Property 11: Projection Matrix Update
*For any* camera, changing its projection parameters (FOV, size, near, far) should result in a recomputed projection matrix that differs from the previous matrix.
**Validates: Requirements 6.2**

Property 12: Camera Aspect Ratio Update
*For any* camera, when the window is resized, the camera's aspect ratio should be updated to match the new window dimensions (width / height).
**Validates: Requirements 6.5**

### 2D Sprite Rendering Properties

Property 13: Sprite Batching by Texture
*For any* set of sprites that share the same texture handle, the batch renderer should group them into a single batch (or multiple batches if exceeding max batch size), minimizing the number of draw calls.
**Validates: Requirements 3.1**

Property 14: Sprite Instance Data Correctness
*For any* sprite with a texture, color, transform, and optional texture rect, the batch renderer should create a sprite instance with the correct transform matrix, color values, and UV coordinates.
**Validates: Requirements 3.2, 3.4**

Property 15: Sprite Sorting
*For any* set of sprites with z-order and texture values, the batch renderer should sort them first by z-order (depth) and then by texture to minimize state changes.
**Validates: Requirements 3.3**

### Physics Properties

Property 16: Collision Detection and Events
*For any* two entities with collider components that overlap in space, the physics engine should detect the collision and emit a collision event containing both entity IDs.
**Validates: Requirements 7.2, 7.6**

Property 17: Physics Transform Synchronization
*For any* entity with both a RigidBody and Transform component, after a physics simulation step, the entity's Transform should match the physics body's position and rotation.
**Validates: Requirements 7.4, 7.7**

Property 18: Rigid Body Type Behavior
*For any* entity with a RigidBody component, the physics engine should handle it according to its type: Dynamic bodies should respond to forces and collisions, Kinematic bodies should move only when explicitly set, and Static bodies should not move at all.
**Validates: Requirements 7.5**

### Audio Properties

Property 19: Audio File Loading
*For any* valid audio file in a supported format, the audio system should successfully decode it into a playable AudioClip.
**Validates: Requirements 8.1**

Property 20: Audio Playback Control
*For any* audio source, triggering play should start playback with the specified volume and pitch, and pause/resume/stop commands should correctly control the playback state.
**Validates: Requirements 8.2, 8.6**

Property 21: Spatial Audio Distance Attenuation
*For any* spatial audio source, the effective volume should decrease as the distance between the source and the audio listener increases, reaching zero at or beyond the max_distance.
**Validates: Requirements 8.3**

Property 22: Audio Looping
*For any* audio source with looping enabled, when the audio clip finishes playing, it should automatically restart from the beginning.
**Validates: Requirements 8.4**

### Plugin System Properties

Property 23: Plugin Build Invocation
*For any* plugin registered with the engine, the engine should call the plugin's build method exactly once during initialization.
**Validates: Requirements 9.1**

Property 24: Plugin System Registration
*For any* plugin that adds systems to specific stages, those systems should be present in the schedule at the specified stages after the plugin is built.
**Validates: Requirements 9.2**

Property 25: Plugin Component and Resource Registration
*For any* plugin that registers custom components or resources, those types should be available in the world after the plugin is built.
**Validates: Requirements 9.3**

Property 26: Plugin Execution Order
*For any* sequence of plugins registered with the engine, the plugins should be built in the order they were registered.
**Validates: Requirements 9.4**

## Error Handling

### Asset Loading Errors

The asset system must handle various error conditions gracefully:

- **File Not Found**: Log error, return fallback asset (e.g., pink checkerboard texture, default cube mesh)
- **Parse Errors**: Log detailed error with file path and line number, return fallback asset
- **Unsupported Format**: Log error with supported formats list, return fallback asset
- **Out of Memory**: Log error, attempt to free unused assets, retry or return fallback

```rust
pub enum AssetLoadError {
    FileNotFound(PathBuf),
    ParseError { path: PathBuf, message: String },
    UnsupportedFormat { path: PathBuf, format: String },
    OutOfMemory,
}

impl AssetServer {
    fn handle_load_error(&mut self, error: AssetLoadError, handle: AssetId) {
        match error {
            AssetLoadError::FileNotFound(path) => {
                error!("Asset not found: {:?}", path);
                self.storage.insert(handle, Self::fallback_asset());
            }
            AssetLoadError::ParseError { path, message } => {
                error!("Failed to parse asset {:?}: {}", path, message);
                self.storage.insert(handle, Self::fallback_asset());
            }
            // ... handle other errors
        }
    }
}
```

### Physics Errors

Physics simulation errors should not crash the engine:

- **Invalid Collider Shape**: Log warning, skip collider creation
- **NaN in Transform**: Log error, reset transform to identity
- **Physics World Overflow**: Log error, remove oldest dynamic bodies

```rust
pub fn physics_sync_system(
    mut physics_world: ResMut<RapierPhysicsWorld>,
    mut query: Query<(Entity, &mut Transform, &RigidBody)>,
) {
    for (entity, mut transform, _) in query.iter_mut() {
        if let Some(body_handle) = physics_world.entity_to_body.get(&entity) {
            if let Some(body) = physics_world.bodies.get(*body_handle) {
                let position = body.translation();
                let rotation = body.rotation();
                
                // Check for NaN
                if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
                    error!("NaN detected in physics body for entity {:?}, resetting", entity);
                    transform.translation = Vec3::ZERO;
                    transform.rotation = Quat::IDENTITY;
                    continue;
                }
                
                transform.translation = Vec3::new(position.x, position.y, position.z);
                transform.rotation = Quat::from_xyzw(rotation.i, rotation.j, rotation.k, rotation.w);
            }
        }
    }
}
```

### Scene Deserialization Errors

Scene loading errors should provide clear feedback:

- **Missing Component Type**: Log warning, skip component
- **Invalid Component Data**: Log error with component name and entity, use default values
- **Circular Parent References**: Log error, break cycle by removing parent reference

```rust
impl SceneSerializer {
    pub fn deserialize(scene: &Scene, world: &mut World) -> Result<(), DeserializeError> {
        let mut entity_map = HashMap::new();
        
        // First pass: create all entities
        for entity_data in &scene.entities {
            let entity = world.spawn();
            entity_map.insert(entity_data.name.clone(), entity);
        }
        
        // Second pass: add components and set up hierarchy
        for entity_data in &scene.entities {
            let entity = entity_map[&entity_data.name];
            
            for (component_type, component_data) in &entity_data.components {
                match deserialize_component(component_type, component_data) {
                    Ok(component) => {
                        world.insert(entity, component);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize component {} for entity {}: {}",
                              component_type, entity_data.name, e);
                        // Continue with other components
                    }
                }
            }
            
            // Set up parent-child relationship
            if let Some(parent_name) = &entity_data.parent {
                if let Some(&parent_entity) = entity_map.get(parent_name) {
                    world.insert(entity, Parent(parent_entity));
                    // Add to parent's children list
                } else {
                    warn!("Parent entity {} not found for {}", parent_name, entity_data.name);
                }
            }
        }
        
        Ok(())
    }
}
```

### Rendering Errors

Rendering errors should degrade gracefully:

- **Shader Compilation Error**: Log error with shader source, use fallback shader
- **Texture Upload Failure**: Log error, use fallback texture
- **Out of GPU Memory**: Log error, reduce quality settings, retry

```rust
pub fn render_system(
    mut render_context: ResMut<RenderContext>,
    meshes: Query<(&MeshRenderer, &GlobalTransform)>,
) {
    for (mesh_renderer, transform) in meshes.iter() {
        match render_context.draw_mesh(mesh_renderer, transform) {
            Ok(_) => {}
            Err(RenderError::ShaderCompilationFailed { message }) => {
                error!("Shader compilation failed: {}", message);
                // Use fallback shader
                render_context.use_fallback_shader();
            }
            Err(RenderError::OutOfMemory) => {
                error!("GPU out of memory, reducing quality");
                render_context.reduce_quality();
            }
            Err(e) => {
                error!("Render error: {:?}", e);
            }
        }
    }
}
```

## Testing Strategy

Phase 1 requires a dual testing approach combining unit tests for specific scenarios and property-based tests for comprehensive validation.

### Unit Testing

Unit tests focus on:
- **Specific examples**: Known input-output pairs (e.g., specific transform hierarchy)
- **Edge cases**: Empty scenes, single-entity scenes, deeply nested hierarchies
- **Integration points**: Plugin initialization order, system execution order
- **Error conditions**: Invalid asset paths, malformed scene files, NaN values

Example unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_scene_serialization() {
        let scene = Scene {
            meta: SceneMeta {
                name: "Empty".to_string(),
                version: "1.0.0".to_string(),
                description: "".to_string(),
            },
            entities: vec![],
        };
        
        let ron = ron::to_string(&scene).unwrap();
        let deserialized: Scene = ron::from_str(&ron).unwrap();
        
        assert_eq!(scene.entities.len(), deserialized.entities.len());
    }
    
    #[test]
    fn test_transform_hierarchy_single_child() {
        let mut world = World::new();
        
        let parent = world.spawn((
            Transform {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            GlobalTransform::default(),
        ));
        
        let child = world.spawn((
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            GlobalTransform::default(),
            Parent(parent),
        ));
        
        transform_propagate_system(&mut world);
        
        let child_global = world.get::<GlobalTransform>(child).unwrap();
        let expected_pos = Vec3::new(2.0, 2.0, 3.0);
        
        assert!((child_global.matrix().w_axis.truncate() - expected_pos).length() < 0.001);
    }
}
```

### Property-Based Testing

Property-based tests validate universal properties across randomized inputs using a PBT library (e.g., `proptest` for Rust).

**Configuration**:
- Minimum 100 iterations per property test
- Each test tagged with: `Feature: phase-1-core-engine, Property N: [property text]`
- Generators for: scenes, transforms, hierarchies, materials, physics bodies

Example property tests:
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    // Generator for random transforms
    fn arb_transform() -> impl Strategy<Value = Transform> {
        (
            prop::array::uniform3(-100.0f32..100.0f32),
            prop::array::uniform4(-1.0f32..1.0f32),
            prop::array::uniform3(0.1f32..10.0f32),
        ).prop_map(|(pos, rot, scale)| {
            let rotation = Quat::from_xyzw(rot[0], rot[1], rot[2], rot[3]).normalize();
            Transform {
                translation: Vec3::from_array(pos),
                rotation,
                scale: Vec3::from_array(scale),
            }
        })
    }
    
    // Generator for random scenes
    fn arb_scene() -> impl Strategy<Value = Scene> {
        // Generate scenes with 0-20 entities
        prop::collection::vec(arb_entity_data(), 0..20)
            .prop_map(|entities| Scene {
                meta: SceneMeta {
                    name: "Test Scene".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Generated test scene".to_string(),
                },
                entities,
            })
    }
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        // Feature: phase-1-core-engine, Property 1: Scene Round-Trip Consistency
        #[test]
        fn prop_scene_round_trip(scene in arb_scene()) {
            // Serialize to RON
            let ron_string = ron::to_string(&scene).unwrap();
            
            // Deserialize back
            let deserialized: Scene = ron::from_str(&ron_string).unwrap();
            
            // Verify equivalence
            assert_eq!(scene.entities.len(), deserialized.entities.len());
            
            for (original, deserialized) in scene.entities.iter().zip(deserialized.entities.iter()) {
                assert_eq!(original.name, deserialized.name);
                assert_eq!(original.parent, deserialized.parent);
                assert_eq!(original.tags, deserialized.tags);
                assert_eq!(original.components.len(), deserialized.components.len());
            }
        }
        
        // Feature: phase-1-core-engine, Property 8: Transform Hierarchy Propagation
        #[test]
        fn prop_transform_hierarchy_propagation(
            parent_transform in arb_transform(),
            child_transform in arb_transform(),
        ) {
            let mut world = World::new();
            
            let parent = world.spawn((
                parent_transform,
                GlobalTransform::default(),
            ));
            
            let child = world.spawn((
                child_transform,
                GlobalTransform::default(),
                Parent(parent),
            ));
            
            transform_propagate_system(&mut world);
            
            let parent_global = world.get::<GlobalTransform>(parent).unwrap();
            let child_global = world.get::<GlobalTransform>(child).unwrap();
            
            // Child's world transform should equal parent_world * child_local
            let expected = parent_global.matrix() * child_transform.to_matrix();
            let actual = child_global.matrix();
            
            // Compare matrices (with floating point tolerance)
            for i in 0..4 {
                for j in 0..4 {
                    let diff = (expected.col(i)[j] - actual.col(i)[j]).abs();
                    assert!(diff < 0.001, "Matrix mismatch at [{}, {}]: {} vs {}", i, j, expected.col(i)[j], actual.col(i)[j]);
                }
            }
        }
        
        // Feature: phase-1-core-engine, Property 13: Sprite Batching by Texture
        #[test]
        fn prop_sprite_batching(sprite_count in 1usize..100) {
            let mut world = World::new();
            let mut asset_server = AssetServer::new();
            
            // Create a single texture handle
            let texture = asset_server.load::<Texture>("test.png");
            
            // Spawn sprites with the same texture
            for _ in 0..sprite_count {
                world.spawn((
                    Sprite {
                        texture: texture.clone(),
                        color: Color::WHITE,
                        rect: None,
                        flip_x: false,
                        flip_y: false,
                        anchor: Anchor::Center,
                    },
                    GlobalTransform::default(),
                ));
            }
            
            let mut batcher = SpriteBatcher::new(1000);
            let sprites = world.query::<(&Sprite, &GlobalTransform, Option<&ZOrder>)>();
            batcher.prepare(sprites);
            
            // All sprites should be in a single batch (or ceil(sprite_count / 1000) batches)
            let expected_batches = (sprite_count + 999) / 1000;
            assert_eq!(batcher.batches.len(), expected_batches);
            
            // Total instances should equal sprite count
            let total_instances: usize = batcher.batches.iter().map(|b| b.instances.len()).sum();
            assert_eq!(total_instances, sprite_count);
        }
    }
}
```

### Integration Testing

Integration tests verify the complete Phase 1 demo:
- Scene loading from .ron file
- Asset loading (meshes, textures, audio)
- Physics simulation with collision detection
- Rendering with PBR materials and shadows
- Audio playback

These tests are implemented as example applications that can be run manually or in CI with headless rendering.

### Test Coverage Goals

- **Unit tests**: 80%+ code coverage for core systems
- **Property tests**: All 26 correctness properties implemented
- **Integration tests**: Phase 1 demo runs without errors for 60 seconds
- **Error handling**: All error paths tested with invalid inputs

