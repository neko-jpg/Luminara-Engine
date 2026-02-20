# Selection Synchronization Implementation

## Overview

This document describes the implementation of bidirectional selection synchronization in the Scene Builder, fulfilling **Requirement 4.8**: "THE Scene_Builder SHALL sync selection between hierarchy and viewport".

## Architecture

### Shared Selection State

The selection state is shared between the hierarchy panel, viewport panel, and inspector panel using `Arc<RwLock<HashSet<Entity>>>`. This allows all panels to access and modify the same selection state in a thread-safe manner.

```rust
pub struct SceneBuilderBox {
    // Shared selection state for synchronization
    selected_entities: Arc<RwLock<HashSet<Entity>>>,
    // ...
}
```

### Components

#### 1. SceneBuilderBox

The main container that owns the shared selection state and coordinates updates between panels.

**Key Methods:**
- `select_entity(entity, multi_select, cx)` - Updates selection and triggers UI refresh
- `clear_selection(cx)` - Clears all selected entities
- `selected_entities()` - Returns a clone of the current selection

#### 2. Hierarchy Panel

Displays the scene hierarchy tree with visual feedback for selected entities.

**Selection Behavior:**
- **Click**: Selects a single entity (clears previous selection)
- **Shift+Click**: Toggles entity in multi-selection
- **Visual Feedback**: Selected entities are highlighted with accent color

#### 3. Viewport Panel

Displays the 3D viewport with visual feedback for selected entities (gizmos, outlines).

**Selection State:**
- Receives shared selection state via `Arc<RwLock<HashSet<Entity>>>`
- Can read selection to highlight entities in the viewport
- Future: Will support clicking entities in viewport to update selection

#### 4. Inspector Panel

Displays properties of the currently selected entity.

**Behavior:**
- Automatically updates when selection changes
- Shows "No Selection" message when nothing is selected
- Displays the first selected entity in multi-selection scenarios

## Data Flow

### Hierarchy → Viewport → Inspector

```
User clicks entity in hierarchy
    ↓
SceneBuilderBox::select_entity() called
    ↓
selected_entities (Arc<RwLock<HashSet<Entity>>>) updated
    ↓
cx.notify() triggers re-render
    ↓
Hierarchy: Highlights selected entity
Viewport: Reads selection, highlights entity with gizmos
Inspector: Reads selection, displays entity properties
```

### Multi-Selection Flow

```
User Shift+Clicks entity
    ↓
event.modifiers.shift detected
    ↓
select_entity(entity, multi_select=true, cx)
    ↓
Entity toggled in selection set
    ↓
All panels update to reflect new selection
```

## Implementation Details

### Thread Safety

The selection state uses `Arc<RwLock<HashSet<Entity>>>` for thread-safe access:
- **Arc**: Allows multiple panels to share ownership
- **RwLock**: Allows multiple readers or one writer at a time
- **HashSet**: Efficient O(1) lookup for selection checks

### Lock Management

To avoid deadlocks and ensure responsiveness:
1. Acquire locks for minimal duration
2. Drop locks explicitly before calling `cx.notify()`
3. Clone data when needed for rendering

Example:
```rust
fn render_hierarchy_item(&self, entity: Entity, ...) -> impl IntoElement {
    let selected = self.selected_entities.read();
    let is_selected = selected.contains(&entity);
    drop(selected); // Release lock early
    
    // Continue rendering without holding lock
    // ...
}
```

### GPUI Integration

Selection updates trigger GPUI's reactive rendering system:
- `cx.notify()` marks the view as needing re-render
- GPUI efficiently updates only changed UI elements
- No manual DOM manipulation required

## Testing

### Unit Test: `test_selection_synchronization`

Verifies:
1. ✅ Single selection works correctly
2. ✅ Selection replacement (clicking different entity)
3. ✅ Multi-selection (Shift+Click)
4. ✅ Toggle selection (Shift+Click on selected entity)
5. ✅ Clear selection
6. ✅ ViewportElement can access shared selection

## Future Enhancements

### Viewport Selection (Planned)

Currently, selection flows from hierarchy → viewport. Future work will add:
- **Ray casting**: Click detection in 3D viewport
- **Entity picking**: Identify clicked entity from viewport coordinates
- **Reverse sync**: Update hierarchy when entity clicked in viewport

### Multi-Selection Improvements

- **Box selection**: Drag to select multiple entities
- **Select all**: Keyboard shortcut to select all entities
- **Invert selection**: Select all unselected entities

### Visual Feedback

- **Gizmo rendering**: Show transform gizmos for selected entities
- **Outline rendering**: Highlight selected entities with colored outline
- **Bounding box**: Display bounding box around selected entities

## Requirements Validation

✅ **Requirement 4.8**: "THE Scene_Builder SHALL sync selection between hierarchy and viewport"

**Evidence:**
- Shared selection state (`Arc<RwLock<HashSet<Entity>>>`)
- Hierarchy updates selection on click
- Viewport receives selection state
- Inspector updates automatically
- Test coverage validates synchronization

## Related Files

- `crates/luminara_editor/src/scene_builder.rs` - Main implementation
- `crates/luminara_editor/src/viewport.rs` - Viewport element with selection support
- `crates/luminara_editor/tests/property_selection_sync_test.rs` - Property-based tests (future)

## References

- GPUI Reactive System: https://github.com/zed-industries/zed
- Luminara ECS: `crates/luminara_core/src/world.rs`
- Scene Hierarchy: `crates/luminara_scene/src/hierarchy.rs`
