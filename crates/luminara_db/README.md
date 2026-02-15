# Luminara DB

Graph-based database integration for Luminara Engine using SurrealDB embedded mode.

## Features

- **Embedded SurrealDB**: No external server required, runs in-process
- **Graph Relationships**: Use Record Links to represent entity hierarchies and asset dependencies
- **SurrealQL Queries**: Powerful graph traversal and complex queries
- **Persistent Undo/Redo**: Store operation timeline for recovery across sessions
- **WASM Support**: Works in browser with IndexedDB backend (see [WASM documentation](docs/wasm_indexeddb_support.md))

## Architecture

The database stores four main types of records:

1. **Entities**: Game entities with metadata, tags, and relationships
2. **Components**: Component data attached to entities
3. **Assets**: Asset metadata with dependency graph
4. **Operations**: Operation timeline for undo/redo functionality

All records use Record Links to represent relationships, enabling powerful graph queries.

## Usage

### Initialize Database

```rust
use luminara_db::LuminaraDatabase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create embedded database
    let db = LuminaraDatabase::new("./data/luminara.db").await?;
    
    Ok(())
}
```

### Store and Query Entities

```rust
use luminara_db::{EntityRecord, QueryBuilder};

// Create entity
let entity = EntityRecord::new(Some("Player".to_string()))
    .with_tag("player")
    .with_tag("controllable");

let entity_id = db.store_entity(entity).await?;

// Query entities by tag
let players = db.query_entities(
    "SELECT * FROM entity WHERE 'player' IN tags"
).await?;

// Or use query builder
let query = QueryBuilder::entities_with_tag("player")
    .limit(10)
    .build();
let players = db.query_entities(&query).await?;
```

### Store Components

```rust
use luminara_db::ComponentRecord;
use serde_json::json;

let component_data = json!({
    "position": [0.0, 1.0, 2.0],
    "rotation": [0.0, 0.0, 0.0, 1.0],
    "scale": [1.0, 1.0, 1.0]
});

let component = ComponentRecord::new(
    "Transform",
    "luminara_scene::Transform",
    component_data,
    entity_id.clone(),
);

let component_id = db.store_component(component).await?;
```

### Track Asset Dependencies

```rust
use luminara_db::{AssetRecord, AssetMetadata};
use serde_json::json;

let metadata = AssetMetadata {
    size_bytes: 2048,
    modified_timestamp: 1234567890,
    custom: json!({"format": "PNG", "width": 512, "height": 512}),
};

// Create a texture
let texture = AssetRecord::new(
    "assets/textures/player.png",
    "Texture",
    "abc123hash",
    metadata.clone(),
);
let texture_id = db.store_asset(texture).await?;

// Create a material that depends on the texture
let material = AssetRecord::new(
    "assets/materials/player.mat",
    "Material",
    "def456hash",
    metadata,
)
.with_dependency(texture_id.clone());
let material_id = db.store_asset(material).await?;

// Find direct dependencies
let dependencies = db.find_asset_dependencies(&material_id).await?;
println!("Material has {} direct dependencies", dependencies.len());

// Find transitive dependencies (dependencies of dependencies)
let all_deps = db.find_asset_dependencies_transitive(&material_id).await?;
println!("Material has {} total dependencies", all_deps.len());

// Find what depends on the texture (reverse lookup)
let dependents = db.find_asset_dependents(&texture_id).await?;
println!("{} assets depend on this texture", dependents.len());
```

### Operation Timeline (Undo/Redo)

```rust
use luminara_db::OperationRecord;
use serde_json::json;

let commands = vec![json!({"type": "SpawnEntity", "name": "Player"})];
let inverse_commands = vec![json!({"type": "DestroyEntity", "id": "entity:123"})];

let operation = OperationRecord::new(
    "SpawnEntity",
    "Spawn player entity",
    commands,
    inverse_commands,
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64,
)
.with_branch("main");

let operation_id = db.store_operation(operation).await?;

// Load recent operations
let history = db.load_operation_history(10, Some("main")).await?;
```

## Asset Dependency Tracking

The database provides powerful asset dependency tracking with graph traversal:

### Direct Dependencies

```rust
// Find immediate dependencies of an asset
let deps = db.find_asset_dependencies(&material_id).await?;
```

### Transitive Dependencies

```rust
// Find all dependencies recursively (handles circular dependencies)
let all_deps = db.find_asset_dependencies_transitive(&mesh_id).await?;

// Example: Mesh -> Material -> Texture
// This will find both the Material and the Texture
```

### Reverse Dependencies

```rust
// Find what depends on an asset
let dependents = db.find_asset_dependents(&texture_id).await?;

// Example: Find all materials that use this texture
```

### Scene Asset Queries

```rust
// Find all assets of a specific type used in a scene
let textures = db.find_assets_in_scene(&scene_id, "Texture").await?;
let meshes = db.find_assets_in_scene(&scene_id, "Mesh").await?;

// Find all textures used by materials in a scene
// This performs complex graph traversal:
// Scene -> Entities -> Components -> Materials -> Textures
let textures = db.find_textures_used_by_materials_in_scene(&scene_id).await?;
```

### Complex Dependency Graph Example

```rust
// Create a complex dependency graph:
// Mesh -> Material1 -> Texture1, Texture2
//      -> Material2 -> Texture2, Texture3

let texture1 = AssetRecord::new("tex1.png", "Texture", "hash1", metadata.clone());
let texture1_id = db.store_asset(texture1).await?;

let texture2 = AssetRecord::new("tex2.png", "Texture", "hash2", metadata.clone());
let texture2_id = db.store_asset(texture2).await?;

let texture3 = AssetRecord::new("tex3.png", "Texture", "hash3", metadata.clone());
let texture3_id = db.store_asset(texture3).await?;

let material1 = AssetRecord::new("mat1.mat", "Material", "hash4", metadata.clone())
    .with_dependency(texture1_id.clone())
    .with_dependency(texture2_id.clone());
let material1_id = db.store_asset(material1).await?;

let material2 = AssetRecord::new("mat2.mat", "Material", "hash5", metadata.clone())
    .with_dependency(texture2_id.clone())
    .with_dependency(texture3_id.clone());
let material2_id = db.store_asset(material2).await?;

let mesh = AssetRecord::new("cube.mesh", "Mesh", "hash6", metadata)
    .with_dependency(material1_id)
    .with_dependency(material2_id);
let mesh_id = db.store_asset(mesh).await?;

// Find all transitive dependencies
let all_deps = db.find_asset_dependencies_transitive(&mesh_id).await?;
// Returns: 2 materials + 3 textures = 5 assets

// Find what uses texture2 (shared by both materials)
let dependents = db.find_asset_dependents(&texture2_id).await?;
// Returns: material1, material2
```

## Graph Queries

SurrealDB's Record Links enable powerful graph traversal:

```sql
-- Find all components of an entity
SELECT ->components->component.* FROM entity:player

-- Find all child entities recursively
SELECT ->children->entity.* FROM entity:root

-- Find all textures used by materials in a scene
SELECT ->uses->texture.* FROM material WHERE scene = $scene_id

-- Find all assets that depend on a specific texture
SELECT <-dependencies<-asset.* FROM asset:texture123
```

## Performance

- **Sync Latency**: Target <16ms for ECS to database synchronization
- **Query Performance**: Optimized for 10,000+ entity scenes
- **Storage Backend**: RocksDB for production, in-memory for testing

## Requirements

This crate implements **Requirement 21.1** from the Pre-Editor Engine Audit spec:
- Embedded SurrealDB setup
- Schema for entities, components, assets
- Basic CRUD operations
- Graph-based relationships

## Future Work

- Advanced query optimization
- Incremental migration support
- Asset migration enhancements
- Query optimization and caching
- Incremental sync for large scenes


## Entity Relationships and Graph Traversal

### Load Entity with Components

```rust
// Load entity with all its components in one operation
let (entity, components) = db.load_entity_with_components(&entity_id).await?;

println!("Entity {} has {} components", 
    entity.name.unwrap_or_default(), 
    components.len());

for component in components {
    println!("  - {}: {:?}", component.type_name, component.data);
}
```

### Entity Hierarchies

```rust
// Create parent-child hierarchy
let parent = EntityRecord::new(Some("Parent".to_string()));
let parent_id = db.store_entity(parent).await?;

let child = EntityRecord::new(Some("Child".to_string()));
let child_id = db.store_entity(child).await?;

// Link them together
let mut parent = db.load_entity(&parent_id).await?;
parent.children = vec![child_id.clone()];
db.update_entity(&parent_id, parent).await?;

let mut child = db.load_entity(&child_id).await?;
child.parent = Some(parent_id.clone());
db.update_entity(&child_id, child).await?;

// Load hierarchy (parent and children)
let hierarchy = db.load_entity_hierarchy(&parent_id).await?;
println!("Parent has {} children", hierarchy.children.len());
if let Some(parent) = hierarchy.parent {
    println!("Has parent: {}", parent.name.unwrap_or_default());
}
```

### Load Entity with All Relationships

```rust
// Load entity with components, parent, and children in one call
let full_entity = db.load_entity_with_relationships(&entity_id).await?;

println!("Entity: {}", full_entity.entity.name.unwrap_or_default());
println!("Components: {}", full_entity.components.len());
println!("Children: {}", full_entity.hierarchy.children.len());

if let Some(parent) = full_entity.hierarchy.parent {
    println!("Parent: {}", parent.name.unwrap_or_default());
}
```

### Find Entities by Component Type

```rust
// Find all entities that have a Transform component
let entities = db.find_entities_with_component("Transform").await?;
println!("Found {} entities with Transform", entities.len());

// Find all entities with Mesh component
let renderable = db.find_entities_with_component("Mesh").await?;
```

### Recursive Graph Traversal

```rust
// Find all descendants (children, grandchildren, etc.)
let descendants = db.find_entity_descendants(&root_id).await?;
println!("Root has {} total descendants", descendants.len());

// Find all ancestors (parent, grandparent, etc.)
let ancestors = db.find_entity_ancestors(&leaf_id).await?;
println!("Leaf has {} ancestors", ancestors.len());
```

### Complex Scene Graph Example

```rust
// Build a complete scene graph
let scene = EntityRecord::new(Some("Scene".to_string())).with_tag("scene");
let scene_id = db.store_entity(scene).await?;

let camera = EntityRecord::new(Some("Camera".to_string())).with_tag("camera");
let camera_id = db.store_entity(camera).await?;

let player = EntityRecord::new(Some("Player".to_string())).with_tag("player");
let player_id = db.store_entity(player).await?;

// Add components to player
let transform = ComponentRecord::new(
    "Transform",
    "luminara_scene::Transform",
    json!({"position": [0.0, 0.0, 0.0]}),
    player_id.clone(),
);
let transform_id = db.store_component(transform).await?;

let mesh = ComponentRecord::new(
    "Mesh",
    "luminara_render::Mesh",
    json!({"vertices": 100}),
    player_id.clone(),
);
let mesh_id = db.store_component(mesh).await?;

// Setup hierarchy
let mut scene = db.load_entity(&scene_id).await?;
scene.children = vec![camera_id.clone(), player_id.clone()];
db.update_entity(&scene_id, scene).await?;

let mut camera = db.load_entity(&camera_id).await?;
camera.parent = Some(scene_id.clone());
db.update_entity(&camera_id, camera).await?;

let mut player = db.load_entity(&player_id).await?;
player.parent = Some(scene_id.clone());
player.components = vec![transform_id, mesh_id];
db.update_entity(&player_id, player).await?;

// Query the scene
let scene_hierarchy = db.load_entity_hierarchy(&scene_id).await?;
println!("Scene has {} entities", scene_hierarchy.children.len());

let entities_with_transform = db.find_entities_with_component("Transform").await?;
println!("Scene has {} renderable entities", entities_with_transform.len());
```

## Testing

```bash
# Run all tests with memory backend (no RocksDB required)
cargo test --package luminara_db --no-default-features --features memory

# Run specific test suite
cargo test --package luminara_db --test entity_relationships_test --no-default-features --features memory
cargo test --package luminara_db --test basic_crud_test --no-default-features --features memory
```

## Implementation Status

### ✅ Completed (Task 9.1)
- Basic CRUD operations for entities, components, assets, operations
- Embedded SurrealDB with in-memory backend
- Schema initialization

### ✅ Completed (Task 9.2)
- Entity storage with Record Links
- Load entities with relationships (components, parent, children)
- Graph traversal queries (find by component, descendants, ancestors)
- Comprehensive test coverage

### ✅ Completed (Task 9.3)
- Asset dependency tracking with direct and transitive queries
- Reverse dependency lookups (find what depends on an asset)
- Scene asset queries (find all assets of a type used in a scene)
- Complex queries like "find all textures used by materials in a scene"
- Circular dependency handling
- Comprehensive test coverage

### 🚧 In Progress
- (No tasks currently in progress)

### ✅ Completed (Task 9.4)
- ECS synchronization with World state
- Sync latency optimization (<16ms target)
- Concurrent access handling
- Comprehensive test coverage

### ✅ Completed (Task 9.6)
- Operation timeline storage with inverse commands
- Persistent undo/redo across sessions
- Branch management support
- Comprehensive test coverage

### ✅ Completed (Task 9.7)
- WASM/IndexedDB backend support
- Browser-based persistent storage
- Same API as native backends
- Comprehensive test coverage and documentation

### ✅ Completed (Task 9.8)
- RON to database migration tool
- Two-pass migration preserving all relationships
- Batch processing support
- Migration verification
- Comprehensive documentation and examples
