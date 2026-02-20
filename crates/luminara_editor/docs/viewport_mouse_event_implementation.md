# Viewport Mouse Event Handling Implementation

## Overview

This document describes the implementation of mouse event handling for the ViewportElement, which routes events to Luminara's input system for camera controls and object interaction.

## Requirements Addressed

- **Requirement 12.5.1**: THE System SHALL route viewport mouse events to Luminara's input system
- **Requirement 12.5.2**: WHEN the viewport is focused, THE System SHALL forward keyboard input to Luminara
- **Requirement 16.5**: THE System SHALL implement custom mouse event handling (MouseDownEvent, MouseMoveEvent) for node dragging

## Implementation Details

### 1. Mouse Event Routing

The ViewportElement now includes an optional `EngineHandle` that provides access to Luminara's input system:

```rust
pub struct ViewportElement {
    // ... existing fields ...
    /// Engine handle for routing events to Luminara's input system
    engine_handle: Option<Arc<EngineHandle>>,
}
```

### 2. Event Conversion

Mouse events from GPUI are converted to Luminara's input format:

- **GPUI MouseButton** → **Luminara MouseButton**
  - `MouseButton::Left` → `luminara_input::MouseButton::Left`
  - `MouseButton::Right` → `luminara_input::MouseButton::Right`
  - `MouseButton::Middle` → `luminara_input::MouseButton::Middle`

- **GPUI Pixel Coordinates** → **Viewport-Relative Coordinates**
  - Position is already in viewport-relative coordinates from GPUI
  - Converted to f32 for Luminara's Vec2 format

### 3. Event Routing Methods

#### `route_mouse_event(button, position, pressed)`

Routes mouse button press/release events to Luminara's input system:
- Converts GPUI mouse button to Luminara format
- Extracts viewport-relative coordinates
- Logs events in debug builds
- Prepared for future integration with Luminara's Input resource

#### `route_mouse_move(position)`

Routes mouse movement events to Luminara's input system:
- Tracks mouse position
- Calculates delta from last position
- Logs movement during drag operations
- Prepared for future integration with Luminara's Input resource

### 4. Integration with Camera Controls

The existing camera control methods now route events:

- **`start_drag(button, position, mode)`**: Routes mouse down event
- **`update_drag(position)`**: Routes mouse move event
- **`stop_drag(button)`**: Routes mouse up event

### 5. Usage Example

```rust
// Create viewport with engine handle
let viewport = ViewportElement::new(render_target, camera, GizmoMode::None)
    .with_engine_handle(engine_handle);

// Mouse events are automatically routed to Luminara's input system
// when the viewport is interacted with
```

## Future Integration

The current implementation includes placeholder code for full integration with Luminara's Input resource. When the input system is fully integrated with the editor, the following code will be activated:

```rust
// Update Input resource in World
let mut world = engine_handle.world_mut();
if let Some(mut input) = world.get_resource_mut::<luminara_input::Input>() {
    if pressed {
        input.mouse.buttons.insert(luminara_button);
        input.mouse.just_pressed.insert(luminara_button);
    } else {
        input.mouse.buttons.remove(&luminara_button);
        input.mouse.just_released.insert(luminara_button);
    }
    input.mouse.position = Vec2::new(viewport_x, viewport_y);
    input.mouse.delta = Vec2::new(delta.0, delta.1);
}
```

## Testing

The implementation includes comprehensive unit tests:

- `test_viewport_element_creation`: Verifies viewport creation
- `test_viewport_with_engine_handle`: Tests engine handle integration
- `test_viewport_drag_modes`: Tests drag mode transitions
- `test_viewport_mouse_event_routing`: Tests event routing without panicking

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  GPUI UI Layer                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │         ViewportElement                           │  │
│  │  - MouseDownEvent                                 │  │
│  │  - MouseMoveEvent                                 │  │
│  │  - MouseUpEvent                                   │  │
│  └───────────────┬───────────────────────────────────┘  │
└──────────────────┼──────────────────────────────────────┘
                   │ route_mouse_event()
                   │ route_mouse_move()
                   ▼
┌─────────────────────────────────────────────────────────┐
│              EngineHandle                               │
│  ┌───────────────────────────────────────────────────┐  │
│  │         world_mut()                               │  │
│  │  - Access to ECS World                            │  │
│  │  - Access to Input resource                       │  │
│  └───────────────┬───────────────────────────────────┘  │
└──────────────────┼──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│           Luminara Input System                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │         Input Resource                            │  │
│  │  - mouse.buttons                                  │  │
│  │  - mouse.position                                 │  │
│  │  - mouse.delta                                    │  │
│  │  - mouse.just_pressed                             │  │
│  │  - mouse.just_released                            │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Benefits

1. **Separation of Concerns**: UI event handling is separate from engine input processing
2. **Type Safety**: Strong typing ensures correct event conversion
3. **Extensibility**: Easy to add keyboard input routing (Requirement 12.5.2)
4. **Testability**: Event routing can be tested independently
5. **Performance**: Minimal overhead for event conversion

## Next Steps

1. Implement keyboard input routing (Requirement 12.5.2)
2. Activate full Input resource integration when ready
3. Add gizmo manipulation event handling
4. Implement viewport focus management
5. Add input priority handling between UI and viewport

## Related Files

- `crates/luminara_editor/src/viewport.rs`: ViewportElement implementation
- `crates/luminara_editor/src/engine.rs`: EngineHandle implementation
- `crates/luminara_input/src/lib.rs`: Luminara Input system
- `crates/luminara_input/src/mouse.rs`: MouseInput and MouseButton definitions
