# Asset Dependency Tracking

**Task 9.3 Implementation Summary**

## Overview

This document describes the asset dependency tracking system implemented for the Luminara Database. The system enables complex graph queries on asset relationships, supporting use cases like "find all textures used by materials in this scene" with transitive dependency resolution.

## Requirements Validated

**Requirement 21.3**: SurrealDB統合とグラフベースアセット管理
- ✅ Store asset relationships in graph
- ✅ Query asset dependencies with SurrealQL
- ✅ Support transitive dependency queries

## Features Implemented

### 1. Direct Dependency Queries

```rust
pub async fn find_asset_dependencies(&self, asset_id: &RecordId) -> DbResult<Vec<AssetRecord>>
```

Returns the immediate dependencies of an asset. For example, if a Material depends on two Textures, this returns those two Textures.

**Use Case**: "What does this material directly depend on?"

### 2. Transitive Dependency Queries

```rust
pub async fn find_asset_dependencies_transitive(&self, asset_id: &RecordId) -> DbResult<Vec<AssetRecord>>
```

Returns all dependencies recursively, following the dependency chain. Handles circular dependencies by tracking visited nodes.

**Use Case**: "What are ALL the assets I need to load to use this mesh?" (Mesh → Material → Textures)

**Algorithm**:
- Uses breadth-first search with visited set
- Prevents infinite loops from circular dependencies
- Returns unique set of all transitive dependencies

### 3. Reverse Dependency Queries

```rust
pub async fn find_asset_dependents(&self, asset_id: &RecordId) -> DbResult<Vec<AssetRecord>>
```

Returns all assets that depend on the specified asset (reverse lookup).

**Use Case**: "If I modify this texture, what materials will be affected?"

### 4. Scene Asset Queries

```rust
pub async fn find_assets_in_scene(&self, scene_id: &RecordId, asset_type: &str) -> DbResult<Vec<AssetRecord>>
```

Finds all assets of a specific type used by entities in a scene. Performs graph traversal:
1. Find all entities in scene (descendants)
2. Load components for each entity
3. Extract asset references from component data
4. Filter by asset type

**Use Case**: "Show me all textures used in this scene"

### 5. Material-Texture Scene Queries

```rust
pub async fn find_textures_used_by_materials_in_scene(&self, scene_id: &RecordId) -> DbResult<Vec<AssetRecord>>
```

Specialized query that finds materials in a scene and then finds all textures those materials depend on.

**Use Case**: "What textures do I need to load for this scene's materials?"

**Algorithm**:
1. Find all Material assets in scene
2. For each material, find its dependencies
3. Filter dependencies for Texture type
4. Return unique set of textures

### 6. Asset Reference Extraction

```rust
fn extract_asset_references(&self, data: &serde_json::Value) -> Option<Vec<RecordId>>
```

Helper function that recursively searches component data for asset references. Looks for fields named `asset_id` containing RecordId values.

**Use Case**: Extract asset references from arbitrary component data structures.

## Data Model

### AssetRecord

```rust
pub struct AssetRecord {
    pub id: Option<RecordId>,
    pub path: String,
    pub asset_type: String,  // "Texture", "Mesh", "Material", "Scene"
    pub hash: String,
    pub dependencies: Vec<RecordId>,  // Record Links to other assets
    pub metadata: AssetMetadata,
}
```

The `dependencies` field uses SurrealDB Record Links to create a directed graph of asset relationships.

## Example Usage

### Simple Dependency Chain

```rust
// Create: Mesh -> Material -> Texture
let texture = AssetRecord::new("albedo.png", "Texture", "hash1", metadata);
let texture_id = db.store_asset(texture).await?;

let material = AssetRecord::new("metal.mat", "Material", "hash2", metadata)
    .with_dependency(texture_id.clone());
let material_id = db.store_asset(material).await?;

let mesh = AssetRecord::new("cube.mesh", "Mesh", "hash3", metadata)
    .with_dependency(material_id.clone());
let mesh_id = db.store_asset(mesh).await?;

// Query transitive dependencies
let all_deps = db.find_asset_dependencies_transitive(&mesh_id).await?;
// Returns: [material, texture]
```

### Complex Dependency Graph

```rust
// Material1 -> Texture1, Texture2
// Material2 -> Texture2, Texture3
// Mesh -> Material1, Material2

let material1 = AssetRecord::new("mat1.mat", "Material", "hash", metadata)
    .with_dependency(texture1_id)
    .with_dependency(texture2_id);

let material2 = AssetRecord::new("mat2.mat", "Material", "hash", metadata)
    .with_dependency(texture2_id)  // Shared texture
    .with_dependency(texture3_id);

let mesh = AssetRecord::new("cube.mesh", "Mesh", "hash", metadata)
    .with_dependency(material1_id)
    .with_dependency(material2_id);

// Find all dependencies
let deps = db.find_asset_dependencies_transitive(&mesh_id).await?;
// Returns: 2 materials + 3 unique textures = 5 assets
```

### Scene Queries

```rust
// Find all textures used in a scene
let textures = db.find_assets_in_scene(&scene_id, "Texture").await?;

// Find textures used by materials (more specific)
let material_textures = db.find_textures_used_by_materials_in_scene(&scene_id).await?;
```

## Performance Considerations

### Transitive Query Optimization

The transitive dependency query uses a visited set to prevent:
- Infinite loops from circular dependencies
- Duplicate processing of shared dependencies
- Redundant database queries

### Caching Opportunities

Future optimizations could include:
- Cache frequently queried dependency graphs
- Invalidate cache when assets are modified
- Pre-compute transitive closures for large graphs

### Query Complexity

- **Direct dependencies**: O(n) where n = number of dependencies
- **Transitive dependencies**: O(V + E) where V = vertices (assets), E = edges (dependencies)
- **Scene queries**: O(E * C) where E = entities in scene, C = components per entity

## Testing

Comprehensive test coverage in `tests/asset_dependency_test.rs`:

1. ✅ Direct asset dependencies
2. ✅ Transitive asset dependencies (3-level chain)
3. ✅ Complex dependency graphs (shared dependencies)
4. ✅ Circular dependency handling
5. ✅ Reverse dependency lookups
6. ✅ Scene asset queries
7. ✅ Material-texture scene queries
8. ✅ Query by asset type
9. ✅ Empty dependency handling

All tests use in-memory database for fast execution.

## Future Enhancements

### Dependency Analysis

- Detect unused assets (no dependents)
- Find orphaned assets (not in any scene)
- Calculate asset reference counts
- Generate dependency visualization graphs

### Advanced Queries

- Find shortest dependency path between two assets
- Find all assets within N hops of a given asset
- Group assets by dependency depth
- Find strongly connected components (circular dependency groups)

### Performance Monitoring

- Track query execution times
- Identify slow dependency traversals
- Optimize hot paths with caching
- Add query result pagination for large graphs

### Editor Integration

- Real-time dependency updates as assets change
- Dependency impact analysis ("what breaks if I delete this?")
- Asset usage statistics
- Dependency graph visualization in editor

## Conclusion

The asset dependency tracking system provides a solid foundation for graph-based asset management in Luminara Engine. It supports the complex queries needed for editor functionality while maintaining good performance through careful algorithm design and circular dependency handling.

The implementation validates Requirement 21.3 and prepares the engine for advanced editor features like:
- Asset impact analysis
- Intelligent asset loading
- Dependency-aware asset deletion
- Scene optimization based on asset usage
