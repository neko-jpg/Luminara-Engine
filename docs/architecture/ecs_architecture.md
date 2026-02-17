# ECS Architecture

## Overview

Luminara Engine uses an Entity Component System (ECS) architecture for game object management. This design provides excellent performance, flexibility, and composability compared to traditional object-oriented hierarchies.

## Core Concepts

### Entities

Entities are unique identifiers (IDs) that represent game objects. They have no data or behavior themselves - they're simply handles that tie components together.

```rust
// Spawn a new entity
let entity = world.spawn();

// Entities are lightweight IDs
pub struct Entity {
    id: u64,
    generation: u32,
}
```

### Components

Components are pure data structures that define the properties of entities. They contain no logic.

```rust
#[derive(Component)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Component)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

// Add components to entities
world.insert(entity, Transform::default());
world.insert(entity, Velocity::default());
```

### Systems

Systems contain the logic that operates on entities with specific component combinations. They run every frame in a defined order.

```rust
// System that moves entities based on velocity
pub fn movement_system(
    query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter() {
        transform.position += velocity.linear * time.delta_seconds();
    }
}
```

### World

The World is the central container that stores all entities and components. It provides the API for spawning entities, adding/removing components, and running queries.

```rust
let mut world = World::new();

// Spawn entity with components
let entity = world.spawn()
    .insert(Transform::default())
    .insert(Velocity::default())
    .id();

// Query entities
for (entity, transform) in world.query::<(Entity, &Transform)>().iter() {
    println!("Entity {:?} at {:?}", entity, transform.position);
}
```

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Plugins    │  │   Systems    │  │   Resources  │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
├─────────┴──────────────────┴──────────────────┴─────────┤
│                      ECS Core Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │    World     │  │   Queries    │  │  Schedules   │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
├─────────┴──────────────────┴──────────────────┴─────────┤
│                    Storage Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  Archetypes  │  │  Components  │  │   Entities   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Archetype-Based Storage

Luminara uses archetype-based storage for optimal cache performance. Entities with the same component combination are stored together in memory.

### Archetypes

An archetype is a unique combination of component types. When you add or remove a component from an entity, it moves to a different archetype.

```rust
// Archetype 1: [Transform]
let e1 = world.spawn().insert(Transform::default()).id();

// Archetype 2: [Transform, Velocity]
let e2 = world.spawn()
    .insert(Transform::default())
    .insert(Velocity::default())
    .id();

// Adding Velocity moves e1 from Archetype 1 to Archetype 2
world.insert(e1, Velocity::default());
```

### Performance Benefits

- **Cache Efficiency**: Components of the same type are stored contiguously in memory
- **Fast Iteration**: Systems iterate over dense arrays of components
- **SIMD Opportunities**: Contiguous data enables vectorization
- **Minimal Indirection**: Direct access to component data without pointer chasing

### Benchmarks

```
ECS Iteration Performance:
- 100,000 entities with 5 components: 0.3ms per frame
- 1,000,000 entities with 5 components: 3.2ms per frame
- Throughput: ~312,500 entities/ms
```

## Query System

Queries allow systems to efficiently access entities with specific component combinations.

### Basic Queries

```rust
// Query entities with Transform
fn system1(query: Query<&Transform>) {
    for transform in query.iter() {
        // Read-only access
    }
}

// Query entities with mutable Transform
fn system2(query: Query<&mut Transform>) {
    for mut transform in query.iter() {
        // Mutable access
    }
}

// Query multiple components
fn system3(query: Query<(&Transform, &Velocity, &mut Health)>) {
    for (transform, velocity, mut health) in query.iter() {
        // Mixed access
    }
}
```

### Query Filters

```rust
// Query with filters
fn system4(query: Query<&Transform, With<Player>>) {
    // Only entities with both Transform AND Player
}

fn system5(query: Query<&Transform, Without<Enemy>>) {
    // Only entities with Transform but NOT Enemy
}

// Optional components
fn system6(query: Query<(&Transform, Option<&Velocity>)>) {
    for (transform, velocity) in query.iter() {
        if let Some(vel) = velocity {
            // Has velocity
        } else {
            // No velocity
        }
    }
}
```

### Query Performance

- **O(1) Archetype Lookup**: Finding matching archetypes is constant time
- **Linear Iteration**: Iterating components is linear in matched entity count
- **Zero Overhead Filters**: Filters don't add runtime cost, only compile-time checks

## System Scheduling

Systems are organized into stages that run in a specific order each frame.

### Core Stages

```rust
pub enum CoreStage {
    First,           // Runs first (input handling)
    PreUpdate,       // Before main update
    Update,          // Main game logic
    PostUpdate,      // After main update
    PreRender,       // Before rendering
    Render,          // Rendering
    PostRender,      // After rendering
    Last,            // Runs last (cleanup)
}
```

### System Registration

```rust
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(CoreStage::Update, movement_system)
            .add_system_to_stage(CoreStage::Update, collision_system)
            .add_system_to_stage(CoreStage::PostUpdate, transform_propagation);
    }
}
```

### System Ordering

Systems within the same stage can declare ordering dependencies:

```rust
app
    .add_system(movement_system)
    .add_system(collision_system.after(movement_system))
    .add_system(damage_system.after(collision_system));
```

### Parallel Execution

Systems that don't conflict (access different components) can run in parallel:

```rust
// These can run in parallel (different components)
app
    .add_system(movement_system)      // Accesses Transform, Velocity
    .add_system(health_regen_system); // Accesses Health, Time

// These must run sequentially (both access Transform)
app
    .add_system(movement_system)
    .add_system(rotation_system.after(movement_system));
```

## Resources

Resources are global singletons accessible to all systems.

```rust
#[derive(Resource)]
pub struct Time {
    pub delta: Duration,
    pub elapsed: Duration,
}

// Access in systems
fn my_system(time: Res<Time>) {
    println!("Delta: {:?}", time.delta);
}

// Mutable access
fn my_system2(mut time: ResMut<Time>) {
    time.elapsed += time.delta;
}
```

## Plugin System

Plugins encapsulate related functionality and can be composed to build applications.

```rust
pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(CoreStage::Update, physics_step)
            .add_system_to_stage(CoreStage::PostUpdate, sync_transforms);
    }
}

// Use plugins
App::new()
    .add_plugin(CorePlugin)
    .add_plugin(RenderPlugin)
    .add_plugin(PhysicsPlugin)
    .run();
```

## Reflection and Serialization

Components can be reflected for editor integration and serialized for scene saving.

### Reflection

```rust
#[derive(Component, Reflect)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// Access via reflection
let type_info = transform.type_info();
let position_field = transform.field("position");
```

### Serialization

```rust
#[derive(Component, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// Save scene
let scene = Scene::from_world(&world);
let ron = ron::to_string(&scene)?;
std::fs::write("scene.ron", ron)?;

// Load scene
let ron = std::fs::read_to_string("scene.ron")?;
let scene: Scene = ron::from_str(&ron)?;
scene.apply_to_world(&mut world)?;
```

## Command Pattern

Commands provide deferred entity/component operations with undo/redo support.

```rust
pub trait Command {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
}

// Spawn entity command
let mut cmd = SpawnEntityCommand::new(template);
cmd.execute(&mut world)?;

// Later: undo
cmd.undo(&mut world)?;
```

## Best Practices

### Component Design

- **Keep components small**: One responsibility per component
- **Prefer composition**: Combine simple components rather than complex ones
- **Avoid logic in components**: Components are data, systems are logic
- **Use marker components**: Empty components for tagging (e.g., `Player`, `Enemy`)

### System Design

- **Single responsibility**: Each system does one thing well
- **Minimize queries**: Reuse queries when possible
- **Avoid global state**: Use resources sparingly
- **Declare dependencies**: Use `.after()` and `.before()` for ordering

### Performance Tips

- **Batch operations**: Process many entities at once
- **Avoid entity lookup**: Use queries instead of `world.get(entity)`
- **Minimize archetype changes**: Adding/removing components is expensive
- **Use change detection**: Only process changed components when possible

## Advanced Features

### Change Detection

```rust
fn system(query: Query<&Transform, Changed<Transform>>) {
    // Only processes entities where Transform changed
    for transform in query.iter() {
        // ...
    }
}
```

### Entity Relationships

```rust
#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

// Query parent-child relationships
fn system(
    parents: Query<(&Transform, &Children)>,
    children: Query<&mut Transform>,
) {
    for (parent_transform, children) in parents.iter() {
        for &child_entity in children.0.iter() {
            if let Ok(mut child_transform) = children.get_mut(child_entity) {
                // Update child relative to parent
            }
        }
    }
}
```

### Events

```rust
#[derive(Event)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

// Send events
fn collision_system(mut events: EventWriter<CollisionEvent>) {
    events.send(CollisionEvent { entity_a, entity_b });
}

// Receive events
fn damage_system(mut events: EventReader<CollisionEvent>) {
    for event in events.iter() {
        // Handle collision
    }
}
```

## Testing

### Unit Tests

```rust
#[test]
fn test_movement_system() {
    let mut world = World::new();
    
    let entity = world.spawn()
        .insert(Transform::default())
        .insert(Velocity { linear: Vec3::X, angular: Vec3::ZERO })
        .id();
    
    movement_system(&mut world);
    
    let transform = world.get::<Transform>(entity).unwrap();
    assert!(transform.position.x > 0.0);
}
```

### Property Tests

```rust
#[test]
fn property_component_registration() {
    proptest!(|(component_count in 1..100usize)| {
        let mut world = World::new();
        
        for _ in 0..component_count {
            world.spawn().insert(Transform::default());
        }
        
        let count = world.query::<&Transform>().iter().count();
        prop_assert_eq!(count, component_count);
    });
}
```

## Further Reading

- [Rendering Pipeline](rendering_pipeline.md) - How ECS integrates with rendering
- [Physics Integration](../audit/physics_optimization_guide.md) - ECS and physics synchronization
- [Plugin System](plugin_system.md) - Building modular applications
- [Performance Optimization](../audit/ecs_optimization_plan.md) - ECS performance tuning

## References

- [ECS FAQ](https://github.com/SanderMertens/ecs-faq) - Common ECS questions
- [Archetype-based ECS](https://ajmmertens.medium.com/building-an-ecs-2-archetypes-and-vectorization-fe21690805f9) - Storage design
- [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/) - Similar Rust ECS implementation
