# Luminara Render Examples

This directory contains examples demonstrating the rendering capabilities of Luminara Engine.

## Shader Generation Demo

**File:** `shader_generation_demo.rs`

Demonstrates dynamic shader generation from symbolic mathematical expressions using the `ShaderGenerator`. This example showcases the foundation for AI-driven shader creation.

### Running the Example

```bash
cargo run --package luminara_render --example shader_generation_demo
```

### What It Demonstrates

The example creates four different shader effects:

#### 1. Pulsating Glow Effect

Creates a shader that pulsates at 2Hz using a sinusoidal function.

**Mathematical Expression:** `sin(time * 2π * 2) * intensity`

**Use Cases:**
- Glowing effects for UI elements
- Breathing animations for characters
- Attention-grabbing indicators
- Energy field effects

**Key Concepts:**
- Time-based animation
- Sinusoidal modulation
- Frequency control (2Hz = 2 cycles per second)
- Normalization to [0, 1] range

#### 2. Procedural Noise Pattern

Creates a checkerboard-like pattern using trigonometric functions.

**Mathematical Expression:** `sin(x * 10) * cos(y * 10)`

**Use Cases:**
- Procedural textures
- Background patterns
- Visual effects
- Terrain generation

**Key Concepts:**
- Spatial frequency-based effects
- Trigonometric interference patterns
- Coordinate-based computation
- Adjustable pattern density (frequency parameter)

#### 3. Dynamic Color Gradient

Creates a time-based color gradient that smoothly transitions.

**Mathematical Expression:** `sin(t * 2π) * 0.5 + 0.5`

**Use Cases:**
- Animated backgrounds
- UI transitions
- Atmospheric effects
- Color cycling

**Key Concepts:**
- Temporal color transitions
- Oscillating gradients
- Smooth interpolation
- Normalized color values

#### 4. Complex Combined Effect

Combines temporal and spatial effects for sophisticated visuals.

**Mathematical Expression:** `sin(time * 2π) * (sin(x * 5) + cos(y * 5)) * 0.5`

**Use Cases:**
- Water surface effects
- Energy fields
- Magical effects
- Dynamic environmental effects

**Key Concepts:**
- Combining temporal and spatial components
- Multi-dimensional effects
- Compositional shader design
- Complex visual patterns

### Technical Details

#### Shader Caching

The `ShaderGenerator` implements automatic caching:

- **First Generation:** Expression is compiled to WGSL (~1-10ms)
- **Subsequent Generations:** Retrieved from cache (~0.01ms)
- **Cache Key:** Based on expression structure
- **Statistics:** Hit rate, miss rate, total generated

#### Performance Characteristics

```
Cache Statistics (from example run):
- Total shaders generated: 4
- Cache hits: 0 (first run)
- Cache misses: 4
- Cache hit rate: 0.00% (increases on subsequent runs)
- Cache size: 4 shaders
```

#### Generated WGSL Structure

Each generated shader includes:

```wgsl
struct Input {
    time: f32,
    uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> input: Input;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Vertex shader implementation
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Fragment shader with generated expression
}
```

### AI Integration

This example demonstrates the foundation for AI-driven shader generation:

1. **Natural Language Input:** "Create a pulsating glow effect at 2Hz"
2. **AI Expression Generation:** AI converts to mathematical expression
3. **Shader Compilation:** `ShaderGenerator` converts to WGSL
4. **Runtime Application:** Shader applied to materials

**Example Workflow:**

```rust
// Pseudocode for AI integration
async fn generate_shader_from_description(
    description: &str,
    llm_client: &LlmClient,
    generator: &mut ShaderGenerator,
) -> Result<String, Error> {
    // AI generates mathematical expression
    let expr = llm_client.generate_expression(description).await?;
    
    // Convert to shader
    let wgsl = generator.generate_from_expr(&expr);
    
    Ok(wgsl)
}
```

### Testing

The example includes comprehensive tests:

```bash
cargo test --package luminara_render --example shader_generation_demo
```

**Test Coverage:**
- ✅ All examples generate valid shaders
- ✅ Shader caching works correctly
- ✅ Pulsating glow contains sine function
- ✅ Procedural noise contains trigonometric functions
- ✅ Complex effect combines temporal and spatial components

### Requirements Satisfied

This example satisfies **Requirement 13.4** from the Pre-Editor Engine Audit:

> WHEN generating shaders, THE System SHALL use SymExpr to enable AI-driven shader generation from mathematical expressions

**Acceptance Criteria Met:**
- ✅ Uses SymExpr for shader generation
- ✅ Generates WGSL from mathematical expressions
- ✅ Implements shader caching for performance
- ✅ Provides foundation for AI-driven creation
- ✅ Demonstrates multiple effect types
- ✅ Includes comprehensive documentation

### Further Reading

- **ShaderGenerator Documentation:** `crates/luminara_render/docs/shader_generator.md`
- **SymExpr Documentation:** `crates/luminara_math/src/symbolic/mod.rs`
- **WGSL Codegen:** `crates/luminara_math/src/symbolic/wgsl_codegen.rs`
- **AI Shader Pipeline:** `crates/luminara_render/docs/ai_shader_pipeline.md`

### Extending the Examples

To create your own shader effects:

```rust
use luminara_math::symbolic::SymExpr;
use luminara_render::shader_generator::ShaderGenerator;
use std::rc::Rc;

fn create_custom_effect(generator: &mut ShaderGenerator) -> String {
    // Define your mathematical expression
    let x = Rc::new(SymExpr::Var("x".to_string()));
    let time = Rc::new(SymExpr::Var("time".to_string()));
    
    // Combine operations
    let expr = Rc::new(SymExpr::Sin(
        Rc::new(SymExpr::Add(x, time))
    ));
    
    // Generate WGSL
    generator.generate_from_expr(&expr)
}
```

**Supported Operations:**
- Constants: `SymExpr::Const(value)`
- Variables: `SymExpr::Var(name)` - `time`, `x`, `y`, `t`
- Arithmetic: `Add`, `Sub`, `Mul`, `Div`
- Trigonometric: `Sin`, `Cos`
- Power: `Pow`

### Performance Tips

1. **Reuse ShaderGenerator:** Create once, use many times
2. **Monitor Cache:** Check hit rate with `generator.stats()`
3. **Clear When Needed:** Use `clear_cache()` if memory is constrained
4. **Batch Generation:** Generate all shaders at startup for best performance

### Common Patterns

**Time-Based Animation:**
```rust
let time = Rc::new(SymExpr::Var("time".to_string()));
let freq = Rc::new(SymExpr::Const(frequency));
let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time, freq))));
```

**Spatial Pattern:**
```rust
let x = Rc::new(SymExpr::Var("x".to_string()));
let scale = Rc::new(SymExpr::Const(10.0));
let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(x, scale))));
```

**Normalization to [0, 1]:**
```rust
let one = Rc::new(SymExpr::Const(1.0));
let half = Rc::new(SymExpr::Const(0.5));
let normalized = Rc::new(SymExpr::Mul(
    Rc::new(SymExpr::Add(expr, one)),
    half
));
```

## Contributing

When adding new examples:

1. Include comprehensive documentation
2. Add tests to verify functionality
3. Update this README
4. Follow existing code style
5. Demonstrate practical use cases


## Transform Debug Visualization Demo

**File:** `transform_debug_demo.rs`

Demonstrates the transform debug visualization features of the GizmoSystem, including coordinate axes rendering, hierarchy connections, and entity selection highlighting.

### Running the Example

```bash
cargo run --package luminara_render --example transform_debug_demo
```

### What It Demonstrates

The example showcases seven different aspects of transform debug visualization:

#### 1. Coordinate Axes Rendering

Renders RGB coordinate axes (X=red, Y=green, Z=blue) at entity positions.

**Use Cases:**
- Visualizing entity orientation
- Debugging transform hierarchies
- Understanding local coordinate systems
- Editor gizmo rendering

**Key Features:**
- Configurable axes length
- Automatic color coding (RGB = XYZ)
- Respects visualization mode settings
- Scales with entity size

#### 2. Hierarchy Connections

Draws lines connecting parent and child entities to visualize scene hierarchy.

**Use Cases:**
- Understanding parent-child relationships
- Debugging transform propagation
- Visualizing scene graph structure
- Editor hierarchy view

**Key Features:**
- Configurable line color
- Toggle on/off independently
- Automatic parent position tracking
- Works with complex hierarchies

#### 3. Entity Selection Highlighting

Highlights selected entities with a colored sphere.

**Use Cases:**
- Visual feedback for selection
- Editor entity picking
- Multi-selection visualization
- Focus indication

**Key Features:**
- Configurable highlight color (default: yellow)
- Radius scales with entity size
- Semi-transparent for visibility
- Independent of other visualizations

#### 4. Complete Entity Transform Visualization

Combines axes, hierarchy, and selection into a single convenient method.

**Parameters:**
- `position`: Entity world position
- `parent_position`: Optional parent position for hierarchy line
- `is_selected`: Whether to show selection highlight
- `scale`: Scale factor for axes and highlight

**Example:**
```rust
gizmo_system.draw_entity_transform(
    &mut buffer,
    entity_position,
    parent_position,
    is_selected,
    1.0
);
```

#### 5. Customizable Settings

All visualization aspects can be customized:

```rust
let settings = gizmo_system.transform_settings_mut();
settings.axes_length = 2.0;                                    // Longer axes
settings.hierarchy_color = Color::rgb(0.0, 1.0, 1.0);         // Cyan lines
settings.selection_color = Color::rgb(1.0, 0.0, 1.0);         // Magenta highlight
settings.show_axes = true;                                     // Toggle axes
settings.show_hierarchy = true;                                // Toggle hierarchy
```

#### 6. Complex Hierarchy Visualization

Demonstrates rendering a multi-level hierarchy:

```
Root (0, 0, 0)
├── Child 1 (2, 0, 0)
│   └── Grandchild (2, 2, 0)
└── Child 2 (-2, 0, 0) [SELECTED]
```

**Features:**
- Multiple levels of nesting
- Selective highlighting
- Clear visual relationships
- Efficient rendering

#### 7. Runtime Toggle Control

Shows how to enable/disable visualization features at runtime:

```rust
// Disable axes
gizmo_system.transform_settings_mut().show_axes = false;

// Disable hierarchy
gizmo_system.transform_settings_mut().show_hierarchy = false;

// Disable entire transform mode
gizmo_system.disable_mode(VisualizationMode::Transforms);
```

### Integration with ECS

In a real application, integrate with your ECS system:

```rust
use luminara_core::{Query, Res, ResMut};
use luminara_scene::{Transform, Parent, Children};
use luminara_render::{GizmoSystem, CommandBuffer};

fn transform_debug_system(
    mut gizmo_system: ResMut<GizmoSystem>,
    mut buffer: ResMut<CommandBuffer>,
    transforms: Query<(&Transform, Option<&Parent>)>,
    selected: Res<SelectedEntities>,
) {
    if !gizmo_system.is_mode_active(VisualizationMode::Transforms) {
        return;
    }

    for (entity, (transform, parent)) in transforms.iter() {
        let parent_pos = parent
            .and_then(|p| transforms.get(p.0).ok())
            .map(|(t, _)| t.position);
        
        let is_selected = selected.contains(entity);
        
        gizmo_system.draw_entity_transform(
            &mut buffer,
            transform.position,
            parent_pos,
            is_selected,
            transform.scale.x,
        );
    }
}
```

### Performance Characteristics

**Rendering Cost:**
- Axes: 3 draw calls per entity (X, Y, Z arrows)
- Hierarchy: 1 draw call per parent-child connection
- Selection: 1 draw call per selected entity

**Optimization Tips:**
1. Only enable when needed (editor mode)
2. Cull entities outside view frustum
3. Use LOD for distant entities
4. Batch similar gizmo types

**Example Performance:**
```
Scene with 100 entities:
- Axes only: 300 draw calls
- Axes + hierarchy: 399 draw calls (99 connections)
- Axes + hierarchy + 5 selected: 404 draw calls
```

### Keyboard Shortcuts (Suggested)

When integrating into an editor:

- `T` - Toggle transform visualization mode
- `Shift+A` - Toggle axes display
- `Shift+H` - Toggle hierarchy display
- `Shift+S` - Toggle selection highlights

### Requirements Satisfied

This implementation satisfies **Requirement 15.3** from the Pre-Editor Engine Audit:

> WHEN debugging transforms, THE System SHALL render coordinate axes for selected entities

**Acceptance Criteria Met:**
- ✅ Renders coordinate axes for entities
- ✅ Shows hierarchy connections
- ✅ Highlights selected entities
- ✅ Integrates with existing GizmoSystem
- ✅ Configurable visualization settings
- ✅ Respects visualization mode toggles

### API Reference

#### Core Methods

**`draw_transform_axes(buffer, position, scale)`**
- Draws RGB coordinate axes at position
- Scale multiplies axes_length setting
- Only draws if show_axes is enabled

**`draw_hierarchy_connection(buffer, parent_pos, child_pos)`**
- Draws line from parent to child
- Uses hierarchy_color setting
- Only draws if show_hierarchy is enabled

**`draw_entity_highlight(buffer, position, radius)`**
- Draws selection sphere at position
- Uses selection_color setting
- Always draws when called (check is_selected externally)

**`draw_entity_transform(buffer, position, parent_pos, is_selected, scale)`**
- Convenience method combining all features
- Automatically handles conditional rendering
- Recommended for most use cases

#### Settings

**`TransformVisualizationSettings`**
```rust
pub struct TransformVisualizationSettings {
    pub show_axes: bool,           // Toggle axes rendering
    pub show_hierarchy: bool,      // Toggle hierarchy lines
    pub axes_length: f32,          // Base axes length (default: 1.0)
    pub hierarchy_color: Color,    // Hierarchy line color
    pub selection_color: Color,    // Selection highlight color
}
```

### Testing

Comprehensive tests verify all functionality:

```bash
cargo test --package luminara_render --test gizmo_system_test
```

**Test Coverage:**
- ✅ Hierarchy connection respects mode and settings
- ✅ Entity highlight respects mode
- ✅ Complete transform visualization works
- ✅ Selection highlighting works
- ✅ Settings customization works
- ✅ Runtime toggling works
- ✅ Axes length scaling works
- ✅ Selection radius scaling works

### Further Reading

- **GizmoSystem Documentation:** `crates/luminara_render/docs/gizmo_system.md`
- **Debug Rendering:** `crates/luminara_render/docs/debug_rendering.md`
- **Scene Hierarchy:** `crates/luminara_scene/src/hierarchy.rs`

### Common Patterns

**Editor Selection System:**
```rust
fn render_selected_entities(
    gizmo_system: &GizmoSystem,
    buffer: &mut CommandBuffer,
    selected: &[Entity],
    transforms: &Query<&Transform>,
) {
    for entity in selected {
        if let Ok(transform) = transforms.get(*entity) {
            gizmo_system.draw_entity_highlight(
                buffer,
                transform.position,
                1.0
            );
        }
    }
}
```

**Hierarchy Visualization:**
```rust
fn render_hierarchy(
    gizmo_system: &GizmoSystem,
    buffer: &mut CommandBuffer,
    query: &Query<(&Transform, &Parent)>,
    transforms: &Query<&Transform>,
) {
    for (child_transform, parent) in query.iter() {
        if let Ok(parent_transform) = transforms.get(parent.0) {
            gizmo_system.draw_hierarchy_connection(
                buffer,
                parent_transform.position,
                child_transform.position,
            );
        }
    }
}
```

**Conditional Visualization:**
```rust
fn render_debug_gizmos(
    gizmo_system: &GizmoSystem,
    buffer: &mut CommandBuffer,
    editor_mode: bool,
) {
    if editor_mode {
        gizmo_system.enable_mode(VisualizationMode::Transforms);
    } else {
        gizmo_system.disable_mode(VisualizationMode::Transforms);
    }
    
    // Rendering code...
}
```
